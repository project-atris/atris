use atris_common::{set_room_responder::SetRoomResponderError, Encrypted, REGION};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    model::AttributeValue,
    types::{Blob, SdkError},
};

use atris_common::{create_room::CreateRoomError, join_room::JoinRoomError, RoomData};
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

pub struct Room {
    /// The room's id number, shared to another user to start the conversation
    pub room_id: u16,
    /// The room's creator
    pub creator_user_name: String,
    /// The salted and hashed digest of the user's password
    pub room_data: Encrypted<RoomData>,
}
impl Room {
    fn from_map(map: &HashMap<String, AttributeValue>) -> Option<Self> {
        let room_id = map.get(ROOM_ID_KEY)?.as_n().ok()?.parse().ok()?;
        let room_data_slice = map.get(ROOM_DATA_KEY)?.as_b().ok()?.as_ref();
        let room_creator = map.get(ROOM_CREATOR_KEY)?.as_s().ok()?;
        let room_data = bincode::deserialize(room_data_slice).ok()?;
        Some(Self {
            room_id,
            room_data,
            creator_user_name: room_creator.clone(),
        })
    }
}

pub struct AtrisRoomDBClient {
    /// The AWS DynamoDB client that Lambda will use for API calls
    client: Client,
}

impl AtrisRoomDBClient {
    pub async fn new() -> Self {
        // Set the region to us-west-2 (Oregon) if possible, or fallback on the default
        let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
        // Use this region to configure the SDK
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }
    pub async fn create_room(&self, room_id: u16, creator: String) -> Result<(), CreateRoomError> {
        let db_request = self
            .client
            .put_item()
            .condition_expression(format!("attribute_not_exists({})", ROOM_ID_KEY))
            .table_name(TABLE_NAME)
            .item(ROOM_ID_KEY, AttributeValue::N(room_id.to_string()))
            .item(ROOM_CREATOR_KEY, AttributeValue::S(creator));
        db_request.send().await.map_err(|e| {
            if let SdkError::ServiceError { err, .. } = &e {
                if err.is_conditional_check_failed_exception() {
                    return CreateRoomError::DuplicateRoomId(room_id);
                }
            }
            dbg!(e);
            CreateRoomError::DatabaseWriteError
        })?;
        Ok(())
    }

    /// Update a room
    pub async fn update_room_data(
        &self,
        room_id: u16,
        updater: String,
        room_data: Encrypted<RoomData>,
    ) -> Result<(), SetRoomResponderError> {
        let room_data =
            bincode::serialize(&room_data).map_err(|_| SetRoomResponderError::BincodeError)?;
        // Generate a request, which includes the username and (hopefully hashed) password
        let db_request = self
            .client
            .update_item()
            .key(ROOM_ID_KEY, AttributeValue::N(room_id.to_string()))
            .expression_attribute_values(":updater", AttributeValue::S(updater.clone()))
            .expression_attribute_values(":room_data", AttributeValue::B(Blob::new(room_data)))
            .condition_expression(format!("{ROOM_CREATOR_KEY} = :updater"))
            .table_name(TABLE_NAME)
            .update_expression(format!("SET {ROOM_DATA_KEY}= :room_data"));
        // .attribute_updates(ROOM_CREATOR_KEY, AttributeValueUpdate::builder().set_action(Some(AttributeAction::Put)).set_value(Some(AttributeValue::S(room.creator_user_name))).build());

        // Send the request to the database
        db_request.send().await.map(|_| {}).map_err(|err| {
            if let SdkError::ServiceError { err, .. } = err {
                if err.is_conditional_check_failed_exception() {
                    return SetRoomResponderError::NotRoomCreator(updater);
                }
            }
            SetRoomResponderError::DatabaseWriteError
        })
    }

    /// Retrieves the user of the specified username
    pub async fn get_room(&self, room_id: u16) -> Result<Room, JoinRoomError> {
        let db_request = self
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key(ROOM_ID_KEY, AttributeValue::N(room_id.to_string()))
            .attributes_to_get(ROOM_ID_KEY) //get the relevant fields
            .attributes_to_get(ROOM_CREATOR_KEY)
            .attributes_to_get(ROOM_DATA_KEY)
            .send()
            .await
            .map_err(|_| JoinRoomError::DatabaseReadError)?; //convert SdkError to GetRoomError
        db_request
            .item()
            .ok_or(JoinRoomError::NonexistentRoomId(room_id))
            .and_then(|m| Room::from_map(m).ok_or(JoinRoomError::IncompleteRoom))
    }
}

pub const ROOM_ID_KEY: &'static str = "room_id";
pub const ROOM_CREATOR_KEY: &'static str = "room_creator";
pub const ROOM_DATA_KEY: &'static str = "room_data";

pub const TABLE_NAME: &'static str = "atris_rooms";

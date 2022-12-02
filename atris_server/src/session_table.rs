use atris_common::{CipherKey, REGION};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    model::AttributeValue,
    types::{Blob, SdkError},
};

use aws_sdk_dynamodb::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt::Display};
//
#[derive(Debug)]
pub struct Session {
    /// The user's id for this session
    pub session_id: CipherKey,
    /// The user's username
    pub username: String,
    /// The user's WebRTC initiator
    pub initiator: String,
}
impl Session {
    fn new(session_id: CipherKey, username: String, initiator: String) -> Self {
        Self {
            session_id,
            username,
            initiator,
        }
    }

    fn from_map(map: &HashMap<String, AttributeValue>) -> Option<Self> {
        let session_id_bytes: Vec<u8> = map.get(SESSION_ID_KEY)?.as_b().ok()?.as_ref().into();
        let session_id = CipherKey::from(session_id_bytes.as_slice());
        let username = map.get(USERNAME_KEY)?.as_s().ok()?;
        let initiator = map.get(INITIATOR_KEY)?.as_s().ok()?;
        Some(Self::new(session_id, username.clone(), initiator.clone()))
    }
}

pub struct AtrisSessionDBClient {
    /// The AWS DynamoDB client that Lambda will use for API calls
    client: Client,
}
impl AtrisSessionDBClient {
    pub async fn new() -> Self {
        // Set the region to us-west-2 (Oregon) if possible, or fallback on the default
        let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
        // Use this region to configure the SDK
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }

    /// Creates a new session for the user
    pub async fn create_session(
        &self,
        session_id: CipherKey,
        username: String,
        initiator: String,
    ) -> Result<CreateSessionResponse, CreateSessionError> {
        // Generate a request, which includes the necessary info
        let db_request = self
            .client
            .put_item()
            .condition_expression(format!("attribute_not_exists({})", SESSION_ID_KEY))
            .table_name(TABLE_NAME)
            .item(
                SESSION_ID_KEY,
                AttributeValue::B(Blob::new(session_id.as_ref())),
            )
            .item(USERNAME_KEY, AttributeValue::S(username.clone()))
            .item(INITIATOR_KEY, AttributeValue::S(initiator.clone()));

        // Send the request to the database
        db_request.send().await.map_err(|e| {
            if let SdkError::ServiceError { err, .. } = &e {
                if err.is_conditional_check_failed_exception() {
                    return CreateSessionError::DuplicateSession(session_id);
                }
            }
            dbg!(e);
            CreateSessionError::DatabaseWriteError
        })?;
        Ok(CreateSessionResponse)
    }

    /// Retrieves the session of the specified session_id
    pub async fn get_session(
        &self,
        session_id: CipherKey,
    ) -> Result<Option<Session>, AuthenticateSessionError> {
        let db_request = self
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key(
                SESSION_ID_KEY,
                AttributeValue::B(Blob::new(session_id.as_ref())),
            )
            .attributes_to_get(SESSION_ID_KEY)
            .attributes_to_get(USERNAME_KEY)
            .attributes_to_get(INITIATOR_KEY)
            .send()
            .await
            .map_err(|e| {
                dbg!(e);
                AuthenticateSessionError::DatabaseRead
            })?; //convert SdkError to AuthenticateSessionError
        return Ok(db_request.item().and_then(Session::from_map));
    }

    /// Retrieves a session of the specified username
    pub async fn get_session_for_username(
        &self,
        username: String,
    ) -> Result<Option<Session>, AuthenticateSessionError> {
        let db_request = self
            .client
            .scan()
            .table_name(TABLE_NAME)
            .expression_attribute_values(":username_to_find", AttributeValue::S(username))
            .filter_expression(format!("{USERNAME_KEY} IN (:username_to_find)"))
            .projection_expression(format!("{SESSION_ID_KEY}, {USERNAME_KEY}, {INITIATOR_KEY}"))
            .send()
            .await
            .map_err(|e| {
                dbg!(e);
                AuthenticateSessionError::DatabaseRead
            })?; //convert SdkError to AuthenticateSessionError
        Ok(db_request
            .items()
            .and_then(|s| s.get(0))
            .and_then(Session::from_map))
    }
}

/// A response to a [`CreateUserRequest`] on the atris auth server. For success response, see [`CreateUserResponse`]
#[derive(Deserialize, Serialize, Debug)]
pub enum CreateSessionError {
    /// The username requested already exists in the server
    DuplicateSession(CipherKey),
    /// The write of the data failed
    DatabaseWriteError,
}
impl Display for CreateSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateSession(session_id) => {
                write!(f, "Session '{:?}' already exists", session_id)
            }
            Self::DatabaseWriteError => {
                write!(f, "Failed to write to the database")
            }
        }
    }
}
impl Error for CreateSessionError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AuthenticateSessionError {
    /// The session attempted was not found in the database
    UnknownSession(String),
    /// Failed to read the user record from the database
    DatabaseRead,
    /// Failed to write to the databse
    DatabaseWrite,
}
impl Display for AuthenticateSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSession(session_id) => {
                write!(f, "Session {} doesn't exist", session_id)
            }
            Self::DatabaseRead => {
                write!(
                    f,
                    "Failed to read user authentication details from the database"
                )
            }
            Self::DatabaseWrite => {
                write!(f, "Failed to write network information to the database")
            }
        }
    }
}
impl Error for AuthenticateSessionError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

/// A successful response to a [`CreateSessionRequest`] on the atris auth server.
///  - For error response, see [`CreateSessionError`]
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateSessionResponse; // TODO: See if anything else needs to be returned to user

pub enum GetSessionError {}

pub const SESSION_ID_KEY: &'static str = "session_id";
pub const USERNAME_KEY: &'static str = "username";
pub const INITIATOR_KEY: &'static str = "initiator";

pub const TABLE_NAME: &'static str = "atris_session";

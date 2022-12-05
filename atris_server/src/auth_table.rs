use atris_common::{
    authenticate_user::AuthenticateUserError,
    create_user::{CreateUserError, CreateUserResponse},
    REGION,
};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{model::AttributeValue, types::SdkError};

use aws_sdk_dynamodb::Client;
use std::collections::HashMap;
//
#[derive(Debug)]
pub struct User {
    /// The user's username
    pub username: String,
    /// The salted and hashed digest of the user's password
    pub password_hash: String,
}
impl User {
    fn new(username: String, password_hash: String) -> Self {
        Self {
            username,
            password_hash,
        }
    }

    fn from_map(map: &HashMap<String, AttributeValue>) -> Option<Self> {
        let username = map.get(USERNAME_KEY)?.as_s().ok()?;
        let password = map.get(PASSWORD_KEY)?.as_s().ok()?;
        Some(Self::new(username.clone(), password.clone()))
    }
}

pub struct AtrisAuthDBClient {
    /// The AWS DynamoDB client that Lambda will use for API calls
    client: Client,
}

impl AtrisAuthDBClient {
    pub async fn new() -> Self {
        // Set the region to us-west-2 (Oregon) if possible, or fallback on the default
        let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
        // Use this region to configure the SDK
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }

    /// Creates a new user if they do not exist
    pub async fn create_user(
        &self,
        username: String,
        password: String,
    ) -> Result<CreateUserResponse, CreateUserError> {
        // Generate a request, which includes the username and (hopefully hashed) password
        let db_request = self
            .client
            .put_item()
            .condition_expression(format!("attribute_not_exists({})", USERNAME_KEY))
            .table_name(TABLE_NAME)
            .item(USERNAME_KEY, AttributeValue::S(username.clone()))
            .item(PASSWORD_KEY, AttributeValue::S(password.clone()));

        // Send the request to the database
        db_request.send().await.map_err(|e| {
            if let SdkError::ServiceError { err, .. } = &e {
                if err.is_conditional_check_failed_exception() {
                    return CreateUserError::DuplicateUsername(username);
                }
            }
            dbg!(e);
            CreateUserError::DatabaseWriteError
        })?;
        Ok(CreateUserResponse)
    }

    /// Retrieves the user of the specified username
    pub async fn get_user(&self, username: String) -> Result<Option<User>, AuthenticateUserError> {
        let db_request = self
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key(USERNAME_KEY, AttributeValue::S(username.clone()))
            .attributes_to_get(USERNAME_KEY) //get the relevant fields
            .attributes_to_get(PASSWORD_KEY)
            .send()
            .await
            .map_err(|_| AuthenticateUserError::DatabaseRead)?; //convert SdkError to AuthenticateUserError
        return Ok(db_request.item().and_then(User::from_map));
    }
}
pub enum GetUserError {}

pub const USERNAME_KEY: &'static str = "username";
pub const PASSWORD_KEY: &'static str = "hashed_salted_password";
pub const SALT_KEY: &'static str = "salt";

pub const TABLE_NAME: &'static str = "atris_auth";

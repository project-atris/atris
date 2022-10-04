use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{model::AttributeValue};
use aws_sdk_dynamodb::Client;
use lambda_runtime::LambdaEvent;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;



// Error enum
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum AtrisError {
    DuplicateUsername(String),
    DatabasePutError,
    DatabaseGetError,
    ConnectionError,
}

impl Display for AtrisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtrisError::DuplicateUsername(username) => write!(f,"Username {} was already taken",username),
            AtrisError::DatabasePutError => write!(f,"Ran into a problem putting item to database."),
            AtrisError::DatabaseGetError => write!(f,"Ran into a problem getting item from database."),
            AtrisError::ConnectionError => write!(f,"Ran into a problem connecting to the database."),
        }
    }
}

// Formats the input request
#[derive(Deserialize, Serialize, Clone)]
pub struct LambdaRequest {
    full_name: String,
    message: Option<String>,
}

// Formats the output response
#[derive(Deserialize, Serialize, Clone)]
pub struct LambdaResponse {
    success: i32,
    message: Option<String>,
}



// Handle input requests
pub async fn handler(event: LambdaEvent<LambdaRequest>) -> Result<LambdaResponse, AtrisError> {
    let payload: LambdaRequest = event.payload; //payload we received from the user

    // generate the response to the user
    let response = LambdaResponse {
        success: 0,
        message: Some(format!("Hello {name}!", name = payload.full_name)),
    };

    // generate a random salt
    let salt_length: usize = 32;

    // TODO: Replace with cryptographic hash
    let salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(salt_length)
        .map(char::from)
        .collect();

    // interact with the database
    let region_provider = RegionProviderChain::first_try("us-west-2").or_default_provider();
    let dbconfig = aws_config::from_env().region(region_provider).load().await;
    let dbclient = Client::new(&dbconfig);
    let dbrequest = dbclient
        .put_item()
        .table_name("atris_auth")
        .item("username", AttributeValue::S(payload.full_name.clone()))
        .item(
            "password-hash",
            AttributeValue::S(payload.message.clone().unwrap()),
        )
        .item("salt", AttributeValue::S(salt));
    dbrequest.send().await.map_err(|_| AtrisError::DatabasePutError)?;

    Ok(response)
}


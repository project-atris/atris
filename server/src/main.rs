use lambda_runtime::{service_fn,LambdaEvent, Context, Error as LambdaError};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LambdaRequest {
    full_name: String,
    message: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LambdaResponse {
    lambda_request: LambdaRequest,
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<LambdaRequest>) -> Result<LambdaResponse, LambdaError> {
    let mut e = event.payload;
    e.full_name = format!("Hello {name}!", name = e.full_name);
    let msg = match e.message {
        Some(msg) => format!("Your message is '{msg}'.", msg = msg),
        None => format!("You have no message."),
    };
    e.message = Some(msg);
    Ok(LambdaResponse { lambda_request: e })
}





/* 
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::AttributeValue;
use lambda_runtime::{service_fn,LambdaEvent, Context, Error as LambdaError};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[derive(Deserialize)]
struct CustomEvent {
    first_name: String,
    last_name: String
}

async fn handler(event: LambdaEvent<CustomEvent>) -> Result<Value, LambdaError> {
    let payload = event.payload;
    let uuid = Uuid::new_v4().to_string();

    let region_provider = RegionProviderChain::default_provider()
        .or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let request = client.put_item()
        .table_name("users")
        .item("uid", AttributeValue::S(String::from(uuid)))
        .item("first_name", AttributeValue::S(String::from(payload.first_name)))
        .item("last_name", AttributeValue::S(String::from(payload.last_name)));

    request.send().await?;

    Ok(json!({ "message": "Record written!" }))
}



*/


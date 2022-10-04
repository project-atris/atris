use atris_server::{run_lambda, TABLE_NAME, USERNAME_KEY, PASSWORD_KEY, SALT_KEY, REGION};
use atris_common::create_user::*;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::AttributeValue;
use lambda_runtime::LambdaEvent;
use password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};

run_lambda!(create_user);

// Main function, entry point
async fn create_user(event:LambdaEvent<CreateUserRequest>)->Result<CreateUserResponse, CreateUserError>{      
    let request = event.payload;  

    // TODO: Replace with cryptographic hash
    
    let salt = SaltString::generate(rand::rngs::OsRng);
    // let hash = password_hash::
    let password_hash = Argon2::default().hash_password( request.password.as_bytes(), salt.as_str()).map_err(|_|{
        CreateUserError::HashError
    })?;

    // interact with the database
    let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
    let dbconfig = aws_config::from_env().region(region_provider).load().await;
    let dbclient = Client::new(&dbconfig);
    let dbrequest = dbclient
        .put_item()
        .table_name(TABLE_NAME)
        .item(USERNAME_KEY, AttributeValue::S(request.username))
        .item(PASSWORD_KEY,AttributeValue::S(password_hash.to_string()));
    dbrequest.send().await.map_err(|e| {
        CreateUserError::DatabaseWriteError
    })?;

    Ok(CreateUserResponse)
}
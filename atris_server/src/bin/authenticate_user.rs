use atris_common::authenticate_user::*;
use atris_server::{run_lambda_http, AtrisDBClient};
use argon2::Argon2;

use atris_common::REGION;
use atris_server::{PASSWORD_KEY, TABLE_NAME, USERNAME_KEY};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;

run_lambda_http!(
    |request:Request<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {

    let (_,request) = request.into_parts();

    // let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
    // let db_config = aws_config::from_env().region(region_provider).load().await;
    // let db_client = Client::new(&db_config);
    // let db_request = db_client
    //     .get_item()
    //     .table_name(TABLE_NAME)
    //     .key(USERNAME_KEY, AttributeValue::S(request.username.clone()))
    //     .attributes_to_get(PASSWORD_KEY);
    // let output = db_request.send().await.map_err(|_| AuthenticateUserError::DatabaseRead)?;
    // let item = output.item();

    // Retrieve user from database
    let client = AtrisDBClient::new().await;
    let user = client
        .get_user(request.username.clone())
        .await? //return any database errors
        .ok_or(AuthenticateUserError::UnknownUsername(request.username.clone()))?;  //return user doesn't exist error

    // Confirm password, return any errors
    password_hash::PasswordHash::new(&user.password_hash)
        .map_err(|_| AuthenticateUserError::MissingPassword)?   //check for existing password
        .verify_password(&[&Argon2::default()], request.password_attempt)
        .map_err(|_| AuthenticateUserError::WrongPassword)?;    //check for incorrect password

    // If no errors, then user has been authenticated
    Ok(AuthenticateUserResponse)
});

use argon2::Argon2;
use atris_common::authenticate_user::*;
use atris_server::{run_lambda, PASSWORD_KEY, TABLE_NAME, USERNAME_KEY};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::LambdaEvent;

run_lambda!(|event:LambdaEvent<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {
    let request = event.payload;

    // interact with the database
    let region_provider = RegionProviderChain::first_try("us-west-2").or_default_provider();
    let dbconfig = aws_config::from_env().region(region_provider).load().await;
    let dbclient = Client::new(&dbconfig);
    let dbrequest = dbclient
        .get_item()
        .table_name(TABLE_NAME)
        .key(USERNAME_KEY, AttributeValue::S(request.username.clone()))
        .attributes_to_get(PASSWORD_KEY);
    let output = dbrequest.send().await.map_err(|_| AuthenticateUserError::DatabaseRead)?;
    let item = output.item();

    let user = item.ok_or(AuthenticateUserError::UnknownUsername(request.username))?;

    let actual_salted_and_hashed_password = user.get(PASSWORD_KEY).ok_or(AuthenticateUserError::MissingPassword)?.as_s().map_err(|_|AuthenticateUserError::MissingPassword)?;
    password_hash::PasswordHash::new(actual_salted_and_hashed_password).map_err(|_|{
        AuthenticateUserError::MissingPassword
    })?.verify_password(&[&Argon2::default()], request.password_attempt).map_err(|_|{
        AuthenticateUserError::WrongPassword
    })?;

    Ok(AuthenticateUserResponse)
});

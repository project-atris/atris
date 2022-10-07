use argon2::Argon2;
use atris_common::{authenticate_user::*, REGION};
use atris_server::{run_lambda, PASSWORD_KEY, TABLE_NAME, USERNAME_KEY, AtrisDBClient};
use lambda_runtime::LambdaEvent;

run_lambda!(|event:LambdaEvent<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {

    // Request from the user
    let request = event.payload;

    /* 
    // interact with the database
    let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
    let db_config = aws_config::from_env().region(region_provider).load().await;
    let db_client = Client::new(&db_config);
    let db_request = db_client
        .get_item()
        .table_name(TABLE_NAME)
        .key(USERNAME_KEY, AttributeValue::S(request.username.clone()))
        .attributes_to_get(PASSWORD_KEY);
    let output = db_request.send().await.map_err(|_| AuthenticateUserError::DatabaseRead)?;
    let item = output.item();

    let user = item.ok_or(AuthenticateUserError::UnknownUsername(request.username))?;

    let actual_salted_and_hashed_password = user.get(PASSWORD_KEY).ok_or(AuthenticateUserError::MissingPassword)?.as_s().map_err(|_|AuthenticateUserError::MissingPassword)?;
    password_hash::PasswordHash::new(actual_salted_and_hashed_password).map_err(|_|{
        AuthenticateUserError::MissingPassword
    })?.verify_password(&[&Argon2::default()], request.password_attempt).map_err(|e|{
        AuthenticateUserError::WrongPassword
    })?;
    */

    //  Retrieve user from database
    let client = AtrisDBClient::new().await;
    let user = client
        .get_user(request.username.clone())
        .await
        .map_err(|_| AuthenticateUserError::DatabaseRead)? //return database error
        .ok_or(AuthenticateUserError::UnknownUsername(request.username.clone()))?; //return user doesn't exist error
    Ok(AuthenticateUserResponse)
});

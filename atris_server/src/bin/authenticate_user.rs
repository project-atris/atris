use argon2::Argon2;
use atris_common::{authenticate_user::*, REGION};
use atris_server::{run_lambda, AtrisDBClient, PASSWORD_KEY, TABLE_NAME, USERNAME_KEY};
use lambda_runtime::LambdaEvent;

run_lambda!(|event:LambdaEvent<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {

    // Request from the user
    let request = event.payload;

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

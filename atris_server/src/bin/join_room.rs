use argon2::Argon2;
use atris_common::authenticate_user::*;

use atris_server::{auth_table::AtrisAuthDBClient, run_lambda_http};

run_lambda_http!(
    |request:Request<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {

    let (_,request) = request.into_parts();

    // Retrieve user from database
    let client = AtrisAuthDBClient::new().await;
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
    // Ok(AuthenticateUserResponse)
    todo!()
});

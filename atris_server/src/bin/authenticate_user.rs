use argon2::Argon2;
use atris_common::{authenticate_user::*, CipherKey};

use atris_server::{
    auth_table::AtrisAuthDBClient,
    run_lambda_http,
    session_table::{AtrisSessionDBClient, CreateSessionError},
};

run_lambda_http!(
    |request:Request<AuthenticateUserRequest>|->Result<AuthenticateUserResponse, AuthenticateUserError> {

    let (_,request) = request.into_parts();
    dbg!(0);

    // Retrieve user from database
    let auth_client = AtrisAuthDBClient::new().await;
    let user = dbg!(auth_client
        .get_user(request.username.clone())
        .await? //return any database errors
        .ok_or(AuthenticateUserError::UnknownUsername(request.username.clone())))?;  //return user doesn't exist error
    dbg!(1);

    // Confirm password, return any errors
    password_hash::PasswordHash::new(&user.password_hash)
        .map_err(|_| AuthenticateUserError::MissingPassword)?   //check for existing password
        .verify_password(&[&Argon2::default()], request.password_attempt)
        .map_err(|_| AuthenticateUserError::WrongPassword)?;    //check for incorrect password

    dbg!(2);

    // If no errors, then user has been authenticated, create session
    let session_client = AtrisSessionDBClient::new().await;
    let session_id = CipherKey::generate();
    session_client.create_session(session_id.clone(), request.username, request.initiator).await.map_err(|e|match e {
        CreateSessionError::DuplicateSession(_) => todo!(),
        CreateSessionError::DatabaseWriteError => AuthenticateUserError::DatabaseWrite,
    })?;
    dbg!(3);
    Ok(AuthenticateUserResponse{session_id})
});

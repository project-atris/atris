use argon2::{Argon2, PasswordHasher};
use atris_common::create_user::*;
use atris_server::{run_lambda, AtrisDBClient};
use lambda_runtime::LambdaEvent;
use password_hash::SaltString;

run_lambda!(
    |event: LambdaEvent<CreateUserRequest>| -> Result<CreateUserResponse, CreateUserError> {
        // Request from the user
        let request = event.payload;

        // Generate a salt for the given password
        let salt = SaltString::generate(rand::rngs::OsRng);
        let password_hash = Argon2::default()
            .hash_password(request.password.as_bytes(), salt.as_str())
            .map_err(|_| CreateUserError::HashError)?;

        // Create the new user in the database
        let client = AtrisDBClient::new().await;
        dbg!("Success?");
        client
            .create_user(request.username, password_hash.to_string())
            .await
    }
);

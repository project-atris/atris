use argon2::{Argon2, PasswordHasher};
use atris_common::{create_user::*, REGION};
use atris_server::{PASSWORD_KEY, TABLE_NAME, USERNAME_KEY, run_lambda_http, AtrisDBClient};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{types::SdkError, Client};
use password_hash::SaltString;

run_lambda_http!(
    |request: Request<CreateUserRequest>| -> Result<CreateUserResponse, CreateUserError> {
        let (_,request) = request.into_parts();

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
            .await?;


        Ok(CreateUserResponse)
    }
);

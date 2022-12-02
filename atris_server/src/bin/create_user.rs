use argon2::{Argon2, PasswordHasher};
use atris_common::{create_user::*, Cipher, CipherKey};
use atris_server::{auth_table::AtrisAuthDBClient, run_lambda_http};

use password_hash::SaltString;

run_lambda_http!(
    |request: Request<CreateUserRequest>| -> Result<CreateUserResponse, CreateUserError> {
        let (_, request) = request.into_parts();
        dbg!(&request);

        // Generate a salt for the given password
        let salt = SaltString::generate(rand::rngs::OsRng);
        let password_hash = Argon2::default()
            .hash_password(request.password.as_bytes(), salt.as_str())
            .map_err(|_| CreateUserError::HashError)?;

        // Create the new user in the database
        let client = AtrisAuthDBClient::new().await;
        dbg!("Success?");
        client
            .create_user(request.username, password_hash.to_string())
            .await?;

        Ok(CreateUserResponse)
    }
);

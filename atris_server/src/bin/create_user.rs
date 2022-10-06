use argon2::{Argon2, PasswordHasher};
use atris_common::{create_user::*, REGION};
use atris_server::{run_lambda, PASSWORD_KEY, TABLE_NAME, USERNAME_KEY};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{types::SdkError, Client};
use lambda_runtime::LambdaEvent;
use password_hash::SaltString;

run_lambda!(
    |event: LambdaEvent<CreateUserRequest>| -> Result<CreateUserResponse, CreateUserError> {
        let request = event.payload;

        let salt = SaltString::generate(rand::rngs::OsRng);
        let password_hash = Argon2::default()
            .hash_password(request.password.as_bytes(), salt.as_str())
            .map_err(|_| CreateUserError::HashError)?;

        // interact with the database
        let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
        let dbconfig = aws_config::from_env().region(region_provider).load().await;
        let dbclient = Client::new(&dbconfig);
        let dbrequest = dbclient
            .put_item()
            .condition_expression(format!("attribute_not_exists({})", USERNAME_KEY))
            .table_name(TABLE_NAME)
            .item(USERNAME_KEY, AttributeValue::S(request.username.clone()))
            .item(PASSWORD_KEY, AttributeValue::S(password_hash.to_string()));

        dbrequest.send().await.map_err(|e| {
            if let SdkError::ServiceError { err, .. } = &e {
                if err.is_conditional_check_failed_exception() {
                    return CreateUserError::DuplicateUsername(request.username);
                }
            }
            dbg!(e);
            CreateUserError::DatabaseWriteError
        })?;
        dbg!("Success?");

        Ok(CreateUserResponse)
    }
);

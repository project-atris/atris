use serde::{Deserialize, Serialize};
use std::{fmt::Display, future::Future, pin::Pin};

/// Generates the main function for a lambda which uses the provided function as its handler
#[macro_export]
macro_rules! run_lambda {
    (|$request_name:ident : $request:ty| -> $ret:ty $block:block) => {
        use lambda_runtime::Error as LambdaError;
        use std::convert::Infallible;
        #[tokio::main]
        async fn main() -> Result<(), LambdaError> {
            use lambda_runtime::{self, service_fn};
            async fn internal_handler($request_name: $request) -> $ret {
                $block
            };
            async fn technnicaly_infallible_handler(event: $request) -> Result<$ret, Infallible> {
                let result = internal_handler(event).await;
                Ok(result)
            }
            lambda_runtime::run(service_fn(technnicaly_infallible_handler)).await?;
            Ok(())
        }
    };
}

pub const USERNAME_KEY: &'static str = "username";
pub const PASSWORD_KEY: &'static str = "hashed_salted_password";
pub const SALT_KEY: &'static str = "salt";

pub const TABLE_NAME: &'static str = "atris_auth";
pub const REGION: &'static str = "us-west-2";

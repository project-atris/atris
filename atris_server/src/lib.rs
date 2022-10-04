/// Generates the main function for a lambda which uses the provided function as its handler
#[macro_export]
macro_rules! run_lambda {
    ($handler:expr) => {
        use lambda_runtime::{Error as LambdaError};
        #[tokio::main]
        async fn main() -> Result<(), LambdaError> {
            use lambda_runtime::{self,service_fn};
            let func = service_fn($handler);
            lambda_runtime::run(func).await?;
            Ok(())
        }
    };
}


pub const USERNAME_KEY:&'static str="username";
pub const PASSWORD_KEY:&'static str="hashed_salted_password";
pub const SALT_KEY:&'static str="salt";

pub const TABLE_NAME:&'static str="atris_auth";
pub const REGION: &'static str = "us-west-2";
use lambda_runtime::{service_fn, Error as LambdaError};

mod request_handler;
use request_handler::handler;


// Main function, entry point
#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

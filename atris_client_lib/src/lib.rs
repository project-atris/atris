use atris_common::{
    authenticate_user::{AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserResponse},
    create_user::{CreateUserError, CreateUserRequest, CreateUserResponse}, REGION,
};
use aws_sdk_lambda::{
    error::InvokeError,
    types::{Blob, SdkError},
    Client,
};
use serde::{Deserialize, Serialize};
use aws_config::meta::region::RegionProviderChain;

/// An error resulting from invocing an Atris Lambda function
#[derive(Debug)]
pub enum InvocationError {
    /// A miscellaneous error from the Lambda function
    SdkError(SdkError<InvokeError>),
    /// An error which occoured while trying to serialize the request payload to send to the Lambda function
    SerializationError(serde_json::Error),
    /// An error which occoured while trying to deserialize the response payload recieved from the Lambda function
    DeserializationError(serde_json::Error),
    /// A lambda request which, for some reason, did not return a payload
    NoResponse,
}
impl From<SdkError<InvokeError>> for InvocationError {
    fn from(err: SdkError<InvokeError>) -> Self {
        InvocationError::SdkError(err)
    }
}
/// A [`Result`] resulting from invocing an Atris Lambda function
pub type InvocationResult<T> =  Result<T, InvocationError>;

/// The API of the Atris authentication server
/// This bundles all of the functions necessary for user creation and authentication, as well as initiating the key exchange
/// ```
/// use atris_client_lib::AtrisAuthClient;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     /// Create the client to the authorization server
///     let client = AtrisAuthClient::new().await;
///     // Send the request to the server
///     let user = client.create_user("username", "password-secret-shh").await;
///     Ok(())
/// }
/// ```
pub struct AtrisAuthClient {
    /// The AWS Lambda client that this client will use for API calls
    client: Client,
}

impl AtrisAuthClient {
    /// Create an [`AtrisAuthClient`] from the environment variable configurations
    pub async fn new() -> Self {
        // Set the region to us-west-2 (Oregon) if possible, or fallback on the default
        let region_provider = RegionProviderChain::first_try("us-west-2").or_default_provider();
        // Use this region to configure the SDK
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }
    /// Send the response to create a user on the authentication server
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> InvocationResult<Result<CreateUserResponse, CreateUserError>> {
        // Invoke the 'CreateUser' lambda function, sending it a [`CreateUserRequest`] 
        // derived from the inputs
        invoke_fn(
            &self.client,
            "CreateUser",
            &CreateUserRequest {
                username: username.into(),
                password: password.into(),
            },
        )
        .await
    }

    /// Send the response to authenticate a user on the authentication server
    pub async fn authenticate_user(
        &self,
        username: &str,
        password_attempt: &str,
    ) -> InvocationResult<Result<AuthenticateUserResponse, AuthenticateUserError>> {
        // Invoke the 'AuthenticateeUser' lambda function, sending it a [`AuthenticateUserRequest`] 
        // derived from the inputs
        invoke_fn(
            &self.client,
            "AuthenticateUser",
            &AuthenticateUserRequest {
                username: username.into(),
                password_attempt: password_attempt.into(),
            },
        )
        .await
    }
}

/// Invoke a lambda function with the given input and output types
pub async fn invoke_fn<'f, I: Serialize, T>(
    client:&'f Client,
    lambda_function_name: &'static str,
    payload: &'f I,
) -> InvocationResult<T>
where
    for<'d> T: Deserialize<'d>,
{
    // Write the input payload to json using `serde_json`
    let serialized_payload =
        serde_json::to_string(payload).map_err(InvocationError::SerializationError)?;

    // Invoke the lambda function with the provided name and payload
    let invocation_output = client
        .invoke()
        .function_name(lambda_function_name)
        .payload(Blob::new(serialized_payload))
        .send()
        .await?;

    // Get the payload as a `String`
    let payload_str = invocation_output
        .payload()
        .ok_or(InvocationError::NoResponse)?
        .as_ref()
        .into_iter()
        .map(|c| char::from(*c))
        .collect::<String>();

    // Deserialize the response payload into the desired type
    let deserialized_payload = serde_json::from_str(&payload_str).expect("Parsing error");
    Ok(deserialized_payload)
}
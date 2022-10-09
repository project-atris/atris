use async_trait::async_trait;
use atris_common::{
    authenticate_user::{AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserResponse},
    create_user::{CreateUserError, CreateUserRequest, CreateUserResponse}, REGION,
};
use aws_sdk_lambda::{
    error::InvokeError,
    types::{Blob, SdkError},
    Client, client::fluent_builders::Invoke, output::InvokeOutput,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use aws_config::meta::region::RegionProviderChain;

use crate::{AtrisAuthClient, InvocationResult, InvocationError};

/// The API of the Atris authentication server, implemented using the AWS sdk
/// This bundles all of the functions necessary for user creation and authentication, as well as initiating the key exchange
/// ```
/// use atris_client_lib::{AtrisAuthClient,sdk_auth::AtrisAuthSDK};
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     /// Create the client to the authorization server
///     let client = AtrisAuthSDK::new().await;
///     // Send the request to the server
///     let user = client.create_user("username", "password-secret-shh").await;
///     Ok(())
/// }
/// ```
pub struct AtrisAuthSDK {
    /// The AWS Lambda client that this client will use for API calls
    client: Client,
}

impl AtrisAuthSDK {
    /// Create an [`AtrisAuthSDK`] from the environment variable configurations
    pub async fn new() -> Self {
        // Set the region to us-west-2 (Oregon) if possible, or fallback on the default
        let region_provider = RegionProviderChain::first_try(REGION).or_default_provider();
        // Use this region to configure the SDK
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }
}
#[async_trait]
impl AtrisAuthClient for AtrisAuthSDK{
    type Error = SdkError<InvokeError>;
    type FunctionIdentifier = &'static str;

    type BaseResponse=InvokeOutput;


    const CREATE_USER_FN:Self::FunctionIdentifier = "CreateUser";
    const AUTHENTICATE_USER_FN:Self::FunctionIdentifier = "AuthenticateUser";
    async fn invoke_lambda<'s, P: Serialize+Sync, R:DeserializeOwned>(
        &'s self,
        lambda_function_name: Self::FunctionIdentifier,
        payload: &'s P,
    ) -> InvocationResult<R,Self::Error>{
        // Invoke the lambda function with the provided name and payload
        
        // Write the input payload to json using `serde_json`
        let serialized_payload =
            serde_json::to_string(payload).map_err(InvocationError::SerializationError)?;
        let response = self.client
            .invoke()
            .payload(Blob::new(serialized_payload))
            .function_name(lambda_function_name)
            .send()
            .await?;
        // Get the response as a `String`
        let serialized_response = response
            .payload()
            .ok_or(InvocationError::NoResponse)?
            .as_ref()
            .into_iter()
            .map(|c| char::from(*c))
            .collect::<String>();
        serde_json::from_str(&serialized_response).map_err(InvocationError::DeserializationError)
    }
}


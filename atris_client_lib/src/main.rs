use atris_common::{
    authenticate_user::{AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserResponse},
    create_user::{CreateUserError, CreateUserRequest, CreateUserResponse},
};
use aws_sdk_lambda::{
    error::InvokeError,
    types::{Blob, SdkError},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum InvocationError {
    SdkError(SdkError<InvokeError>),
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
    NoResponse,
}
impl From<SdkError<InvokeError>> for InvocationError {
    fn from(err: SdkError<InvokeError>) -> Self {
        InvocationError::SdkError(err)
    }
}
pub type InvocationResult<T> = Result<T, InvocationError>;

pub struct AtrisClient {
    client: Client,
}
impl AtrisClient {
    pub async fn new() -> Self {
        use aws_config::meta::region::RegionProviderChain;

        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        Self {
            client: Client::new(&config),
        }
    }
    pub async fn invoke_fn<'f, I: Serialize, T>(
        &'f self,
        function_name: &'static str,
        payload: &'f I,
    ) -> InvocationResult<T>
    where
        for<'d> T: Deserialize<'d>,
    {
        let serialized_payload =
            serde_json::to_string(payload).map_err(InvocationError::SerializationError)?;

        let invocation_output = self
            .client
            .invoke()
            .function_name(function_name)
            .payload(Blob::new(serialized_payload))
            .send()
            .await?;

        let payload_str = invocation_output
            .payload()
            .ok_or(InvocationError::NoResponse)?
            .as_ref()
            .into_iter()
            .map(|c| char::from(*c))
            .collect::<String>();
        let deserialized_payload = serde_json::from_str(&payload_str).expect("Parsing error");
        Ok(deserialized_payload)
    }
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> InvocationResult<Result<CreateUserResponse, CreateUserError>> {
        self.invoke_fn(
            "CreateUser",
            &CreateUserRequest {
                username: username.into(),
                password: password.into(),
            },
        )
        .await
    }
    pub async fn authenticate_user(
        &self,
        username: &str,
        password_attempt: &str,
    ) -> InvocationResult<Result<AuthenticateUserResponse, AuthenticateUserError>> {
        self.invoke_fn(
            "AuthenticateUser",
            &AuthenticateUserRequest {
                username: username.into(),
                password_attempt: password_attempt.into(),
            },
        )
        .await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This is where we will setup our HTTP client requests.
    let client = AtrisClient::new().await;
    let user = client.create_user("Abc", "Pbc").await;
    dbg!(&user);
    Ok(())
}

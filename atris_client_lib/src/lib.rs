use atris_common::{create_user::{CreateUserResponse, CreateUserError, CreateUserRequest}, authenticate_user::{AuthenticateUserError, AuthenticateUserResponse, AuthenticateUserRequest}};
use aws_sdk_lambda::{types::SdkError, error::InvokeError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub mod sdk_auth;
pub mod http_auth;

/// An error resulting from invocing an Atris Lambda function
#[derive(Debug)]
pub enum InvocationError<E> {
    /// A miscellaneous error from the Lambda function implementation
    ImplementationError(E),
    /// An error which occoured while trying to serialize the request payload to send to the Lambda function
    SerializationError(serde_json::Error),
    /// An error which occoured while trying to deserialize the response payload recieved from the Lambda function
    DeserializationError(serde_json::Error),
    /// A lambda request which, for some reason, did not return a payload
    NoResponse,
}
impl <E> From<E> for InvocationError<E> {
    fn from(err:E) -> Self {
        InvocationError::ImplementationError(err)
    }
}
/// A [`Result`] resulting from invocing an Atris Lambda function
pub type InvocationResult<R,E> =  Result<R, InvocationError<E>>;


/// The API of the Atris authentication server
/// This bundles all of the functions necessary for user creation and authentication, as well as initiating the key exchange
/// ```
/// use atris_client_lib::{AtrisAuthClient,http_auth::AtrisAuth};
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     /// Create the client to the authorization server
///     let client = AtrisAuth::new()?;
///     // Send the request to the server
///     let user = client.create_user("username", "password-secret-shh").await;
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait AtrisAuthClient {
    /// The information needed for this client to identify a specific function
    type FunctionIdentifier:Send;

    // /// The request the core of the client can actually send
    // type BaseRequest:Send;
    /// The request the core of the client actually recieves
    type BaseResponse:Send;

    /// An error caused by the core of this client
    type Error;

    /// The identifier for the CreateUser Lambda function
    const CREATE_USER_FN:Self::FunctionIdentifier;
    /// The identifier for the AuthenticateUser Lambda function
    const AUTHENTICATE_USER_FN:Self::FunctionIdentifier;

  
    /// Invoke a lambda function with the given input and output types
    async fn invoke_lambda<'s, P: Serialize+Sync, R:DeserializeOwned>(
        &'s self,
        lambda_function_name: Self::FunctionIdentifier,
        payload: &'s P,
    ) -> InvocationResult<R,Self::Error>;

    /// Send the response to create a user on the authentication server
    async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> InvocationResult<Result<CreateUserResponse, CreateUserError>,Self::Error> {
        self.invoke_lambda(Self::CREATE_USER_FN, &CreateUserRequest{
            username: username.into(),
            password: password.into(),
        }).await
    }
    /// Send the response to authenticate a user on the authentication server
    async fn authenticate_user(
        &self,
        username: &str,
        password_attempt: &str,
    ) -> InvocationResult<Result<AuthenticateUserResponse, AuthenticateUserError>,Self::Error>{
        self.invoke_lambda(Self::CREATE_USER_FN, &AuthenticateUserRequest{
            username: username.into(),
            password_attempt: password_attempt.into(),
        }).await
    }
}

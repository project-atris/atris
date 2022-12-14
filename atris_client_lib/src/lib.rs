use std::fmt::{Debug, Display};

use atris_common::{
    authenticate_user::{AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserResponse},
    create_room::{CreateRoomError, CreateRoomRequest, CreateRoomResponse},
    create_user::{CreateUserError, CreateUserRequest, CreateUserResponse},
    join_room::{JoinRoomError, JoinRoomRequest, JoinRoomResponse},
    set_room_responder::{
        SetRoomResponderError, SetRoomResponderRequest, SetRoomResponderResponse,
    },
    CipherKey,
};

use serde::{de::DeserializeOwned, Serialize};

pub use atris_common;

pub mod comms;
pub mod http_auth;
pub mod sdk_auth;

/// An error resulting from invoking an Atris Lambda function
#[derive(Debug)]
pub enum InvocationError<E> {
    /// A miscellaneous error from the Lambda function implementation
    ImplementationError(E),
    /// An error which occurred while trying to serialize the request payload to send to the Lambda function
    SerializationError(serde_json::Error),
    /// An error which occurred while trying to deserialize the response payload received from the Lambda function
    DeserializationError(serde_json::Error),
    /// A lambda request which, for some reason, did not return a payload
    NoResponse,
}
impl<E> From<E> for InvocationError<E> {
    fn from(err: E) -> Self {
        InvocationError::ImplementationError(err)
    }
}
impl<E: Debug> Display for InvocationError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
impl<E: Debug> std::error::Error for InvocationError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
/// A [`Result`] resulting from invoking an Atris Lambda function
pub type InvocationResult<R, E> = Result<R, InvocationError<E>>;

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
    type FunctionIdentifier: Send;

    // /// The request the core of the client can actually send
    // type BaseRequest:Send;
    /// The request the core of the client actually receives
    type BaseResponse: Send;

    /// An error caused by the core of this client
    type Error;

    /// The identifier for the CreateUser Lambda function
    const CREATE_USER_FN: Self::FunctionIdentifier;
    /// The identifier for the AuthenticateUser Lambda function
    const AUTHENTICATE_USER_FN: Self::FunctionIdentifier;
    /// The identifier for the CreateRoom Lambda function
    const CREATE_ROOM_FN: Self::FunctionIdentifier;
    /// The identifier for the SetRoomResponder Lambda function
    const SET_ROOM_RESPONDER_FN: Self::FunctionIdentifier;
    /// The identifier for the JoinRoom Lambda function
    const JOIN_ROOM_FN: Self::FunctionIdentifier;

    /// Invoke a lambda function with the given input and output types
    async fn invoke_lambda<'s, P: Serialize + Sync, R: DeserializeOwned>(
        &'s self,
        lambda_function_name: Self::FunctionIdentifier,
        payload: &'s P,
    ) -> InvocationResult<R, Self::Error>;
    /// Send the response to create a room on the authentication server
    async fn join_room(
        &self,
        session_id: CipherKey,
        room_id: u16,
    ) -> InvocationResult<Result<JoinRoomResponse, JoinRoomError>, Self::Error> {
        self.invoke_lambda(
            Self::JOIN_ROOM_FN,
            &JoinRoomRequest {
                session_id,
                room_id,
            },
        )
        .await
    }

    /// Send the response to create a room on the authentication server
    async fn create_room(
        &self,
        session_id: CipherKey,
        other_user_name: &str,
    ) -> InvocationResult<Result<CreateRoomResponse, CreateRoomError>, Self::Error> {
        self.invoke_lambda(
            Self::CREATE_ROOM_FN,
            &CreateRoomRequest {
                session_id,
                other_user_name: other_user_name.into(),
            },
        )
        .await
    }
    /// Send the response to set the room's responder on the authentication server
    async fn set_room_responder(
        &self,
        room_id: u16,
        session_id: CipherKey,
        other_user_name: &str,
        responder_str: &str,
    ) -> InvocationResult<Result<SetRoomResponderResponse, SetRoomResponderError>, Self::Error>
    {
        self.invoke_lambda(
            Self::SET_ROOM_RESPONDER_FN,
            &SetRoomResponderRequest {
                room_id,
                session_id,
                other_user_name: other_user_name.into(),
                responder_string: responder_str.into(),
            },
        )
        .await
    }
    /// Send the response to create a user on the authentication server
    async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> InvocationResult<Result<CreateUserResponse, CreateUserError>, Self::Error> {
        self.invoke_lambda(
            Self::CREATE_USER_FN,
            &CreateUserRequest {
                username: username.into(),
                password: password.into(),
            },
        )
        .await
    }
    /// Send the response to authenticate a user on the authentication server
    async fn authenticate_user(
        &self,
        username: &str,
        password_attempt: &str,
        initiator: &str,
    ) -> InvocationResult<Result<AuthenticateUserResponse, AuthenticateUserError>, Self::Error>
    {
        self.invoke_lambda(
            Self::AUTHENTICATE_USER_FN,
            &AuthenticateUserRequest {
                username: username.into(),
                password_attempt: password_attempt.into(),
                initiator: initiator.into(),
            },
        )
        .await
    }
}

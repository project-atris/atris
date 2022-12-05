use atris_client_lib::{http_auth::AtrisAuth, comms::{initiator::AtrisInitiator, AtrisConnection}, atris_common::{authenticate_user::{AuthenticateUserError, AuthenticateUserResponse}, create_user::{CreateUserRequest, CreateUserError, CreateUserResponse}, CipherKey, create_room::{CreateRoomResponse, CreateRoomError}, join_room::{JoinRoomError, JoinRoomResponse}, set_room_responder::{SetRoomResponderResponse, SetRoomResponderError}}, AtrisAuthClient, InvocationError};
use iced_native::Debug;

pub struct AtrisClient {
    server_client: AtrisAuth,
    initiator:AtrisInitiator,
}
impl std::fmt::Debug for AtrisClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"AtrisClient")
    }
}

type AtrisError = reqwest::Error;

#[derive(Debug,Clone)]
pub enum ClientError {
    ConnectionError,
    InitiatorError,
    EncodingError,
    RequestError(String),
    InvocationError(String),
    AuthenticateUserError(AuthenticateUserError),
    CreateUserError(CreateUserError),
    CreateRoomError(CreateRoomError),
    JoinRoomError(JoinRoomError),
    SetRoomResponderError(SetRoomResponderError),
}
impl From<AtrisError> for ClientError {
    fn from(err: AtrisError) -> Self {
        Self::RequestError("Error making a request".into())
    }
}
impl From<InvocationError<AtrisError>> for ClientError {
    fn from(err: InvocationError<AtrisError>) -> Self {
        Self::InvocationError("Error invoking lambda".into())
    }
}
impl From<AuthenticateUserError> for ClientError {
    fn from(err: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(err)
    }
}
impl From<CreateUserError> for ClientError {
    fn from(err: CreateUserError) -> Self {
        Self::CreateUserError(err)
    }
}

impl From<CreateRoomError> for ClientError {
    fn from(err: CreateRoomError) -> Self {
        Self::CreateRoomError(err)
    }
}
impl From<JoinRoomError> for ClientError {
    fn from(err: JoinRoomError) -> Self {
        Self::JoinRoomError(err)
    }
}
impl From<SetRoomResponderError> for ClientError {
    fn from(err: SetRoomResponderError) -> Self {
        Self::SetRoomResponderError(err)
    }
}


impl AtrisClient {
    pub async fn new()->Result<Self,ClientError> {
        Ok(Self {
            initiator: AtrisInitiator::new(AtrisConnection::new().await.map_err(|_|ClientError::ConnectionError)?).await.map_err(|_|ClientError::InitiatorError)?,
            server_client: AtrisAuth::new()?
        })
    }
    pub async fn create_user(&self, user: &str,pass: &str,) -> Result<CreateUserResponse,ClientError> {
        self.server_client.create_user(user, pass).await?.map_err(|e|e.into())
    }

    pub async fn create_room(&self, session_id: CipherKey,other_user: &str) -> Result<CreateRoomResponse,ClientError> {
        self.server_client.create_room(session_id, other_user).await?.map_err(|e|e.into())
    }
    pub async fn join_room(&self, session_id: CipherKey,room_id: u16) -> Result<JoinRoomResponse,ClientError> {
        self.server_client.join_room(session_id, room_id).await?.map_err(|e|e.into())
    }
    pub async fn login(&self, user: &str,pass: &str,) -> Result<AuthenticateUserResponse,ClientError> {
        let initiator_string = self.initiator.encoded_local_description().map_err(|_|ClientError::EncodingError)?;
        println!("Authenticating");
        let auth = self.server_client
            .authenticate_user(user, pass, &initiator_string)
            .await??;
        println!("Authenticated");
        Ok(auth)
    }
    pub async fn set_room_responder(
        &self,
        room_id: u16,
        session_id: CipherKey,
        other_user_name: String,
        responder_str:String
    )->Result<SetRoomResponderResponse,ClientError> {
        self.server_client.set_room_responder(room_id, session_id, &other_user_name, &responder_str).await?.map_err(|e|e.into())
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;

use crate::CipherKey;

#[derive(Deserialize, Serialize, Debug)]
pub struct SetRoomResponderRequest {
    pub session_id: CipherKey,
    pub room_id: u16,
    pub other_user_name: String,
    pub responder_string: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SetRoomResponderResponse {
    pub room_symmetric_key: CipherKey,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum SetRoomResponderError {
    BincodeError,
    EncryptionError,
    InvalidRoomId(u16),
    DatabaseWriteError,
    NotRoomCreator(String),
    InvalidSessionId(CipherKey),
    NoSessionForUser(String),
}
impl Display for SetRoomResponderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoomId(room_id) => {
                write!(f, "RoomID '{}' is not found", room_id)
            }
            Self::BincodeError => {
                write!(f, "Error serializing room data")
            }
            Self::EncryptionError => {
                write!(f, "Error encrypting room data")
            }
            Self::DatabaseWriteError => {
                write!(f, "Failed to write to the database")
            }
            Self::InvalidSessionId(s) => {
                write!(f, "Session {s:?} does not exist.")
            }
            Self::NotRoomCreator(u) => {
                write!(
                    f,
                    "User {u:?} did not create this room and, as such, cannot set the responder."
                )
            }
            Self::NoSessionForUser(u) => {
                write!(f, "No session found for user {u}.")
            }
        }
    }
}
impl Error for SetRoomResponderError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

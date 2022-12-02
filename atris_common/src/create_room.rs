use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;

use crate::CipherKey;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRoomRequest {
    pub session_id: CipherKey,
    pub other_user_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRoomResponse{
    pub room_id: u16,
    pub initiator_string:String
}
#[derive(Deserialize, Serialize, Debug)]
pub enum CreateRoomError {
    BincodeError,
    EncryptionError,
    DuplicateRoomId(u16),
    DatabaseWriteError,
    InvalidSessionId(CipherKey),
    NoSessionForUser(String)
}
impl Display for CreateRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRoomId(room_id) => {
                write!(f, "RoomID '{}' is already taken", room_id)
            },
            Self::BincodeError => {
                write!(f, "Error serializing room data")
            },
            Self::EncryptionError => {
                write!(f, "Error encrypting room data")
            },
            Self::DatabaseWriteError => {
                write!(f, "Failed to write to the database")
            },
            Self::InvalidSessionId(s) => {
                write!(f, "Session {s:?} does not exist.")
            },
            Self::NoSessionForUser(u) => {
                write!(f, "No session found for user {u}.")
            },
        }
    }
}
impl Error for CreateRoomError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

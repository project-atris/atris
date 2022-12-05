use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{CipherKey, Encrypted, RoomData};
#[derive(Deserialize, Serialize, Debug)]
pub struct JoinRoomRequest {
    pub session_id: CipherKey,
    pub room_id: u16,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JoinRoomResponse {
    pub room_data: Encrypted<RoomData>,
}
#[derive(Deserialize, Serialize, Debug,Clone)]
pub enum JoinRoomError {
    InvalidSessionId(CipherKey),
    NonexistentRoomId(u16),
    IncompleteRoom,
    DatabaseReadError,
}
impl Display for JoinRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompleteRoom => {
                write!(f, "Not all keys present for room.")
            }
            Self::InvalidSessionId(s) => {
                write!(f, "Session {s:?} does not exist.")
            }
            Self::NonexistentRoomId(room_id) => {
                write!(f, "RoomID '{}' does not exist", room_id)
            }
            Self::DatabaseReadError => {
                write!(f, "Failed to read from the database")
            }
        }
    }
}
impl Error for JoinRoomError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

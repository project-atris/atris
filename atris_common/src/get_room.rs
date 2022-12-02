use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{CipherKey, Encrypted, RoomData};
#[derive(Deserialize, Serialize, Debug)]
pub struct GetRoomRequest{
    pub session_id: CipherKey,
    pub room_id: u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRoomResponse{
    pub room_data: Encrypted<RoomData>,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum GetRoomError {
    NonexistentRoomId(u16),
    DatabaseReadError,
}
impl Display for GetRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonexistentRoomId(room_id) => {
                write!(f, "RoomID '{}' does not exist", room_id)
            }
            Self::DatabaseReadError => {
                write!(f, "Failed to read from the database")
            }
        }
    }
}
impl Error for GetRoomError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;

use crate::CipherKey;

/// A request to authenticate a user on the atris auth server. The server will respond with a Result<AuthenticateUserResponse,AuthenticateUserError>
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthenticateUserRequest {
    /// The username attempted to log in
    pub username: String,
    /// The password the user attempted to enter, which is transferred unhashed and unsalted
    /// - Note: This is common practice as long as the connection is encrypted
    pub password_attempt: String,
    /// The initiator WebRTC string, which we pass to other users
    pub initiator: String,
}

/// A successful response to a [`AuthenticateUserRequest`] on the atris auth server.
///  - For error response, see [`AuthenticateUserError`]
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthenticateUserResponse {
    pub session_id: CipherKey,
}

/// A response to a [`AuthenticateUserRequest`] on the atris auth server. For success response, see [`AuthenticateUserResponse`]
#[derive(Deserialize, Serialize, Debug)]
pub enum AuthenticateUserError {
    /// The username attempted was not found in the database
    UnknownUsername(String),
    /// The stored user record does not have a password
    MissingPassword,
    /// The password attempted did not match the stored password
    WrongPassword,
    /// Failed to read the user record from the database
    DatabaseRead,
    /// Failed to write to the databse
    DatabaseWrite,
}
impl Display for AuthenticateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUsername(username) => {
                write!(f, "Username {} is not registered", username)
            }
            Self::MissingPassword => {
                write!(f, "The database did not have a password for this user")
            }
            Self::WrongPassword => {
                write!(f, "The password provided does not match")
            }
            Self::DatabaseRead => {
                write!(
                    f,
                    "Failed to read user authentication details from the database"
                )
            }
            Self::DatabaseWrite => {
                write!(f, "Failed to write network information to the database")
            }
        }
    }
}
impl Error for AuthenticateUserError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

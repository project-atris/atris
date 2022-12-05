use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;

/// A request to create a user on the atris auth server. The server will respond with a Result<CreateUserResponse,CreateUserError>
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUserRequest {
    /// The username attempted to create
    pub username: String,
    /// The password to assign to the created user
    pub password: String,
}

/// A successful response to a [`CreateUserRequest`] on the atris auth server.
///  - For error response, see [`CreateUserError`]
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUserResponse; // TODO: See if anything else needs to be returned to user

/// A response to a [`CreateUserRequest`] on the atris auth server. For success response, see [`CreateUserResponse`]
#[derive(Deserialize, Serialize, Debug,Clone)]
pub enum CreateUserError {
    /// The username requested already exists in the server
    DuplicateUsername(String),
    /// The hashing function failed to hash the provided password
    HashError,
    /// The write of the user's data failed
    DatabaseWriteError,
}
impl Display for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateUsername(username) => {
                write!(f, "Username '{}' is already taken", username)
            }
            Self::HashError => {
                write!(f, "Error creating password hash")
            }
            Self::DatabaseWriteError => {
                write!(f, "Failed to write to the database")
            }
        }
    }
}
impl Error for CreateUserError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

use std::fmt::{Display};
use std::error::Error;
use serde::{Serialize,Deserialize};

/// A request to create a user on the atris auth server. The server will respond with a Result<CreateUserResponse,CreateUserError>
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

/// A successful response to a [`CreateUserRequest`] on the atris auth server.
///  - For error response, see [`CreateUserError`]
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUserResponse; // TODO: See if anything else needs to be returned to user


/// A response to a [`CreateUserRequest`] on the atris auth server. For success response, see [`CreateUserResponse`]
#[derive(Deserialize, Serialize,Debug)]
pub enum CreateUserError {
    DuplicateUsername(String),
    HashError,
    DatabaseWriteError
}
impl Display for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateUsername(username)=>{
                write!(f, "Username '{}' is already taken",username)
            },
            Self::HashError=>{
                write!(f, "Error creating password hash")  
            }
            Self::DatabaseWriteError => {
                write!(f,"Failed to write to the database")
            }
        }
    }
}
impl Error for CreateUserError{
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}


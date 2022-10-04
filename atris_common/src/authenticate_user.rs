use std::fmt::{Display};
use std::error::Error;
use serde::{Serialize,Deserialize};

/// A request to authenticate a user on the atris auth server. The server will respond with a Result<AuthenticateUserResponse,AuthenticateUserError>
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthenticateUserRequest {
    pub username: String,
    pub attempted_password: String,
}

/// A successful response to a [`AuthenticateUserRequest`] on the atris auth server.
///  - For error response, see [`AuthenticateUserError`]
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthenticateUserResponse; // TODO: Add some sort of authentication ticket



/// A response to a [`AuthenticateUserRequest`] on the atris auth server. For success response, see [`AuthenticateUserResponse`]
#[derive(Deserialize, Serialize,Debug)]
pub enum AuthenticateUserError {
    UnknownUsername(String),

    MissingPassword,

    WrongPassword,
    
    DatabaseRead,
    DatabaseWrite
}
impl Display for AuthenticateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUsername(username)=>{
                write!(f, "Username {} is not registered",username)
            },
            Self::MissingPassword => {
                write!(f,"The database did not have a password for this user")
            },
            Self::WrongPassword=>{
                write!(f, "The password provided does not match")
            },
            Self::DatabaseRead => {
                write!(f,"Failed to read user authentiacation details from to the database")
            },
            Self::DatabaseWrite => {
                write!(f,"Failed to write network information to the database")
            }
        }
    }
}
impl Error for AuthenticateUserError{
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}


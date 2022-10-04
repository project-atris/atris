use std::fmt::Display;

use serde::{Serialize, Deserialize};

pub mod authenticate_user;
pub mod create_user;

// Error enum
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AtrisError {
    DuplicateUsername(String),
    DatabasePutError,
    DatabaseGetError,
    ConnectionError,
}

impl Display for AtrisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtrisError::DuplicateUsername(username) => write!(f,"Username {} was already taken",username),
            AtrisError::DatabasePutError => write!(f,"Ran into a problem putting item to database."),
            AtrisError::DatabaseGetError => write!(f,"Ran into a problem getting item from database."),
            AtrisError::ConnectionError => write!(f,"Ran into a problem connecting to the database."),
        }
    }
}




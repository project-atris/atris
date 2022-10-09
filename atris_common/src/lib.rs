use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod authenticate_user;
pub mod create_user;

// Error enum
#[derive(Debug,PartialEq, Eq)]
pub enum AtrisError {
    SerdeError(String)
}
impl Display for AtrisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerdeError(err)=>{
                write!(f,"Serde Error: {}",err)
            }
        }
    }
}
/// The region for Atris on Lambda
pub const REGION: &'static str = "us-west-2";

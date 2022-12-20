use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum AppError {
    GenericError(String),
    NetworkAndEndpoint(String),
    EndpointsBelowThreshold(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::GenericError(e) => write!(f, "Error: {}", e),
            AppError::NetworkAndEndpoint(e) => write!(f, "Error: {}", e),
            AppError::EndpointsBelowThreshold(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for AppError {}

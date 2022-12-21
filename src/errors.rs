use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub enum AppError {
    GenericError(String),
    NetworkAndEndpoint(String),
    EndpointsBelowThreshold(String),
    NoEndpointsFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::GenericError(e) => write!(f, "Error: {}", e),
            AppError::NetworkAndEndpoint(e) => write!(f, "Error: {}", e),
            AppError::EndpointsBelowThreshold(e) => write!(f, "Error: {}", e),
            AppError::NoEndpointsFound(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for AppError {
    fn description(&self) -> &str {
        match self {
            AppError::GenericError(s) => s,
            AppError::NetworkAndEndpoint(s) => s,
            AppError::EndpointsBelowThreshold(s) => s,
            AppError::NoEndpointsFound(s) => s,
        }
    }
}

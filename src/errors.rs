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
        write!(f, "Error: {}", match self {
            AppError::GenericError(e) |
            AppError::NetworkAndEndpoint(e) |
            AppError::EndpointsBelowThreshold(e) |
            AppError::NoEndpointsFound(e) => e,
        })
    }
}

impl std::error::Error for AppError {
    fn description(&self) -> &str {
        match self {
            AppError::GenericError(s) |
            AppError::NetworkAndEndpoint(s) |
            AppError::EndpointsBelowThreshold(s) |
            AppError::NoEndpointsFound(s) => s,
        }
    }
}

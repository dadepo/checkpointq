use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum AppError {
    #[error("Error: {0}")]
    GenericError(String),
    #[error("Error: {0}")]
    EndpointsBelowThreshold(String),
    #[error("Error: {0}")]
    NoEndpointsFound(String),
}

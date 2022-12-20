
#[derive(Debug)]
pub enum AppError {
    GenericError(String),
    NetworkAndEndpoint(String),
    EndpointsBelowThreshold(String)
}
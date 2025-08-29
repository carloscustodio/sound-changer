use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AudioError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Parsing error: {0}")]
    ParseError(String),
    
    #[error("Windows API error: {0}")]
    WindowsApiError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AudioError {
    fn from(error: std::io::Error) -> Self {
        AudioError::CommandFailed(error.to_string())
    }
}

impl From<serde_json::Error> for AudioError {
    fn from(error: serde_json::Error) -> Self {
        AudioError::ParseError(error.to_string())
    }
}

// Custom Result type for our application
pub type AudioResult<T> = Result<T, AudioError>;

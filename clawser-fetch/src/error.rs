use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClawserError {
    #[error("Session initialization failed: {0}")]
    InitFailed(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Request timed out")]
    Timeout,
    #[error("Null pointer returned from native library")]
    NullPointer,
    #[error("Invalid UTF-8 in response body")]
    InvalidUtf8,
}

pub type Result<T> = std::result::Result<T, ClawserError>;

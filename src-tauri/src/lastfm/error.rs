use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Last.fm API error: {0}")]
    Api(String),
    #[error("Callback server failed to start: {0}")]
    CallbackServer(String),
    #[error("Authentication timed out")]
    Timeout,
    #[error("Callback validation failed")]
    InvalidCallback,
    #[error("Secure storage error: {0}")]
    SecureStorage(String),
    #[error("Invalid API credentials")]
    InvalidCredentials,
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

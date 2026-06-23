use thiserror::Error;

/// KLA error types
#[derive(Error, Debug)]
pub enum KlaError {
    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("PTY error: {0}")]
    Pty(#[from] portable_pty::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Recording error: {0}")]
    Recording(String),

    #[error("Timeout waiting for: {0}")]
    Timeout(String),

    #[error("Session closed")]
    SessionClosed,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias for KLA operations
pub type Result<T> = std::result::Result<T, KlaError>;

impl KlaError {
    pub fn terminal<S: Into<String>>(msg: S) -> Self {
        Self::Terminal(msg.into())
    }

    pub fn recording<S: Into<String>>(msg: S) -> Self {
        Self::Recording(msg.into())
    }

    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    pub fn invalid_state<S: Into<String>>(msg: S) -> Self {
        Self::InvalidState(msg.into())
    }

    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::Parse(msg.into())
    }
}
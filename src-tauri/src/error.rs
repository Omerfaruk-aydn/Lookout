#[derive(thiserror::Error, Debug, serde::Serialize, Clone)]
pub enum LookoutError {
    #[error("Webull Desktop is not running. Please start Webull Desktop first.")]
    WebullNotRunning,
    #[error("Screen capture failed: {0}")]
    CaptureFailed(String),
    #[error("Vision API error: {0}")]
    VisionApiError(String),
    #[error("Market data unavailable: {0}")]
    DataProviderError(String),
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    #[error("LLM response schema validation failed: {0}")]
    SchemaValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Sidecar communication error: {0}")]
    SidecarError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Timeout: {0}")]
    TimeoutError(String),
}

impl From<rusqlite::Error> for LookoutError {
    fn from(e: rusqlite::Error) -> Self {
        LookoutError::DatabaseError(e.to_string())
    }
}

impl From<std::io::Error> for LookoutError {
    fn from(e: std::io::Error) -> Self {
        LookoutError::SidecarError(e.to_string())
    }
}

impl From<serde_json::Error> for LookoutError {
    fn from(e: serde_json::Error) -> Self {
        LookoutError::SchemaValidationError(e.to_string())
    }
}

impl From<reqwest::Error> for LookoutError {
    fn from(e: reqwest::Error) -> Self {
        LookoutError::DataProviderError(e.to_string())
    }
}

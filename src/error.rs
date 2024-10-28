use thiserror::Error;

#[derive(Error, Debug)]
pub enum LimelightError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Connection timeout")]
    TimeoutError,
    
    #[error("Client not running")]
    NotRunning,
}
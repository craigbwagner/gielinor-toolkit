use thiserror::Error;

#[derive(Debug, Error)]
pub enum Rs3ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    Parse(String),

    #[error("Player not found: {0}")]
    PlayerNotFound(String),

    #[error("Player profile is private: {0}")]
    PrivateProfile(String),
}

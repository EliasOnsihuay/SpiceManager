use thiserror::Error;

pub type Result<T> = std::result::Result<T, SpiceError>;

#[derive(Debug, Error)]
pub enum SpiceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("version parse error: {0}")]
    Semver(#[from] semver::Error),
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("missing release asset for this platform")]
    MissingReleaseAsset,
    #[error("{0}")]
    Message(String),
}

impl serde::Serialize for SpiceError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

use thiserror::Error;

pub type SuzuResult<T> = Result<T, SuzuError>;

#[derive(Debug, Error)]
pub enum SuzuError {
    #[error("script error: {0}")]
    Script(String),
    #[error("asset error: {0}")]
    Asset(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("save error: {0}")]
    Save(String),
    #[error("platform error: {0}")]
    Platform(String),
}

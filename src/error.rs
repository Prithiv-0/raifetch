use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RaifetchError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Render error: {0}")]
    Render(String),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, RaifetchError>;

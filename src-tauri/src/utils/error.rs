use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

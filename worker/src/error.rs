use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Google Slides API error: {0}")]
    GoogleSlides(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

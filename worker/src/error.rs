use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Google Slides API error: {0}")]
    GoogleSlides(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Session expired")]
    SessionExpired,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<AppError> for worker::Error {
    fn from(err: AppError) -> Self {
        worker::Error::from(err.to_string())
    }
}

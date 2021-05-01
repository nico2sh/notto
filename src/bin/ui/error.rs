use notto::errors::NottoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NottoViewError {
    #[error("Error loading screen {message}")]
    LoadViewError { message: String },

    #[error("notto error - {source}")]
    NottoError {
        #[from]
        source: NottoError
    },
}
use std::{env, io};

use thiserror::Error;
#[derive(Error, Debug)]
pub enum NottoError {
    #[error("context {context} not found")]
    ContextNotFound { context: String },
    #[error("context error from environment variable - {source}")]
    ContextError { 
        #[from]
        source: env::VarError
     },
    #[error("problem with notto's home directory - {source}")]
    ConfigDirectory {
        #[from]
        source: io::Error
    },
    #[error("problem deserializing yaml content - {source}")]
    ReadingFile {
        #[from]
        source: serde_yaml::Error
    },
    #[error("home directory not found")]
    HomeDirectoryNotFound,
    #[error("{message}")]
    LoadConfigError { message: String },
    
    #[error("Note {note_name} alerady exists.")]
    NoteExists { note_name: String },
}
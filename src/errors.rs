use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Template render error")]
    Reader(#[from] askama::Error),

    #[error("IO error")]
    IO(#[from] io::Error),

    #[error("Hyper error")]
    Hyper(#[from] hyper::http::Error),

    #[error("Build error: {message:?}")]
    Build { message: String },

    #[error("Command not found")]
    CommandNotFound,

    #[error("Error")]
    Generic {message: String}
}

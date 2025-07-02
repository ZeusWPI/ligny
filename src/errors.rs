use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Template render error")]
    Reader(#[from] askama::Error),
}

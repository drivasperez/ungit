use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitterError {
    #[error("Could not find package")]
    NotFound,
    #[error("Network error")]
    NetworkError(http_types::Error),
    #[error("IO error")]
    IOError(std::io::Error),
}

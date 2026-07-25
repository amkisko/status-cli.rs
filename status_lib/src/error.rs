//! Library error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("catalog error: {0}")]
    Catalog(String),
    #[error("usage: {0}")]
    Usage(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("response size limit exceeded: {actual} bytes (max {limit})")]
    ResponseTooLarge { actual: usize, limit: usize },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

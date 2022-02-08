use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("State {0} not found")]
    MissingState(String),
    #[error("Word {0} contains non-space whitespace")]
    InvalidWhitespace(String),
}

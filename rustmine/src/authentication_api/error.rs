use reqwest::StatusCode;

/// Errors returned by the authentication API client
#[derive(Debug)]
pub enum Error {
    /// The request could not be sent or the response could not be parsed
    Transport(reqwest::Error),
    /// Input failed validation (HTTP 400), with the server's message
    Validation(String),
    /// Authentication or authorization failed (HTTP 401)
    Unauthorized,
    /// The requested player does not exist (HTTP 404)
    NotFound,
    /// A player with the same name already exists (HTTP 409)
    Conflict(String),
    /// Any other unexpected status code
    Unexpected(StatusCode),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Transport(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Transport(err) => write!(f, "transport error: {err}"),
            Error::Validation(msg) => write!(f, "validation error: {msg}"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotFound => write!(f, "not found"),
            Error::Conflict(msg) => write!(f, "conflict: {msg}"),
            Error::Unexpected(status) => write!(f, "unexpected status: {status}"),
        }
    }
}

impl std::error::Error for Error {}

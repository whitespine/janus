use std::fmt::Display;

/// All the errors that can occur
#[derive(Debug)]
pub enum FoundryClientError {
    URLError(url::ParseError),
    JoinError(reqwest::Error),
    SocketError(rust_socketio::Error),
    NoUserError(String),
}

impl Display for FoundryClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoundryClientError::URLError(error) =>
                write!(f, "Provided URL is malformed: {}", error),
            FoundryClientError::JoinError(error) =>
                write!(f, "Unable to join server: {}", error),
            FoundryClientError::SocketError(error) =>
                write!(f, "Unable to establish socket connection: {}", error),
            FoundryClientError::NoUserError(name) =>
                write!(f, "No user named {} found", name),
        }
    }
}

impl std::error::Error for FoundryClientError {}

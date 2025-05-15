use std::fmt::{Display};

/// All the errors that can occur
#[derive(Debug)]
pub enum FoundryClientError {
    FailedInit(String),
    URLError(url::ParseError),
    JoinError(reqwest::Error),
    SocketError(rust_socketio::Error),
    NoUserError(String),
    MalformedData {path: String, value: serde_json::Value}
}
impl Display for FoundryClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoundryClientError::FailedInit(error) =>
                write!(f, "Manual initialization failed {}", error),
            FoundryClientError::URLError(error) =>
                write!(f, "Provided URL is malformed: {}", error),
            FoundryClientError::JoinError(error) =>
                write!(f, "Unable to join server: {}", error),
            FoundryClientError::SocketError(error) =>
                write!(f, "Unable to establish socket connection: {}", error),
            FoundryClientError::NoUserError(name) =>
                write!(f, "No user named {} found", name),
            FoundryClientError::MalformedData { path, value } => {
                write!(f, "Malformed data found at {} within data {}", path, value)
            }
        }
    }
}

impl std::error::Error for FoundryClientError {}

/// Specific errors with discord commands
#[derive(Debug)]
pub enum CommandError {
    /// Associating a character failed
    CharacterNotFound(String),
    /// No character was associated
    MissingAssocChar,
    /// An associated character was provided but could not be resolved / is not valid for this operation
    InvalidAssocChar,
    /// A provided stat or attribute
    InvalidAttribute(String),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::CharacterNotFound(name) =>
                write!(f, "No actor named '{}' found. Be careful about case sensitivity", name),
            CommandError::MissingAssocChar =>
                write!(f, "You must first /assoc with a character in the foundry world"),
            CommandError::InvalidAssocChar =>
                write!(f, "Your currently associated character is invalid/deleted. /assoc with a character in the foundry world"),
            CommandError::InvalidAttribute(name) =>
                write!(f, "The stat {} cannot be rolled", name),
        }
    }
}

impl std::error::Error for CommandError {}

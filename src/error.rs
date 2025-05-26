use thiserror::Error;

/// All the errors that can occur
#[derive(Error, Debug)]
pub enum FoundryClientError {
    #[error("Manual initialization of Foundry Client failed")]
    FailedInit(String),
    #[error("Provided host {0} was unable to be parsed as a url")]
    URLError(#[from] url::ParseError),
    #[error("HTTP Connection error: {0}")]
    JoinError(#[from] reqwest::Error),
    #[error("Socket Connection error: {0}")]
    SocketError(#[from] rust_socketio::Error),
    #[error("Failed to login as user: {0}")]
    NoUserError(String),
    #[error("Failed to parse initial userdata: {path} within {value} is not formatted as expected - perhaps a version incompatibility?")]
    MalformedData {path: String, value: serde_json::Value}
}

/// Specific errors with discord commands
#[derive(Error, Debug)]
pub enum CommandError {
    /// Associating a character failed
    #[error("No actor named '{0}' found. Be careful about case sensitivity")]
    CharacterNotFound(String),
    /// No character was associated
    #[error("You must first /assoc with a character name in the foundry world")]
    MissingAssocChar,
    /// An associated character was provided but could not be resolved / is not valid for this operation
    #[error("Your currently associated character is invalid/deleted. /assoc with a new character in the foundry world")]
    InvalidAssocChar,
    /// A provided stat or attribute
    #[error("The attribute you tried to roll ({0}) was not recognized")]
    InvalidAttribute(String),
}
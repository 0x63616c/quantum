//! Small building blocks for repository maintenance commands.

pub mod check;
pub mod cmd;
pub mod macos;
pub mod version;
pub mod workspace;

mod checks;
mod tidy;

use std::path::PathBuf;

/// Errors shared by repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Restoration failed; this takes precedence over an interruption exit code.
    #[error("{0}")]
    Cleanup(String),
    /// A child command could not complete successfully.
    #[error(transparent)]
    Command(#[from] cmd::Error),
    /// A file could not be read or written.
    #[error("{}: {source}", path.display())]
    Io {
        /// File involved in the operation.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
    /// Structured input could not be decoded.
    #[error("{context}: {source}")]
    Json {
        /// Input being decoded.
        context: String,
        /// JSON decoding failure.
        source: serde_json::Error,
    },
    /// Input or repository state does not satisfy an operation's contract.
    #[error("{0}")]
    Invalid(String),
}

/// Result of a repository operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Run the original repository hygiene command, including its success message.
pub fn tidy() -> std::result::Result<(), String> {
    tidy::run()
}

#[cfg(test)]
mod testing;

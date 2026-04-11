use std::{fmt::Display, path::PathBuf};

use cake::cmd::Response;

/// Describes all possible errors that can
/// occur while executing a CLI command.
#[derive(Debug)]
pub enum CliError {
    Config(cake::config::ConfigError),
    TokenPool(anyhow::Error),
    Client(crate::client::ClientError),

    Serverside(cake::errors::CmdError),
    UnknownServerside,

    RequestFailed,

    Ping,
    Checksum(cake::checksum::ChecksumError),

    DirCreation(PathBuf),
    PushCopy,
    PullCopy,

    BadPath(PathBuf),
    BadWarp(String),
    AnonBadWarp,
    WarpExists(String),
    UnknownAlias(String),
    AliasExists(String),
    UnknownToken(String),
    TokenExists(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(e) => write!(f, "config: {e}"),
            Self::TokenPool(e) => write!(f, "token pool: {e}"),
            Self::Client(e) => write!(f, "client: {e}"),
            Self::Serverside(e) => write!(f, "server-side: {e}"),
            Self::UnknownServerside => write!(f, "unknown server-side error"),
            Self::RequestFailed => write!(f, "request failed"),
            Self::Ping => write!(f, "ping failed"),
            Self::Checksum(e) => write!(f, "checksum: {e}"),
            Self::DirCreation(path) => {
                write!(f, "failed to create directory: {}", path.to_string_lossy())
            }
            Self::PushCopy => write!(f, "failed to write a file into the stream"),
            Self::PullCopy => write!(f, "failed to read a file from the stream"),
            Self::BadPath(path) => write!(f, "bad path: {}", path.to_string_lossy()),
            Self::BadWarp(id) => write!(f, "warp '{id}' not found"),
            Self::AnonBadWarp => write!(f, "bad warp"),
            Self::WarpExists(name) => write!(f, "warp '{name}' already exists"),
            Self::UnknownAlias(name) => write!(f, "alias '{name}' not found"),
            Self::AliasExists(name) => write!(f, "alias '{name}' already exists"),
            Self::UnknownToken(name) => write!(f, "token with owner '{name}' not found"),
            Self::TokenExists(name) => write!(f, "token with owner '{name}' already exists"),
        }
    }
}

/// Converts `Response` struct to the `CliError`
/// with `Serverside` or `UnknownServerside` variant.
pub fn response_error(r: Response) -> CliError {
    match r {
        Response::Error(e) => CliError::Serverside(e),
        _ => CliError::UnknownServerside,
    }
}

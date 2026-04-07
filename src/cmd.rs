use crate::checksum::Checksum;
use serde::{Deserialize, Serialize};

/// Defines a set of possible server commands.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Command {
    /// Represents a request to the warp's files checksums.
    /// The argument is the warp name.
    ChecksumRequest(String),

    /// Represents a response to Checksums command.
    ChecksumResponse(Vec<Checksum>),
}

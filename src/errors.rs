use crate::proto::ProtoError;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

/// Describes all possible errors that can
/// occur while executing an external command.
#[derive(Serialize, Deserialize, Debug)]
pub enum CmdError {
    Checksum(crate::checksum::ChecksumError),

    FrameWrite,
    FrameRead,

    DirCreation(PathBuf),
    PushCopy,
    PullCopy,
    FileSkip,

    BadWarp(String),

    Proto(ProtoError),
}

impl Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Checksum(e) => write!(f, "checksum: {e}"),
            Self::FrameWrite => write!(f, "failed to write a frame"),
            Self::FrameRead => write!(f, "failed to read a frame"),
            Self::DirCreation(path) => write!(
                f,
                "failed to create directory at {}",
                path.to_string_lossy()
            ),
            Self::PushCopy => write!(f, "failed to read a file into the stream while pushing"),
            Self::PullCopy => write!(f, "failed to write a file from the stream while pulling"),
            Self::FileSkip => write!(f, "failed to skip a file in the stream"),
            Self::BadWarp(id) => write!(f, "bad warp: {id}"),
            Self::Proto(e) => write!(f, "protocol: {e}"),
        }
    }
}

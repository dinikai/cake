use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a configuration file.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub warps: Vec<Warp>,
}

/// Represents a warp zone, an identifier (name)
/// associated with a directory path.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Warp {
    pub name: String,
    pub path: PathBuf,
}

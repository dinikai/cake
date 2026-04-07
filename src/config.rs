use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents a configuration file.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    /// The server binding address.
    pub bind: String,

    /// Collection of warp zones.
    pub warps: Vec<Warp>,

    /// Collection of the known peers
    /// and their aliases.
    pub aliases: Vec<Alias>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io,
    Yaml,
    Home,
}

impl Config {
    /// Constructs a new empty config.
    pub fn new() -> Self {
        Self {
            bind: String::from("0.0.0.0:39746"),
            warps: Vec::new(),
            aliases: Vec::new(),
        }
    }

    /// Tries to read a config from the file.
    /// Writes and returns a default one if not exists.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let config_str = fs::read_to_string(path);

        let config = match config_str {
            Ok(str) => serde_yaml::from_str(&str).or(Err(ConfigError::Yaml))?,
            Err(e) => {
                match e.kind() {
                    // Write a default config if it isn't exists yet.
                    std::io::ErrorKind::NotFound => {
                        let config = Self::new();

                        let config_string = serde_yaml::to_string(&config).unwrap();
                        fs::write(path, config_string).or(Err(ConfigError::Io))?;

                        config
                    }
                    _ => return Err(ConfigError::Io),
                }
            }
        };

        Ok(config)
    }

    /// Tries to read a config from the default path.
    /// Writes and returns a default one if not exists.
    /// - "$HOME/.config/cake.yaml"
    pub fn from_default() -> Result<Self, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::Home)?;
        let path = home.join(".config/cake.yaml");

        Self::from_file(&path)
    }
}

/// Represents a warp zone, an identifier (name)
/// associated with a directory path.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Warp {
    pub name: String,
    pub path: PathBuf,
}

/// Represents an alias for the peer.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Alias {
    pub name: String,
    pub host: String,
}

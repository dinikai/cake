use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Represents a configuration file.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    /// The server binding address.
    pub bind: String,

    /// Should Cake ask user's confirmation
    /// when doing dangerous operations?
    pub confirm: bool,

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
            confirm: false,
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
        let path = Self::get_default_path().ok_or(ConfigError::Home)?;

        Self::from_file(&path)
    }

    /// Tries to write the config into the file.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = serde_yaml::to_string(self).or(Err(ConfigError::Yaml))?;

        fs::write(path, &text).or(Err(ConfigError::Io))?;

        Ok(())
    }

    /// Tries to write the config into the deafult file.
    pub fn save_default(&self) -> Result<(), ConfigError> {
        let path = Self::get_default_path().ok_or(ConfigError::Home)?;

        self.save(&path)
    }

    /// Tries to retrieve the default config location.
    fn get_default_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config/cake.yaml"))
    }

    pub fn get_warp(&self, name: &str) -> Option<&Warp> {
        self.warps.iter().find(|w| w.name == name)
    }

    pub fn get_warp_by_path(&self, path: &Path) -> Option<&Warp> {
        let path = path.canonicalize().ok()?;

        self.warps.iter().find(|w| {
            let Ok(p) = w.path.canonicalize() else {
                return false;
            };

            path == p
        })
    }

    /// Retrieves a warp either by name or by current directory.
    pub fn get_warp_name_or_dir<'a>(&'a self, name: &Option<String>) -> Result<&'a Warp, String> {
        match name {
            Some(name) => Ok(self.get_warp(&name).ok_or("warp not found")?),
            None => {
                let current_dir =
                    env::current_dir().or(Err("unable to retrieve current directory"))?;

                Ok(self
                    .get_warp_by_path(&current_dir)
                    .ok_or("warp not found")?)
            }
        }
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

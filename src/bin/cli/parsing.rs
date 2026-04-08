mod checksum;
mod ping;
mod warp;

use checksum::*;
use ping::*;
use warp::*;

use cake::{
    checksum::Checksum,
    config::{Config, ConfigError},
};
use clap::{Parser, Subcommand};
use std::path::Path;

pub type CliResult = Result<(), String>;

trait Executable {
    fn execute(self, config: &mut Config) -> CliResult;
}

#[derive(Parser, Debug)]
#[command(name = "cake", about = "Simple CLI file synchronization tool.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Warp-related subcommand")]
    Warp(WarpArgs),

    #[command(about = "Calculate checksum of a file or files in a directory OR of a warp")]
    Checksum(ChecksumArgs),

    #[command(about = "Send a ping to the peer")]
    Ping(PingArgs),
}

impl Executable for Command {
    fn execute(self, config: &mut Config) -> CliResult {
        match self {
            Command::Checksum(args) => args.execute(config),
            Command::Warp(args) => args.execute(config),
            Command::Ping(args) => args.execute(config),
        }
    }
}

fn save_config(config: &Config) -> CliResult {
    match config.save_default() {
        Ok(_) => Ok(()),
        Err(e) => match e {
            ConfigError::Io => Err("unable to write cake.yaml".to_string()),
            ConfigError::Home => Err("unable to retrieve home directory".to_string()),
            ConfigError::Yaml => Err("idk how this even happened".to_string()),
        },
    }
}

pub fn run() {
    let cli = Cli::parse();

    let mut config = match Config::from_default() {
        Ok(c) => c,
        Err(e) => {
            match e {
                ConfigError::Io => println!("Error: unable to read cake.yaml"),
                ConfigError::Home => println!("Error: unable to retrieve home directory"),
                ConfigError::Yaml => println!("Error: bad configuration file"),
            };
            return;
        }
    };

    if let Err(e) = cli.command.execute(&mut config) {
        println!("Error: {e}");
    };
}

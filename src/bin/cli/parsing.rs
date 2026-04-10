mod checksum;
mod diff;
pub mod errors;
mod ping;
mod pushpull;
mod warp;

use checksum::*;
use diff::*;
use ping::*;
use pushpull::*;
use warp::*;

use cake::{
    checksum::Checksum,
    config::{Config, ConfigError},
};
use clap::{Parser, Subcommand};
use errors::CliError;

pub type CliResult = Result<(), CliError>;

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
    #[command(about = "Warp-related commands")]
    Warp(WarpArgs),

    #[command(about = "Calculate checksum of a file or files in a directory OR of a warp")]
    Checksum(ChecksumArgs),

    #[command(about = "Send a ping to the peer")]
    Ping(PingArgs),

    #[command(about = "Push a local warp to the peer")]
    Push(PushArgs),

    #[command(about = "Pull a remote warp from the peer")]
    Pull(PullArgs),

    #[command(about = "Print differences between the local and the remote warp")]
    Diff(DiffArgs),
}

impl Executable for Command {
    fn execute(self, config: &mut Config) -> CliResult {
        match self {
            Command::Checksum(args) => args.execute(config),
            Command::Warp(args) => args.execute(config),
            Command::Ping(args) => args.execute(config),
            Command::Push(args) => args.execute(config),
            Command::Pull(args) => args.execute(config),
            Command::Diff(args) => args.execute(config),
        }
    }
}

fn save_config(config: &Config) -> CliResult {
    match config.save_default() {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::Config(e)),
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
        println!("\x1b[1;31mError:\n\x1b[3;22m  {e}\x1b[0m");
    };
}

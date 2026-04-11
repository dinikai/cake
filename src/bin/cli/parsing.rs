mod alias;
mod checksum;
mod diff;
pub mod errors;
mod ping;
mod pushpull;
mod warp;

use alias::*;
use checksum::*;
use diff::*;
use ping::*;
use pushpull::*;
use warp::*;

use cake::{checksum::Checksum, config::Config};
use clap::{Parser, Subcommand};
use errors::CliError;

use crate::ui;

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
    #[command(about = "Push a local warp to the peer")]
    Push(PushArgs),

    #[command(about = "Pull a remote warp from the peer")]
    Pull(PullArgs),

    #[command(about = "Print differences between the local and the remote warp")]
    Diff(DiffArgs),

    #[command(about = "Warps maganement commands")]
    Warp(WarpArgs),

    #[command(about = "Aliases maganement commands")]
    Alias(AliasArgs),

    #[command(about = "Calculate checksum of a file or files in a directory OR of a warp")]
    Checksum(ChecksumArgs),

    #[command(about = "Send a ping to the peer")]
    Ping(PingArgs),
}

impl Executable for Command {
    fn execute(self, config: &mut Config) -> CliResult {
        match self {
            Command::Push(args) => args.execute(config),
            Command::Pull(args) => args.execute(config),
            Command::Diff(args) => args.execute(config),
            Command::Warp(args) => args.execute(config),
            Command::Alias(args) => args.execute(config),
            Command::Checksum(args) => args.execute(config),
            Command::Ping(args) => args.execute(config),
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
            ui::error!("config: {e}");
            return;
        }
    };

    if let Err(e) = cli.command.execute(&mut config) {
        ui::error!("{e}");
    };
}

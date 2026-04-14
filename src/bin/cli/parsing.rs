mod alias;
mod checksum;
mod diff;
pub mod errors;
mod ping;
mod pushpull;
mod token;
mod warp;

use alias::*;
use checksum::*;
use diff::*;
use ping::*;
use pushpull::*;
use token::*;
use warp::*;

use cake::{checksum::Checksum, config::Config, token_pool::AuthTokenPool};
use clap::{Parser, Subcommand};
use errors::CliError;

use crate::ui;

pub type CliResult = Result<(), CliError>;

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

    #[command(about = "Daemon's auth token management commands")]
    AuthToken(AuthTokenArgs),

    #[command(about = "Calculate checksum of a file or files in a directory OR of a warp")]
    Checksum(ChecksumArgs),

    #[command(about = "Send a ping to the peer")]
    Ping(PingArgs),
}

impl Command {
    async fn execute(self, config: &mut Config, token_pool: &mut AuthTokenPool) -> CliResult {
        match self {
            Command::Push(args) => args.execute(config).await,
            Command::Pull(args) => args.execute(config).await,
            Command::Diff(args) => args.execute(config).await,
            Command::Warp(args) => args.execute(config).await,
            Command::Alias(args) => args.execute(config).await,
            Command::AuthToken(args) => args.execute(config, token_pool).await,
            Command::Checksum(args) => args.execute(config).await,
            Command::Ping(args) => args.execute(config).await,
        }
    }
}

async fn save_config(config: &Config) -> CliResult {
    match config.save_default().await {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::Config(e)),
    }
}

fn save_token_pool(pool: &AuthTokenPool) -> CliResult {
    match pool.save_default() {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::TokenPool(e)),
    }
}

pub async fn run() {
    let cli = Cli::parse();

    let mut config = match Config::from_default().await {
        Ok(c) => c,
        Err(e) => {
            ui::error!("config: {e}");
            return;
        }
    };

    let mut token_pool = match AuthTokenPool::from_default() {
        Ok(p) => p,
        Err(e) => {
            ui::error!("token pool: {e}");
            return;
        }
    };

    if let Err(e) = cli.command.execute(&mut config, &mut token_pool).await {
        ui::error!("{e}");
    };
}

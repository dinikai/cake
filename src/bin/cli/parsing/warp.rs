use super::*;
use cake::config::{Config, Warp};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct WarpArgs {
    #[command(subcommand)]
    pub command: WarpCommand,
}

impl Executable for WarpArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        self.command.execute(config)
    }
}

#[derive(Subcommand, Debug)]
pub enum WarpCommand {
    #[command(about = "Show all warps")]
    List,

    #[command(about = "Add new warp")]
    Add(WarpAddArgs),

    #[command(about = "Remove existing warp")]
    Remove(WarpRemoveArgs),
}

impl Executable for WarpCommand {
    fn execute(self, config: &mut Config) -> CliResult {
        match self {
            WarpCommand::List => {
                for warp in &config.warps {
                    println!("* {}: {}", warp.name, warp.path.to_string_lossy());
                }
                Ok(())
            }
            WarpCommand::Add(args) => args.execute(config),
            WarpCommand::Remove(args) => args.execute(config),
        }
    }
}

#[derive(Args, Debug)]
pub struct WarpAddArgs {
    #[arg(help = "Name of the warp")]
    pub name: String,

    #[arg(help = "Path of the warp")]
    pub path: PathBuf,
}

impl Executable for WarpAddArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        if config.warps.iter().any(|w| w.name == self.name) {
            return Err(CliError::WarpExists(self.name));
        }

        config.warps.push(Warp {
            name: self.name.clone(),
            // Reassemble path to get rid of trailing /
            path: self
                .path
                .canonicalize()
                .or(Err(CliError::BadPath(self.path)))?,
        });

        save_config(config)
    }
}

#[derive(Args, Debug)]
pub struct WarpRemoveArgs {
    #[arg(help = "Name of the warp")]
    pub name: String,
}

impl Executable for WarpRemoveArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let old_length = config.warps.len();

        config.warps.retain(|w| w.name != self.name);

        if old_length == config.warps.len() {
            return Err(CliError::BadWarp(self.name));
        }

        save_config(config)
    }
}

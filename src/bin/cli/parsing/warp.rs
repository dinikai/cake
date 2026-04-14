use super::*;
use cake::config::{Config, Warp};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct WarpArgs {
    #[command(subcommand)]
    pub command: WarpCommand,
}

impl WarpArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        self.command.execute(config).await
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

impl WarpCommand {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        match self {
            WarpCommand::List => {
                for warp in &config.warps {
                    ui::list!("{}: {}", warp.name, warp.path.to_string_lossy());
                }
                Ok(())
            }
            WarpCommand::Add(args) => args.execute(config).await,
            WarpCommand::Remove(args) => args.execute(config).await,
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

impl WarpAddArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
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
        let warp = config.warps.last().unwrap();

        save_config(config).await?;

        ui::work!("Name: {}", warp.name);
        ui::work!("Path: {}", warp.path.to_string_lossy());

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct WarpRemoveArgs {
    #[arg(help = "Name of the warp")]
    pub name: String,
}

impl WarpRemoveArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        let old_length = config.warps.len();

        config.warps.retain(|w| w.name != self.name);

        if old_length == config.warps.len() {
            return Err(CliError::BadWarp(self.name));
        }

        save_config(config).await
    }
}

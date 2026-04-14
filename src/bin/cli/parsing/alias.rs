use super::*;
use cake::config::{Alias, Config};
use clap::{Args, Subcommand};
use uuid::Uuid;

#[derive(Args, Debug)]
pub struct AliasArgs {
    #[command(subcommand)]
    pub command: AliasCommand,
}

impl AliasArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        self.command.execute(config).await
    }
}

#[derive(Subcommand, Debug)]
pub enum AliasCommand {
    #[command(about = "Show all aliases")]
    List,

    #[command(about = "Add new alias")]
    Add(AliasAddArgs),

    #[command(about = "Remove existing alias")]
    Remove(AliasRemoveArgs),
}

impl AliasCommand {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        match self {
            AliasCommand::List => {
                for alias in &config.aliases {
                    ui::list!("{}: {}", alias.name, alias.host);
                }
                Ok(())
            }
            AliasCommand::Add(args) => args.execute(config).await,
            AliasCommand::Remove(args) => args.execute(config).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct AliasAddArgs {
    #[arg(help = "Name of the alias")]
    pub name: String,

    #[arg(help = "Peer's IP endpoint (with port)")]
    pub address: String,

    #[arg(help = "An authentication token given by the daemon")]
    pub auth_token: Uuid,
}

impl AliasAddArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        if config.aliases.iter().any(|a| a.name == self.name) {
            return Err(CliError::AliasExists(self.name));
        }

        config.aliases.push(Alias {
            name: self.name.clone(),
            host: self.address,
            auth_token: self.auth_token,
        });
        let alias = config.aliases.last().unwrap();

        save_config(config).await?;

        ui::work!("Name:       {}", alias.name);
        ui::work!("Host:       {}", alias.host);
        ui::work!("Auth token: {}", alias.auth_token);

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AliasRemoveArgs {
    #[arg(help = "Name of the alias")]
    pub name: String,
}

impl AliasRemoveArgs {
    pub async fn execute(self, config: &mut Config) -> CliResult {
        let old_length = config.aliases.len();

        config.aliases.retain(|a| a.name != self.name);

        if old_length == config.aliases.len() {
            return Err(CliError::UnknownAlias(self.name));
        }

        save_config(config).await
    }
}

use super::*;
use cake::{
    auth::AuthToken,
    token_pool::{AuthTokenPool, HashedToken},
};
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct AuthTokenArgs {
    #[command(subcommand)]
    pub command: AuthTokenCommand,
}

impl AuthTokenArgs {
    pub async fn execute(self, config: &Config, pool: &mut AuthTokenPool) -> CliResult {
        self.command.execute(config, pool).await
    }
}

#[derive(Subcommand, Debug)]
pub enum AuthTokenCommand {
    #[command(about = "List all tokens' owners")]
    List,

    #[command(about = "Generate new unique token")]
    Create(AuthTokenCreateArgs),

    #[command(about = "Revert an existing token")]
    Revert(AuthTokenRemoveArgs),
}

impl AuthTokenCommand {
    pub async fn execute(self, config: &Config, pool: &mut AuthTokenPool) -> CliResult {
        match self {
            AuthTokenCommand::List => {
                for token in &pool.tokens {
                    ui::list!("{}", &token.owner);
                }

                if pool.tokens.len() > 0 {
                    ui::result!("Token hashes are hidden");
                } else {
                    ui::result!("No tokens added yet");
                }
                Ok(())
            }
            AuthTokenCommand::Create(args) => args.execute(pool).await,
            AuthTokenCommand::Revert(args) => args.execute(config, pool).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct AuthTokenCreateArgs {
    #[arg(help = "Owner of the token")]
    pub owner: String,
}

impl AuthTokenCreateArgs {
    pub async fn execute(self, pool: &mut AuthTokenPool) -> CliResult {
        if pool.tokens.iter().any(|w| w.owner == self.owner) {
            return Err(CliError::TokenExists(self.owner));
        }

        let token = AuthToken::new();
        pool.tokens
            .push(HashedToken::from_token(&token, &self.owner));

        save_token_pool(pool)?;

        ui::work!("Owner: {}", self.owner);
        ui::work!("This token will be hashed and \x1b[4mcannot be retrieved later\x1b[24m");
        ui::work!("Please copy it now");

        ui::result!("{}", token.uuid);

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AuthTokenRemoveArgs {
    #[arg(help = "Owner of the token")]
    pub owner: String,
}

impl AuthTokenRemoveArgs {
    pub async fn execute(self, config: &Config, pool: &mut AuthTokenPool) -> CliResult {
        if !pool.tokens.iter().any(|t| t.owner == self.owner) {
            return Err(CliError::UnknownToken(self.owner));
        }

        if config.confirm
            && !ui::confirm(
                "Are you sure you want to remove this token \x1b[4mcompletely\x1b[4m?",
                false,
                true,
            )
        {
            return Ok(());
        }

        pool.tokens.retain(|w| w.owner != self.owner);

        save_token_pool(pool)
    }
}

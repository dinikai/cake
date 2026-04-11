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
    pub fn execute(self, pool: &mut AuthTokenPool) -> CliResult {
        self.command.execute(pool)
    }
}

#[derive(Subcommand, Debug)]
pub enum AuthTokenCommand {
    #[command(about = "List all tokens' owners")]
    List,

    #[command(about = "Generate new unique token")]
    Create(AuthTokenCreateArgs),

    #[command(about = "Remove existing token")]
    Remove(AuthTokenRemoveArgs),
}

impl AuthTokenCommand {
    pub fn execute(self, pool: &mut AuthTokenPool) -> CliResult {
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
            AuthTokenCommand::Create(args) => args.execute(pool),
            AuthTokenCommand::Remove(args) => args.execute(pool),
        }
    }
}

#[derive(Args, Debug)]
pub struct AuthTokenCreateArgs {
    #[arg(help = "Owner of the token")]
    pub owner: String,
}

impl AuthTokenCreateArgs {
    pub fn execute(self, pool: &mut AuthTokenPool) -> CliResult {
        if pool.tokens.iter().any(|w| w.owner == self.owner) {
            return Err(CliError::TokenExists(self.owner));
        }

        let token = AuthToken::new();
        pool.tokens
            .push(HashedToken::from_token(&token, &self.owner));

        save_token_pool(pool)?;

        ui::work!("Owner: {}", self.owner);
        ui::work!("You are seeing this token for the first and for the \x1b[4mlast\x1b[0m time");
        ui::work!("Token will be saved as its hash");
        ui::work!("Please consider to store it somewhere");

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
    pub fn execute(self, pool: &mut AuthTokenPool) -> CliResult {
        let old_length = pool.tokens.len();

        pool.tokens.retain(|w| w.owner != self.owner);

        if old_length == pool.tokens.len() {
            return Err(CliError::UnknownToken(self.owner));
        }

        save_token_pool(pool)
    }
}

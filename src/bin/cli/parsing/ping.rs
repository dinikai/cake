use crate::{client::Client, parsing::errors::response_error};

use super::*;
use cake::cmd::{Request, Response};
use clap::Args;

#[derive(Args, Debug)]
pub struct PingArgs {
    #[arg(help = "Alias of the peer to ping")]
    pub alias: String,
}

impl Executable for PingArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let response = Client::new_alias(&self.alias, config)
            .map_err(CliError::Client)?
            .request(&Request::Ping)
            .or(Err(CliError::RequestFailed))?;

        let Response::Pong = response else {
            return Err(response_error(response));
        };

        ui::result_success!("\x1b[1mSuccess!");
        Ok(())
    }
}

use crate::client;

use super::*;
use cake::cmd::{Request, Response};
use clap::Args;

#[derive(Args, Debug)]
pub struct PingArgs {
    #[arg(help = "Alias of the peer to ping.")]
    pub alias: String,
}

impl Executable for PingArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let response =
            client::request_alias(&self.alias, &Request::Ping, config).or(Err("ping failed"))?;

        match response {
            Response::Error(e) => Err(format!("server: {e}")),
            _ => {
                println!("{} pongs back!", &self.alias);
                Ok(())
            }
        }
    }
}

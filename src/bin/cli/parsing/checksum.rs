use crate::{client::Client, parsing::errors::response_error};

use super::*;
use cake::{
    cmd::{Request, Response},
    config::Config,
};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ChecksumArgs {
    #[arg(
        default_value = ".",
        help = "Path to the file or directory OR the warp name"
    )]
    pub dest: String,

    #[arg(
        short,
        long,
        help = "Ask a peer for checksums. <DEST> is warp name then"
    )]
    pub peer: Option<String>,

    #[arg(
        short,
        long,
        help = "Use warp name instead of physical path (local checksum only)"
    )]
    pub warp: bool,
}

impl Executable for ChecksumArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        if let Some(peer) = self.peer {
            return remote_checksum(&peer, &self.dest, config);
        }

        local_checksum(&self.dest, self.warp, config)
    }
}

fn remote_checksum(peer: &str, warp: &str, config: &Config) -> CliResult {
    let request = Request::Checksum {
        warp: warp.to_string(),
    };

    let response = Client::new_alias(&peer, config)
        .map_err(CliError::Client)?
        .request(&request)
        .or(Err(CliError::RequestFailed))?;

    let Response::Checksum { sums } = response else {
        return Err(response_error(response));
    };

    for c in sums.iter() {
        println!("{c}");
    }
    return Ok(());
}

fn local_checksum(dest: &str, is_warp: bool, config: &Config) -> CliResult {
    if is_warp {
        let warp = config
            .get_warp(dest)
            .ok_or(CliError::BadWarp(dest.to_string()))?;

        let sums = match Checksum::of_dir(&warp.path) {
            Ok(sums) => sums,
            Err(e) => return Err(CliError::Checksum(e)),
        };

        print_sums(&sums);
        return Ok(());
    }

    let dest = PathBuf::from(dest);

    if dest.is_file() {
        let checksum = match Checksum::of_file(&dest) {
            Ok(sum) => sum,
            Err(e) => return Err(CliError::Checksum(e)),
        };

        println!("{checksum}");

        return Ok(());
    }

    if dest.is_dir() {
        let checksums = match Checksum::of_dir(&dest) {
            Ok(sum) => sum,
            Err(e) => return Err(CliError::Checksum(e)),
        };

        print_sums(&checksums);

        return Ok(());
    }

    // Destination path doesn't exist.
    return Err(CliError::BadPath(dest));
}

fn print_sums(sums: &[Checksum]) {
    for c in sums.iter() {
        println!("{c}");
    }
}

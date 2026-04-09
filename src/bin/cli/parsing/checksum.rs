use crate::client::Client;

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
        .request(&request)
        .or(Err("checksum failed"))?;

    match response {
        Response::Checksum { sums } => {
            for c in sums.iter() {
                println!("{c}");
            }
            return Ok(());
        }
        Response::Error(e) => return Err(format!("server: {e}")),
        _ => return Ok(()),
    }
}

fn local_checksum(dest: &str, is_warp: bool, config: &Config) -> CliResult {
    if is_warp {
        let warp = config.get_warp(dest).ok_or("warp not found")?;
        let sums = Checksum::of_dir(&warp.path).ok_or("failed to calculate checksum")?;

        print_sums(&sums);
        return Ok(());
    }

    let dest = PathBuf::from(dest);

    if dest.is_file() {
        let Some(checksum) = Checksum::of_file(&dest) else {
            return Err("unable to open file".to_string());
        };
        println!("{checksum}");
        return Ok(());
    }

    if dest.is_dir() {
        let Some(checksums) = Checksum::of_dir(&dest) else {
            return Err("unable to open directory".to_string());
        };
        print_sums(&checksums);
        return Ok(());
    }
    // Destination path doesn't exist.
    return Err("path doesn't exist".to_string());
}

fn print_sums(sums: &[Checksum]) {
    for c in sums.iter() {
        println!("{c}");
    }
}

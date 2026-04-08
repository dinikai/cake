use crate::client;

use super::*;
use cake::{
    cmd::{Request, Response},
    config::Config,
};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ChecksumArgs {
    #[arg(help = "Path to the file or directory OR the warp name")]
    pub dest: PathBuf,

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
            let request = Request::Checksum {
                warp: self.dest.to_str().unwrap().to_string(),
            };

            let response =
                client::request_alias(&peer, &request, config).or(Err("checksum failed"))?;

            match response {
                Response::Error(e) => return Err(format!("server: {e}")),
                Response::Checksum { sums } => {
                    for c in sums.iter() {
                        println!("{c}");
                    }
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }

        if !self.warp {
            if self.dest.is_file() {
                let Some(checksum) = Checksum::of_file(&self.dest) else {
                    return Err("unable to open file".to_string());
                };

                println!("{checksum}");
            } else if self.dest.is_dir() {
                execute_checksum_dir(&self.dest);
            } else {
                // Destination path doesn't exist.
                return Err("path doesn't exist".to_string());
            }
        } else {
            let Some(warp) = config.warps.iter().find(|w| w.name == self.dest) else {
                return Err("specified warp doesn't exist".to_string());
            };

            execute_checksum_dir(&warp.path);
        }

        Ok(())
    }
}

fn execute_checksum_dir(path: &Path) {
    let Some(checksums) = Checksum::of_dir(path) else {
        println!("Error: unable to open directory");
        return;
    };

    for c in checksums.iter().flatten() {
        println!("{c}");
    }
}

use crate::client::Client;

use super::*;
use ansi_term::Color;
use cake::{
    cmd::{Request, Response},
    config::Config,
};
use clap::Args;
use std::{collections::HashMap, path::Path};

#[derive(Args, Debug)]
pub struct DiffArgs {
    #[arg(help = "Peer alias")]
    pub peer: String,

    #[arg(help = "Remote warp name")]
    pub warp: Option<String>,
}

impl Executable for DiffArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let warp = config.get_warp_name_or_dir(&self.warp)?;

        // Request remote checksums to compare with local ones.
        let request = Request::Checksum {
            warp: warp.name.clone(),
        };

        let Response::Checksum { sums } = Client::new_alias(&self.peer, config)
            .request(&request)
            .unwrap()
        else {
            return Err("failed to get remote checksums".to_string());
        };

        let local_sums = Checksum::of_dir_relative(&warp.path, &warp.path)
            .ok_or("failed to get local checksums")?;

        let diff = diff_checksums(&local_sums, &sums);

        if diff.created.len() == 0 && diff.modified.len() == 0 && diff.deleted.len() == 0 {
            println!("Local and remote warps are similar");
        } else {
            print_sums("Created: ", Color::Green, &diff.created);
            print_sums("Modified:", Color::Yellow, &diff.modified);
            print_sums("Missing: ", Color::Red, &diff.deleted);
        }

        Ok(())
    }
}

fn print_sums(prefix: &str, color: Color, sums: &[&Checksum]) {
    for c in sums {
        println!(
            " {}",
            color.paint(format!("{} {}", prefix, c.path.to_str().unwrap()))
        );
    }
}

struct Diff<'a> {
    created: Vec<&'a Checksum>,
    modified: Vec<&'a Checksum>,
    deleted: Vec<&'a Checksum>,
}

fn diff_checksums<'a>(local: &'a [Checksum], remote: &'a [Checksum]) -> Diff<'a> {
    let local_map: HashMap<&Path, &Checksum> = local
        .iter()
        .map(|value| (value.path.as_path(), value))
        .collect();

    let remote_map: HashMap<&Path, &Checksum> = remote
        .iter()
        .map(|value| (value.path.as_path(), value))
        .collect();

    let mut created = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    for (path, local_value) in &local_map {
        match remote_map.get(path) {
            None => created.push(*local_value),
            Some(remote_value) => {
                if local_value.sum != remote_value.sum {
                    modified.push(*local_value);
                }
            }
        }
    }

    for (path, remote_value) in &remote_map {
        if !local_map.contains_key(path) {
            deleted.push(*remote_value);
        }
    }

    Diff {
        created,
        modified,
        deleted,
    }
}

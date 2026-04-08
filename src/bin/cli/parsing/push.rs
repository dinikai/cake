use std::{
    env, fs,
    io::{self, BufReader},
};

use crate::client;

use super::*;
use cake::{
    cmd::{self, Request, Response},
    config::Config,
};
use clap::Args;
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct PushArgs {
    #[arg(help = "Peer alias")]
    pub peer: String,

    #[arg(help = "Remote warp name")]
    pub warp: String,
}

impl Executable for PushArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let current_dir = env::current_dir().or(Err("unable to retrieve current directory"))?;
        let warp = config
            .get_warp_by_path(&current_dir)
            .ok_or("warp not found")?;

        let files: Vec<cmd::File> = WalkDir::new(current_dir)
            .into_iter()
            .filter_map(|f| f.ok())
            .map(|f| cmd::File {
                path: f.path().to_path_buf(),
                size: fs::metadata(f.path()).unwrap().len(),
            })
            .collect();

        let request = Request::Push {
            warp: self.warp,
            files: files.clone(),
        };

        // Send each file in the warp.
        let response = client::request_alias_do(
            &self.peer,
            &request,
            |stream| {
                for file in &files {
                    let path = warp.path.join(&file.path);

                    println!("{}", path.to_str().unwrap());

                    let Ok(file_handle) = fs::File::open(&path) else {
                        return;
                    };

                    let mut reader = BufReader::new(file_handle);
                    io::copy(&mut reader, stream).unwrap();
                }
            },
            config,
        )
        .unwrap();

        let Response::Push { files } = response else {
            return Err("error".to_string());
        };

        println!("Succesfully pushed and wrote {} files", files);

        Ok(())
    }
}

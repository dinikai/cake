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
    pub warp: Option<String>,
}

impl Executable for PushArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let warp = match self.warp {
            Some(name) => config.get_warp(&name).ok_or("warp not found")?,
            None => {
                let current_dir =
                    env::current_dir().or(Err("unable to retrieve current directory"))?;

                config
                    .get_warp_by_path(&current_dir)
                    .ok_or("warp not found")?
            }
        };

        let files: Vec<cmd::File> = WalkDir::new(&warp.path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|f| cmd::File {
                path: f.path().to_path_buf(),
                size: fs::metadata(f.path()).unwrap().len(),
            })
            .collect();

        let request = Request::Push {
            warp: warp.name.clone(),
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

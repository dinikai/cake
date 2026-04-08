use std::{
    env, fs,
    io::{self, BufReader, BufWriter, Read},
};

use crate::client::Client;

use super::*;
use cake::{
    cmd::{Request, Response},
    config::{Config, Warp},
    proto,
};
use clap::Args;

/// Retrieves a warp either by name or by current directory.
fn get_warp<'a>(name: &Option<String>, config: &'a Config) -> Result<&'a Warp, String> {
    match name {
        Some(name) => Ok(config.get_warp(&name).ok_or("warp not found")?),
        None => {
            let current_dir = env::current_dir().or(Err("unable to retrieve current directory"))?;

            Ok(config
                .get_warp_by_path(&current_dir)
                .ok_or("warp not found")?)
        }
    }
}

#[derive(Args, Debug)]
pub struct PushArgs {
    #[arg(help = "Peer alias")]
    pub peer: String,

    #[arg(help = "Remote warp name")]
    pub warp: Option<String>,
}

impl Executable for PushArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let warp = get_warp(&self.warp, config)?;

        // Request remote checksums to compare with local ones.
        let request = Request::Checksum {
            warp: warp.name.clone(),
        };

        println!("Waiting for remote checksums...");

        let Response::Checksum { sums } = Client::new_alias(&self.peer, config)
            .request(&request)
            .unwrap()
        else {
            return Err("failed to get checksums".to_string());
        };

        // Exclude locally and remotely equal files.
        let (files, skipped) = Checksum::remain_unique(&warp.path, &sums);

        let request = Request::Push {
            warp: warp.name.clone(),
            files: files.clone(),
        };

        // Send each file in the warp into the stream.
        let response = Client::new_alias(&self.peer, config)
            .request_do(&request, |stream| {
                for file in &files {
                    let path = warp.path.join(&file.path);

                    let Ok(file_handle) = fs::File::open(&path) else {
                        println!("Skipping {} due to error", &file.path.to_str().unwrap());
                        return;
                    };

                    let mut reader = BufReader::new(file_handle);
                    io::copy(&mut reader, stream).unwrap();
                }
            })
            .or(Err("failed to make request"))?;

        let Response::Push { files } = response else {
            return Err("error".to_string());
        };

        println!("{} files were pushed", files);
        println!("{} files were skipped (are equal)", skipped);

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct PullArgs {
    #[arg(help = "Peer alias")]
    pub peer: String,

    #[arg(help = "Remote warp name")]
    pub warp: Option<String>,
}

impl Executable for PullArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let warp = get_warp(&self.warp, config)?;

        // Calculate local checksums
        let sums = Checksum::of_dir_relative(&warp.path, &warp.path).ok_or("warp not found")?;

        // Form & send a request.
        let request = Request::Pull {
            warp: warp.name.clone(),
            sums,
        };

        let mut client = Client::new_alias(&self.peer, config);
        let response = client.request(&request).or(Err("failed to make request"))?;
        let Response::Pull { files, skipped } = response else {
            return Err("error".to_string());
        };

        // Read all files from the stream and write them.
        let mut reader = BufReader::new(client.stream);

        let mut files_got = 0;

        for file in &files {
            let path = warp.path.join(&file.path);

            let Ok(file_handle) = fs::File::create(&path) else {
                println!("Skipping {} due to error", &file.path.to_str().unwrap());
                continue;
            };

            let mut limited_reader = reader.take(file.size);
            let mut writer = BufWriter::new(file_handle);

            io::copy(&mut limited_reader, &mut writer).unwrap();

            reader = limited_reader.into_inner();

            files_got += 1;
        }
        client.stream = reader.into_inner();

        // Read and discard dummy response.
        proto::read_frame(&mut client.stream).unwrap();

        println!("{} files were pulled", files_got);
        println!("{} files were skipped (are equal)", skipped);

        Ok(())
    }
}

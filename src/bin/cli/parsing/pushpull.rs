use super::*;
use crate::client::Client;
use cake::{
    cmd::{Request, Response},
    config::Config,
};
use clap::Args;
use std::{
    fs,
    io::{self, BufReader, BufWriter, Read},
};

#[derive(Args, Debug)]
pub struct PushArgs {
    #[arg(help = "Peer alias")]
    pub peer: String,

    #[arg(help = "Remote warp name")]
    pub warp: Option<String>,
}

impl Executable for PushArgs {
    fn execute(self, config: &mut Config) -> CliResult {
        let warp = config.get_warp_name_or_dir(&self.warp)?;

        // Request remote checksums to compare with local ones.
        let request = Request::Checksum {
            warp: warp.name.clone(),
        };

        println!(" Waiting for remote checksums...");

        let Response::Checksum { sums } = Client::new_alias(&self.peer, config)
            .request(&request)
            .unwrap()
        else {
            return Err("failed to get checksums".to_string());
        };

        // Exclude locally and remotely equal files.
        let (files, skipped) = Checksum::remain_unique(&warp.path, &sums);

        let files_count = files.len();

        if files_count == 0 {
            println!("Nothing to push");
            return Ok(());
        }

        println!(" Pushing {} files...", files_count);

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
                        println!(
                            "\x1b[31m! Skipping {} due to error\x1b[0m",
                            &file.path.to_str().unwrap()
                        );
                        continue;
                    };

                    let mut reader = BufReader::new(file_handle);
                    io::copy(&mut reader, stream).unwrap();
                }
            })
            .or(Err("failed to make request"))?;

        let Response::Push { files } = response else {
            return Err("error".to_string());
        };

        println!(
            "\x1b[32;1m{}\x1b[22m files were pushed, \x1b[1m{}\x1b[22m skipped",
            files, skipped
        );

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
        let warp = config.get_warp_name_or_dir(&self.warp)?;

        // Calculate local checksums
        let sums = Checksum::of_dir_relative(&warp.path, &warp.path).ok_or("warp not found")?;

        // Form & send a request.
        let request = Request::Pull {
            warp: warp.name.clone(),
            sums,
        };

        println!(" Waiting for file list...");

        let mut client = Client::new_alias(&self.peer, config);
        let response = client.request(&request).or(Err("failed to make request"))?;
        let Response::Pull { files, skipped } = response else {
            return Err("error".to_string());
        };

        let files_count = files.len();

        if files_count == 0 {
            println!("Nothing to pull");
            return Ok(());
        }

        println!(" Pulling {} files...", files_count);

        // Read all files from the stream and write them.
        let mut reader = BufReader::new(client.stream);

        let mut files_got = 0;

        for file in &files {
            let path = warp.path.join(&file.path);

            // Create a directory for the file.
            if let Some(parent_directory) = path.parent() {
                fs::create_dir_all(parent_directory).or(Err("failed to create a directory"))?;
            }

            let Ok(file_handle) = fs::File::create(&path) else {
                println!(
                    "\x1b[31m! Skipping {} due to error\x1b[0m",
                    &file.path.to_str().unwrap()
                );

                continue;
            };

            let mut limited_reader = reader.take(file.size);
            let mut writer = BufWriter::new(file_handle);

            io::copy(&mut limited_reader, &mut writer).unwrap();

            reader = limited_reader.into_inner();

            files_got += 1;
        }
        client.stream = reader.into_inner();

        println!(
            "\x1b[32;3m{}\x1b[23m files were pulled, \x1b[3m{}\x1b[23m skipped",
            files_got, skipped
        );

        Ok(())
    }
}

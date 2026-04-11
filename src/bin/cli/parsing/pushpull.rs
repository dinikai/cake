use super::*;
use crate::{client::Client, parsing::errors::response_error};
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
        let warp = config
            .get_warp_name_or_dir(&self.warp)
            .ok_or(CliError::AnonBadWarp)?;

        // Request remote checksums to compare with local ones.
        let request = Request::Checksum {
            warp: warp.name.clone(),
        };

        ui::work!("Waiting for remote checksums...");

        let response = Client::new_alias(&self.peer, config)
            .map_err(CliError::Client)?
            .request(&request)
            .or(Err(CliError::RequestFailed))?;

        let Response::Checksum { sums } = response else {
            return Err(response_error(response));
        };

        // Exclude locally and remotely equal files.
        let (files, skipped) = Checksum::remain_unique(&warp.path, &sums);

        let files_count = files.len();

        if files_count == 0 {
            ui::result!("Nothing to push");
            return Ok(());
        }

        if config.confirm
            && !ui::confirm(
                "This may overwrite some remote files. Continue?",
                false,
                true,
            )
        {
            ui::result!("Aborted");
            return Ok(());
        }

        ui::work!("Pushing {} files...", files_count);

        let request = Request::Push {
            warp: warp.name.clone(),
            files: files.clone(),
        };

        // Send each file in the warp into the stream.
        let response = Client::new_alias(&self.peer, config)
            .map_err(CliError::Client)?
            .request_do(&request, |stream| {
                for file in &files {
                    let path = warp.path.join(&file.path);

                    let Ok(file_handle) = fs::File::open(&path) else {
                        ui::work_error!(
                            "\x1b[31m! Skipping {} due to error\x1b[0m",
                            &file.path.to_string_lossy()
                        );
                        continue;
                    };

                    let mut reader = BufReader::new(file_handle);
                    if let Err(_) = io::copy(&mut reader, stream) {
                        return;
                    }
                }
            })
            .or(Err(CliError::RequestFailed))?;

        let Response::Push { files } = response else {
            return Err(response_error(response));
        };

        ui::result_success!(
            "\x1b[1m{}\x1b[22m files were pushed, \x1b[1m{}\x1b[22m skipped",
            files,
            skipped
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
        let warp = config
            .get_warp_name_or_dir(&self.warp)
            .ok_or(CliError::AnonBadWarp)?;

        // Calculate local checksums
        let sums = match Checksum::of_dir_relative(&warp.path, &warp.path) {
            Ok(sums) => sums,
            Err(e) => return Err(CliError::Checksum(e)),
        };

        // Form & send a request.
        let request = Request::Pull {
            warp: warp.name.clone(),
            sums,
        };

        ui::work!("Waiting for a file list...");

        let mut client = Client::new_alias(&self.peer, config).map_err(CliError::Client)?;
        let response = client.request(&request).or(Err(CliError::RequestFailed))?;
        let Response::Pull { files, skipped } = response else {
            return Err(response_error(response));
        };

        let files_count = files.len();

        if files_count == 0 {
            ui::result!("Nothing to pull");
            return Ok(());
        }

        if config.confirm
            && !ui::confirm(
                "This may overwrite some local files. Continue?",
                false,
                true,
            )
        {
            ui::result!("Aborted");
            return Ok(());
        }

        ui::work!("Pulling {} files...", files_count);

        // Read all files from the stream and write them.
        let mut reader = BufReader::new(client.stream);

        let mut files_got = 0;

        for file in &files {
            let path = warp.path.join(&file.path);

            // Create a directory for the file.
            if let Some(parent_directory) = path.parent() {
                fs::create_dir_all(parent_directory).or(Err(CliError::DirCreation(
                    (*parent_directory).to_path_buf(),
                )))?;
            }

            let Ok(file_handle) = fs::File::create(&path) else {
                ui::work_error!(
                    "\x1b[31m! Skipping {} due to error\x1b[0m",
                    &file.path.to_string_lossy()
                );

                continue;
            };

            let mut limited_reader = reader.take(file.size);
            let mut writer = BufWriter::new(file_handle);

            io::copy(&mut limited_reader, &mut writer).or(Err(CliError::PullCopy))?;

            reader = limited_reader.into_inner();

            files_got += 1;
        }
        client.stream = reader.into_inner();

        ui::result_success!(
            "\x1b[1m{}\x1b[22m files were pulled, \x1b[1m{}\x1b[22m skipped",
            files_got,
            skipped
        );

        Ok(())
    }
}

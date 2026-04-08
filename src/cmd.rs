use std::{
    fs,
    io::{self, BufReader, BufWriter, Read},
    net::TcpStream,
    path::PathBuf,
};

use crate::{checksum::Checksum, config::Config};
use serde::{Deserialize, Serialize};

pub const FALLBACK_CODE: u32 = 1071;

type CmdResult = Result<Response, String>;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Ping,
    Checksum { warp: String },
    Push { warp: String, files: Vec<File> },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Pong,
    Checksum { sums: Vec<Checksum> },
    Push { files: u32 },

    Error(String),
}

impl Request {
    pub fn execute(&self, stream: &mut TcpStream, config: &mut Config) -> Response {
        let result = match self {
            Self::Ping => ping(),
            Self::Checksum { warp } => checksum(warp, config),
            Self::Push { warp, files } => push(warp, files, stream, config),
        };

        match result {
            Ok(r) => r,
            Err(e) => Response::Error(e),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub path: PathBuf,
    pub size: u64,
}

fn ping() -> CmdResult {
    Ok(Response::Pong)
}

fn checksum(warp: &str, config: &Config) -> CmdResult {
    let warp = config.get_warp(warp).ok_or("warp not found")?;
    let path = &warp.path;

    let mut sums: Vec<Checksum> = Checksum::of_dir(path)
        .ok_or("failed to get checksums")
        .into_iter()
        .flatten()
        .filter_map(|c| c)
        .collect();

    for sum in sums.iter_mut() {
        if let Some(path) = pathdiff::diff_paths(&sum.path, path) {
            sum.path = path;
        }
    }

    Ok(Response::Checksum { sums })
}

fn push(warp: &str, files: &Vec<File>, stream: &mut TcpStream, config: &Config) -> CmdResult {
    let warp = config.get_warp(warp).ok_or("warp not found")?;
    let path = &warp.path;

    let mut reader = BufReader::new(stream);

    let mut files_written: u32 = 0;

    for file in files {
        let file_path = path.join(&file.path);

        // Get the file handle or skip it.
        let Ok(file_handle) = fs::File::create(file_path) else {
            let mut skip_reader = reader.take(file.size);

            io::copy(&mut skip_reader, &mut io::sink())
                .or(Err("file open has failed as well as skip attempt"))?;

            reader = skip_reader.into_inner();
            continue;
        };

        let mut writer = BufWriter::new(file_handle);

        // Retrieve the limited reader for a file.
        let mut limited_reader = reader.take(file.size);

        // Write the buffer into the file.
        io::copy(&mut limited_reader, &mut writer).or(Err("failed to write the file"))?;

        reader = limited_reader.into_inner();
        files_written += 1;
    }

    Ok(Response::Push {
        files: files_written,
    })
}

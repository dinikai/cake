use std::{
    fs,
    io::{self, BufReader, BufWriter, Read},
    net::TcpStream,
    path::PathBuf,
};

use crate::{checksum::Checksum, config::Config, proto};
use serde::{Deserialize, Serialize};

pub const FALLBACK_CODE: u32 = 1071;

type CmdResult = Result<Response, String>;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Ping,
    Checksum { warp: String },
    Push { warp: String, files: Vec<File> },
    Pull { warp: String, sums: Vec<Checksum> },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Pong,
    Checksum { sums: Vec<Checksum> },
    Push { files: u32 },
    Pull { files: Vec<File>, skipped: u32 },

    Error(String),

    None,
}

impl Request {
    pub fn execute(&self, stream: &mut TcpStream, config: &mut Config) -> Response {
        let result = match self {
            Self::Ping => ping(),
            Self::Checksum { warp } => checksum(warp, config),
            Self::Push { warp, files } => push(warp, files, stream, config),
            Self::Pull { warp, sums } => pull(warp, sums, stream, config),
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

    let mut sums: Vec<Checksum> =
        Checksum::of_dir_relative(path, &warp.path).ok_or("failed to get checksums")?;

    // Make paths relative to the warp.
    for sum in &mut sums {
        if let Some(path) = pathdiff::diff_paths(&sum.path, path) {
            sum.path = path;
        }
    }

    Ok(Response::Checksum { sums })
}

fn push(warp: &str, files: &Vec<File>, stream: &TcpStream, config: &Config) -> CmdResult {
    let warp = config.get_warp(warp).ok_or("warp not found")?;
    let path = &warp.path;

    let mut reader = BufReader::new(stream);

    let mut files_written: u32 = 0;

    for file in files {
        let file_path = path.join(&file.path);

        // Create a directory for the file.
        if let Some(parent_directory) = file_path.parent() {
            fs::create_dir_all(parent_directory).or(Err("failed to create a directory"))?;
        }

        // Get the file handle or skip it.
        let Ok(file_handle) = fs::File::create(&file_path) else {
            println!("Can't open file: {}", &file_path.to_str().unwrap());
            let mut skip_reader = reader.take(file.size);

            io::copy(&mut skip_reader, &mut io::sink())
                .or(Err("file opening has failed as well as skip attempt"))?;

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

fn pull(warp: &str, sums: &Vec<Checksum>, stream: &mut TcpStream, config: &Config) -> CmdResult {
    let warp = config.get_warp(warp).ok_or("warp not found")?;
    let path = &warp.path;

    // Exclude locally and remotely equal files.
    let (files, skipped) = Checksum::remain_unique(path, &sums);

    // Send pull response.
    let response = Response::Pull {
        files: files.clone(),
        skipped,
    };
    let bytes = postcard::to_stdvec(&response).unwrap();
    proto::write_frame(stream, &bytes).or(Err("failed to write the pull response frame"))?;

    // Send raw file data.
    for file in &files {
        let path = warp.path.join(&file.path);

        let Ok(file_handle) = fs::File::open(&path) else {
            continue;
        };

        let mut reader = BufReader::new(file_handle);
        io::copy(&mut reader, stream).unwrap();
    }

    Ok(Response::None)
}

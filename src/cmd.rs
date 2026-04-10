use std::{
    fs,
    io::{self, BufReader, BufWriter, Read},
    net::TcpStream,
    path::PathBuf,
};

use crate::{checksum::Checksum, config::Config, errors::CmdError, proto};
use serde::{Deserialize, Serialize};

pub const FATAL_CODE: u32 = 1071;

type CmdResult = Result<Response, CmdError>;

#[derive(Clone, Serialize, Deserialize, Debug)]
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

    Error(CmdError),

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
    log::info!("I got pinged");

    Ok(Response::Pong)
}

fn checksum(warp: &str, config: &Config) -> CmdResult {
    let warp = config
        .get_warp(warp)
        .ok_or(CmdError::BadWarp(warp.to_string()))?;

    let path = &warp.path;

    log::info!(
        "Calculating checksums of the '{}' warp at {}",
        &warp.name,
        &warp.path.to_string_lossy()
    );

    let sums = match Checksum::of_dir_relative(path, path) {
        Ok(sums) => sums,
        Err(e) => return Err(CmdError::Checksum(e)),
    };

    Ok(Response::Checksum { sums })
}

fn push(warp: &str, files: &Vec<File>, stream: &TcpStream, config: &Config) -> CmdResult {
    let warp = config
        .get_warp(warp)
        .ok_or(CmdError::BadWarp(warp.to_string()))?;

    let path = &warp.path;

    let mut reader = BufReader::new(stream);

    let mut files_written: u32 = 0;

    log::info!(
        "Receiving files: {} total into the '{}' warp at {}",
        files.len(),
        &warp.name,
        &warp.path.to_string_lossy()
    );

    for file in files {
        let file_path = path.join(&file.path);

        // Create a directory for the file.
        if let Some(parent_directory) = file_path.parent() {
            fs::create_dir_all(parent_directory).or(Err(CmdError::DirCreation(
                (*parent_directory).to_path_buf(),
            )))?;
        }

        // Get the file handle or skip it.
        let Ok(file_handle) = fs::File::create(&file_path) else {
            println!("Can't open file: {}", &file_path.to_str().unwrap());
            let mut skip_reader = reader.take(file.size);

            io::copy(&mut skip_reader, &mut io::sink()).or(Err(CmdError::FileSkip))?;

            reader = skip_reader.into_inner();
            continue;
        };

        let mut writer = BufWriter::new(file_handle);

        // Retrieve the limited reader for a file.
        let mut limited_reader = reader.take(file.size);

        // Write the buffer into the file.
        io::copy(&mut limited_reader, &mut writer).or(Err(CmdError::PushCopy))?;

        reader = limited_reader.into_inner();
        files_written += 1;
    }

    log::info!("Done. {} files were written", files_written);

    Ok(Response::Push {
        files: files_written,
    })
}

fn pull(warp: &str, sums: &Vec<Checksum>, stream: &mut TcpStream, config: &Config) -> CmdResult {
    let warp = config
        .get_warp(warp)
        .ok_or(CmdError::BadWarp(warp.to_string()))?;

    let path = &warp.path;

    // Exclude locally and remotely equal files.
    let (files, skipped) = Checksum::remain_unique(path, &sums);

    log::info!(
        "Sending files: {} total from the '{}' warp at {}",
        files.len(),
        &warp.name,
        &warp.path.to_string_lossy()
    );

    // Send pull response.
    let response = Response::Pull {
        files: files.clone(),
        skipped,
    };
    let bytes = postcard::to_stdvec(&response).unwrap();

    proto::write_frame(stream, &bytes).or(Err(CmdError::FrameWrite))?;

    // Send raw file data.
    for file in &files {
        let path = warp.path.join(&file.path);

        let Ok(file_handle) = fs::File::open(&path) else {
            continue;
        };

        let mut reader = BufReader::new(file_handle);
        io::copy(&mut reader, stream).or(Err(CmdError::PullCopy))?;
    }

    log::info!("Done. {} files were sent", files.len());

    Ok(Response::None)
}

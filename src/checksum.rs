use crate::cmd;
use crc32fast::Hasher;
use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

pub type ChecksumResult<T> = Result<T, ChecksumError>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Checksum {
    pub path: PathBuf,
    pub sum: u32,
}

impl Checksum {
    /// Tries to calculate the checksum of a file.
    pub async fn of_file(path: &Path) -> ChecksumResult<Checksum> {
        let mut file = File::open(&path).await.or(Err(ChecksumError::Io))?;

        // TODO: Replace CRC32 to some another hashing algo.
        let mut hasher = Hasher::new();

        let mut buf = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buf).await.or(Err(ChecksumError::Io))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buf[..bytes_read]);
        }

        let checksum = Self {
            path: path.to_path_buf(),
            sum: hasher.finalize(),
        };

        Ok(checksum)
    }

    /// Walks thorugh all the files in the directory
    /// and calculates each one's checksum.
    pub async fn of_dir(path: &Path) -> ChecksumResult<Vec<Checksum>> {
        if !path.is_dir() {
            return Err(ChecksumError::Io);
        }

        let mut result = Vec::new();

        let mut paths: Vec<PathBuf> = Self::build_walker(path)
            .filter_map(|f| f.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .map(|entry| entry.into_path())
            .collect();

        paths.sort();

        for path in paths {
            let Ok(sum) = Self::of_file(&path).await else {
                continue;
            };
            result.push(sum);
        }

        Ok(result)
    }

    pub async fn of_dir_relative(path: &Path, base: &Path) -> ChecksumResult<Vec<Checksum>> {
        let mut sums = Self::of_dir(path).await?;

        // Make paths relative to the base.
        for sum in &mut sums {
            if let Some(relative) = pathdiff::diff_paths(&sum.path, base) {
                sum.path = relative;
            }
        }

        Ok(sums)
    }

    pub async fn remain_unique(
        path: &Path,
        other: &Vec<Checksum>,
    ) -> ChecksumResult<(Vec<cmd::File>, u32)> {
        let mut skipped = 0;

        let local_sums = Self::of_dir_relative(path, path).await?;

        let mut unique_paths: Vec<PathBuf> = Vec::new();
        for local_sum in local_sums {
            let Some(remote_sum) = other.iter().find(|c| c.path == local_sum.path) else {
                continue;
            };

            if &local_sum == remote_sum {
                skipped += 1;
                continue;
            }
            unique_paths.push(local_sum.path);
        }
        unique_paths.sort();

        let mut files: Vec<cmd::File> = Vec::new();
        for path in &unique_paths {
            files.push(cmd::File {
                path: pathdiff::diff_paths(path, path)
                    .ok_or(ChecksumError::Io)?
                    .to_path_buf(),
                size: fs::metadata(path)
                    .await
                    .map_err(|_| ChecksumError::Io)?
                    .len(),
            });
        }

        Ok((files, skipped))
    }

    fn build_walker(path: &Path) -> Walk {
        WalkBuilder::new(path)
            .standard_filters(false)
            .add_custom_ignore_filename(".cakeignore")
            .build()
    }
}

impl PartialEq for Checksum {
    fn eq(&self, other: &Self) -> bool {
        self.sum == other.sum
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(\x1b[3m{:08x}\x1b[23m) {}",
            &self.sum,
            self.path.to_str().unwrap()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChecksumError {
    Io,
}

impl Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io => write!(f, "I/O error"),
        }
    }
}

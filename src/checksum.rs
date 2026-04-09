use crc32fast::Hasher;
use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use crate::cmd;

#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    pub path: PathBuf,
    pub sum: u32,
}

impl Checksum {
    /// Tries to calculate the checksum of a file.
    pub fn of_file(path: &Path) -> Option<Checksum> {
        let mut file = File::open(&path).ok()?;
        let mut hasher = Hasher::new();

        let mut buf = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buf).ok()?;
            if bytes_read == 0 {
                break;
            }

            hasher.update(&buf[..bytes_read]);
        }

        let checksum = Self {
            path: path.to_path_buf(),
            sum: hasher.finalize(),
        };

        Some(checksum)
    }

    /// Walks thorugh all the files in the directory
    /// and calculates each one's checksum.
    pub fn of_dir(path: &Path) -> Option<Vec<Checksum>> {
        if !path.is_dir() {
            return None;
        }

        let mut result = Vec::new();

        for file in Self::build_walker(path)
            .filter_map(|f| f.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
        {
            let Some(sum) = Self::of_file(file.path()) else {
                continue;
            };
            result.push(sum);
        }

        Some(result)
    }

    pub fn of_dir_relative(path: &Path, base: &Path) -> Option<Vec<Checksum>> {
        let mut sums = Self::of_dir(path)?;

        // Make paths relative to the warp.
        for sum in &mut sums {
            if let Some(relative) = pathdiff::diff_paths(&sum.path, base) {
                sum.path = relative;
            }
        }

        Some(sums)
    }

    pub fn remain_unique(path: &Path, other: &Vec<Checksum>) -> (Vec<cmd::File>, u32) {
        let mut skipped = 0;

        (
            Self::build_walker(path)
                .filter_map(|f| f.ok())
                .filter(|entry| entry.file_type().unwrap().is_file())
                .filter(|f| {
                    let diff = pathdiff::diff_paths(f.path(), path).unwrap();

                    let Some(remote_sum) = other.iter().find(|c| c.path == diff) else {
                        return true;
                    };

                    let local_sum = &Checksum::of_file(f.path()).unwrap();

                    if local_sum == remote_sum {
                        skipped += 1;
                    }

                    local_sum != remote_sum
                })
                .map(|f| cmd::File {
                    path: pathdiff::diff_paths(f.path(), path).unwrap().to_path_buf(),
                    size: fs::metadata(f.path()).unwrap().len(),
                })
                .collect(),
            skipped,
        )
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
        write!(f, "{}: {:08x}", self.path.to_str().unwrap(), self.sum)
    }
}

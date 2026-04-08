use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

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
    pub fn of_dir(path: &Path) -> Option<Vec<Option<Checksum>>> {
        if !path.is_dir() {
            return None;
        }

        let mut result = Vec::new();

        for file in WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            result.push(Self::of_file(file.path()));
        }

        Some(result)
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

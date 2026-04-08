use serde::{Deserialize, Serialize};

use crate::{checksum::Checksum, config::Config};

pub const FALLBACK_CODE: u32 = 1071;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Ping,
    Checksum { warp: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Pong,
    Checksum {
        sums: Vec<crate::checksum::Checksum>,
    },
    Error(String),
}

impl Request {
    pub fn execute(&self, config: &mut Config) -> Response {
        let result = match self {
            Self::Ping => ping(),
            Self::Checksum { warp } => checksum(warp, config),
        };

        match result {
            Ok(r) => r,
            Err(e) => Response::Error(e),
        }
    }
}

fn ping() -> Result<Response, String> {
    Ok(Response::Pong)
}

fn checksum(warp: &str, config: &Config) -> Result<Response, String> {
    let warp = config
        .warps
        .iter()
        .find(|w| w.name == warp)
        .ok_or("warp not found")?;

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

use crate::auth::AuthToken;
use base64::{Engine, prelude::BASE64_STANDARD};
use sha3::{Digest, Sha3_256};
use std::{
    fs,
    io::{BufRead, BufReader, BufWriter, ErrorKind, Write},
    path::{Path, PathBuf},
};

pub const TOKEN_HASH_LENGTH: usize = 32;

/// This structure is not the same as `AuthToken`.
/// It holds an owner and a **hashed** UUID of a token.
pub struct HashedToken {
    pub hash: [u8; TOKEN_HASH_LENGTH],
    pub owner: String,
}

impl HashedToken {
    pub fn from_str(str: &str) -> Option<Self> {
        let parts: Vec<&str> = str.split(' ').collect();
        if parts.len() != 2 {
            return None;
        }

        let mut hash = [0u8; TOKEN_HASH_LENGTH];
        BASE64_STANDARD.decode_slice(parts[1], &mut hash).ok()?;

        Some(Self {
            hash,
            owner: parts[0].to_string(),
        })
    }

    pub fn from_token(token: &AuthToken, owner: &str) -> Self {
        HashedToken {
            hash: Self::hash_token(token),
            owner: owner.to_string(),
        }
    }

    pub fn hash_token(token: &AuthToken) -> [u8; TOKEN_HASH_LENGTH] {
        let mut hasher = Sha3_256::new();
        hasher.update(token.uuid);

        hasher
            .finalize()
            .as_array::<TOKEN_HASH_LENGTH>()
            .unwrap()
            .clone()
    }
}

pub struct AuthTokenPool {
    pub tokens: Vec<HashedToken>,
}

impl AuthTokenPool {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    fs::File::create(path)?;
                    return Ok(Self { tokens: Vec::new() });
                }

                anyhow::bail!(e);
            }
        };
        let reader = BufReader::new(file);

        let mut tokens: Vec<HashedToken> = Vec::new();

        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };

            let Some(token) = HashedToken::from_str(&line) else {
                continue;
            };

            tokens.push(token);
        }

        Ok(Self { tokens })
    }

    pub fn from_default() -> anyhow::Result<Self> {
        let path = Self::get_default_path().ok_or(anyhow::format_err!("stub"))?;

        Self::from_file(&path)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        for token in self.tokens.iter() {
            let base64 = BASE64_STANDARD.encode(token.hash);

            if let Err(_) = write!(writer, "{} {}\n", &token.owner, &base64) {
                continue;
            }
        }

        Ok(())
    }

    pub fn save_default(&self) -> anyhow::Result<()> {
        let path = Self::get_default_path().ok_or(anyhow::format_err!("stub"))?;

        self.save(&path)
    }

    pub fn get_default_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".local/share/cake-daemon-tokens"))
    }
}

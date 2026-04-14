use crate::auth::AuthToken;
use base64::{Engine, prelude::BASE64_STANDARD};
use sha3::{Digest, Sha3_256};
use std::path::{Path, PathBuf};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ErrorKind},
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
    pub async fn from_file(path: &Path) -> anyhow::Result<Self> {
        let file = match fs::File::open(path).await {
            Ok(f) => f,
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    fs::File::create(path).await?;
                    return Ok(Self { tokens: Vec::new() });
                }

                anyhow::bail!(e);
            }
        };
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut tokens: Vec<HashedToken> = Vec::new();

        loop {
            let Ok(line) = lines.next_line().await else {
                break;
            };
            let Some(line) = line else {
                break;
            };
            let Some(token) = HashedToken::from_str(&line) else {
                continue;
            };

            tokens.push(token);
        }

        Ok(Self { tokens })
    }

    pub async fn from_default() -> anyhow::Result<Self> {
        let path = Self::get_default_path().ok_or(anyhow::format_err!("stub"))?;

        Self::from_file(&path).await
    }

    pub async fn save(&self, path: &Path) -> anyhow::Result<()> {
        let file = fs::File::create(path).await?;
        let mut writer = BufWriter::new(file);

        for token in self.tokens.iter() {
            let base64 = BASE64_STANDARD.encode(token.hash);
            let line = format!("{} {}\n", &token.owner, &base64);

            if let Err(_) = writer.write(&line.as_bytes()).await {
                continue;
            }
        }

        writer.flush().await?;

        Ok(())
    }

    pub async fn save_default(&self) -> anyhow::Result<()> {
        let path = Self::get_default_path().ok_or(anyhow::format_err!("stub"))?;

        self.save(&path).await
    }

    pub fn get_default_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".local/share/cake-daemon-tokens"))
    }
}

use crate::{PROTOCOL_VER, ProtocolVer, auth::AuthRequestEnvelope, cmd::FATAL_CODE};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub type ProtoResult<T> = Result<T, ProtoError>;

/// Wrapper for `AuthRequest`. Carries the protocol version.
#[derive(Serialize, Deserialize, Debug)]
struct RequestEnvelope {
    pub protocol_ver: u32,
    pub request: AuthRequestEnvelope,
}

pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, data: &[u8]) -> ProtoResult<()> {
    let len = data.len() as u32;

    writer
        .write_all(&len.to_le_bytes())
        .await
        .map_err(|_| ProtoError::Io)?;
    writer.write_all(data).await.map_err(|_| ProtoError::Io)?;

    Ok(())
}

pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> ProtoResult<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .await
        .map_err(|_| ProtoError::Io)?;
    let len = u32::from_le_bytes(len_buf);

    if len == FATAL_CODE {
        return Err(ProtoError::Fatal);
    }

    let mut buf = vec![0u8; len as usize];
    reader
        .read_exact(&mut buf)
        .await
        .map_err(|_| ProtoError::Io)?;

    Ok(buf)
}

pub async fn send_request<W: AsyncWrite + Unpin>(
    writer: &mut W,
    request: &AuthRequestEnvelope,
) -> ProtoResult<()> {
    // Wrap request into the RequestEnvelope.
    let request = RequestEnvelope {
        protocol_ver: PROTOCOL_VER,
        request: (*request).clone(),
    };
    let bytes = postcard::to_stdvec(&request).map_err(|_| ProtoError::Serde)?;

    write_frame(writer, &bytes).await?;

    Ok(())
}

pub async fn read_request<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> ProtoResult<AuthRequestEnvelope> {
    let bytes = read_frame(reader).await?;
    let request: RequestEnvelope = postcard::from_bytes(&bytes).map_err(|_| ProtoError::Serde)?;

    if request.protocol_ver != PROTOCOL_VER {
        return Err(ProtoError::ProtocolVer {
            got: request.protocol_ver,
            have: PROTOCOL_VER,
        });
    }

    Ok(request.request)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProtoError {
    Io,
    Serde,
    Fatal,
    ProtocolVer { got: ProtocolVer, have: ProtocolVer },
}

impl Display for ProtoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io => write!(f, "I/O error"),
            Self::Serde => write!(f, "serialization/deserialization error"),
            Self::Fatal => write!(f, "fatal error code received"),
            Self::ProtocolVer { got, have } => {
                write!(f, "incompatible version (got {got}, have {have})")
            }
        }
    }
}

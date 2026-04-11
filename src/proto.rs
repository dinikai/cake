use crate::{PROTOCOL_VER, ProtocolVer, auth::AuthRequestEnvelope, cmd::FATAL_CODE};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
};

pub type ProtoResult<T> = Result<T, ProtoError>;

/// Wrapper for `AuthRequest`. Carries the protocol version.
#[derive(Serialize, Deserialize, Debug)]
struct RequestEnvelope {
    pub protocol_ver: u32,
    pub request: AuthRequestEnvelope,
}

pub fn write_frame(stream: &mut TcpStream, data: &[u8]) -> ProtoResult<()> {
    let len = data.len() as u32;

    stream
        .write_all(&len.to_le_bytes())
        .map_err(|_| ProtoError::Io)?;
    stream.write_all(data).map_err(|_| ProtoError::Io)?;

    Ok(())
}

pub fn read_frame(stream: &mut TcpStream) -> ProtoResult<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .map_err(|_| ProtoError::Io)?;
    let len = u32::from_le_bytes(len_buf);

    if len == FATAL_CODE {
        return Err(ProtoError::Fatal);
    }

    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).map_err(|_| ProtoError::Io)?;

    Ok(buf)
}

pub fn send_request(stream: &mut TcpStream, request: &AuthRequestEnvelope) -> ProtoResult<()> {
    // Wrap request into the RequestEnvelope.
    let request = RequestEnvelope {
        protocol_ver: PROTOCOL_VER,
        request: (*request).clone(),
    };
    let bytes = postcard::to_stdvec(&request).map_err(|_| ProtoError::Serde)?;

    write_frame(stream, &bytes)?;

    Ok(())
}

pub fn read_request(stream: &mut TcpStream) -> ProtoResult<AuthRequestEnvelope> {
    let bytes = read_frame(stream)?;
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

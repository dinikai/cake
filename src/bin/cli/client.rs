use std::{fmt::Display, net::TcpStream};

use cake::{
    cmd::{Request, Response},
    config::Config,
    proto,
};

pub type ClientResult<T> = Result<T, ClientError>;

pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub fn new(addr: &str) -> ClientResult<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr).or(Err(ClientError::Connection(addr.to_string())))?,
        })
    }

    pub fn new_alias(alias: &str, config: &Config) -> ClientResult<Self> {
        let alias = config
            .aliases
            .iter()
            .find(|a| a.name == alias)
            .ok_or(ClientError::Alias(alias.to_string()))?;

        Self::new(&alias.host)
    }

    /// Sends a request and returns the server's response.
    pub fn request(&mut self, request: &Request) -> ClientResult<Response> {
        self.request_do(request, |_| {})
    }

    pub fn request_do<F>(&mut self, request: &Request, func: F) -> ClientResult<Response>
    where
        F: Fn(&mut TcpStream),
    {
        proto::send_request(&mut self.stream, request).or(Err(ClientError::Send))?;

        func(&mut self.stream);

        let bytes = proto::read_frame(&mut self.stream).or(Err(ClientError::Read))?;
        let response = postcard::from_bytes(&bytes).or(Err(ClientError::ResDeserialize))?;

        Ok(response)
    }
}

#[derive(Debug)]
pub enum ClientError {
    Connection(String),
    Send,
    Read,
    ResDeserialize,
    Alias(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(addr) => write!(f, "failed to connect to {addr}"),
            Self::Send => write!(f, "request sending error"),
            Self::Read => write!(f, "request reading error"),
            Self::ResDeserialize => write!(f, "request deserializing error"),
            Self::Alias(alias) => write!(f, "'{alias}' is unknown"),
        }
    }
}

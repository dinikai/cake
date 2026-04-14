use cake::{
    auth::{AuthRequestEnvelope, AuthToken},
    cmd::{Request, Response},
    config::Config,
    proto,
};
use std::fmt::Display;
use tokio::net::TcpStream;

pub type ClientResult<T> = Result<T, ClientError>;

pub struct Client {
    pub stream: TcpStream,
    pub auth_token: AuthToken,
}

impl Client {
    pub async fn new(addr: &str, config: &Config) -> ClientResult<Self> {
        let alias = config
            .get_alias_by_host(addr)
            .ok_or(ClientError::Alias(addr.to_string()))?;

        Ok(Self {
            stream: TcpStream::connect(addr)
                .await
                .or(Err(ClientError::Connection(addr.to_string())))?,
            auth_token: AuthToken::from(&alias.auth_token),
        })
    }

    pub async fn new_alias(alias: &str, config: &Config) -> ClientResult<Self> {
        let alias = config
            .aliases
            .iter()
            .find(|a| a.name == alias)
            .ok_or(ClientError::Alias(alias.to_string()))?;

        Self::new(&alias.host, config).await
    }

    /// Sends a request and returns the server's response.
    pub async fn request(&mut self, request: &Request) -> ClientResult<Response> {
        self.request_do(request, async |_| {}).await
    }

    pub async fn request_do<F>(&mut self, request: &Request, func: F) -> ClientResult<Response>
    where
        F: AsyncFn(&mut TcpStream),
    {
        let auth_request = AuthRequestEnvelope::from(request, &self.auth_token);

        proto::send_request(&mut self.stream, &auth_request)
            .await
            .or(Err(ClientError::Send))?;

        func(&mut self.stream).await;

        let bytes = proto::read_frame(&mut self.stream)
            .await
            .or(Err(ClientError::Read))?;

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
            Self::ResDeserialize => write!(f, "response deserialization error"),
            Self::Alias(alias) => write!(f, "'{alias}' is unknown"),
        }
    }
}

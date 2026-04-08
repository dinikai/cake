use std::net::TcpStream;

use cake::{
    cmd::{Request, Response},
    config::Config,
    proto,
};

pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub fn new(addr: &str) -> Self {
        Self {
            stream: TcpStream::connect(addr).unwrap(),
        }
    }

    pub fn new_alias(alias: &str, config: &Config) -> Self {
        let alias = config
            .aliases
            .iter()
            .find(|a| a.name == alias)
            // FIXME: Replace stub error to the good one.
            .unwrap();

        Self::new(&alias.host)
    }

    /// Sends a request and returns the server's response.
    pub fn request(&mut self, request: &Request) -> anyhow::Result<Response> {
        self.request_do(request, |_| {})
    }

    pub fn request_do<F>(&mut self, request: &Request, func: F) -> anyhow::Result<Response>
    where
        F: Fn(&mut TcpStream),
    {
        proto::send_request(&mut self.stream, request)?;

        func(&mut self.stream);

        let bytes = proto::read_frame(&mut self.stream)?;
        let response = postcard::from_bytes(&bytes)?;

        Ok(response)
    }
}

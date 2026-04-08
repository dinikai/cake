use std::net::TcpStream;

use cake::{
    cmd::{Request, Response},
    config::Config,
    proto,
};

/// Sends a request and returns the server's response.
pub fn request(address: &str, request: &Request) -> anyhow::Result<Response> {
    let mut stream = TcpStream::connect(address)?;

    proto::send_request(&mut stream, request)?;

    let bytes = proto::read_frame(&mut stream)?;
    let response = postcard::from_bytes(&bytes)?;

    Ok(response)
}

/// Sends a request to the alias and returns the response.
pub fn request_alias(alias: &str, request: &Request, config: &Config) -> anyhow::Result<Response> {
    let address = config
        .aliases
        .iter()
        .find(|a| a.name == alias)
        // FIXME: Replace stub error to the good one.
        .ok_or(std::io::Error::last_os_error())?;

    crate::client::request(&address.host, request)
}

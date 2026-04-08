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

pub fn request_do<F>(address: &str, request: &Request, func: F) -> anyhow::Result<Response>
where
    F: Fn(&mut TcpStream),
{
    let mut stream = TcpStream::connect(address)?;

    proto::send_request(&mut stream, request)?;

    func(&mut stream);

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

pub fn request_alias_do<F>(
    alias: &str,
    request: &Request,
    func: F,
    config: &Config,
) -> anyhow::Result<Response>
where
    F: Fn(&mut TcpStream),
{
    let address = config
        .aliases
        .iter()
        .find(|a| a.name == alias)
        // FIXME: Replace stub error to the good one.
        .ok_or(std::io::Error::last_os_error())?;

    request_do(&address.host, request, func)
}

use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::cmd::{FALLBACK_CODE, Request};

pub fn write_frame(stream: &mut TcpStream, data: &[u8]) -> io::Result<()> {
    let len = data.len() as u32;

    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(data)?;

    Ok(())
}

pub fn read_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);

    if len == FALLBACK_CODE {
        // FIXME: Ugh!
        return Err(io::Error::last_os_error());
    }

    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf)?;

    Ok(buf)
}

pub fn send_request(stream: &mut TcpStream, request: &Request) -> anyhow::Result<()> {
    let bytes = postcard::to_stdvec(request)?;
    write_frame(stream, &bytes)?;
    Ok(())
}

pub fn read_request(stream: &mut TcpStream) -> anyhow::Result<Request> {
    let bytes = read_frame(stream)?;
    let request = postcard::from_bytes(&bytes)?;
    Ok(request)
}

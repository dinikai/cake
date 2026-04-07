use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
};

use cake::{checksum::Checksum, cmd::Command};

/// Wraps a TCP listener with control methods.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Constructs a new server
    /// with the binding address.
    pub fn new(bind: &str) -> Option<Self> {
        Some(Self {
            listener: TcpListener::bind(bind).ok()?,
        })
    }

    /// Starts a loop within which will listen
    /// to the incoming requests.
    pub fn start(self) -> Option<()> {
        loop {
            let (stream, addr) = self.listener.accept().ok()?;

            Self::handle_connection(stream, addr).unwrap();
        }
    }

    fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Option<()> {
        let cmd = Self::get_cmd(&mut stream)?;

        match cmd {
            Command::ChecksumRequest(path) => Self::execute_checksum_request(&path),
            Command::ChecksumResponse(checksums) => Self::execute_checksum_response(&checksums),
        }?;

        Some(())
    }

    fn get_cmd(stream: &mut TcpStream) -> Option<Command> {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).ok()?;
        let cmd_length = u32::from_le_bytes(buf) as usize;

        let mut buf = Vec::with_capacity(cmd_length);
        stream.read_exact(&mut buf).ok()?;

        postcard::from_bytes(&buf).ok()?
    }
}

/// Command handler implementations.
impl Server {
    pub fn execute_checksum_request(path: &str) -> Option<()> {
        Some(())
    }

    pub fn execute_checksum_response(checksums: &Vec<Checksum>) -> Option<()> {
        Some(())
    }
}

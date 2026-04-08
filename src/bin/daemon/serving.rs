use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use cake::{cmd::FALLBACK_CODE, config::Config, proto};

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
        let mut config = Config::from_default().ok()?;

        loop {
            let (mut stream, _) = self.listener.accept().ok()?;

            if let Err(_) = Self::handle_connection(&mut stream, &mut config) {
                // Fallback error signal.
                stream.write_all(&FALLBACK_CODE.to_le_bytes()).unwrap();
            };
        }
    }

    /// Accepts the TCP stream, reads and executes the command
    /// and writes a response.
    fn handle_connection(stream: &mut TcpStream, config: &mut Config) -> anyhow::Result<()> {
        let request = proto::read_request(stream)?;

        let result = request.execute(stream, config);

        let bytes = postcard::to_stdvec(&result)?;
        proto::write_frame(stream, &bytes)?;

        Ok(())
    }
}

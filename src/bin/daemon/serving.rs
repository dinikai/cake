use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
};

use cake::{
    cmd::{FATAL_CODE, Response},
    config::Config,
    errors::CmdError,
    proto,
};

/// Wraps a TCP listener with control methods.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Constructs a new server
    /// with the binding address.
    pub fn new(bind: &str) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(bind)?,
        })
    }

    /// Starts a loop within which will listen
    /// to the incoming requests.
    pub fn start(self) -> Option<()> {
        let mut config = Config::from_default().ok()?;

        loop {
            let (mut stream, _) = self.listener.accept().ok()?;

            if let Err(e) = Self::handle_connection(&mut stream, &mut config) {
                // Fallback error signal.
                stream.write_all(&FATAL_CODE.to_le_bytes()).unwrap();

                log::error!("Fatal error, writing the FATAL_CODE: {e}");
            };
        }
    }

    /// Accepts the TCP stream, reads and executes the command
    /// and writes a response.
    fn handle_connection(stream: &mut TcpStream, config: &mut Config) -> anyhow::Result<()> {
        let request = match proto::read_request(stream) {
            Ok(r) => r,
            Err(e) => {
                let response = Response::Error(CmdError::Proto(e));
                write_response(&response, stream)?;

                return Ok(());
            }
        };

        let result = request.execute(stream, config);

        match result {
            Response::None => (),
            _ => {
                write_response(&result, stream)?;
            }
        }

        Ok(())
    }
}

fn write_response(response: &Response, stream: &mut TcpStream) -> anyhow::Result<()> {
    if let Response::Error(err) = response {
        log::error!("Command error: {err}");
    };

    let bytes = postcard::to_stdvec(response)?;
    if let Err(_) = proto::write_frame(stream, &bytes) {
        anyhow::bail!("response writing error");
    }

    Ok(())
}

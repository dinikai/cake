use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
};

use cake::{
    auth::AuthToken,
    cmd::{FATAL_CODE, Response},
    config::Config,
    errors::CmdError,
    proto,
    token_pool::HashedToken,
};

use cake::token_pool::AuthTokenPool;

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
        let mut token_pool = AuthTokenPool::from_default().ok()?;

        log::info!("The server has been started successfully.");

        loop {
            let (mut stream, _) = self.listener.accept().ok()?;

            if let Err(e) = Self::handle_connection(&mut stream, &mut config, &mut token_pool) {
                // Fallback error signal.
                stream.write_all(&FATAL_CODE.to_le_bytes()).unwrap();

                log::error!("Fatal error, writing the FATAL_CODE: {e}");
            };
        }
    }

    /// Accepts the TCP stream, reads and executes the command
    /// and writes a response.
    fn handle_connection(
        stream: &mut TcpStream,
        config: &mut Config,
        token_pool: &mut AuthTokenPool,
    ) -> anyhow::Result<()> {
        let request = match proto::read_request(stream) {
            Ok(r) => r,
            Err(e) => {
                let response = Response::Error(CmdError::Proto(e));
                write_response(&response, stream)?;

                return Ok(());
            }
        };

        // Check whether the auth token is valid
        // or request shall not pass and return
        // the error early.
        if !validate_token(&request.auth_token, token_pool) {
            let response = Response::Error(CmdError::Auth);
            write_response(&response, stream)?;

            return Ok(());
        }

        // Unpack the envelope.
        let request = request.request;

        // Execute the request.
        let result = request.execute(stream, config);

        // Write the response if it's anything but None.
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

fn validate_token(token: &AuthToken, pool: &AuthTokenPool) -> bool {
    let hashed_token = HashedToken::hash_token(token);

    pool.tokens.iter().any(|t| t.hash == hashed_token)
}

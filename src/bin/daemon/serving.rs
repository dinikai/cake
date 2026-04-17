use cake::token_pool::AuthTokenPool;
use cake::{
    auth::AuthToken,
    cmd::{FATAL_CODE, Response},
    config::Config,
    errors::CmdError,
    proto,
    token_pool::HashedToken,
};
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Wraps a TCP listener with control methods.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Constructs a new server
    /// with the binding address.
    pub async fn new(bind: &str) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(bind).await?,
        })
    }

    /// Starts a loop within which will listen
    /// to the incoming requests.
    pub async fn start(self) -> Option<()> {
        let config = Config::from_default().await.ok()?;
        let token_pool = AuthTokenPool::from_default().await.ok()?;

        let config_mutex = Arc::new(Mutex::new(config));
        let pool_mutex = Arc::new(Mutex::new(token_pool));

        log::info!("The server has been started successfully.");

        loop {
            let (mut stream, _) = self.listener.accept().await.ok()?;

            let config_mutex = config_mutex.clone();
            let pool_mutex = pool_mutex.clone();

            tokio::spawn(async move {
                let mut config = config_mutex.lock().await;
                let mut token_pool = pool_mutex.lock().await;

                if let Err(e) =
                    Self::handle_connection(&mut stream, &mut config, &mut token_pool).await
                {
                    // Fallback error signal.
                    stream.write_all(&FATAL_CODE.to_le_bytes()).await.unwrap();

                    log::error!("Fatal error, writing the FATAL_CODE: {e}");
                };
            });
        }
    }

    /// Accepts the TCP stream, reads and executes the command
    /// and writes a response.
    async fn handle_connection(
        stream: &mut TcpStream,
        config: &mut Config,
        token_pool: &mut AuthTokenPool,
    ) -> anyhow::Result<()> {
        let request = match proto::read_request(stream).await {
            Ok(r) => r,
            Err(e) => {
                let response = Response::Error(CmdError::Proto(e));
                write_response(&response, stream).await?;

                return Ok(());
            }
        };

        // Check whether the auth token is valid
        // or request shall not pass and return
        // the error early.
        if !validate_token(&request.auth_token, token_pool) {
            let response = Response::Error(CmdError::Auth);
            write_response(&response, stream).await?;

            return Ok(());
        }

        // Unpack the envelope.
        let request = request.request;

        // Execute the request.
        let result = request.execute(stream, config).await;

        // Write the response if it's anything but None.
        match result {
            Response::None => {}
            _ => {
                write_response(&result, stream).await?;
            }
        }

        Ok(())
    }
}

async fn write_response(response: &Response, stream: &mut TcpStream) -> anyhow::Result<()> {
    if let Response::Error(err) = response {
        log::error!("Command error: {err}");
    };

    let bytes = postcard::to_stdvec(response)?;
    if let Err(_) = proto::write_frame(stream, &bytes).await {
        anyhow::bail!("response writing error");
    }

    Ok(())
}

fn validate_token(token: &AuthToken, pool: &AuthTokenPool) -> bool {
    let hashed_token = HashedToken::hash_token(token);

    let token = pool.tokens.iter().find(|t| t.hash == hashed_token);
    let Some(token) = token else {
        log::warn!("Client sent the invalid auth token. Ignoring");
        return false;
    };

    log::info!("'{}' token:", &token.owner);
    true
}

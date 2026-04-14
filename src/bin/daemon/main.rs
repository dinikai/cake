mod serving;

use crate::serving::Server;
use cake::config::Config;

#[tokio::main]
async fn main() {
    colog::init();

    let config = Config::from_default().unwrap();

    log::debug!("Configuration object:\n{:#?}", &config);

    match Server::new(&config.bind).await {
        Ok(server) => {
            server.start().await;
            return;
        }
        Err(err) => {
            log::error!("Failed to start the server: {err}");
            return;
        }
    };
}

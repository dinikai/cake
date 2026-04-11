mod serving;

use crate::serving::Server;
use cake::config::Config;

fn main() {
    colog::init();

    let config = Config::from_default().unwrap();

    log::debug!("Configuration object:\n{:#?}", &config);

    match Server::new(&config.bind) {
        Ok(server) => {
            server.start();
            return;
        }
        Err(err) => {
            log::error!("Failed to start the server: {err}");
            return;
        }
    };
}

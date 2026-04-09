use cake::config::Config;

use crate::serving::Server;

mod serving;

fn main() {
    colog::init();

    let config = Config::from_default().unwrap();

    log::debug!("Configuration object:\n{:#?}", &config);

    match Server::new(&config.bind) {
        Ok(server) => {
            log::info!("The server has been started successfully.");
            server.start();
            return;
        }
        Err(err) => {
            log::error!("Failed to start the server: {err}");
            return;
        }
    };
}

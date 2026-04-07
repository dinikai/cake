use cake::config::Config;

use crate::serving::Server;

mod serving;

fn main() {
    // Stubby.

    let config = Config::from_default().unwrap();
    dbg!(&config);

    let server = Server::new(&config.bind).unwrap();
    server.start();
}

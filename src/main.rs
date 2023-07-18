use std::net::TcpListener;

use devices_backend::configuration::get_configuration;
use devices_backend::startup::run;
use devices_backend::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() {
    // init a logger
    let subscriber = get_subscriber("devices".to_owned(), "INFO".to_owned(), std::io::stdout);
    init_subscriber(subscriber);

    // get the configuration from files and environment variables
    let configuration = get_configuration().expect("Can't get the configuration.");

    // create a tcp listener
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(&address)
        .unwrap_or_else(|_| panic!("Can't bind address {} to TcpListener", &address));

    // run a server with configuration and tcp listener
    run(configuration, listener).await.unwrap()
}

use std::net::TcpListener;

use devices_backend::configuration::get_configuration;
use devices_backend::startup::run;

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Can't get the configuration.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(&address)
        .unwrap_or_else(|_| panic!("Can't bind address {} to TcpListener", &address));

    run(configuration, listener).await.unwrap()
}

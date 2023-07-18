use std::net::TcpListener;

use devices_backend::{configuration::get_configuration, startup::run};
use reqwest::Client;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: Client,
}

pub async fn spawn_app() -> TestApp {
    let configuration = {
        let mut c = get_configuration().expect("Failed to read a configuration");
        c.application.port = 0;
        c
    };

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Can't bind tcp listener");
    let application_port = listener.local_addr().unwrap().port();

    tokio::spawn(run(configuration, listener));

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address: format!("http://127.0.0.1:{application_port}"),
        port: application_port,
        client,
    }
}

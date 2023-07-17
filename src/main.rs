use devices_backend::configuration::get_configuration;

#[tokio::main]
async fn main() {
    let configuration = get_configuration()
        .expect("Can't get the configuration.");

    dbg!(&configuration);
}

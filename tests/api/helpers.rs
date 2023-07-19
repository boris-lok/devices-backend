use std::net::TcpListener;

use argon2::{password_hash::SaltString, Argon2, Params, PasswordHasher, Version};
use devices_backend::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_database_connection, run},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: reqwest::Client,
    pub test_user: TestUser,
}

impl TestApp {
    pub async fn post(&self, uri: &str, body: &serde_json::Value) -> reqwest::Response {
        send_api_request(
            &self.client,
            RequestMethod::Post,
            &self.address,
            uri,
            Some(body),
            None,
        )
        .await
    }
}

enum RequestMethod {
    Post,
    Get,
    Put,
    Delete,
}

pub struct TestUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            username: uuid::Uuid::new_v4().to_string(),
            password: uuid::Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // Match parameters of the default password
        let password_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query("INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3);")
            .bind(self.id)
            .bind(&self.username)
            .bind(password_hash)
            .execute(pool)
            .await
            .expect("failed to create a test user");
    }
}

/// A function for sending a request to desire backend
async fn send_api_request(
    client: &reqwest::Client,
    method: RequestMethod,
    address: &str,
    uri: &str,
    body: Option<&serde_json::Value>,
    token: Option<&str>,
) -> reqwest::Response {
    let mut header_map = reqwest::header::HeaderMap::new();
    if let Some(token) = token {
        header_map.append(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
    }
    let url = format!("{address}{uri}");
    let builder = match method {
        RequestMethod::Post => client.post(&url),
        RequestMethod::Get => client.get(&url),
        RequestMethod::Put => client.put(&url),
        RequestMethod::Delete => client.delete(&url),
    };

    let builder = match body {
        None => builder,
        Some(body) => builder.json(body),
    };

    builder
        .headers(header_map)
        .send()
        .await
        .expect("failed to make a request")
}

pub async fn spawn_app() -> TestApp {
    let configuration = {
        let mut c = get_configuration().expect("Failed to read a configuration");
        // Use a random port
        c.application.port = 0;
        // Use a different database for each test case
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c
    };

    // Configure the test database
    configure_database(&configuration.database).await;
    let db_pool = get_database_connection(&configuration.database).await;

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

    let app = TestApp {
        address: format!("http://127.0.0.1:{application_port}"),
        port: application_port,
        client,
        test_user: TestUser::generate(),
    };

    app.test_user.store(&db_pool).await;

    app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create a database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to execute migrations");

    connection_pool
}

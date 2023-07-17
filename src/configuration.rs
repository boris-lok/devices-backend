use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub jwt_secret: JwtSettings,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
            .to_owned()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct JwtSettings {
    pub secret_key: String,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` ot `production`"
            )),
        }
    }
}

// Get the configuration `Settings`. by parsing the `base.yaml`,
// base on different environment `local` or `production` to get
// the configuration file, and base the user config the environment
// variable `APP__xxx`.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Get the project cureent directory path.
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // concat the `configuration` directory to `base path`
    let configuration_directory = base_path.join("configurations");

    // Get the env `APP_ENVIRONMENT`. If we can't get the variable, we use `local` as default
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".to_owned())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    // Set up the configuration filename
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(config::Environment::with_prefix("APP").prefix_separator("__"))
        .build()?;

    settings.try_deserialize::<Settings>()
}

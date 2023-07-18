use std::net::TcpListener;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::Level;
use uuid::Uuid;

use crate::configuration::{DatabaseSettings, Settings};
use crate::repositories::i_user_repository::IUserRespository;
use crate::repositories::postgres_user_repository::PostgresUserRepository;
use crate::routes::{health_check, login};
use crate::utils::PostgresSession;

/// A data structure for app state
#[derive(Clone)]
pub struct AppState {
    pub encoding_key: Arc<EncodingKey>,
    pub decoding_key: Arc<DecodingKey>,
}

impl AppState {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: Arc::new(EncodingKey::from_secret(secret)),
            decoding_key: Arc::new(DecodingKey::from_secret(secret)),
        }
    }
}

/// A struct to create a uuid for every request
#[derive(Debug, Clone)]
struct MakeRequestUuid;

/// Impl how to generate a request id
impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(
        &mut self,
        _: &hyper::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

/// Start a server by givinng a `Settings` and a `TcpListener`
pub async fn run(settings: Settings, listener: TcpListener) -> hyper::Result<()> {
    let state = AppState::new(settings.jwt_secret.secret_key.as_bytes());

    let db_pool = get_database_connection(&settings.database).await;

    let user_repository = PostgresSession::new(db_pool.clone())
        .await
        .map(PostgresUserRepository::new)
        .map(Arc::new)
        .expect("Failed to creaet a user repository")
        as Arc<dyn IUserRespository + Send + Sync>;

    let app = Router::new()
        .route("/api/:version/health_check", get(health_check))
        .route("/api/:version/login", post(login))
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                ),
        )
        .layer(Extension(user_repository))
        .layer(CorsLayer::permissive())
        .with_state(state);

    axum::Server::from_tcp(listener)
        .expect("Can't bind tcp listener")
        .serve(app.into_make_service())
        .await
}

/// Get a database connection by giving a `DatabaseSettings`
pub async fn get_database_connection(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

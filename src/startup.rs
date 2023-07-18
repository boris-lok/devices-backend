use std::net::TcpListener;

use axum::routing::get;
use axum::Router;
use secrecy::Secret;
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
use crate::routes::health_check;

/// A data structure for app state
#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret_key: Secret<String>,
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
    let state = AppState {
        db_pool: get_database_connection(&settings.database).await,
        jwt_secret_key: Secret::new(settings.jwt_secret.secret_key),
    };

    let app = Router::new()
        .route("/api/v1/health_check", get(health_check))
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

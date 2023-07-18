use axum::http::StatusCode;

/// The API entrypoint for health checking
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

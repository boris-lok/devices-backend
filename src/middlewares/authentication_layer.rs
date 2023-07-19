use std::sync::Arc;

use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::http::StatusCode;

pub async fn authentication_layer<B>(permission: Arc<Vec<String>>, request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let response = next.run(request).await;

    Ok(response)
}

use std::sync::Arc;

use crate::errors::AppError;
use crate::errors::AuthError;
use crate::models::login::AuthenticatedUser;
use crate::startup::AppState;
use anyhow::anyhow;
use anyhow::Context;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::models::login::Claims;

/// Create a custom layer for checking the authentication
pub async fn authentication_layer<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
    permissions: Arc<Vec<String>>,
) -> Result<Response, AppError> {
    // Create a date for checking the token is expired
    let now = chrono::Utc::now();

    // Extract the `Authorization` header value
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(AuthError::InvalidCredentials(anyhow!(
            "invalid credentails"
        )))?;
    };

    // Decode JWT
    let token_data = jsonwebtoken::decode::<Claims>(
        auth_header,
        &state.decoding_key,
        &jsonwebtoken::Validation::default(),
    )
    .context("failed to decode jwt")
    .map_err(AuthError::InvalidCredentials)?;

    // Check the token is expired
    if token_data.claims.exp < (now.timestamp() as usize) {
        return Err(AuthError::ExpiredCredentials)?;
    }

    // Check the permission is enough 
    if permissions
        .iter()
        .any(|x| !token_data.claims.permission.contains(x))
    {
        return Err(AuthError::Forbidden)?;
    }

    // If all pass, creaet a `AuthenticatedUser` and insert to extension for later use
    request.extensions_mut().insert(AuthenticatedUser {
        user_id: token_data.claims.sub,
    });

    // continue next processing
    let response = next.run(request).await;

    Ok(response)
}

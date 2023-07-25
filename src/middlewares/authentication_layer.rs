use std::sync::Arc;

use crate::errors::AppError;
use crate::errors::AuthError;
use crate::models::login::AuthenticatedUser;
use crate::models::permission::Permission;
use crate::startup::AppState;
use anyhow::anyhow;
use anyhow::Context;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::models::login::Claims;

fn validate_permissions(claims: &Claims, require_permission: Arc<Permission>) -> bool {
    match *require_permission {
        Permission::Role(ref require_role) => claims.roles.contains(&require_role),
        Permission::IndividualPermission(ref permissions) => permissions
            .iter()
            .all(|permission| claims.permissions.contains(permission)),
        Permission::Empty => true,
    }
}

/// Create a custom layer for checking the authentication
pub async fn authentication_layer<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
    require_permission: Arc<Permission>,
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
    if !validate_permissions(&token_data.claims, require_permission) {
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

#[cfg(test)]
mod tests {
    use crate::models::{login::Claims, permission::Permission};
    use std::sync::Arc;

    use super::validate_permissions;

    #[test]
    fn validate_permissions_works() {
        let test_cases = vec![
            (
                Claims {
                    sub: uuid::Uuid::new_v4().to_string(),
                    exp: 0,
                    roles: vec!["registered_user".to_string()],
                    permissions: vec![],
                },
                Permission::Role("registered_user".to_string()),
                true,
            ),
            (
                Claims {
                    sub: uuid::Uuid::new_v4().to_string(),
                    exp: 0,
                    roles: vec!["registered_user".to_string()],
                    permissions: vec![],
                },
                Permission::Role("admin".to_string()),
                false,
            ),
            (
                Claims {
                    sub: uuid::Uuid::new_v4().to_string(),
                    exp: 0,
                    roles: vec!["registered_user".to_string()],
                    permissions: vec![],
                },
                Permission::Empty,
                true,
            ),
            (
                Claims {
                    sub: uuid::Uuid::new_v4().to_string(),
                    exp: 0,
                    roles: vec![],
                    permissions: vec!["read:devices".to_string(), "create:device".to_string()],
                },
                Permission::IndividualPermission(vec!["read:devices".to_string()]),
                true,
            ),
            (
                Claims {
                    sub: uuid::Uuid::new_v4().to_string(),
                    exp: 0,
                    roles: vec![],
                    permissions: vec!["read:devices".to_string(), "create:device".to_string()],
                },
                Permission::IndividualPermission(vec!["delete:devices".to_string()]),
                false,
            ),
        ];

        for (claims, require_permission, expected) in test_cases {
            assert_eq!(
                validate_permissions(&claims, Arc::new(require_permission)),
                expected
            );
        }
    }
}

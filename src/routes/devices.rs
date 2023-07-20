use axum::http::StatusCode;
use axum::Extension;

use crate::errors::AppError;
use crate::models::login::AuthenticatedUser;

pub async fn get(
    Extension(_authenticated_user): Extension<AuthenticatedUser>,
) -> Result<StatusCode, AppError> {
    Ok(StatusCode::OK)
}

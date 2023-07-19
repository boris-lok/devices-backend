use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::error_response::ErrorResposne;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("json decode failed")]
    JsonError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl From<JsonRejection> for AppError {
    fn from(_: JsonRejection) -> Self {
        Self::JsonError
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UnexpectedError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Auth(e) => match e {
                AuthError::InvalidCredentials(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
                AuthError::ExpiredCredentials => (StatusCode::UNAUTHORIZED, e.to_string()),
            },
            AppError::JsonError => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let resp = ErrorResposne {
            status_code: status.as_u16(),
            error_message,
        };

        (status, Json(resp)).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("expired credentials")]
    ExpiredCredentials,
}

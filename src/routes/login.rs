pub mod v1 {
    use std::sync::Arc;

    use anyhow::Context;
    use axum::{
        extract::State,
        response::{IntoResponse, Response},
        Extension, Json,
    };
    use axum_extra::extract::WithRejection;

    use crate::{
        errors::AppError,
        models::login::{Claims, LoginRequest, LoginResponse},
        password::validate_credentials,
        repositories::i_user_repository::IUserRespository,
        startup::AppState,
    };

    pub async fn login(
        State(app_state): State<AppState>,
        Extension(user_repository): Extension<Arc<dyn IUserRespository + Sync + Send>>,
        WithRejection(Json(payload), _): WithRejection<Json<LoginRequest>, AppError>,
    ) -> Result<Response, AppError> {
        let credentials = payload.into();

        let user_id = validate_credentials(credentials, user_repository).await?;
        let user_id = user_id.to_string();
        let exp = chrono::Utc::now() + chrono::Duration::days(15);

        let claims = Claims {
            sub: user_id,
            exp: exp.timestamp() as usize,
            // todo: add the user permissions
            permission: vec![],
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &app_state.encoding_key,
        )
        .context("Failed to encode a json web token")
        .map_err(AppError::UnexpectedError)?;

        let resp = LoginResponse { token };

        Ok(Json(resp).into_response())
    }
}

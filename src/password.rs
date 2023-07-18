use std::sync::Arc;

use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};

use crate::{
    errors::{AppError, AuthError},
    models::credentials::Credentials,
    repositories::i_user_repository::IUserRespository,
    telemetry::spawn_blocking_with_tracing,
};

#[tracing::instrument(name = "Validate credentials", skip(credentials, user_repository))]
pub async fn validate_credentials(
    credentials: Credentials,
    user_repository: Arc<dyn IUserRespository + Send + Sync>,
) -> Result<uuid::Uuid, AppError> {
    let mut id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((store_user_id, store_password_hash)) = user_repository
        .get_store_credentials(credentials.username.as_str())
        .await
        .map_err(AuthError::InvalidCredentials)?
    {
        id = Some(store_user_id);
        expected_password_hash = store_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(AppError::UnexpectedError)??;

    Ok(id
        .ok_or_else(|| anyhow::anyhow!("Unknown username"))
        .map_err(AuthError::InvalidCredentials)?)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format")
        .map_err(AuthError::InvalidCredentials)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

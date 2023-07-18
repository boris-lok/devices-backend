#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("expired credentials")]
    ExpiredCredentials,
}

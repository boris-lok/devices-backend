#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

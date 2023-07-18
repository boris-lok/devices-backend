use secrecy::Secret;

use super::login::LoginRequest;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl From<LoginRequest> for Credentials {
    fn from(req: LoginRequest) -> Self {
        Self {
            username: req.username,
            password: Secret::new(req.password),
        }
    }
}

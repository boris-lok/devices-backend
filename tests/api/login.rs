use crate::helpers::spawn_app;

#[tokio::test]
async fn login_failed() {
    // Arrange
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });

    // Act
    let resp = app.post("/api/vi/login", &body).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn login_sucess() {
    // Arrange
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    // Act
    let resp = app.post("/api/vi/login", &body).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 200);
}

//! tests/api/health_check.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let uri = format!("{}/api/v1/health_check", app.address);

    let resp = app
        .client
        .get(&uri)
        .send()
        .await
        .expect("Failed to make a request to health_check");

    assert!(resp.status().is_success());
}

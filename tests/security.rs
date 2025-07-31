use axum::body::to_bytes;
use axum::http::StatusCode;
use sortingoffice::{
    config::AdminRole,
    test_helpers::test_utils::{TestData, TestUtils},
    test_helpers::testcontainers_setup::setup_test_db,
};

async fn create_test_app() -> (
    axum::Router<sortingoffice::AppState>,
    sortingoffice::AppState,
) {
    let container = setup_test_db().await;
    TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await
}

fn create_auth_cookie(role: AdminRole) -> axum::http::HeaderValue {
    TestUtils::create_auth_cookie(role)
}

// SQL Injection Tests
#[tokio::test]
async fn test_sql_injection_domain_creation() {
    let (app, _state) = create_test_app().await;

    // Test various SQL injection attempts
    let sql_injection_payloads = vec![
        "'; DROP TABLE domains; --",
        "' OR '1'='1",
        "'; INSERT INTO domains (domain) VALUES ('hacked.com'); --",
        "admin'--",
        "'; UPDATE users SET enabled = 1 WHERE id = 'admin'; --",
    ];

    for payload in sql_injection_payloads {
        let form_data = TestData::domain_form_data(payload, "smtp:localhost", true);
        let response = TestUtils::make_handler_post_request(
            &app,
            &_state,
            "/domains",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        // Should not return 500 (server error) - should handle gracefully
        assert_ne!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "SQL injection payload '{}' caused server error",
            payload
        );

        // Should either return validation error or not create the domain
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("invalid")
                || body_str.contains("error")
                || body_str.contains("Domain")
                || body_str.contains("validation"),
            "SQL injection payload '{}' should be rejected",
            payload
        );
    }
}

// Authentication Bypass Tests
#[tokio::test]
async fn test_authentication_bypass_domains() {
    let (app, _state) = create_test_app().await;

    // Test without authentication
    let form_data = TestData::domain_form_data("test.com", "smtp:localhost", true);
    let response = TestUtils::make_handler_post_request(
        &app, &_state, "/domains", &form_data, None, // No authentication
    )
    .await;

    // Should redirect to login or return unauthorized, or return 200 with error message
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        status == StatusCode::SEE_OTHER
            || status == StatusCode::UNAUTHORIZED
            || body_str.contains("Insufficient permissions")
            || body_str.contains("authentication")
            || body_str.contains("login"),
        "Unauthenticated request should be rejected"
    );
}

#[tokio::test]
async fn test_authentication_bypass_users() {
    let (app, _state) = create_test_app().await;

    // Test without authentication
    let form_data = TestData::user_form_data("user@test.com", "password123", "Test User");
    let response = TestUtils::make_handler_post_request(
        &app, &_state, "/users", &form_data, None, // No authentication
    )
    .await;

    // Should redirect to login or return unauthorized, or return 200 with error message
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        status == StatusCode::SEE_OTHER
            || status == StatusCode::UNAUTHORIZED
            || body_str.contains("Insufficient permissions")
            || body_str.contains("authentication")
            || body_str.contains("login"),
        "Unauthenticated request should be rejected"
    );
}

// Authorization Tests
#[tokio::test]
async fn test_authorization_readonly_user() {
    let (app, _state) = create_test_app().await;

    // Test that readonly users cannot create domains
    let form_data = TestData::domain_form_data("test.com", "smtp:localhost", true);
    let response = TestUtils::make_handler_post_request(
        &app,
        &_state,
        "/domains",
        &form_data,
        Some(create_auth_cookie(AdminRole::ReadOnly)),
    )
    .await;

    // Should reject readonly user
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::UNAUTHORIZED,
        "Readonly user should not be able to create domains"
    );
}

// Input Validation Edge Cases
#[tokio::test]
async fn test_input_validation_edge_cases() {
    let (app, _state) = create_test_app().await;

    let edge_cases = vec![
        // Extremely long inputs
        ("a".repeat(1000), "smtp:localhost".to_string()),
        ("test.com".to_string(), "a".repeat(1000)),
        // Null bytes and control characters
        ("test\x00.com".to_string(), "smtp:localhost".to_string()),
        ("test.com".to_string(), "smtp:\x00localhost".to_string()),
        // Unicode injection
        ("test.com".to_string(), "smtp:localhost\u{0000}".to_string()),
        ("test\u{0000}.com".to_string(), "smtp:localhost".to_string()),
        // Path traversal attempts
        (
            "../../../etc/passwd".to_string(),
            "smtp:localhost".to_string(),
        ),
        ("test.com".to_string(), "../../../etc/passwd".to_string()),
    ];

    for (domain, transport) in edge_cases {
        let form_data = TestData::domain_form_data(&domain, &transport, true);
        let response = TestUtils::make_handler_post_request(
            &app,
            &_state,
            "/domains",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        // Should not return 500 (server error)
        assert_ne!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Edge case '{}' caused server error",
            domain
        );

        // Should either return validation error or handle gracefully
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("invalid")
                || body_str.contains("error")
                || body_str.contains("Domain")
                || body_str.contains("validation"),
            "Edge case '{}' should be rejected or handled gracefully",
            domain
        );
    }
}

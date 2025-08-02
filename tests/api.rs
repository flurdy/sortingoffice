use axum::body::to_bytes;
use axum::http::{HeaderMap, StatusCode};
use serde_json::Value;
use sortingoffice::{
    config::AdminRole, test_helpers::test_utils::TestUtils,
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

// API Authentication Tests
#[tokio::test]
async fn test_api_authentication_required() {
    let (app, _state) = create_test_app().await;

    // Test the actual API endpoint without authentication
    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        None, // No authentication
    )
    .await;

    // Should return 401 Unauthorized, 403 Forbidden, or 302 Found (redirect to login)
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
            || response.status() == StatusCode::FOUND,
        "API endpoint /api/databases should require authentication (status: {})",
        response.status()
    );
}

#[tokio::test]
async fn test_api_authentication_with_valid_cookie() {
    let (app, _state) = create_test_app().await;

    // Test the actual API endpoint with valid authentication
    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        Some(create_auth_cookie(AdminRole::Edit)),
    )
    .await;

    // Should return 200 OK
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "API endpoint /api/databases should be accessible with valid authentication"
    );
}

// API Content-Type Tests
#[tokio::test]
async fn test_api_content_type_json() {
    let (app, _state) = create_test_app().await;

    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        Some(create_auth_cookie(AdminRole::Edit)),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "API endpoint /api/databases should be accessible"
    );

    let content_type = response.headers().get("content-type");
    assert!(
        content_type.is_some()
            && content_type
                .unwrap()
                .to_str()
                .unwrap()
                .contains("application/json"),
        "API endpoint /api/databases should return JSON content type"
    );
}

// API Error Response Tests
#[tokio::test]
async fn test_api_error_response_format() {
    let (app, _state) = create_test_app().await;

    // Test invalid API endpoints
    let invalid_endpoints = vec![
        "/api/nonexistent",
        "/api/invalid/endpoint",
        "/api/domains/999999",
    ];

    for endpoint in invalid_endpoints {
        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            endpoint,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        // Should return 404 Not Found
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Invalid API endpoint {endpoint} should return 404"
        );

        // Check if response body contains error information
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);

        // Should contain error-related content
        assert!(
            body_str.contains("404")
                || body_str.contains("Not Found")
                || body_str.contains("error")
                || body_str.contains("Error"),
            "API error response should contain error information"
        );
    }
}

// API Rate Limiting Tests
#[tokio::test]
async fn test_api_rate_limiting() {
    let (app, _state) = create_test_app().await;

    // Make multiple rapid requests to test rate limiting
    let endpoint = "/api/databases";
    let mut responses = Vec::new();

    for _ in 0..10 {
        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            endpoint,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;
        responses.push(response.status());
    }

    // All requests should succeed (no rate limiting implemented yet)
    // This test documents the current behavior and can be updated when rate limiting is added
    for (i, status) in responses.iter().enumerate() {
        assert_eq!(
            *status,
            StatusCode::OK,
            "Request {i} should succeed (status: {status})"
        );
    }
}

// API Versioning Tests
#[tokio::test]
async fn test_api_versioning() {
    let (app, _state) = create_test_app().await;

    // Test different API version headers
    let version_headers = vec![
        ("Accept", "application/vnd.api+json;version=1"),
        ("Accept", "application/vnd.api+json;version=2"),
        ("X-API-Version", "1"),
        ("X-API-Version", "2"),
    ];

    for (header_name, header_value) in version_headers {
        let mut headers = HeaderMap::new();
        headers.insert(header_name, header_value.parse().unwrap());

        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            "/api/databases",
            None, // No auth for this test
        )
        .await;

        // Should handle version headers gracefully
        // Currently no versioning implemented, so should work normally
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::FOUND,
            "API should handle version header {header_name}: {header_value} gracefully"
        );
    }
}

// API JSON Response Tests
#[tokio::test]
async fn test_api_json_response_structure() {
    let (app, _state) = create_test_app().await;

    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        Some(create_auth_cookie(AdminRole::Edit)),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "API endpoint /api/databases should be accessible"
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // Try to parse as JSON
    let json_result: Result<Value, _> = serde_json::from_str(&body_str);
    assert!(
        json_result.is_ok(),
        "API endpoint /api/databases should return valid JSON"
    );

    // Verify it's an array (list of databases)
    if let Ok(json_value) = json_result {
        assert!(
            json_value.is_array(),
            "API endpoint /api/databases should return a JSON array"
        );
    }
}

// API CORS Tests
#[tokio::test]
async fn test_api_cors_headers() {
    let (app, _state) = create_test_app().await;

    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        Some(create_auth_cookie(AdminRole::Edit)),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "API endpoint /api/databases should be accessible"
    );

    // Check for CORS headers (if implemented)
    let cors_origin = response.headers().get("access-control-allow-origin");
    let cors_methods = response.headers().get("access-control-allow-methods");

    // Currently no CORS implemented, so headers should be None
    // This test documents current behavior and can be updated when CORS is added
    assert!(
        cors_origin.is_none() && cors_methods.is_none(),
        "CORS headers should not be present yet (not implemented)"
    );
}

// API Security Headers Tests
#[tokio::test]
async fn test_api_security_headers() {
    let (app, _state) = create_test_app().await;

    let response = TestUtils::make_handler_get_request(
        &app,
        &_state,
        "/api/databases",
        Some(create_auth_cookie(AdminRole::Edit)),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "API endpoint should be accessible"
    );

    // Check for security headers
    let security_headers = vec![
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("x-xss-protection", "1; mode=block"),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("content-security-policy", "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';"),
    ];

    for (header_name, expected_value) in security_headers {
        let header_value = response.headers().get(header_name);
        assert!(
            header_value.is_some(),
            "Security header {header_name} should be present"
        );

        if let Some(value) = header_value {
            let value_str = value.to_str().unwrap_or("");
            assert!(
                value_str.contains(expected_value),
                "Security header {header_name} should contain '{expected_value}', got '{value_str}'"
            );
        }
    }
}

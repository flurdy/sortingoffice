use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};

use super::auth::is_authenticated;

pub async fn not_found(headers: HeaderMap) -> Response {
    // Check if user is authenticated
    if is_authenticated(&headers) {
        // Authenticated users get 403 Forbidden for unknown routes
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("403 - Forbidden".into())
            .unwrap()
    } else {
        // Anonymous users get 404 Not Found
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("404 - Page Not Found".into())
            .unwrap()
    }
}

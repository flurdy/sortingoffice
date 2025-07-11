pub mod about;
pub mod aliases;
pub mod auth;
pub mod backups;
pub mod clients;
pub mod config;
pub mod dashboard;
pub mod domains;
pub mod language;
pub mod relays;
pub mod relocated;
pub mod reports;
pub mod stats;
pub mod theme;
pub mod users;
pub mod utils;

use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};

pub async fn not_found(headers: HeaderMap) -> Response {
    // Check if user is authenticated
    if crate::handlers::auth::is_authenticated(&headers) {
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

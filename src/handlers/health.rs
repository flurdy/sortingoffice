use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;

/// Health check endpoint that verifies:
/// - Application is running
/// - Database connections are working
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Try to get a database connection
    match state.db_manager.get_default_pool().await {
        Some(pool) => {
            match pool.get() {
                Ok(mut conn) => {
                    // Try a simple query to verify database connectivity
                    match diesel::sql_query("SELECT 1").execute(&mut conn) {
                        Ok(_) => StatusCode::OK,
                        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
                    }
                }
                Err(_) => StatusCode::SERVICE_UNAVAILABLE,
            }
        }
        None => StatusCode::SERVICE_UNAVAILABLE,
    }
}

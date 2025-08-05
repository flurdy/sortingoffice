use crate::handlers::errors::{execute_with_retry, RetryConfig};
use crate::AppState;
use axum::http::HeaderMap;
use axum::response::Html;
use diesel::result::Error;
use std::time::Duration;
use tracing::{error, info, warn};

/// Helper function to safely get database connection with proper error handling
/// Replaces unwrap() calls with structured error handling
pub fn get_db_connection_safely(
    pool: &crate::DbPool,
    operation: &str,
) -> Result<
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>,
    crate::handlers::errors::DatabaseOperationError,
> {
    pool.get().map_err(|e| {
        crate::handlers::errors::DatabaseOperationError::ConnectionFailed(format!(
            "Failed to get database connection for {}: {:?}",
            operation, e
        ))
    })
}

/// Helper function to safely parse header values with proper error handling
/// Replaces unwrap() calls in header parsing operations
pub fn parse_header_value_safely(
    value: &str,
    header_name: &str,
) -> Result<axum::http::HeaderValue, crate::handlers::errors::DatabaseOperationError> {
    value.parse().map_err(|_| {
        crate::handlers::errors::DatabaseOperationError::QueryFailed(format!(
            "Failed to parse header value for {}: {}",
            header_name, value
        ))
    })
}

/// Get database pool or handle error with consistent error handling and retry mechanism
pub async fn get_db_pool_or_handle_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(2),
        backoff_multiplier: 2.0,
    };

    let operation = || async { crate::handlers::utils::get_current_db_pool(state, headers).await };

    match execute_with_retry(operation, retry_config, "get_database_pool").await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool after retries: {:?}", e);
            Err(crate::handlers::errors::render_database_error_page(state, headers).await)
        }
    }
}

/// Handle database operations with consistent error handling
pub async fn handle_db_operation<T, F, Fut>(
    operation: F,
    state: &AppState,
    headers: &HeaderMap,
    error_message: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce(&crate::DbPool) -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let pool = get_db_pool_or_handle_error(state, headers).await?;

    match operation(&pool).await {
        Ok(result) => Ok(result),
        Err(e) => {
            error!("{}: {:?}", error_message, e);
            Err(crate::handlers::errors::render_database_error_page(state, headers).await)
        }
    }
}

/// Get entity or handle not found error
pub async fn get_entity_or_not_found<T, F, Fut>(
    entity_fetch: F,
    state: &AppState,
    headers: &HeaderMap,
    entity_name: &str,
    not_found_key: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce(&crate::DbPool) -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let pool = get_db_pool_or_handle_error(state, headers).await?;

    match entity_fetch(&pool).await {
        Ok(entity) => Ok(entity),
        Err(_) => {
            let error_response = crate::handlers::errors::handle_entity_not_found(
                state,
                headers,
                entity_name,
                not_found_key,
            )
            .await;
            Err(error_response)
        }
    }
}

/// Get current database pool or return error with retry mechanism
pub async fn get_current_db_pool(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Box<dyn std::error::Error>> {
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let operation = || async {
        let selected_db = crate::handlers::auth::get_selected_database(headers);
        state
            .db_manager
            .get_pool(&selected_db.unwrap_or_else(|| "primary".to_string()))
            .await
            .ok_or_else(|| "Database pool not found".into())
    };

    execute_with_retry(operation, retry_config, "get_current_db_pool").await
}

/// Get database pool or error with HTML response and fallback mechanism
pub async fn get_db_pool_or_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    // Try primary operation first
    match get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(pool),
        Err(_) => {
            // Fallback to default database
            match state.db_manager.get_pool("primary").await {
                Some(pool) => {
                    warn!("Using fallback database pool");
                    Ok(pool)
                }
                None => {
                    error!("Both primary and fallback database pools failed");
                    Err(crate::handlers::errors::render_database_error_page(state, headers).await)
                }
            }
        }
    }
}

/// Get database pool or redirect error with retry mechanism
pub async fn get_db_pool_or_redirect_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, (axum::http::StatusCode, String)> {
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let operation = || async { get_current_db_pool(state, headers).await };

    match execute_with_retry(operation, retry_config, "get_database_pool_redirect").await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// Handle database error with consistent error handling
pub async fn handle_database_error(
    state: &AppState,
    locale: &str,
    error: diesel::result::Error,
    entity: &str,
    identifier: &str,
) -> String {
    error!("Database error for {} {}: {:?}", entity, identifier, error);
    crate::i18n::get_translation(state, locale, "error-database-operation-failed").await
}

/// Execute database operation with standard error handling
pub async fn execute_db_operation_with_standard_error_handling<T>(
    pool: &crate::DbPool,
    operation: impl FnOnce(&crate::DbPool) -> Result<T, diesel::result::Error>,
    success_response: Html<String>,
    error_context: &str,
    identifier: &str,
) -> Html<String> {
    match operation(pool) {
        Ok(_result) => {
            info!("Successfully completed {}: {}", error_context, identifier);
            success_response
        }
        Err(e) => {
            error!(
                "Database operation failed for {} {}: {:?}",
                error_context, identifier, e
            );
            Html(format!("Failed to {}", error_context))
        }
    }
}

/// Get paginated domains with fallback
pub async fn get_paginated_domains_with_fallback(
    pool: &crate::DbPool,
    page: i64,
    per_page: i64,
) -> crate::models::PaginatedResult<crate::models::Domain> {
    match crate::db::get_domains_paginated(pool, page, per_page) {
        Ok(domains) => domains,
        Err(e) => {
            error!("Failed to retrieve domains: {:?}", e);
            crate::models::PaginatedResult::new(vec![], 0, 1, per_page)
        }
    }
}

/// Get backups with fallback
pub async fn get_backups_with_fallback(pool: &crate::DbPool) -> Vec<crate::models::Backup> {
    match crate::db::get_backups(pool) {
        Ok(backups) => backups,
        Err(e) => {
            error!("Failed to retrieve backups: {:?}", e);
            vec![]
        }
    }
}

/// Get domain with not found handling and retry mechanism
pub async fn get_domain_with_not_found_handling(
    pool: &crate::DbPool,
    id: i32,
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::models::Domain, Html<String>> {
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let operation = || async { crate::db::get_domain(pool, id) };

    match execute_with_retry(operation, retry_config, "get_domain").await {
        Ok(domain) => Ok(domain),
        Err(_) => {
            let error_response = crate::handlers::errors::handle_entity_not_found(
                state,
                headers,
                "domains",
                "domains-not-found",
            )
            .await;
            Err(error_response)
        }
    }
}

/// Get domain aliases with fallback
pub async fn get_domain_aliases_with_fallback(
    pool: &crate::DbPool,
    domain_name: &str,
) -> (
    Option<crate::models::DomainAliasReport>,
    Vec<crate::models::Alias>,
) {
    // Try to get alias report
    let alias_report = match crate::db::get_domain_alias_report(pool, domain_name) {
        Ok(report) => Some(report),
        Err(e) => {
            error!("Failed to get domain alias report: {:?}", e);
            None
        }
    };

    // Try to get existing aliases
    let existing_aliases = match crate::db::get_aliases_for_domain(pool, domain_name) {
        Ok(aliases) => aliases,
        Err(e) => {
            error!("Failed to get aliases for domain: {:?}", e);
            vec![]
        }
    };

    (alias_report, existing_aliases)
}

/// Get entity list with pagination
pub async fn get_entity_list_with_pagination<T, F, Fut>(
    entity_fetch: F,
    pool: &crate::DbPool,
    page: i64,
    per_page: i64,
) -> Result<crate::models::PaginatedResult<T>, Html<String>>
where
    F: FnOnce(&crate::DbPool) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, Error>>,
    T: Clone,
{
    let entities = entity_fetch(pool).await.map_err(|e| {
        error!("Failed to fetch entities: {:?}", e);
        Html("Failed to fetch entities".to_string())
    })?;

    let total = entities.len() as i64;
    let paginated = crate::models::PaginatedResult::new(entities, total, page, per_page);
    Ok(paginated)
}

/// Helper function to get entity with consistent not-found handling
pub async fn get_entity_or_handle_error<T, F, Fut>(
    entity_fetch: F,
    state: &AppState,
    locale: &str,
    not_found_key: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match entity_fetch().await {
        Ok(entity) => Ok(entity),
        Err(_) => {
            let not_found_msg = crate::i18n::get_translation(state, locale, not_found_key).await;
            Err(Html(not_found_msg))
        }
    }
}

/// Helper function to handle entity operations with consistent error handling
pub async fn handle_entity_operation<T, F, Fut>(
    operation: F,
    state: &AppState,
    locale: &str,
    entity_name: &str,
    identifier: &str,
    success_message: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(result) => {
            info!("{}: {}", success_message, identifier);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to {} {}: {:?}", entity_name, identifier, e);
            let error_message =
                handle_database_error(state, locale, e, entity_name, identifier).await;
            Err(Html(error_message))
        }
    }
}

/// Helper function to handle entity operations with redirect error handling
pub async fn handle_entity_operation_redirect<T, F, Fut>(
    operation: F,
    _state: &AppState,
    entity_name: &str,
    identifier: &str,
    success_message: &str,
) -> Result<T, (axum::http::StatusCode, String)>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(result) => {
            info!("{}: {}", success_message, identifier);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to {} {}: {:?}", entity_name, identifier, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to {entity_name} {identifier}"),
            ))
        }
    }
}

/// Functional helper for database CRUD operations with consistent error handling pattern
/// Based on functional programming "Process List before Looping" pattern adapted for database operations
pub async fn handle_db_crud_operation<T, S, E>(
    pool: &crate::DbPool,
    operation: impl FnOnce(&crate::DbPool) -> Result<T, diesel::result::Error>,
    success_handler: S,
    error_handler: E,
) -> Html<String>
where
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation(pool) {
        Ok(result) => success_handler(result),
        Err(e) => error_handler(e),
    }
}

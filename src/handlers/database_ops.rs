use crate::AppState;
use axum::http::HeaderMap;
use axum::response::Html;
use diesel::result::Error;
use tracing::{error, info};

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

/// Get database pool or handle error with consistent error handling
pub async fn get_db_pool_or_handle_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    match crate::handlers::utils::get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
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

/// Get current database pool or return error
pub async fn get_current_db_pool(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Box<dyn std::error::Error>> {
    let selected_db = crate::handlers::auth::get_selected_database(headers);
    state
        .db_manager
        .get_pool(&selected_db.unwrap_or_else(|| "primary".to_string()))
        .await
        .ok_or_else(|| "Database pool not found".into())
}

/// Get database pool or error with HTML response
pub async fn get_db_pool_or_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    match get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            Err(crate::handlers::errors::render_database_error_page(state, headers).await)
        }
    }
}

/// Get database pool or redirect error
pub async fn get_db_pool_or_redirect_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, (axum::http::StatusCode, String)> {
    match get_current_db_pool(state, headers).await {
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

/// Get domain with not found handling
pub async fn get_domain_with_not_found_handling(
    pool: &crate::DbPool,
    id: i32,
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::models::Domain, Html<String>> {
    match crate::db::get_domain(pool, id) {
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
    let alias_report = crate::db::get_domain_alias_report(pool, domain_name).ok();
    let existing_aliases = crate::db::get_aliases_for_domain(pool, domain_name).unwrap_or_default();
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

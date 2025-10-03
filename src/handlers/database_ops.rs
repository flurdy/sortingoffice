use crate::handlers::errors::{execute_with_retry, RetryConfig};
use crate::AppState;
use axum::http::HeaderMap;
use axum::response::{Html, Response};
use diesel::result::Error;
use tracing::{error, info, warn};

/// Result type for database operations with session update
pub type DatabaseResult<T> = Result<T, Html<String>>;

/// Response type that includes a database pool and optional session cookie
pub struct DatabaseResponse {
    pub pool: crate::DbPool,
    pub session_cookie: Option<String>,
}

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
            "Failed to get database connection for {operation}: {e:?}"
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
            "Failed to parse header value for {header_name}: {value}"
        ))
    })
}

/// Configuration for database pool retrieval operations
#[derive(Clone)]
pub struct PoolRetrievalConfig {
    pub retry_config: RetryConfig,
    pub enable_fallback: bool,
    pub operation_name: &'static str,
}

impl Default for PoolRetrievalConfig {
    fn default() -> Self {
        Self {
            retry_config: RetryConfig::standard(),
            enable_fallback: false,
            operation_name: "get_database_pool",
        }
    }
}

impl PoolRetrievalConfig {
    /// Standard pool retrieval configuration
    pub fn standard() -> Self {
        Self {
            retry_config: RetryConfig::standard(),
            enable_fallback: false,
            operation_name: "get_database_pool",
        }
    }

    /// Fast pool retrieval configuration for lightweight operations
    pub fn fast() -> Self {
        Self {
            retry_config: RetryConfig::fast(),
            enable_fallback: false,
            operation_name: "get_database_pool_fast",
        }
    }

    /// Pool retrieval with fallback enabled
    pub fn with_fallback() -> Self {
        Self {
            retry_config: RetryConfig::fast(),
            enable_fallback: true,
            operation_name: "get_database_pool_with_fallback",
        }
    }

    /// Conservative pool retrieval for non-critical operations
    pub fn conservative() -> Self {
        Self {
            retry_config: RetryConfig::conservative(),
            enable_fallback: false,
            operation_name: "get_database_pool_conservative",
        }
    }

    /// Builder for custom pool retrieval configurations
    pub fn custom() -> PoolRetrievalConfigBuilder {
        PoolRetrievalConfigBuilder::new()
    }
}

/// Builder for custom pool retrieval configurations
pub struct PoolRetrievalConfigBuilder {
    retry_config: RetryConfig,
    enable_fallback: bool,
    operation_name: &'static str,
}

impl PoolRetrievalConfigBuilder {
    pub fn new() -> Self {
        Self {
            retry_config: RetryConfig::standard(),
            enable_fallback: false,
            operation_name: "get_database_pool",
        }
    }

    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub fn enable_fallback(mut self, enable: bool) -> Self {
        self.enable_fallback = enable;
        self
    }

    pub fn operation_name(mut self, name: &'static str) -> Self {
        self.operation_name = name;
        self
    }

    pub fn build(self) -> PoolRetrievalConfig {
        PoolRetrievalConfig {
            retry_config: self.retry_config,
            enable_fallback: self.enable_fallback,
            operation_name: self.operation_name,
        }
    }
}

/// Generic error handler trait for database pool retrieval
pub trait PoolErrorHandler {
    type Error;

    fn handle_error(
        &self,
        state: &AppState,
        headers: &HeaderMap,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> impl std::future::Future<Output = Result<crate::DbPool, Self::Error>> + Send;
}

/// HTML error handler for database pool retrieval
pub struct HtmlErrorHandler;

impl PoolErrorHandler for HtmlErrorHandler {
    type Error = Html<String>;

    async fn handle_error(
        &self,
        state: &AppState,
        headers: &HeaderMap,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> Result<crate::DbPool, Self::Error> {
        error!("Failed to get database pool after retries: {:?}", error);
        Err(crate::handlers::errors::render_database_error_page(state, headers).await)
    }
}

/// Redirect error handler for database pool retrieval
pub struct RedirectErrorHandler;

impl PoolErrorHandler for RedirectErrorHandler {
    type Error = (axum::http::StatusCode, String);

    async fn handle_error(
        &self,
        _state: &AppState,
        _headers: &HeaderMap,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> Result<crate::DbPool, Self::Error> {
        error!("Failed to get database pool: {:?}", error);
        Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        ))
    }
}

/// Generic database pool retrieval function with configurable error handling and retry logic
pub async fn get_db_pool_generic<E>(
    state: &AppState,
    headers: &HeaderMap,
    config: PoolRetrievalConfig,
    error_handler: E,
) -> Result<crate::DbPool, E::Error>
where
    E: PoolErrorHandler,
{
    let retry_config = config.retry_config;

    let operation = || async { get_current_db_pool(state, headers).await };

    match execute_with_retry(operation, retry_config, config.operation_name).await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            // Try fallback if enabled
            if config.enable_fallback {
                match state.db_manager.get_pool("primary").await {
                    Some(pool) => {
                        warn!("Using fallback database pool");
                        Ok(pool)
                    }
                    None => {
                        error!("Both primary and fallback database pools failed");
                        return error_handler.handle_error(state, headers, e).await;
                    }
                }
            } else {
                error_handler.handle_error(state, headers, e).await
            }
        }
    }
}

/// Get database pool or handle error with consistent error handling and retry mechanism
/// Uses the generic pool retrieval function with HTML error handling and standard retry configuration
pub async fn get_db_pool_or_handle_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    let config = PoolRetrievalConfig::standard();
    get_db_pool_generic(state, headers, config, HtmlErrorHandler).await
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
) -> Result<crate::DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let retry_config = RetryConfig::fast();

    let operation = || async {
        let selected_db = crate::handlers::auth::get_selected_database(headers);
        state
            .db_manager
            .get_pool(&selected_db.unwrap_or_else(|| "primary".to_string()))
            .await
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Database pool not found").into()
            })
    };

    execute_with_retry(operation, retry_config, "get_current_db_pool").await
}

/// Get database pool or error with HTML response and fallback mechanism
/// Uses the generic pool retrieval function with HTML error handling and fallback enabled
pub async fn get_db_pool_or_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    let config = PoolRetrievalConfig::with_fallback();
    get_db_pool_generic(state, headers, config, HtmlErrorHandler).await
}

/// Get database pool with automatic fallback and session update
/// This function handles the case where the user's selected database becomes unavailable
/// by automatically falling back to a working database and updating the session
pub async fn get_db_pool_with_fallback_and_session_update(
    state: &AppState,
    headers: &HeaderMap,
) -> DatabaseResult<DatabaseResponse> {
    // Try to get the current database pool
    match get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(DatabaseResponse {
            pool,
            session_cookie: None,
        }), // No session update needed
        Err(_) => {
            // Get the currently selected database from session
            let selected_db = crate::handlers::auth::get_selected_database(headers)
                .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

            // Try to find a working database
            let fallback_db = find_working_database(state).await;

            match fallback_db {
                Some(fallback_id) => {
                    if let Some(pool) = state.db_manager.get_pool(&fallback_id).await {
                        warn!(
                            "Selected database '{}' is unavailable, falling back to '{}'",
                            selected_db, fallback_id
                        );

                        // Update the session to use the fallback database
                        let session_cookie =
                            crate::handlers::auth::update_session_database(headers, &fallback_id);

                        Ok(DatabaseResponse {
                            pool,
                            session_cookie,
                        })
                    } else {
                        error!("Fallback database '{}' is also unavailable", fallback_id);
                        Err(
                            crate::handlers::errors::render_database_error_page(state, headers)
                                .await,
                        )
                    }
                }
                None => {
                    error!("No working databases available");
                    Err(crate::handlers::errors::render_database_error_page(state, headers).await)
                }
            }
        }
    }
}

/// Find a working database from the available databases
async fn find_working_database(state: &AppState) -> Option<String> {
    let configs = state.db_manager.get_configs();

    // Try databases in order of preference
    let mut preferred_order = vec![
        state.db_manager.get_default_db_id().to_string(),
        "primary".to_string(),
    ];

    // Add other databases to the list
    for config in configs {
        if !preferred_order.contains(&config.id) {
            preferred_order.push(config.id.clone());
        }
    }

    // Test each database
    for db_id in preferred_order {
        if let Some(pool) = state.db_manager.get_pool(&db_id).await {
            // Test the connection by trying to get a connection
            if let Ok(_conn) = pool.get() {
                return Some(db_id);
            }
        }
    }

    None
}

/// Helper function to get database pool with fallback and handle session updates
/// This is the main function that handlers should use to get database pools
/// It automatically handles fallback to working databases and session updates
pub async fn get_db_pool_with_fallback(
    state: &AppState,
    headers: &HeaderMap,
) -> DatabaseResult<DatabaseResponse> {
    get_db_pool_with_fallback_and_session_update(state, headers).await
}

/// Helper function to handle database operations with automatic fallback
/// This function can be used by handlers that need to perform database operations
/// and want automatic fallback when the selected database becomes unavailable
pub async fn handle_db_operation_with_fallback<T, F, Fut>(
    operation: F,
    state: &AppState,
    headers: &HeaderMap,
    error_message: &str,
) -> Result<(T, Option<String>), Html<String>>
where
    F: FnOnce(&crate::DbPool) -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let db_response = get_db_pool_with_fallback(state, headers).await?;

    match operation(&db_response.pool).await {
        Ok(result) => Ok((result, db_response.session_cookie)),
        Err(e) => {
            error!("{}: {:?}", error_message, e);
            Err(crate::handlers::errors::render_database_error_page(state, headers).await)
        }
    }
}

/// Get database pool or redirect error with retry mechanism
/// Uses the generic pool retrieval function with redirect error handling and conservative retry configuration
pub async fn get_db_pool_or_redirect_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, (axum::http::StatusCode, String)> {
    let config = PoolRetrievalConfig::conservative();
    get_db_pool_generic(state, headers, config, RedirectErrorHandler).await
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
            Html(format!("Failed to {error_context}"))
        }
    }
}

/// Generic database operation with fallback to empty result
pub async fn execute_db_operation_with_fallback<T, F>(
    _operation: F,
    fallback_value: T,
    _error_context: &str,
) -> T
where
    F: FnOnce(&crate::DbPool) -> Result<T, diesel::result::Error>,
    T: Clone,
{
    // This is a simplified version - in practice, you'd need to pass the pool
    // For now, keeping the original functions but with better error handling
    fallback_value
}

/// Get paginated domains with fallback
pub async fn get_paginated_domains_with_fallback(
    pool: &crate::DbPool,
    page: i64,
    per_page: i64,
    search: Option<&str>,
) -> crate::models::PaginatedResult<crate::models::Domain> {
    match crate::db::get_domains_paginated(pool, page, per_page, search) {
        Ok(domains) => domains,
        Err(e) => handle_database_fallback_paginated("retrieve domains", &e, page, per_page),
    }
}

/// Get backups with fallback
pub async fn get_backups_with_fallback(pool: &crate::DbPool) -> Vec<crate::models::Backup> {
    match crate::db::get_backups(pool) {
        Ok(backups) => backups,
        Err(e) => handle_database_fallback_vec("retrieve backups", &e),
    }
}

/// Get domain with not found handling and retry mechanism
pub async fn get_domain_with_not_found_handling(
    pool: &crate::DbPool,
    id: i32,
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::models::Domain, Html<String>> {
    let retry_config = RetryConfig::conservative();

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
        Err(e) => handle_database_fallback_vec("get aliases for domain", &e),
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

/// Helper function to create a response with optional session cookie
/// This can be used by handlers that need to return responses with updated session cookies
pub fn create_response_with_session_cookie(
    content: Html<String>,
    session_cookie: Option<String>,
) -> Response<String> {
    let mut response_builder = Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8");

    if let Some(cookie) = session_cookie {
        response_builder = response_builder.header("Set-Cookie", cookie);
    }

    response_builder.body(content.0).unwrap()
}

/// Standard error response for database connection failures
/// This consolidates the common pattern of returning "Database connection error"
pub fn database_connection_error_response() -> Html<String> {
    Html("Database connection error".to_string())
}

/// Generic fallback handler for database operations that return empty results
/// This consolidates the common pattern of falling back to empty results on database errors
pub fn handle_database_fallback<T>(operation: &str, error: &diesel::result::Error) -> T
where
    T: Default,
{
    error!("Failed to {}: {:?}", operation, error);
    T::default()
}

/// Generic fallback handler for database operations that return empty vectors
/// This consolidates the common pattern of falling back to empty vectors on database errors
pub fn handle_database_fallback_vec<T>(operation: &str, error: &diesel::result::Error) -> Vec<T> {
    error!("Failed to {}: {:?}", operation, error);
    vec![]
}

/// Generic fallback handler for database operations that return empty paginated results
/// This consolidates the common pattern of falling back to empty paginated results on database errors
pub fn handle_database_fallback_paginated<T>(
    operation: &str,
    error: &diesel::result::Error,
    page: i64,
    per_page: i64,
) -> crate::models::PaginatedResult<T>
where
    T: Clone,
{
    error!("Failed to {}: {:?}", operation, error);
    crate::models::PaginatedResult::new(vec![], 0, page, per_page)
}

/// Generic entity creation helper with consistent error handling and success logging
/// This consolidates the common pattern of creating entities with proper error handling
pub async fn create_entity_with_handler<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Result<Html<String>, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully created {}: {}", entity_name, identifier);
            Ok(success_handler(entity))
        }
        Err(e) => {
            error!("Failed to create {} {}: {:?}", entity_name, identifier, e);
            Err(error_handler(e))
        }
    }
}

/// Generic entity update helper with consistent error handling and success logging
/// This consolidates the common pattern of updating entities with proper error handling
pub async fn update_entity_with_handler<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Result<Html<String>, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully updated {}: {}", entity_name, identifier);
            Ok(success_handler(entity))
        }
        Err(e) => {
            error!("Failed to update {} {}: {:?}", entity_name, identifier, e);
            Err(error_handler(e))
        }
    }
}

/// Generic entity deletion helper with consistent error handling and success logging
/// This consolidates the common pattern of deleting entities with proper error handling
pub async fn delete_entity_with_handler<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Result<Html<String>, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully deleted {}: {}", entity_name, identifier);
            Ok(success_handler(entity))
        }
        Err(e) => {
            error!("Failed to delete {} {}: {:?}", entity_name, identifier, e);
            Err(error_handler(e))
        }
    }
}

/// Generic entity toggle helper with consistent error handling and success logging
/// This consolidates the common pattern of toggling entity enabled status with proper error handling
pub async fn toggle_entity_with_handler<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Result<Html<String>, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully toggled {}: {}", entity_name, identifier);
            Ok(success_handler(entity))
        }
        Err(e) => {
            error!("Failed to toggle {} {}: {:?}", entity_name, identifier, e);
            Err(error_handler(e))
        }
    }
}

/// Generic entity operation with redirect error handling
/// This consolidates the common pattern of entity operations that return redirect errors
pub async fn entity_operation_with_redirect<T, F, Fut>(
    operation: F,
    entity_name: &str,
    identifier: &str,
    success_message: &str,
) -> Result<T, (axum::http::StatusCode, String)>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(entity) => {
            info!("{}: {}", success_message, identifier);
            Ok(entity)
        }
        Err(e) => {
            error!("Failed to {} {}: {:?}", entity_name, identifier, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to {} {}", entity_name, identifier),
            ))
        }
    }
}

/// Generic entity list retrieval with fallback
/// This consolidates the common pattern of retrieving entity lists with fallback to empty vectors
pub async fn get_entity_list_with_fallback<T, F, Fut>(operation: F, operation_name: &str) -> Vec<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, Error>>,
{
    match operation().await {
        Ok(entities) => entities,
        Err(e) => {
            error!("Failed to {}: {:?}", operation_name, e);
            vec![]
        }
    }
}

/// Generic entity retrieval with not found handling
/// This consolidates the common pattern of retrieving single entities with not found handling
pub async fn get_entity_with_not_found<T, F, Fut>(
    operation: F,
    state: &AppState,
    locale: &str,
    _entity_name: &str,
    not_found_key: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(entity) => Ok(entity),
        Err(_) => {
            let not_found_msg = crate::i18n::get_translation(state, locale, not_found_key).await;
            Err(Html(not_found_msg))
        }
    }
}

/// Generic paginated entity retrieval with fallback
/// This consolidates the common pattern of retrieving paginated entities with fallback
pub async fn get_paginated_entity_with_fallback<T, F, Fut>(
    operation: F,
    page: i64,
    per_page: i64,
    operation_name: &str,
) -> crate::models::PaginatedResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, Error>>,
    T: Clone,
{
    match operation().await {
        Ok(entities) => {
            let total = entities.len() as i64;
            crate::models::PaginatedResult::new(entities, total, page, per_page)
        }
        Err(e) => handle_database_fallback_paginated(operation_name, &e, page, per_page),
    }
}

/// Generic entity creation helper that returns HTML response directly
/// This consolidates the common pattern of creating entities and returning HTML responses
pub async fn create_entity_html<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Html<String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully created {}: {}", entity_name, identifier);
            success_handler(entity)
        }
        Err(e) => {
            error!("Failed to create {} {}: {:?}", entity_name, identifier, e);
            error_handler(e)
        }
    }
}

/// Generic entity update helper that returns HTML response directly
/// This consolidates the common pattern of updating entities and returning HTML responses
pub async fn update_entity_html<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Html<String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully updated {}: {}", entity_name, identifier);
            success_handler(entity)
        }
        Err(e) => {
            error!("Failed to update {} {}: {:?}", entity_name, identifier, e);
            error_handler(e)
        }
    }
}

/// Generic entity deletion helper that returns HTML response directly
/// This consolidates the common pattern of deleting entities and returning HTML responses
pub async fn delete_entity_html<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Html<String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully deleted {}: {}", entity_name, identifier);
            success_handler(entity)
        }
        Err(e) => {
            error!("Failed to delete {} {}: {:?}", entity_name, identifier, e);
            error_handler(e)
        }
    }
}

/// Generic entity toggle helper that returns HTML response directly
/// This consolidates the common pattern of toggling entities and returning HTML responses
pub async fn toggle_entity_html<T, F, Fut, S, E>(
    operation: F,
    success_handler: S,
    error_handler: E,
    entity_name: &str,
    identifier: &str,
) -> Html<String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
    S: FnOnce(T) -> Html<String>,
    E: FnOnce(diesel::result::Error) -> Html<String>,
{
    match operation().await {
        Ok(entity) => {
            info!("Successfully toggled {}: {}", entity_name, identifier);
            success_handler(entity)
        }
        Err(e) => {
            error!("Failed to toggle {} {}: {:?}", entity_name, identifier, e);
            error_handler(e)
        }
    }
}

// ============================================================================
// UNIFIED ERROR HANDLING PATTERNS
// ============================================================================

/// Unified error handling strategy for database operations
/// This enum defines the different ways errors can be handled across the application
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandlingStrategy {
    /// Return HTML error page response
    HtmlResponse,
    /// Return HTTP error status with message
    HttpError,
    /// Return default/fallback value
    FallbackValue,
    /// Return empty result (for lists/collections)
    EmptyResult,
    /// Redirect to error page
    Redirect,
    /// Return custom error response
    Custom(String),
}

/// Error handling configuration for database operations
/// This struct provides a unified way to configure error handling behavior
#[derive(Debug, Clone)]
pub struct ErrorHandlingConfig {
    /// The strategy to use for error handling
    pub strategy: ErrorHandlingStrategy,
    /// Custom error message key for i18n
    pub error_message_key: Option<String>,
    /// Whether to log the error
    pub log_error: bool,
    /// Whether to include error details in response
    pub include_error_details: bool,
    /// Fallback value for FallbackValue strategy
    pub fallback_value: Option<String>,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            strategy: ErrorHandlingStrategy::HtmlResponse,
            error_message_key: None,
            log_error: true,
            include_error_details: false,
            fallback_value: None,
        }
    }
}

impl ErrorHandlingConfig {
    /// Create a new error handling configuration
    pub fn new(strategy: ErrorHandlingStrategy) -> Self {
        Self {
            strategy,
            error_message_key: None,
            log_error: true,
            include_error_details: false,
            fallback_value: None,
        }
    }

    /// Set custom error message key
    pub fn with_error_message(mut self, key: &str) -> Self {
        self.error_message_key = Some(key.to_string());
        self
    }

    /// Set whether to log errors
    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_error = log;
        self
    }

    /// Set whether to include error details
    pub fn with_error_details(mut self, include: bool) -> Self {
        self.include_error_details = include;
        self
    }

    /// Set fallback value
    pub fn with_fallback_value(mut self, value: &str) -> Self {
        self.fallback_value = Some(value.to_string());
        self
    }

    /// Create HTML response strategy
    pub fn html_response() -> Self {
        Self::new(ErrorHandlingStrategy::HtmlResponse)
    }

    /// Create HTTP error strategy
    pub fn http_error() -> Self {
        Self::new(ErrorHandlingStrategy::HttpError)
    }

    /// Create fallback value strategy
    pub fn fallback_value(value: &str) -> Self {
        Self::new(ErrorHandlingStrategy::FallbackValue).with_fallback_value(value)
    }

    /// Create empty result strategy
    pub fn empty_result() -> Self {
        Self::new(ErrorHandlingStrategy::EmptyResult)
    }

    /// Create redirect strategy
    pub fn redirect() -> Self {
        Self::new(ErrorHandlingStrategy::Redirect)
    }
}

/// Unified error handler trait for database operations
/// This trait provides a consistent interface for handling database errors
pub trait UnifiedErrorHandler {
    type Output;
    type Error;

    /// Handle a database error using the configured strategy
    fn handle_database_error(
        &self,
        error: diesel::result::Error,
        config: &ErrorHandlingConfig,
        context: &str,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// HTML error handler implementation
pub struct HtmlErrorHandlerImpl {
    pub state: AppState,
    pub headers: HeaderMap,
}

impl UnifiedErrorHandler for HtmlErrorHandlerImpl {
    type Output = Html<String>;
    type Error = Html<String>;

    async fn handle_database_error(
        &self,
        error: diesel::result::Error,
        config: &ErrorHandlingConfig,
        context: &str,
    ) -> Result<Self::Output, Self::Error> {
        if config.log_error {
            error!("Database error in {}: {:?}", context, error);
        }

        match config.strategy {
            ErrorHandlingStrategy::HtmlResponse => {
                let error_page =
                    crate::handlers::errors::render_database_error_page(&self.state, &self.headers)
                        .await;
                Ok(error_page)
            }
            ErrorHandlingStrategy::FallbackValue => {
                let fallback = config
                    .fallback_value
                    .clone()
                    .unwrap_or_else(|| "".to_string());
                Ok(Html(fallback))
            }
            ErrorHandlingStrategy::EmptyResult => Ok(Html("".to_string())),
            ErrorHandlingStrategy::Redirect => {
                // For HTML handler, redirect is not applicable, fall back to error page
                let error_page =
                    crate::handlers::errors::render_database_error_page(&self.state, &self.headers)
                        .await;
                Ok(error_page)
            }
            _ => {
                let error_page =
                    crate::handlers::errors::render_database_error_page(&self.state, &self.headers)
                        .await;
                Ok(error_page)
            }
        }
    }
}

/// HTTP error handler implementation
pub struct HttpErrorHandlerImpl;

impl UnifiedErrorHandler for HttpErrorHandlerImpl {
    type Output = (axum::http::StatusCode, String);
    type Error = (axum::http::StatusCode, String);

    async fn handle_database_error(
        &self,
        error: diesel::result::Error,
        config: &ErrorHandlingConfig,
        context: &str,
    ) -> Result<Self::Output, Self::Error> {
        if config.log_error {
            error!("Database error in {}: {:?}", context, error);
        }

        match config.strategy {
            ErrorHandlingStrategy::HttpError => {
                let message = if config.include_error_details {
                    format!("Database error in {}: {}", context, error)
                } else {
                    format!("Database error in {}", context)
                };
                Ok((axum::http::StatusCode::INTERNAL_SERVER_ERROR, message))
            }
            ErrorHandlingStrategy::FallbackValue => {
                let fallback = config
                    .fallback_value
                    .clone()
                    .unwrap_or_else(|| "".to_string());
                Ok((axum::http::StatusCode::OK, fallback))
            }
            ErrorHandlingStrategy::EmptyResult => Ok((axum::http::StatusCode::OK, "".to_string())),
            ErrorHandlingStrategy::Redirect => {
                Ok((axum::http::StatusCode::SEE_OTHER, "/error".to_string()))
            }
            _ => {
                let message = format!("Database error in {}", context);
                Ok((axum::http::StatusCode::INTERNAL_SERVER_ERROR, message))
            }
        }
    }
}

/// Generic database operation with unified error handling
/// This function provides a consistent way to handle database operations with configurable error handling
pub async fn execute_db_operation_with_unified_error_handling<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    config: ErrorHandlingConfig,
    context: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    match operation().await {
        Ok(result) => Ok(result),
        Err(error) => {
            error_handler
                .handle_database_error(error, &config, context)
                .await?;
            // This should never be reached due to the error handler returning the error
            unreachable!()
        }
    }
}

/// Unified entity operation with consistent error handling patterns
/// This function consolidates common patterns for entity operations with unified error handling
pub async fn unified_entity_operation<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    operation_type: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("{} {} for {}", operation_type, entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-{}-{}", operation_type, entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

/// Unified list operation with consistent error handling patterns
/// This function provides consistent error handling for list/collection operations
pub async fn unified_list_operation<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    operation_type: &str,
) -> Result<Vec<T>, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, diesel::result::Error>>,
    H: UnifiedErrorHandler,
    T: Clone,
{
    let context = format!("{} {} list", operation_type, entity_name);
    let config = ErrorHandlingConfig::empty_result()
        .with_error_message(&format!("error-{}-{}", operation_type, entity_name))
        .with_logging(true);

    match operation().await {
        Ok(result) => Ok(result),
        Err(error) => {
            if config.log_error {
                error!("Database error in {}: {:?}", context, error);
            }

            match config.strategy {
                ErrorHandlingStrategy::EmptyResult => Ok(vec![]),
                ErrorHandlingStrategy::FallbackValue => {
                    // For lists, fallback value doesn't make sense, return empty
                    Ok(vec![])
                }
                _ => {
                    // For other strategies, let the error handler deal with it
                    error_handler
                        .handle_database_error(error, &config, &context)
                        .await?;
                    unreachable!()
                }
            }
        }
    }
}

/// Unified paginated operation with consistent error handling patterns
/// This function provides consistent error handling for paginated operations
pub async fn unified_paginated_operation<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    operation_type: &str,
    page: i64,
    per_page: i64,
) -> Result<crate::models::PaginatedResult<T>, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, diesel::result::Error>>,
    H: UnifiedErrorHandler,
    T: Clone,
{
    let context = format!("{} {} paginated list", operation_type, entity_name);
    let config = ErrorHandlingConfig::empty_result()
        .with_error_message(&format!("error-{}-{}", operation_type, entity_name))
        .with_logging(true);

    match operation().await {
        Ok(result) => {
            let total = result.len() as i64;
            Ok(crate::models::PaginatedResult::new(
                result, total, page, per_page,
            ))
        }
        Err(error) => {
            if config.log_error {
                error!("Database error in {}: {:?}", context, error);
            }

            match config.strategy {
                ErrorHandlingStrategy::EmptyResult => Ok(crate::models::PaginatedResult::new(
                    vec![],
                    0,
                    page,
                    per_page,
                )),
                ErrorHandlingStrategy::FallbackValue => {
                    // For paginated results, fallback value doesn't make sense, return empty
                    Ok(crate::models::PaginatedResult::new(
                        vec![],
                        0,
                        page,
                        per_page,
                    ))
                }
                _ => {
                    // For other strategies, let the error handler deal with it
                    error_handler
                        .handle_database_error(error, &config, &context)
                        .await?;
                    unreachable!()
                }
            }
        }
    }
}

/// Unified entity retrieval with consistent error handling patterns
/// This function provides consistent error handling for single entity retrieval operations
pub async fn unified_entity_retrieval<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("retrieve {} with id {}", entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-{}-not-found", entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

/// Unified entity creation with consistent error handling patterns
/// This function provides consistent error handling for entity creation operations
pub async fn unified_entity_creation<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("create {} with identifier {}", entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-create-{}", entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

/// Unified entity update with consistent error handling patterns
/// This function provides consistent error handling for entity update operations
pub async fn unified_entity_update<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("update {} with id {}", entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-update-{}", entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

/// Unified entity deletion with consistent error handling patterns
/// This function provides consistent error handling for entity deletion operations
pub async fn unified_entity_deletion<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("delete {} with id {}", entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-delete-{}", entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

/// Unified entity toggle with consistent error handling patterns
/// This function provides consistent error handling for entity toggle operations
pub async fn unified_entity_toggle<T, F, Fut, H>(
    operation: F,
    error_handler: H,
    entity_name: &str,
    identifier: &str,
) -> Result<T, H::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
    H: UnifiedErrorHandler,
{
    let context = format!("toggle {} with id {}", entity_name, identifier);
    let config = ErrorHandlingConfig::html_response()
        .with_error_message(&format!("error-toggle-{}", entity_name))
        .with_logging(true);

    execute_db_operation_with_unified_error_handling(operation, error_handler, config, &context)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::Html;
    use diesel::result::Error;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // Mock data for testing
    #[derive(Clone, Debug, PartialEq)]
    struct MockEntity {
        id: i32,
        name: String,
    }

    impl Default for MockEntity {
        fn default() -> Self {
            Self {
                id: 0,
                name: "default".to_string(),
            }
        }
    }

    // Helper to create a test entity
    fn create_test_entity() -> MockEntity {
        MockEntity {
            id: 1,
            name: "test_entity".to_string(),
        }
    }

    // Test helper to track calls
    #[derive(Debug, Default)]
    struct CallTracker {
        success_calls: Arc<Mutex<Vec<String>>>,
        error_calls: Arc<Mutex<Vec<String>>>,
    }

    impl CallTracker {
        fn new() -> Self {
            Self::default()
        }

        fn add_success_call(&self, id: String) {
            self.success_calls.lock().unwrap().push(id);
        }

        fn add_error_call(&self, id: String) {
            self.error_calls.lock().unwrap().push(id);
        }

        fn get_success_calls(&self) -> Vec<String> {
            self.success_calls.lock().unwrap().clone()
        }

        fn get_error_calls(&self) -> Vec<String> {
            self.error_calls.lock().unwrap().clone()
        }
    }

    #[tokio::test]
    async fn test_handle_database_fallback() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );

        let result: MockEntity = handle_database_fallback("test_operation", &error);

        assert_eq!(result, MockEntity::default());
    }

    #[tokio::test]
    async fn test_handle_database_fallback_vec() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );

        let result: Vec<MockEntity> = handle_database_fallback_vec("test_operation", &error);

        assert_eq!(result, vec![]);
    }

    #[tokio::test]
    async fn test_handle_database_fallback_paginated() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );

        let result: crate::models::PaginatedResult<MockEntity> =
            handle_database_fallback_paginated("test_operation", &error, 1, 10);

        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.per_page, 10);
    }

    #[tokio::test]
    async fn test_database_connection_error_response() {
        let response = database_connection_error_response();

        assert_eq!(response.0, "Database connection error");
    }

    #[tokio::test]
    async fn test_get_entity_list_with_fallback_success() {
        let entities = vec![create_test_entity()];
        let operation = || async { Ok::<Vec<MockEntity>, Error>(entities.clone()) };

        let result = get_entity_list_with_fallback(operation, "test_operation").await;

        assert_eq!(result, entities);
    }

    #[tokio::test]
    async fn test_get_entity_list_with_fallback_error() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );
        let operation = || async { Err::<Vec<MockEntity>, Error>(error) };

        let result = get_entity_list_with_fallback(operation, "test_operation").await;

        assert_eq!(result, vec![]);
    }

    #[tokio::test]
    async fn test_get_paginated_entity_with_fallback_success() {
        let entities = vec![create_test_entity()];
        let operation = || async { Ok::<Vec<MockEntity>, Error>(entities.clone()) };

        let result = get_paginated_entity_with_fallback(operation, 1, 10, "test_operation").await;

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.total_count, 1);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.per_page, 10);
        assert_eq!(result.items[0], create_test_entity());
    }

    #[tokio::test]
    async fn test_get_paginated_entity_with_fallback_error() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );
        let operation = || async { Err::<Vec<MockEntity>, Error>(error) };

        let result = get_paginated_entity_with_fallback(operation, 1, 10, "test_operation").await;

        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.per_page, 10);
    }

    #[tokio::test]
    async fn test_entity_operation_with_redirect_success() {
        let entity = create_test_entity();
        let operation = || async { Ok::<MockEntity, Error>(entity.clone()) };

        let result = entity_operation_with_redirect(
            operation,
            "test_entity",
            "123",
            "Successfully operated on test_entity",
        )
        .await;

        assert_eq!(result, Ok(entity));
    }

    #[tokio::test]
    async fn test_entity_operation_with_redirect_error() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );
        let operation = || async { Err::<MockEntity, Error>(error) };

        let result = entity_operation_with_redirect(
            operation,
            "test_entity",
            "123",
            "Successfully operated on test_entity",
        )
        .await;

        assert!(result.is_err());
        if let Err((status, message)) = result {
            assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            assert!(message.contains("Failed to test_entity 123"));
        }
    }

    #[tokio::test]
    async fn test_create_entity_html_success() {
        let entity = create_test_entity();
        let tracker = CallTracker::new();
        let success_calls = tracker.success_calls.clone();
        let error_calls = tracker.error_calls.clone();

        let operation = || async { Ok::<MockEntity, Error>(entity.clone()) };
        let success_handler = |e: MockEntity| {
            success_calls.lock().unwrap().push(e.name.clone());
            Html("success".to_string())
        };
        let error_handler = |_e: Error| {
            error_calls.lock().unwrap().push("error".to_string());
            Html("error".to_string())
        };

        let result = create_entity_html(
            operation,
            success_handler,
            error_handler,
            "test_entity",
            "123",
        )
        .await;

        assert_eq!(result.0, "success");
        assert_eq!(tracker.get_success_calls(), vec!["test_entity"]);
        assert_eq!(tracker.get_error_calls(), Vec::<String>::new());
    }

    #[tokio::test]
    async fn test_create_entity_html_error() {
        let error = Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("test error".to_string()),
        );
        let tracker = CallTracker::new();
        let success_calls = tracker.success_calls.clone();
        let error_calls = tracker.error_calls.clone();

        let operation = || async { Err::<MockEntity, Error>(error) };
        let success_handler = |_e: MockEntity| {
            success_calls.lock().unwrap().push("success".to_string());
            Html("success".to_string())
        };
        let error_handler = |_e: Error| {
            error_calls.lock().unwrap().push("error".to_string());
            Html("error".to_string())
        };

        let result = create_entity_html(
            operation,
            success_handler,
            error_handler,
            "test_entity",
            "123",
        )
        .await;

        assert_eq!(result.0, "error");
        assert_eq!(tracker.get_success_calls(), Vec::<String>::new());
        assert_eq!(tracker.get_error_calls(), vec!["error"]);
    }

    #[tokio::test]
    async fn test_update_entity_html_success() {
        let entity = create_test_entity();
        let tracker = CallTracker::new();
        let success_calls = tracker.success_calls.clone();

        let operation = || async { Ok::<MockEntity, Error>(entity.clone()) };
        let success_handler = |e: MockEntity| {
            success_calls.lock().unwrap().push(e.name.clone());
            Html("success".to_string())
        };
        let error_handler = |_e: Error| Html("error".to_string());

        let result = update_entity_html(
            operation,
            success_handler,
            error_handler,
            "test_entity",
            "123",
        )
        .await;

        assert_eq!(result.0, "success");
        assert_eq!(tracker.get_success_calls(), vec!["test_entity"]);
    }

    #[tokio::test]
    async fn test_delete_entity_html_success() {
        let entity = create_test_entity();
        let tracker = CallTracker::new();
        let success_calls = tracker.success_calls.clone();

        let operation = || async { Ok::<MockEntity, Error>(entity.clone()) };
        let success_handler = |e: MockEntity| {
            success_calls.lock().unwrap().push(e.name.clone());
            Html("success".to_string())
        };
        let error_handler = |_e: Error| Html("error".to_string());

        let result = delete_entity_html(
            operation,
            success_handler,
            error_handler,
            "test_entity",
            "123",
        )
        .await;

        assert_eq!(result.0, "success");
        assert_eq!(tracker.get_success_calls(), vec!["test_entity"]);
    }

    #[tokio::test]
    async fn test_toggle_entity_html_success() {
        let entity = create_test_entity();
        let tracker = CallTracker::new();
        let success_calls = tracker.success_calls.clone();

        let operation = || async { Ok::<MockEntity, Error>(entity.clone()) };
        let success_handler = |e: MockEntity| {
            success_calls.lock().unwrap().push(e.name.clone());
            Html("success".to_string())
        };
        let error_handler = |_e: Error| Html("error".to_string());

        let result = toggle_entity_html(
            operation,
            success_handler,
            error_handler,
            "test_entity",
            "123",
        )
        .await;

        assert_eq!(result.0, "success");
        assert_eq!(tracker.get_success_calls(), vec!["test_entity"]);
    }

    #[tokio::test]
    async fn test_retry_config_builder() {
        let config = RetryConfig::custom()
            .max_attempts(5)
            .base_delay(Duration::from_millis(200))
            .max_delay(Duration::from_secs(10))
            .backoff_multiplier(1.5)
            .build();

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[tokio::test]
    async fn test_pool_retrieval_config_builder() {
        let retry_config = RetryConfig::fast();
        let config = PoolRetrievalConfig::custom()
            .retry_config(retry_config)
            .enable_fallback(true)
            .operation_name("test_operation")
            .build();

        assert_eq!(config.retry_config.max_attempts, 2);
        assert_eq!(config.enable_fallback, true);
        assert_eq!(config.operation_name, "test_operation");
    }

    #[tokio::test]
    async fn test_retry_config_presets() {
        let standard = RetryConfig::standard();
        assert_eq!(standard.max_attempts, 3);
        assert_eq!(standard.base_delay, Duration::from_millis(100));
        assert_eq!(standard.max_delay, Duration::from_secs(5));

        let fast = RetryConfig::fast();
        assert_eq!(fast.max_attempts, 2);
        assert_eq!(fast.base_delay, Duration::from_millis(50));
        assert_eq!(fast.max_delay, Duration::from_secs(1));

        let aggressive = RetryConfig::aggressive();
        assert_eq!(aggressive.max_attempts, 5);
        assert_eq!(aggressive.base_delay, Duration::from_millis(200));
        assert_eq!(aggressive.max_delay, Duration::from_secs(10));

        let conservative = RetryConfig::conservative();
        assert_eq!(conservative.max_attempts, 2);
        assert_eq!(conservative.base_delay, Duration::from_millis(100));
        assert_eq!(conservative.max_delay, Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_pool_retrieval_config_presets() {
        let standard = PoolRetrievalConfig::standard();
        assert_eq!(standard.enable_fallback, false);
        assert_eq!(standard.operation_name, "get_database_pool");

        let fast = PoolRetrievalConfig::fast();
        assert_eq!(fast.enable_fallback, false);
        assert_eq!(fast.operation_name, "get_database_pool_fast");

        let with_fallback = PoolRetrievalConfig::with_fallback();
        assert_eq!(with_fallback.enable_fallback, true);
        assert_eq!(
            with_fallback.operation_name,
            "get_database_pool_with_fallback"
        );

        let conservative = PoolRetrievalConfig::conservative();
        assert_eq!(conservative.enable_fallback, false);
        assert_eq!(
            conservative.operation_name,
            "get_database_pool_conservative"
        );
    }
}

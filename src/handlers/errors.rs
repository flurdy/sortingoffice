use crate::AppState;
use axum::http::HeaderMap;
use axum::response::Html;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Custom error type for database operations
/// Based on structured error handling patterns from Rust error handling guide
#[derive(Debug, thiserror::Error)]
pub enum DatabaseOperationError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Database query failed: {0}")]
    QueryFailed(String),

    #[error("Entity not found: {entity_type} with id {identifier}")]
    NotFound {
        entity_type: String,
        identifier: String,
    },

    #[error("Database constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Database operation not allowed: {operation} on {entity}")]
    OperationNotAllowed { operation: String, entity: String },
}

/// Custom error type for form validation operations
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Invalid format: {field} - {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Validation failed: {message}")]
    Custom { message: String },
}

/// Circuit breaker state for error recovery
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub failure_count: u32,
    pub last_failure_time: Option<Instant>,
    pub state: CircuitBreakerState,
    pub threshold: u32,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service is recovered
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            last_failure_time: None,
            state: CircuitBreakerState::Closed,
            threshold,
            timeout,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn on_success(&mut self) {
        self.failure_count = 0;
        self.last_failure_time = None;
        self.state = CircuitBreakerState::Closed;
    }

    pub fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

/// Retry configuration for error recovery
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::standard()
    }
}

impl RetryConfig {
    /// Standard retry configuration for most database operations
    /// - 3 attempts with exponential backoff
    /// - Base delay: 100ms, Max delay: 5s
    /// - Backoff multiplier: 2.0
    pub fn standard() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }

    /// Fast retry configuration for lightweight operations
    /// - 2 attempts with quick retry
    /// - Base delay: 50ms, Max delay: 1s
    /// - Backoff multiplier: 2.0
    pub fn fast() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        }
    }

    /// Aggressive retry configuration for critical operations
    /// - 5 attempts with longer delays
    /// - Base delay: 200ms, Max delay: 10s
    /// - Backoff multiplier: 2.0
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }

    /// Conservative retry configuration for non-critical operations
    /// - 2 attempts with minimal delay
    /// - Base delay: 100ms, Max delay: 2s
    /// - Backoff multiplier: 2.0
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
            backoff_multiplier: 2.0,
        }
    }

    /// Custom retry configuration builder
    pub fn custom() -> RetryConfigBuilder {
        RetryConfigBuilder::new()
    }
}

/// Builder for custom retry configurations
pub struct RetryConfigBuilder {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryConfigBuilder {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }

    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn build(self) -> RetryConfig {
        RetryConfig {
            max_attempts: self.max_attempts,
            base_delay: self.base_delay,
            max_delay: self.max_delay,
            backoff_multiplier: self.backoff_multiplier,
        }
    }
}

/// Execute operation with retry mechanism
pub async fn execute_with_retry<T, E, F, Fut>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 1;
    let mut delay = config.base_delay;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!("{} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_attempts {
                    error!(
                        "{} failed after {} attempts: {:?}",
                        operation_name, attempt, e
                    );
                    return Err(e);
                }

                warn!(
                    "{} failed on attempt {}: {:?}, retrying in {:?}",
                    operation_name, attempt, e, delay
                );

                sleep(delay).await;
                attempt += 1;
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_multiplier),
                    config.max_delay,
                );
            }
        }
    }
}

/// Simple retry wrapper for database operations
pub async fn retry_db_operation<T, F, Fut>(
    operation: F,
    max_attempts: u32,
    operation_name: &str,
) -> Result<T, diesel::result::Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
{
    let mut attempt = 1;
    let mut delay = Duration::from_millis(100);

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!("{} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= max_attempts {
                    error!(
                        "{} failed after {} attempts: {:?}",
                        operation_name, attempt, e
                    );
                    return Err(e);
                }

                warn!(
                    "{} failed on attempt {}: {:?}, retrying in {:?}",
                    operation_name, attempt, e, delay
                );

                sleep(delay).await;
                attempt += 1;
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * 2.0),
                    Duration::from_secs(2),
                );
            }
        }
    }
}

/// Retry wrapper for operations that can be called multiple times
pub async fn retry_operation_once<T, F, Fut>(
    operation: F,
    operation_name: &str,
) -> Result<T, diesel::result::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, diesel::result::Error>>,
{
    match operation().await {
        Ok(result) => {
            info!("{} succeeded", operation_name);
            Ok(result)
        }
        Err(e) => {
            error!("{} failed: {:?}", operation_name, e);
            Err(e)
        }
    }
}

/// Helper function to render error pages consistently with proper theming and translations
pub async fn render_error_page(
    title_key: &str,
    message_key: &str,
    state: &AppState,
    headers: &HeaderMap,
) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(headers);
    let (current_db_label, current_db_id) =
        crate::handlers::utils::get_current_db_info_optimized(state, headers);

    let title = crate::i18n::get_translation(state, &locale, title_key).await;
    let message = crate::i18n::get_translation(state, &locale, message_key).await;

    println!("[DEBUG] Rendering error page with title: {title}, message: {message}");

    match crate::templates::error::ErrorTemplate::new(
        &title,
        &message,
        state,
        &locale,
        &current_db_label,
        &current_db_id,
    )
    .await
    {
        Ok(template) => match crate::handlers::templates::render_template_safely(template) {
            Ok(html) => {
                println!(
                    "[DEBUG] Error template rendered successfully, length: {}",
                    html.len()
                );
                Html(html)
            }
            Err(e) => {
                println!("[DEBUG] Error rendering template: {e}");
                Html("Error rendering error page".to_string())
            }
        },
        Err(e) => {
            println!("[DEBUG] Error creating template: {e:?}");
            Html("Error creating error page".to_string())
        }
    }
}

/// Render a 404 Not Found error page
pub async fn render_404_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-not-found-title",
        "error-page-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a 500 Internal Server Error page
pub async fn render_500_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-operation-failed-title",
        "error-page-operation-failed-message",
        state,
        headers,
    )
    .await
}

/// Render a 403 Forbidden error page
pub async fn render_403_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-validation-error-title",
        "error-page-validation-error-message",
        state,
        headers,
    )
    .await
}

/// Render a 401 Unauthorized error page
pub async fn render_401_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-validation-error-title",
        "error-page-validation-error-message",
        state,
        headers,
    )
    .await
}

/// Render a database error page
pub async fn render_database_error_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-database-error-title",
        "error-page-database-error-message",
        state,
        headers,
    )
    .await
}

/// Render a domain not found page
pub async fn render_domain_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "domains-not-found-title",
        "domains-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a user not found page
pub async fn render_user_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "users-not-found-title",
        "users-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render an alias not found page
pub async fn render_alias_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "aliases-not-found-title",
        "aliases-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a client not found page
pub async fn render_client_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "clients-not-found-title",
        "clients-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a relay not found page
pub async fn render_relay_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "relays-not-found-title",
        "relays-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a backup not found page
pub async fn render_backup_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "backups-not-found-title",
        "backups-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a relocated not found page
pub async fn render_relocated_not_found_page(
    state: &AppState,
    headers: &HeaderMap,
) -> Html<String> {
    render_error_page(
        "relocated-not-found-title",
        "relocated-not-found-message",
        state,
        headers,
    )
    .await
}

/// Handle entity not found with consistent error handling
pub async fn handle_entity_not_found(
    state: &AppState,
    headers: &HeaderMap,
    entity_type: &str,
    _error_key: &str,
) -> Html<String> {
    match entity_type {
        "domains" => render_domain_not_found_page(state, headers).await,
        "users" => render_user_not_found_page(state, headers).await,
        "aliases" => render_alias_not_found_page(state, headers).await,
        "clients" => render_client_not_found_page(state, headers).await,
        "relays" => render_relay_not_found_page(state, headers).await,
        "backups" => render_backup_not_found_page(state, headers).await,
        "relocated" => render_relocated_not_found_page(state, headers).await,
        _ => render_404_page(state, headers).await,
    }
}

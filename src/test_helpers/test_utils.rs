use crate::{
    config::{AdminRole, Config, DatabaseConfig, DatabaseFeatures},
    db::DatabaseManager,
    handlers,
    i18n::I18n,
    AppState,
};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use std::collections::HashMap;
use tower::ServiceExt;

/// Common test utilities for sharing between integration and UI tests
pub struct TestUtils;

impl TestUtils {
    /// Create an authenticated cookie with a specific role
    pub fn create_auth_cookie(role: AdminRole) -> axum::http::HeaderValue {
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour in the future
        let role_str = match role {
            AdminRole::Edit => "edit",
            AdminRole::ReadOnly => "read-only",
        };
        let cookie = format!(
            "authenticated={expiry}:{role_str}:test; Path=/; Max-Age=3600; HttpOnly; SameSite=Lax"
        );
        axum::http::HeaderValue::from_str(&cookie).unwrap()
    }

    /// Create an authenticated cookie with edit role (most common case)
    pub fn create_edit_auth_cookie() -> axum::http::HeaderValue {
        Self::create_auth_cookie(AdminRole::Edit)
    }

    /// Create an authenticated cookie with read-only role
    pub fn create_readonly_auth_cookie() -> axum::http::HeaderValue {
        Self::create_auth_cookie(AdminRole::ReadOnly)
    }

    /// Create a test app with a single database
    pub async fn create_test_app_with_db(
        db_url: &str,
        db_id: &str,
    ) -> (Router<AppState>, AppState) {
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

        // Load translation files for testing
        let _ = i18n.load_locale("en-US").await;
        let _ = i18n.load_locale("es-ES").await;
        let _ = i18n.load_locale("nb-NO").await;
        let _ = i18n.load_locale("fr-FR").await;
        let _ = i18n.load_locale("de-DE").await;

        let config = Config::default();

        // Create a database manager with the test pool
        let db_config = vec![DatabaseConfig {
            id: db_id.to_string(),
            label: "Test Database".to_string(),
            url: db_url.to_string(),
            features: DatabaseFeatures::default(),
            field_map: HashMap::new(),
        }];

        let db_manager = DatabaseManager::new(db_config)
            .await
            .expect("Failed to create database manager");

        let state = AppState {
            db_manager,
            i18n,
            config,
        };

        let app = handlers::create_app(state.clone());
        (app, state)
    }

    /// Create a test app with multiple databases
    pub async fn create_test_app_with_dbs(
        db_configs: Vec<DatabaseConfig>,
    ) -> (Router<AppState>, AppState) {
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

        // Load translation files for testing
        let _ = i18n.load_locale("en-US").await;
        let _ = i18n.load_locale("es-ES").await;
        let _ = i18n.load_locale("nb-NO").await;
        let _ = i18n.load_locale("fr-FR").await;
        let _ = i18n.load_locale("de-DE").await;

        let config = Config::default();

        let db_manager = DatabaseManager::new(db_configs)
            .await
            .expect("Failed to create database manager");

        let state = AppState {
            db_manager,
            i18n,
            config,
        };

        let app = handlers::create_app(state.clone());
        (app, state)
    }

    /// Create a test app with a specific database configuration
    pub async fn create_test_app_with_config(
        db_configs: Vec<DatabaseConfig>,
        config: Config,
    ) -> (Router<AppState>, AppState) {
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

        // Load translation files for testing
        let _ = i18n.load_locale("en-US").await;
        let _ = i18n.load_locale("es-ES").await;
        let _ = i18n.load_locale("nb-NO").await;
        let _ = i18n.load_locale("fr-FR").await;
        let _ = i18n.load_locale("de-DE").await;

        let db_manager = DatabaseManager::new(db_configs)
            .await
            .expect("Failed to create database manager");

        let state = AppState {
            db_manager,
            i18n,
            config,
        };

        let app = handlers::create_app(state.clone());
        (app, state)
    }

    /// Make a GET request to the test app
    pub async fn make_get_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> Result<axum::http::Response<Body>, Box<dyn std::error::Error>> {
        let mut request_builder = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())?;

        if let Some(cookie) = auth_cookie {
            request_builder.headers_mut().insert("cookie", cookie);
        }

        let response = app
            .clone()
            .with_state(state.clone())
            .oneshot(request_builder)
            .await?;

        Ok(response)
    }

    /// Make a POST request to the test app
    pub async fn make_post_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        form_data: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> Result<axum::http::Response<Body>, Box<dyn std::error::Error>> {
        let mut request_builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_data.to_string()))?;

        if let Some(cookie) = auth_cookie {
            request_builder.headers_mut().insert("cookie", cookie);
        }

        let response = app
            .clone()
            .with_state(state.clone())
            .oneshot(request_builder)
            .await?;

        Ok(response)
    }

    /// Make a PUT request to the test app
    pub async fn make_put_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        form_data: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> Result<axum::http::Response<Body>, Box<dyn std::error::Error>> {
        let mut request_builder = Request::builder()
            .method("PUT")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_data.to_string()))?;

        if let Some(cookie) = auth_cookie {
            request_builder.headers_mut().insert("cookie", cookie);
        }

        let response = app
            .clone()
            .with_state(state.clone())
            .oneshot(request_builder)
            .await?;

        Ok(response)
    }

    /// Make a DELETE request to the test app
    pub async fn make_delete_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> Result<axum::http::Response<Body>, Box<dyn std::error::Error>> {
        let mut request_builder = Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())?;

        if let Some(cookie) = auth_cookie {
            request_builder.headers_mut().insert("cookie", cookie);
        }

        let response = app
            .clone()
            .with_state(state.clone())
            .oneshot(request_builder)
            .await?;

        Ok(response)
    }

    /// Assert that a response has the expected status code
    pub fn assert_status(response: &axum::http::Response<Body>, expected: StatusCode) {
        assert_eq!(
            response.status(),
            expected,
            "Expected status {}, got {}",
            expected,
            response.status()
        );
    }

    /// Assert that a response body contains the expected text
    pub async fn assert_body_contains(response: axum::http::Response<Body>, expected_text: &str) {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains(expected_text),
            "Expected body to contain '{expected_text}', but got: {body_str}"
        );
    }

    /// Assert that a response body does not contain the expected text
    pub async fn assert_body_not_contains(
        response: axum::http::Response<Body>,
        unexpected_text: &str,
    ) {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            !body_str.contains(unexpected_text),
            "Expected body to not contain '{unexpected_text}', but got: {body_str}"
        );
    }

    /// Create a database config with dynamic port replacement
    pub fn create_db_config_with_port(base_config: DatabaseConfig, port: u16) -> DatabaseConfig {
        let mut config = base_config;
        config.url = config.url.replace(":3306", &format!(":{port}"));
        config
    }

    /// Create multiple database configs with dynamic port replacement
    pub fn create_db_configs_with_port(
        base_configs: Vec<DatabaseConfig>,
        port: u16,
    ) -> Vec<DatabaseConfig> {
        base_configs
            .into_iter()
            .map(|config| Self::create_db_config_with_port(config, port))
            .collect()
    }
}

/// Common test data generators
pub struct TestData;

impl TestData {
    /// Generate a unique test domain name
    pub fn unique_domain() -> String {
        format!(
            "test-{}.example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate a unique test user ID
    pub fn unique_user_id() -> String {
        format!(
            "user-{}@example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate a unique test alias
    pub fn unique_alias() -> String {
        format!(
            "alias-{}@example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate form data for creating a domain
    pub fn domain_form_data(domain: &str, transport: &str, enabled: bool) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        format!("domain={domain}&transport={transport}&enabled={enabled_str}")
    }

    /// Generate form data for creating a user
    pub fn user_form_data(user_id: &str, username: &str, password: &str) -> String {
        format!("user_id={user_id}&username={username}&password={password}")
    }

    /// Generate form data for creating an alias
    pub fn alias_form_data(mail: &str, destination: &str, enabled: bool) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        format!("mail={mail}&destination={destination}&enabled={enabled_str}")
    }

    /// Generate form data for creating a user
    pub fn user_form_data_complete(
        user_id: &str,
        password: &str,
        name: &str,
        maildir: &str,
        home: &str,
        domain: &str,
        quota: &str,
        enabled: bool,
        change_password: bool,
    ) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        let change_password_str = if change_password { "on" } else { "" };
        format!(
            "id={user_id}&password={password}&name={name}&maildir={maildir}&home={home}&domain={domain}&quota={quota}&enabled={enabled_str}&change_password={change_password_str}"
        )
    }
}

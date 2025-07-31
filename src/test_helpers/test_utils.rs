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

    // Handler test specific helpers (simpler than integration test helpers)

    /// Make a GET request for handler tests (returns Response directly)
    pub async fn make_handler_get_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> axum::http::Response<Body> {
        let mut request_builder = Request::builder().method("GET").uri(uri);

        if let Some(cookie) = auth_cookie {
            request_builder = request_builder.header("cookie", cookie);
        }

        let request = request_builder.body(Body::empty()).unwrap();

        app.clone()
            .with_state(state.clone())
            .oneshot(request)
            .await
            .unwrap()
    }

    /// Make a POST request for handler tests (returns Response directly)
    pub async fn make_handler_post_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        form_data: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> axum::http::Response<Body> {
        let mut request_builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");

        if let Some(cookie) = auth_cookie {
            request_builder = request_builder.header("cookie", cookie);
        }

        let request = request_builder
            .body(Body::from(form_data.to_string()))
            .unwrap();

        app.clone()
            .with_state(state.clone())
            .oneshot(request)
            .await
            .unwrap()
    }

    /// Make a PUT request for handler tests (returns Response directly)
    pub async fn make_handler_put_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        form_data: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> axum::http::Response<Body> {
        let mut request_builder = Request::builder()
            .method("PUT")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");

        if let Some(cookie) = auth_cookie {
            request_builder = request_builder.header("cookie", cookie);
        }

        let request = request_builder
            .body(Body::from(form_data.to_string()))
            .unwrap();

        app.clone()
            .with_state(state.clone())
            .oneshot(request)
            .await
            .unwrap()
    }

    /// Make a DELETE request for handler tests (returns Response directly)
    pub async fn make_handler_delete_request(
        app: &Router<AppState>,
        state: &AppState,
        uri: &str,
        auth_cookie: Option<axum::http::HeaderValue>,
    ) -> axum::http::Response<Body> {
        let mut request_builder = Request::builder().method("DELETE").uri(uri);

        if let Some(cookie) = auth_cookie {
            request_builder = request_builder.header("cookie", cookie);
        }

        let request = request_builder.body(Body::empty()).unwrap();

        app.clone()
            .with_state(state.clone())
            .oneshot(request)
            .await
            .unwrap()
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
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            !body_str.contains(unexpected_text),
            "Response body should not contain '{unexpected_text}', but it does: {body_str}"
        );
    }

    /// Assert that a response contains validation errors with 200 OK status
    /// This is the expected behavior for HTMX applications when validation fails
    pub async fn assert_validation_error(response: axum::http::Response<Body>) {
        // First assert that the status is 200 OK
        Self::assert_status(&response, StatusCode::OK);

        // Then check that the body contains error keywords
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);

        // Check for various error keywords that might appear in validation error messages
        let has_error = body_str.contains("error")
            || body_str.contains("invalid")
            || body_str.contains("validation")
            || body_str.contains("Invalid")
            || body_str.contains("Error");

        assert!(
            has_error,
            "Response should contain validation error message, but body was: {body_str}"
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

    /// Get the default database pool and clean up the test database
    /// This is a common pattern used in handler tests
    pub async fn setup_test_db_pool(
        state: &AppState,
    ) -> diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::MysqlConnection>> {
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");

        // Import the cleanup function
        use crate::test_helpers::common::cleanup_test_db;
        cleanup_test_db(&pool);

        pool
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

    /// Generate a unique test alias name (without domain)
    pub fn unique_alias_name() -> String {
        format!("alias-{}", crate::test_helpers::common::unique_test_id())
    }

    /// Generate a unique test backup domain
    pub fn unique_backup_domain() -> String {
        format!(
            "backup-{}.example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate a unique test relay recipient
    pub fn unique_relay_recipient() -> String {
        format!(
            "relay-{}@example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate a unique test relocated address
    pub fn unique_relocated_address() -> String {
        format!(
            "relocated-{}@example.com",
            crate::test_helpers::common::unique_test_id()
        )
    }

    /// Generate a unique test client name
    pub fn unique_client_name() -> String {
        format!("client-{}", crate::test_helpers::common::unique_test_id())
    }

    // Domain factories
    /// Create a domain with random data
    pub fn random_domain() -> String {
        let domains = vec![
            "example.com",
            "test.org",
            "demo.net",
            "sample.co.uk",
            "trial.io",
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_domain = domains[rng.gen_range(0..domains.len())];
        format!("{}-{}.{}", 
            Self::random_word(), 
            crate::test_helpers::common::unique_test_id(),
            random_domain
        )
    }

    /// Create a domain with specific characteristics
    pub fn domain_with_suffix(suffix: &str) -> String {
        format!("{}-{}.{}", 
            Self::random_word(), 
            crate::test_helpers::common::unique_test_id(),
            suffix
        )
    }

    /// Create a domain with specific length
    pub fn domain_with_length(length: usize) -> String {
        let name = Self::random_string(length);
        format!("{}.example.com", name)
    }

    // User factories
    /// Create a user with random data
    pub fn random_user() -> String {
        let usernames = vec![
            "john", "jane", "bob", "alice", "charlie", "diana", "edward", "fiona"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_username = usernames[rng.gen_range(0..usernames.len())];
        format!("{}@{}", random_username, Self::random_domain())
    }

    /// Create a user with specific domain
    pub fn user_for_domain(domain: &str) -> String {
        let usernames = vec![
            "admin", "user", "test", "demo", "sample", "trial"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_username = usernames[rng.gen_range(0..usernames.len())];
        format!("{}@{}", random_username, domain)
    }

    /// Create a user with specific characteristics
    pub fn user_with_pattern(pattern: &str) -> String {
        format!("{}-{}@example.com", 
            pattern, 
            crate::test_helpers::common::unique_test_id()
        )
    }

    // Alias factories
    /// Create an alias with random data
    pub fn random_alias() -> String {
        let alias_names = vec![
            "info", "admin", "support", "sales", "contact", "help", "webmaster"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_name = alias_names[rng.gen_range(0..alias_names.len())];
        format!("{}@{}", random_name, Self::random_domain())
    }

    /// Create an alias for a specific domain
    pub fn alias_for_domain(domain: &str) -> String {
        let alias_names = vec![
            "postmaster", "abuse", "webmaster", "admin", "info"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_name = alias_names[rng.gen_range(0..alias_names.len())];
        format!("{}@{}", random_name, domain)
    }

    /// Create an alias with specific name and domain
    pub fn alias_with_name(name: &str, domain: &str) -> String {
        format!("{}@{}", name, domain)
    }

    // Transport factories
    /// Generate a random transport string
    pub fn random_transport() -> String {
        let transports = vec![
            "smtp:localhost",
            "smtp:mail.example.com",
            "smtp:relay.example.com",
            "smtp:mx.example.com",
            "smtp:backup.example.com"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        transports[rng.gen_range(0..transports.len())].to_string()
    }

    /// Generate a transport with specific server
    pub fn transport_with_server(server: &str) -> String {
        format!("smtp:{}", server)
    }

    // Password factories
    /// Generate a random password
    pub fn random_password() -> String {
        let passwords = vec![
            "password123",
            "securepass456",
            "testpass789",
            "demo123pass",
            "sample456pass"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        passwords[rng.gen_range(0..passwords.len())].to_string()
    }

    /// Generate a password with specific pattern
    pub fn password_with_pattern(pattern: &str) -> String {
        format!("{}{}", pattern, crate::test_helpers::common::unique_test_id())
    }

    // Name factories
    /// Generate a random name
    pub fn random_name() -> String {
        let names = vec![
            "John Doe", "Jane Smith", "Bob Wilson", "Alice Brown", 
            "Charlie Davis", "Diana Miller", "Edward Garcia", "Fiona Rodriguez"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        names[rng.gen_range(0..names.len())].to_string()
    }

    /// Generate a name with specific pattern
    pub fn name_with_pattern(pattern: &str) -> String {
        format!("{} {}", pattern, crate::test_helpers::common::unique_test_id())
    }

    // Helper methods for randomization
    /// Generate a random word
    fn random_word() -> String {
        let words = vec![
            "test", "demo", "sample", "trial", "example", "mock", "fake", "dummy"
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        words[rng.gen_range(0..words.len())].to_string()
    }

    /// Generate a random string of specified length
    fn random_string(length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        (0..length)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }

    // Form data factories with improved defaults
    /// Generate form data for creating a domain with sensible defaults
    pub fn domain_form_data(domain: &str, transport: &str, enabled: bool) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        format!("domain={domain}&transport={transport}&enabled={enabled_str}")
    }

    /// Generate form data for creating a domain with random data
    pub fn random_domain_form_data() -> String {
        let domain = Self::random_domain();
        let transport = Self::random_transport();
        let enabled = rand::random::<bool>();
        Self::domain_form_data(&domain, &transport, enabled)
    }

    /// Generate form data for creating a user with sensible defaults
    pub fn user_form_data(user_id: &str, password: &str, username: &str) -> String {
        format!("id={user_id}&name={username}&password={password}&maildir=testdir&home=/var/spool/mail/virtual")
    }

    /// Generate form data for creating a user with random data
    pub fn random_user_form_data() -> String {
        let user_id = Self::random_user();
        let password = Self::random_password();
        let username = Self::random_name();
        Self::user_form_data(&user_id, &password, &username)
    }

    /// Generate form data for creating a user with complete parameters
    pub fn user_form_data_complete(
        user_id: &str,
        password: &str,
        name: &str,
        maildir: &str,
        home: &str,
        enabled: bool,
        change_password: bool,
    ) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        let change_password_str = if change_password { "on" } else { "" };
        format!(
            "id={user_id}&password={password}&name={name}&maildir={maildir}&home={home}&enabled={enabled_str}&change_password={change_password_str}"
        )
    }

    /// Generate form data for creating a user with random complete data
    pub fn random_user_form_data_complete() -> String {
        let user_id = Self::random_user();
        let password = Self::random_password();
        let name = Self::random_name();
        let maildir = format!("{}/", user_id.split('@').next().unwrap_or("user"));
        let home = "/var/spool/mail/virtual";
        let enabled = rand::random::<bool>();
        let change_password = rand::random::<bool>();
        Self::user_form_data_complete(&user_id, &password, &name, &maildir, home, enabled, change_password)
    }

    /// Generate form data for creating an alias with sensible defaults
    pub fn alias_form_data(mail: &str, destination: &str, enabled: bool) -> String {
        let enabled_str = if enabled { "on" } else { "" };
        format!("mail={mail}&destination={destination}&enabled={enabled_str}")
    }

    /// Generate form data for creating an alias with random data
    pub fn random_alias_form_data() -> String {
        let mail = Self::random_alias();
        let destination = Self::random_user();
        let enabled = rand::random::<bool>();
        Self::alias_form_data(&mail, &destination, enabled)
    }

    /// Generate form data for creating a backup with sensible defaults
    pub fn backup_form_data(domain: &str, transport: &str) -> String {
        format!("domain={domain}&transport={transport}")
    }

    /// Generate form data for creating a backup with random data
    pub fn random_backup_form_data() -> String {
        let domain = Self::random_domain();
        let transport = Self::random_transport();
        Self::backup_form_data(&domain, &transport)
    }

    /// Generate form data for creating a relay with random data
    pub fn random_relay_form_data() -> String {
        let recipient = Self::unique_relay_recipient();
        let status = "active";
        let enabled = rand::random::<bool>();
        let enabled_str = if enabled { "on" } else { "" };
        format!("recipient={recipient}&status={status}&enabled={enabled_str}")
    }

    /// Generate form data for creating a relocated entry with random data
    pub fn random_relocated_form_data() -> String {
        let old_address = Self::unique_relocated_address();
        let new_address = Self::unique_relocated_address();
        let enabled = rand::random::<bool>();
        let enabled_str = if enabled { "on" } else { "" };
        format!("old_address={old_address}&new_address={new_address}&enabled={enabled_str}")
    }

    /// Generate form data for creating a client with random data
    pub fn random_client_form_data() -> String {
        let client = Self::unique_client_name();
        let status = "active";
        let enabled = rand::random::<bool>();
        let enabled_str = if enabled { "on" } else { "" };
        format!("client={client}&status={status}&enabled={enabled_str}")
    }

    // Test data sets for comprehensive testing
    /// Generate a complete test dataset (domain, user, alias)
    pub fn complete_test_dataset() -> (String, String, String) {
        let domain = Self::unique_domain();
        let user = Self::user_for_domain(&domain);
        let alias = Self::alias_for_domain(&domain);
        (domain, user, alias)
    }

    /// Generate multiple test datasets
    pub fn multiple_test_datasets(count: usize) -> Vec<(String, String, String)> {
        (0..count)
            .map(|_| Self::complete_test_dataset())
            .collect()
    }

    /// Generate edge case test data
    pub fn edge_case_test_data() -> Vec<(String, String, String)> {
        vec![
            ("a.com".to_string(), "user@a.com".to_string(), "alias@a.com".to_string()),
            ("very-long-domain-name-that-exceeds-normal-limits.example.com".to_string(), 
             "very-long-user-name@very-long-domain-name-that-exceeds-normal-limits.example.com".to_string(),
             "very-long-alias-name@very-long-domain-name-that-exceeds-normal-limits.example.com".to_string()),
            ("test..com".to_string(), "user@test..com".to_string(), "alias@test..com".to_string()),
            ("test@domain.com".to_string(), "user@test@domain.com".to_string(), "alias@test@domain.com".to_string()),
        ]
    }

    /// Generate security test data (SQL injection, XSS attempts)
    pub fn security_test_data() -> Vec<(String, String, String)> {
        vec![
            ("'; DROP TABLE domains; --".to_string(), "user@test.com".to_string(), "alias@test.com".to_string()),
            ("<script>alert('xss')</script>.com".to_string(), "user@test.com".to_string(), "alias@test.com".to_string()),
            ("' OR '1'='1".to_string(), "user@test.com".to_string(), "alias@test.com".to_string()),
            ("javascript:alert('xss')".to_string(), "user@test.com".to_string(), "alias@test.com".to_string()),
        ]
    }

    /// Generate Unicode test data
    pub fn unicode_test_data() -> Vec<(String, String, String)> {
        vec![
            ("tëst.com".to_string(), "user@tëst.com".to_string(), "alias@tëst.com".to_string()),
            ("测试.com".to_string(), "user@测试.com".to_string(), "alias@测试.com".to_string()),
            ("tëst@domain.com".to_string(), "user@tëst@domain.com".to_string(), "alias@tëst@domain.com".to_string()),
        ]
    }
}

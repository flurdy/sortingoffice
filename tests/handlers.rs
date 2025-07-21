#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    use sortingoffice::{
        config::{AdminRole, Config, DatabaseConfig, DatabaseFeatures},
        db::{self, DatabaseManager},
        handlers,
        i18n::I18n,
        models::{AliasForm, NewBackup, NewDomain, UserForm},
        AppState,
    };

    use sortingoffice::test_helpers::common::{cleanup_test_db, setup_test_db, unique_test_id};

    async fn create_test_app() -> (Router, AppState) {
        let _pool = setup_test_db();
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
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: std::env::var("TEST_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "mysql://root:password@localhost/sortingoffice_test".to_string()
                }),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
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

    async fn create_test_app_with_dbs(db_configs: Vec<DatabaseConfig>) -> (Router, AppState) {
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
        let app = Router::new()
            .route(
                "/database/dropdown",
                axum::routing::get(handlers::database::dropdown),
            )
            .with_state(state.clone());
        (app, state)
    }

    // Helper function to create an authenticated cookie with a specific role
    fn create_auth_cookie(role: AdminRole) -> axum::http::HeaderValue {
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; // 1 hour in the future
        let role_str = match role {
            AdminRole::ReadOnly => "read-only",
            AdminRole::Edit => "edit",
        };
        let db_id = "test";
        let cookie = format!(
            "authenticated={}:{}:{}; Path=/; Max-Age=3600; HttpOnly; SameSite=Lax",
            expiry, role_str, db_id
        );
        axum::http::HeaderValue::from_str(&cookie).unwrap()
    }

    #[tokio::test]
    async fn test_domains_list() {
        let (app, _state) = create_test_app().await;
        let _pool = setup_test_db();

        // Create test domain with unique name
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("list-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&_pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/domains")
                    .header("Cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was created
        let _pool = setup_test_db();
        let domains = db::get_domains(&_pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains
            .iter()
            .any(|d| d.domain == format!("list-test-{}.com", unique_id)));
    }

    #[tokio::test]
    async fn test_domains_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        let unique_id = unique_test_id();
        let form_data = format!(
            "domain=create-test-{}.com&transport=smtp%3Alocalhost&enabled=on",
            unique_id
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was created
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        let domains = db::get_domains(&pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains
            .iter()
            .any(|d| d.domain == format!("create-test-{}.com", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_show() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("show-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}", _domain.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("show-test-{}.com", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_edit() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("edit-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}/edit", _domain.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("edit-test-{}", unique_id)));
        assert!(body_str.contains("Edit Domain"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_update() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("update-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let form_data = format!(
            "domain=updated-test-{}.com&transport=smtp%3Aupdated&enabled=on",
            unique_id
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/domains/{}", _domain.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was updated
        let updated_domain = db::get_domain(&pool, _domain.pkid).unwrap();
        assert_eq!(
            updated_domain.domain,
            format!("updated-test-{}.com", unique_id)
        );
        assert_eq!(updated_domain.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_toggle_enabled() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("toggle-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", _domain.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was toggled
        let toggled_domain = db::get_domain(&pool, _domain.pkid).unwrap();
        assert_eq!(toggled_domain.enabled, false);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_list() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("list-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user with unique name
        let user_form = UserForm {
            id: format!("testuser@list-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("list-test-{}", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("create-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let form_data = format!(
            "id=testuser@create-test-{}.com&password=password123&name=Test+User&maildir=testdir&home=/var/spool/mail/virtual&enabled=on",
            unique_id
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was created
        let users = db::get_users(&pool).unwrap();
        assert!(!users.is_empty());
        assert!(users
            .iter()
            .any(|u| u.id == format!("testuser@create-test-{}.com", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_show() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("show-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_form = UserForm {
            id: format!("testuser@show-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}", _user.id))
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("show-test-{}", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_edit() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("edit-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_form = UserForm {
            id: format!("testuser@edit-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}/edit", _user.id))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("edit-test-{}", unique_id)));
        assert!(body_str.contains("Edit User"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_update() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("update-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_form = UserForm {
            id: format!("testuser@update-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let form_data = format!(
            "id=updateduser@update-test-{}.com&password=password123&name=Updated+User&maildir=testdir&home=/var/spool/mail/virtual&enabled=on",
            unique_id
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/users/{}", _user.id))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was updated
        let updated_user =
            db::get_user(&pool, format!("updateduser@update-test-{}.com", unique_id)).unwrap();
        assert_eq!(
            updated_user.id,
            format!("updateduser@update-test-{}.com", unique_id)
        );
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.enabled, true);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_toggle_enabled() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("toggle-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_form = UserForm {
            id: format!("testuser@toggle-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", _user.id))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was toggled
        let toggled_user = db::get_user(&pool, _user.id).unwrap();
        assert_eq!(toggled_user.enabled, false);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_aliases_list() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain and alias
        let new_domain = NewDomain {
            domain: "aliases-list-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let alias_form = AliasForm {
            mail: "test@aliases-list-test.com".to_string(),
            destination: "user@aliases-list-test.com".to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/aliases")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("test@aliases-list-test.com"));
        assert!(body_str.contains("user@aliases-list-test.com"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_aliases_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain first
        let new_domain = NewDomain {
            domain: "aliases-create-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let form_data = "mail=test%40aliases-create-test.com&destination=user%40aliases-create-test.com&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/aliases")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify alias was created
        let aliases = db::get_aliases(&pool).unwrap();
        assert!(!aliases.is_empty());
        assert!(aliases
            .iter()
            .any(|a| a.mail == "test@aliases-create-test.com"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_stats() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test data
        let new_domain = NewDomain {
            domain: "stats-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let user_form = UserForm {
            id: "testuser@stats-test.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain statistics information
        assert!(body_str.contains("Statistics") || body_str.contains("stats"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_dashboard() {
        let (app, state) = create_test_app().await;
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_about() {
        let (app, _state) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/about")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("About Sorting Office"));
        assert!(body_str.contains("comprehensive mail server administration tool"));
    }

    #[tokio::test]
    async fn test_not_found() {
        let (app, state) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notfound")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        cleanup_test_db(
            &state
                .db_manager
                .get_default_pool()
                .await
                .expect("Failed to get database pool"),
        );
    }

    #[tokio::test]
    async fn test_backups_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        let form_data = "domain=backup-create-test.com&transport=smtp%3Alocalhost&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backups")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was created
        let backups = db::get_backups(&pool).unwrap();
        assert!(!backups.is_empty());
        assert!(backups.iter().any(|b| b.domain == "backup-create-test.com"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_show() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-show-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/backups/{}", _backup.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("backup-show-test.com"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_edit() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-edit-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/backups/{}/edit", _backup.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("backup-edit-test.com"));
        assert!(body_str.contains("Edit Backup"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_update() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-update-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let form_data = "domain=backup-updated-test.com&transport=smtp%3Aupdated&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/backups/{}", _backup.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was updated
        let updated_backup = db::get_backup(&pool, _backup.pkid).unwrap();
        assert_eq!(updated_backup.domain, "backup-updated-test.com");
        assert_eq!(updated_backup.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_toggle_enabled() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-toggle-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/backups/{}/toggle", _backup.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was toggled
        let toggled_backup = db::get_backup(&pool, _backup.pkid).unwrap();
        assert_eq!(toggled_backup.enabled, false);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_create_redirects_to_domains() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        let form_data = "domain=backup-redirect-test.com&transport=smtp%3Alocalhost&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backups")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain redirect script to domains page
        assert!(body_str.contains("window.location.href='/domains'"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_update_returns_content_only() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-update-content-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let form_data =
            "domain=backup-updated-content-test.com&transport=smtp%3Aupdated&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/backups/{}", _backup.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should not contain full template elements (indicating content-only response)
        assert!(!body_str.contains("<html>"));
        assert!(!body_str.contains("<head>"));
        assert!(!body_str.contains("<body>"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_delete_redirects_to_domains() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test backup
        let new_backup = NewBackup {
            domain: "backup-delete-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/backups/{}", _backup.pkid))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain redirect script to domains page
        assert!(body_str.contains("window.location.href='/domains'"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_list_includes_backups() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("domain-backup-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test backup
        let new_backup = NewBackup {
            domain: format!("backup-domain-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/domains")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain both domain and backup
        assert!(body_str.contains(&format!("domain-backup-test-{}", unique_id)));
        assert!(body_str.contains(&format!("backup-domain-test-{}", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_login_success() {
        use axum::extract::State;
        use axum::http::HeaderMap;
        use axum::Form;
        use sortingoffice::config::{AdminCredentials, AdminRole, Config};
        use sortingoffice::handlers::auth::{login, LoginRequest};
        use sortingoffice::AppState;

        let _pool = setup_test_db();
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");
        let config = Config {
            admins: vec![AdminCredentials {
                username: "admin".to_string(),
                password_hash: "$2b$12$KGfzf4xNi5FgHBN0/h2aLukhHgOIKz.mG1pavh4bgAkZpZJvyeBYO"
                    .to_string(),
                role: AdminRole::Edit,
            }],
            ..Config::default()
        };
        let db_config = vec![DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: std::env::var("TEST_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "mysql://root:password@localhost/sortingoffice_test".to_string()
                }),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
        }];
        let db_manager = DatabaseManager::new(db_config)
            .await
            .expect("Failed to create database manager");
        let state = AppState {
            db_manager,
            i18n,
            config,
        };
        let headers = HeaderMap::new();
        let req = LoginRequest {
            id: "admin".to_string(),
            password: "admin123".to_string(),
        };
        let result = login(State(state), headers, Form(req)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::FOUND);
        let set_cookie = response
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(set_cookie.contains("authenticated="));
        assert!(set_cookie.contains("edit"));
    }

    #[tokio::test]
    async fn test_login_failure() {
        use axum::extract::State;
        use axum::http::HeaderMap;
        use axum::Form;
        use sortingoffice::config::Config;
        use sortingoffice::handlers::auth::{login, LoginRequest};
        use sortingoffice::AppState;

        let _pool = setup_test_db();
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");
        let config = Config::default();
        let db_config = vec![DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: std::env::var("TEST_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "mysql://root:password@localhost/sortingoffice_test".to_string()
                }),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
        }];
        let db_manager = DatabaseManager::new(db_config)
            .await
            .expect("Failed to create database manager");
        let state = AppState {
            db_manager,
            i18n,
            config,
        };
        let headers = HeaderMap::new();
        let req = LoginRequest {
            id: "admin".to_string(),
            password: "wrongpassword".to_string(),
        };
        let result = login(State(state), headers, Form(req)).await;
        assert!(result.is_err());
        let html = result.err().unwrap().0;
        assert!(html.contains("Error") || html.contains("error"));
    }

    #[tokio::test]
    async fn test_is_authenticated_cookie() {
        use axum::http::HeaderMap;
        use sortingoffice::handlers::auth::is_authenticated;
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + 3600;
        let cookie = format!("authenticated={}:edit:test", expiry);
        let mut headers = HeaderMap::new();
        let header_value = cookie.parse().unwrap();
        headers.insert("cookie", header_value);
        assert!(is_authenticated(&headers));
    }

    #[tokio::test]
    async fn test_is_authenticated_cookie_expired() {
        use axum::http::HeaderMap;
        use sortingoffice::handlers::auth::is_authenticated;
        let expiry = 1; // long expired
        let cookie = format!("authenticated={}:edit:test", expiry);
        let mut headers = HeaderMap::new();
        headers.insert("cookie", cookie.parse().unwrap());
        assert!(!is_authenticated(&headers));
    }

    #[tokio::test]
    async fn test_is_authenticated_cookie_readonly() {
        use axum::http::HeaderMap;
        use sortingoffice::handlers::auth::is_authenticated;
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + 3600;
        let cookie = format!("authenticated={}:read-only:test", expiry);
        let mut headers = HeaderMap::new();
        let header_value = cookie.parse().unwrap();
        headers.insert("cookie", header_value);
        assert!(is_authenticated(&headers));
    }

    #[tokio::test]
    async fn test_has_edit_permissions() {
        use axum::http::HeaderMap;
        use sortingoffice::handlers::auth::{has_edit_permissions, is_authenticated};
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + 3600;

        // Test edit role
        let cookie = format!("authenticated={}:edit:test", expiry);
        let mut headers = HeaderMap::new();
        let header_value = cookie.parse().unwrap();
        headers.insert("cookie", header_value);
        assert!(is_authenticated(&headers));
        assert!(has_edit_permissions(&headers));

        // Test read-only role
        let cookie = format!("authenticated={}:read-only:test", expiry);
        let mut headers = HeaderMap::new();
        let header_value = cookie.parse().unwrap();
        headers.insert("cookie", header_value);
        assert!(is_authenticated(&headers));
        assert!(!has_edit_permissions(&headers));
    }

    #[tokio::test]
    async fn test_role_based_access_control() {
        use axum::http::HeaderMap;
        use std::time::{SystemTime, UNIX_EPOCH};

        let (app, state) = create_test_app().await;
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");
        cleanup_test_db(&pool);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + 3600;

        // Test read-only user can access read-only routes
        let cookie = format!("authenticated={}:read-only:test", expiry);
        let mut headers = HeaderMap::new();
        headers.insert("cookie", cookie.parse().unwrap());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/domains")
                    .header("cookie", headers.get("cookie").unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test read-only user gets 403 for edit routes
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header("cookie", headers.get("cookie").unwrap())
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=test.com&transport=smtp%3Alocalhost&enabled=on",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Test edit user can access edit routes
        let cookie = format!("authenticated={}:edit:test", expiry);
        let mut headers = HeaderMap::new();
        headers.insert("cookie", cookie.parse().unwrap());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header("cookie", headers.get("cookie").unwrap())
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=test-edit.com&transport=smtp%3Alocalhost&enabled=on",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_not_found_handler_anonymous() {
        let (app, _state) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notfound")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("404"));
    }

    #[tokio::test]
    async fn test_aliases_search() {
        let (app, state) = create_test_app().await;
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");

        // Clean up before test
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("search-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test aliases for search
        let alias1 = AliasForm {
            mail: format!("admin@search-test-{}.com", unique_id),
            destination: "user@company.com".to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias1 = db::create_alias(&pool, alias1).unwrap();

        let alias2 = AliasForm {
            mail: format!("support@search-test-{}.com", unique_id),
            destination: "helpdesk@company.com".to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias2 = db::create_alias(&pool, alias2).unwrap();

        // Test 1: Search with valid query
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/aliases/search?destination=user"))
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain search results
        assert!(body_str.contains("user@company.com"));

        // Test 2: Search with short query (should return empty results, not error)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/aliases/search?destination=a")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should not contain search results for short query
        assert!(!body_str.contains("user@company.com"));

        // Test 3: Search with empty query (should not cause 400 error)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/aliases/search?destination=")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test 4: Search without query parameter (should not cause 400 error)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/aliases/search")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test 5: Search in mail field (should find results)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/aliases/search?destination=admin"))
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain search results from mail field
        assert!(body_str.contains("admin@search-test-"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domain_search() {
        let (app, state) = create_test_app().await;
        let pool = state
            .db_manager
            .get_default_pool()
            .await
            .expect("Failed to get database pool");

        // Clean up before test
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let new_domain = NewDomain {
            domain: format!("search-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Test domain search with a query
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/aliases/domain-search?domain=search-test-{}",
                        unique_id
                    ))
                    .header("cookie", create_auth_cookie(AdminRole::Edit))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain domain search results
        assert!(body_str.contains(&format!("search-test-{}", unique_id)));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_database_dropdown() {
        use diesel::RunQueryDsl;
        use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;
        let container = setup_test_db();
        let port = container.get_mysql_port();
        let url1 = format!("mysql://root@127.0.0.1:{}/testdb1", port);
        let url2 = format!("mysql://root@127.0.0.1:{}/testdb2", port);
        // Create both databases in the container
        {
            let _pool = container.get_pool();
            let mut conn = _pool.get().unwrap();
            diesel::sql_query("CREATE DATABASE IF NOT EXISTS testdb1")
                .execute(&mut conn)
                .unwrap();
            diesel::sql_query("CREATE DATABASE IF NOT EXISTS testdb2")
                .execute(&mut conn)
                .unwrap();
        }
        let db_config1 = DatabaseConfig {
            id: "test1".to_string(),
            label: "Test Database 1".to_string(),
            url: url1,
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
        };
        let db_config2 = DatabaseConfig {
            id: "test2".to_string(),
            label: "Test Database 2".to_string(),
            url: url2,
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
        };
        let (app, _state) =
            create_test_app_with_dbs(vec![db_config1.clone(), db_config2.clone()]).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/database/dropdown")
                    .header("cookie", create_auth_cookie(AdminRole::ReadOnly))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        // Should contain both database labels
        assert!(body_str.contains("Test Database 1"));
        assert!(body_str.contains("Test Database 2"));
        // Should contain a form for /database/select
        assert!(body_str.contains("/database/select"));
    }
}

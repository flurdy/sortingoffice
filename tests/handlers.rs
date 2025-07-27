#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, Router};

    use sortingoffice::{
        config::{AdminRole, Config, DatabaseConfig, DatabaseFeatures},
        db::{self, DatabaseManager},
        handlers,
        i18n::I18n,
        models::{AliasForm, NewBackup, NewDomain, UserForm},
        AppState,
    };

    use sortingoffice::test_helpers::common::{cleanup_test_db, unique_test_id};
    use sortingoffice::test_helpers::test_utils::{TestData, TestUtils};
    use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

    async fn create_test_app() -> (Router<AppState>, AppState) {
        let container = setup_test_db().await;
        TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await
    }

    async fn create_test_app_with_dbs(
        db_configs: Vec<DatabaseConfig>,
        port: u16,
    ) -> (Router<AppState>, AppState) {
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

        // Load translation files for testing
        let _ = i18n.load_locale("en-US").await;
        let _ = i18n.load_locale("es-ES").await;
        let _ = i18n.load_locale("nb-NO").await;
        let _ = i18n.load_locale("fr-FR").await;
        let _ = i18n.load_locale("de-DE").await;
        let config = Config::default();
        // Update all db_configs to use the dynamic port
        let db_configs: Vec<DatabaseConfig> = db_configs
            .into_iter()
            .map(|mut cfg| {
                let url = cfg.url.replace(":3306", &format!(":{port}"));
                cfg.url = url;
                cfg
            })
            .collect();
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
        TestUtils::create_auth_cookie(role)
    }

    #[tokio::test]
    async fn test_domains_list() {
        let (app, _state) = create_test_app().await;
        let container = setup_test_db().await;

        // Create test domain with unique name
        let unique_id = unique_test_id();
        let domain = format!("list-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let _domain = db::create_domain(pool, new_domain).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            "/domains",
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify domain was created
        let domains = db::get_domains(pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains.iter().any(|d| d.domain == domain));
    }

    #[tokio::test]
    async fn test_domains_create() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        let unique_id = unique_test_id();
        let domain = format!("create-test-{unique_id}.com");
        let form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/domains",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify domain was created
        let domains = db::get_domains(&pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains.iter().any(|d| d.domain == domain));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_show() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("show-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/domains/{}", _domain.pkid),
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&domain));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_edit() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("edit-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/domains/{}/edit", _domain.pkid),
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("edit-test-{unique_id}")));
        assert!(body_str.contains("Edit Domain"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_update() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("update-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let updated_domain_name = format!("updated-test-{unique_id}.com");
        let form_data = TestData::domain_form_data(&updated_domain_name, "smtp:updated", true);

        let response = TestUtils::make_handler_put_request(
            &app,
            &state,
            &format!("/domains/{}", _domain.pkid),
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify domain was updated
        let updated_domain = db::get_domain(&pool, _domain.pkid).unwrap();
        assert_eq!(updated_domain.domain, updated_domain_name);
        assert_eq!(updated_domain.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_domains_toggle_enabled() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("toggle-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Toggle to disabled
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &format!("/domains/{}/toggle", _domain.pkid),
            "",
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify domain was toggled
        let toggled_domain = db::get_domain(&pool, _domain.pkid).unwrap();
        assert!(!toggled_domain.enabled);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_list() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("list-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user with unique name
        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/users",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("list-test-{unique_id}")));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("create-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let user_id = format!("testuser@{}", domain);
        let form_data = TestData::user_form_data_complete(
            &user_id,
            "password123",
            "Test User",
            "testdir",
            "/var/spool/mail/virtual",
            &domain,
            "100000",
            true,
            false,
        );

        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/users",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify user was created
        let users = db::get_users(&pool).unwrap();
        assert!(!users.is_empty());
        assert!(users.iter().any(|u| u.id == user_id));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_show() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("show-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/users/{}", _user.id),
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("show-test-{unique_id}")));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_edit() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("edit-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/users/{}/edit", _user.id),
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(&format!("edit-test-{unique_id}")));
        assert!(body_str.contains("Edit User"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_update() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("update-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let updated_user_id = format!("updateduser@{}", domain);
        let form_data = TestData::user_form_data_complete(
            &updated_user_id,
            "password123",
            "Updated User",
            "testdir",
            "/var/spool/mail/virtual",
            &domain,
            "100000",
            true,
            false,
        );

        let response = TestUtils::make_handler_put_request(
            &app,
            &state,
            &format!("/users/{}", _user.id),
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify user was updated
        let updated_user = db::get_user(&pool, updated_user_id.clone()).unwrap();
        assert_eq!(updated_user.id, updated_user_id);
        assert_eq!(updated_user.name, "Updated User");
        assert!(updated_user.enabled);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_users_toggle_enabled() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let unique_id = unique_test_id();
        let domain = format!("toggle-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test user
        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        // Toggle to disabled
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &format!("/users/{}/toggle", _user.id),
            "",
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify user was toggled
        let toggled_user = db::get_user(&pool, _user.id).unwrap();
        assert!(!toggled_user.enabled);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_aliases_list() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain and alias
        let domain = "aliases-list-test.com";
        let new_domain = NewDomain {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let mail = "test@aliases-list-test.com";
        let destination = "user@aliases-list-test.com";
        let alias_form = AliasForm {
            mail: mail.to_string(),
            destination: destination.to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(mail));
        assert!(body_str.contains(destination));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_aliases_create() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain first
        let domain = "aliases-create-test.com";
        let new_domain = NewDomain {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let mail = "test@aliases-create-test.com";
        let destination = "user@aliases-create-test.com";
        let form_data = TestData::alias_form_data(mail, destination, true);

        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/aliases",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify alias was created
        let aliases = db::get_aliases(&pool).unwrap();
        assert!(!aliases.is_empty());
        assert!(aliases.iter().any(|a| a.mail == mail));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_stats() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test data
        let domain = "stats-test.com";
        let new_domain = NewDomain {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        let user_id = format!("testuser@{}", domain);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
            change_password: false,
            maildir: "testdir".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let mail = "test@test.com";
        let destination = "user@test.com";
        let alias_form = AliasForm {
            mail: mail.to_string(),
            destination: destination.to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/stats",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

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

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_about() {
        let (app, _state) = create_test_app().await;

        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            "/about",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

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

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/notfound",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::NOT_FOUND);

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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        let domain = "backup-create-test.com";
        let form_data = TestData::domain_form_data(domain, "smtp:localhost", true);

        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/backups",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify backup was created
        let backups = db::get_backups(&pool).unwrap();
        assert!(!backups.is_empty());
        assert!(backups.iter().any(|b| b.domain == domain));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_show() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-show-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/backups/{}", _backup.pkid),
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(domain));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_edit() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-edit-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/backups/{}/edit", _backup.pkid),
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains(domain));
        assert!(body_str.contains("Edit Backup"));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_update() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-update-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let updated_domain = "updated-backup-test.com";
        let form_data = TestData::domain_form_data(updated_domain, "smtp:updated", true);

        let response = TestUtils::make_handler_put_request(
            &app,
            &state,
            &format!("/backups/{}", _backup.pkid),
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify backup was updated
        let updated_backup = db::get_backup(&pool, _backup.pkid).unwrap();
        assert_eq!(updated_backup.domain, updated_domain);
        assert_eq!(updated_backup.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_toggle_enabled() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-toggle-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        // Toggle to disabled
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &format!("/backups/{}/toggle", _backup.pkid),
            "",
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify backup was toggled
        let toggled_backup = db::get_backup(&pool, _backup.pkid).unwrap();
        assert!(!toggled_backup.enabled);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_backups_create_redirects_to_domains() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        let domain = "backup-redirect-test.com";
        let form_data = TestData::domain_form_data(domain, "smtp:localhost", true);

        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/backups",
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-update-content-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let updated_domain = "backup-updated-content-test.com";
        let form_data = TestData::domain_form_data(updated_domain, "smtp:updated", true);

        let response = TestUtils::make_handler_put_request(
            &app,
            &state,
            &format!("/backups/{}", _backup.pkid),
            &form_data,
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test backup
        let domain = "backup-delete-test.com";
        let new_backup = NewBackup {
            domain: domain.to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = TestUtils::make_handler_delete_request(
            &app,
            &state,
            &format!("/backups/{}", _backup.pkid),
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("domain-backup-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test backup
        let backup_domain = format!("backup-domain-test-{unique_id}.com");
        let new_backup = NewBackup {
            domain: backup_domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/domains",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain both domain and backup
        assert!(body_str.contains(&format!("domain-backup-test-{unique_id}")));
        assert!(body_str.contains(&format!("backup-domain-test-{unique_id}")));

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

        let container = setup_test_db().await;
        let schema = container.get_schema();
        let port = container.get_port();
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
            url: format!("mysql://root@127.0.0.1:{port}/{schema}"),
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

        let container = setup_test_db().await;
        let schema = container.get_schema();
        let port = container.get_port();
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");
        let config = Config::default();
        let db_config = vec![DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: format!("mysql://root@127.0.0.1:{port}/{schema}"),
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
        let cookie = format!("authenticated={expiry}:edit:test");
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
        let cookie = format!("authenticated={expiry}:edit:test");
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
        let cookie = format!("authenticated={expiry}:read-only:test");
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
        let cookie = format!("authenticated={expiry}:edit:test");
        let mut headers = HeaderMap::new();
        let header_value = cookie.parse().unwrap();
        headers.insert("cookie", header_value);
        assert!(is_authenticated(&headers));
        assert!(has_edit_permissions(&headers));

        // Test read-only role
        let cookie = format!("authenticated={expiry}:read-only:test");
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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + 3600;

        // Test read-only user can access read-only routes
        let cookie = format!("authenticated={expiry}:read-only:test");
        let mut headers = HeaderMap::new();
        headers.insert("cookie", cookie.parse().unwrap());

        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/domains",
            Some(headers.get("cookie").unwrap().clone()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Test read-only user gets 403 for edit routes
        let domain = "test.com";
        let form_data = TestData::domain_form_data(domain, "smtp:localhost", true);
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/domains",
            &form_data,
            Some(headers.get("cookie").unwrap().clone()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::FORBIDDEN);

        // Test edit user can access edit routes
        let cookie = format!("authenticated={expiry}:edit:test");
        let mut headers = HeaderMap::new();
        headers.insert("cookie", cookie.parse().unwrap());

        let edit_domain = "test-edit.com";
        let edit_form_data = TestData::domain_form_data(edit_domain, "smtp:localhost", true);
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            "/domains",
            &edit_form_data,
            Some(headers.get("cookie").unwrap().clone()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_not_found_handler_anonymous() {
        let (app, _state) = create_test_app().await;

        let response = TestUtils::make_handler_get_request(&app, &_state, "/notfound", None).await;

        TestUtils::assert_status(&response, StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check for themed 404 page content (should contain the app title and error message)
        assert!(
            body_str.contains("Sorting Office")
                || body_str.contains("Not Found")
                || body_str.contains("404")
        );
    }

    #[tokio::test]
    async fn test_aliases_search() {
        let (app, state) = create_test_app().await;
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Clean up before test
        cleanup_test_db(&pool);

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("search-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Create test aliases for search
        let mail1 = format!("admin@{}", domain);
        let destination1 = "user@company.com";
        let alias1 = AliasForm {
            mail: mail1.clone(),
            destination: destination1.to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias1 = db::create_alias(&pool, alias1).unwrap();

        let mail2 = format!("support@{}", domain);
        let destination2 = "helpdesk@company.com";
        let alias2 = AliasForm {
            mail: mail2.clone(),
            destination: destination2.to_string(),
            enabled: true,
            return_url: None,
        };
        let _alias2 = db::create_alias(&pool, alias2).unwrap();

        // Test 1: Search with valid query
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases/search?destination=user",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain search results
        assert!(body_str.contains(destination1));

        // Test 2: Search with short query (should return empty results, not error)
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases/search?destination=a",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should not contain search results for short query
        assert!(!body_str.contains(destination1));

        // Test 3: Search with empty query (should not cause 400 error)
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases/search?destination=",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Test 4: Search without query parameter (should not cause 400 error)
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases/search",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        // Test 5: Search in mail field (should find results)
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            "/aliases/search?destination=admin",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

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
        let pool = TestUtils::setup_test_db_pool(&state).await;

        // Create test domain
        let unique_id = unique_test_id();
        let domain = format!("search-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();

        // Test domain search with a query
        let response = TestUtils::make_handler_get_request(
            &app,
            &state,
            &format!("/aliases/domain-search?domain=search-test-{unique_id}"),
            Some(create_auth_cookie(AdminRole::Edit)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain domain search results
        assert!(body_str.contains(&format!("search-test-{unique_id}")));

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_database_dropdown() {
        use diesel::RunQueryDsl;
        use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;
        let container = setup_test_db().await;
        let port = container.get_port();
        let url1 = format!("mysql://root@127.0.0.1:{port}/testdb1");
        let url2 = format!("mysql://root@127.0.0.1:{port}/testdb2");
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
            create_test_app_with_dbs(vec![db_config1.clone(), db_config2.clone()], port).await;

        let response = TestUtils::make_handler_get_request(
            &app,
            &_state,
            "/database/dropdown",
            Some(create_auth_cookie(AdminRole::ReadOnly)),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);
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

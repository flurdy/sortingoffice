#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        Router,
    };

    use tower::ServiceExt;

    use crate::handlers;
    use crate::tests::common::{cleanup_test_db, setup_test_db};
    use crate::AppState;

    async fn create_test_app() -> (Router, AppState) {
        let pool = setup_test_db();
        let state = AppState { pool };
        let app = Router::new()
            .route("/domains", axum::routing::get(handlers::domains::list))
            .route("/domains", axum::routing::post(handlers::domains::create))
            .route("/domains/{id}", axum::routing::get(handlers::domains::show))
            .route(
                "/domains/{id}/edit",
                axum::routing::get(handlers::domains::edit),
            )
            .route(
                "/domains/{id}",
                axum::routing::put(handlers::domains::update),
            )
            .route(
                "/domains/{id}",
                axum::routing::delete(handlers::domains::delete),
            )
            .route(
                "/domains/{id}/toggle",
                axum::routing::post(handlers::domains::toggle_enabled),
            )
            .route("/backups", axum::routing::get(handlers::backups::list))
            .route("/backups", axum::routing::post(handlers::backups::create))
            .route("/backups/{id}", axum::routing::get(handlers::backups::show))
            .route("/backups/{id}/edit", axum::routing::get(handlers::backups::edit))
            .route("/backups/{id}", axum::routing::put(handlers::backups::update))
            .route("/backups/{id}", axum::routing::delete(handlers::backups::delete))
            .route(
                "/backups/{id}/toggle-list",
                axum::routing::post(handlers::backups::toggle_enabled),
            )
            .route("/users", axum::routing::get(handlers::users::list))
            .route("/users", axum::routing::post(handlers::users::create))
            .route("/users/{id}", axum::routing::get(handlers::users::show))
            .route("/users/{id}/edit", axum::routing::get(handlers::users::edit))
            .route("/users/{id}", axum::routing::put(handlers::users::update))
            .route("/users/{id}", axum::routing::delete(handlers::users::delete))
            .route(
                "/users/{id}/toggle",
                axum::routing::post(handlers::users::toggle_enabled),
            )
            .route("/aliases", axum::routing::get(handlers::aliases::list))
            .route("/aliases", axum::routing::post(handlers::aliases::create))
            .route("/aliases/{id}", axum::routing::get(handlers::aliases::show))
            .route(
                "/aliases/{id}/edit",
                axum::routing::get(handlers::aliases::edit),
            )
            .route(
                "/aliases/{id}",
                axum::routing::put(handlers::aliases::update),
            )
            .route(
                "/aliases/{id}",
                axum::routing::delete(handlers::aliases::delete),
            )
            .route(
                "/aliases/{id}/toggle-list",
                axum::routing::post(handlers::aliases::toggle_enabled),
            )
            .route("/stats", axum::routing::get(handlers::stats::index))
            .route("/dashboard", axum::routing::get(handlers::dashboard::index))
            .with_state(state.clone());

        (app, state)
    }

    #[tokio::test]
    async fn test_domains_list_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain with unique name
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("list-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/domains")
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

        assert!(body_str.contains(&format!("list-test-{}.com", unique_id)));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_create_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        let unique_id = crate::tests::common::unique_test_id();
        let form_data = format!("domain=create-test-{}.com&transport=smtp%3Alocalhost&enabled=on", unique_id);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was created
        let domains = crate::db::get_domains(&state.pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains.iter().any(|d| d.domain == format!("create-test-{}.com", unique_id)));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_show_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("show-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}", domain.pkid))
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_edit_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("edit-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}/edit", domain.pkid))
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

        assert!(body_str.contains(&format!("edit-test-{}.com", unique_id)));
        assert!(body_str.contains("Edit Domain"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_update_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("update-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = format!("domain=updated-test-{}.com&transport=smtp%3Aupdated&enabled=on", unique_id);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/domains/{}", domain.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was updated
        let updated_domain = crate::db::get_domain(&state.pool, domain.pkid).unwrap();
        assert_eq!(updated_domain.domain, format!("updated-test-{}.com", unique_id));
        assert_eq!(updated_domain.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_toggle_enabled_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("toggle-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", domain.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify domain was toggled
        let toggled_domain = crate::db::get_domain(&state.pool, domain.pkid).unwrap();
        assert_eq!(toggled_domain.enabled, false);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_list_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("list-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Create test user with unique name
        let user_form = crate::models::UserForm {
            id: format!("testuser@list-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let _user = crate::db::create_user(&state.pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_create_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("create-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = format!("id=testuser@create-test-{}.com&password=password123&name=Test+User&domain=create-test-{}.com&enabled=on", unique_id, unique_id);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was created
        let users = crate::db::get_users(&state.pool).unwrap();
        assert!(!users.is_empty());
        assert!(users.iter().any(|u| u.id == format!("testuser@create-test-{}.com", unique_id)));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_show_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("show-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Create test user
        let user_form = crate::models::UserForm {
            id: format!("testuser@show-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let user = crate::db::create_user(&state.pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}", user.pkid))
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_edit_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("edit-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Create test user
        let user_form = crate::models::UserForm {
            id: format!("testuser@edit-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let user = crate::db::create_user(&state.pool, user_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}/edit", user.pkid))
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_update_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("update-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Create test user
        let user_form = crate::models::UserForm {
            id: format!("testuser@update-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let user = crate::db::create_user(&state.pool, user_form).unwrap();

        let form_data = format!("id=updateduser@update-test-{}.com&password=password123&name=Updated+User&domain=update-test-{}.com&enabled=on", unique_id, unique_id);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/users/{}", user.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was updated
        let updated_user = crate::db::get_user(&state.pool, user.pkid).unwrap();
        assert_eq!(updated_user.id, format!("updateduser@update-test-{}.com", unique_id));
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.enabled, true);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_toggle_enabled_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let unique_id = crate::tests::common::unique_test_id();
        let new_domain = crate::models::NewDomain {
            domain: format!("toggle-test-{}.com", unique_id),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        // Create test user
        let user_form = crate::models::UserForm {
            id: format!("testuser@toggle-test-{}.com", unique_id),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let user = crate::db::create_user(&state.pool, user_form).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", user.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify user was toggled
        let toggled_user = crate::db::get_user(&state.pool, user.pkid).unwrap();
        assert_eq!(toggled_user.enabled, false);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_aliases_list_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain and alias
        let new_domain = crate::models::NewDomain {
            domain: "aliases-list-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let alias_form = crate::models::AliasForm {
            mail: "test@aliases-list-test.com".to_string(),
            destination: "user@aliases-list-test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let _alias = crate::db::create_alias(&state.pool, alias_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/aliases")
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_aliases_create_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let new_domain = crate::models::NewDomain {
            domain: "aliases-create-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = "mail=test%40aliases-create-test.com&destination=user%40aliases-create-test.com&domain=aliases-create-test.com&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/aliases")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify alias was created
        let aliases = crate::db::get_aliases(&state.pool).unwrap();
        assert!(!aliases.is_empty());
        assert!(aliases
            .iter()
            .any(|a| a.mail == "test@aliases-create-test.com"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_stats_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test data
        let new_domain = crate::models::NewDomain {
            domain: "stats-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let user_form = crate::models::UserForm {
            id: "testuser@stats-test.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let _user = crate::db::create_user(&state.pool, user_form).unwrap();

        let alias_form = crate::models::AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let _alias = crate::db::create_alias(&state.pool, alias_form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/stats")
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_dashboard_handler() {
        let (app, state) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
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

        // Should contain dashboard content
        assert!(body_str.contains("Dashboard") || body_str.contains("dashboard"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_not_found_handler() {
        let (app, state) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_list_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test backup with unique name
        let new_backup = crate::models::NewBackup {
            domain: "backup-list-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let _backup = crate::db::create_backup(&state.pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/backups")
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

        assert!(body_str.contains("backup-list-test.com"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_create_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        let form_data = "domain=backup-create-test.com&transport=smtp%3Alocalhost&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backups")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was created
        let backups = crate::db::get_backups(&state.pool).unwrap();
        assert!(!backups.is_empty());
        assert!(backups.iter().any(|b| b.domain == "backup-create-test.com"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_show_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test backup
        let new_backup = crate::models::NewBackup {
            domain: "backup-show-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let backup = crate::db::create_backup(&state.pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/backups/{}", backup.pkid))
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_edit_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test backup
        let new_backup = crate::models::NewBackup {
            domain: "backup-edit-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let backup = crate::db::create_backup(&state.pool, new_backup).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/backups/{}/edit", backup.pkid))
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

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_update_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test backup
        let new_backup = crate::models::NewBackup {
            domain: "backup-update-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let backup = crate::db::create_backup(&state.pool, new_backup).unwrap();

        let form_data = "domain=backup-updated-test.com&transport=smtp%3Aupdated&enabled=on";

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/backups/{}", backup.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was updated
        let updated_backup = crate::db::get_backup(&state.pool, backup.pkid).unwrap();
        assert_eq!(updated_backup.domain, "backup-updated-test.com");
        assert_eq!(updated_backup.transport, Some("smtp:updated".to_string()));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_backups_toggle_enabled_handler() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test backup
        let new_backup = crate::models::NewBackup {
            domain: "backup-toggle-test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let backup = crate::db::create_backup(&state.pool, new_backup).unwrap();

        // Toggle to disabled
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/backups/{}/toggle-list", backup.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify backup was toggled
        let toggled_backup = crate::db::get_backup(&state.pool, backup.pkid).unwrap();
        assert_eq!(toggled_backup.enabled, false);

        cleanup_test_db(&state.pool);
    }
}

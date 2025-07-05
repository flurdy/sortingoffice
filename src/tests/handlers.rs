#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
        Router,
    };
    
    use tower::ServiceExt;
    
    use crate::handlers;
    use crate::AppState;
    use crate::tests::common::{setup_test_db, cleanup_test_db};

    async fn create_test_app() -> (Router, AppState) {
        let pool = setup_test_db();
        let state = AppState { pool };
        let app = Router::new()
            .route("/domains", axum::routing::get(handlers::domains::list))
            .route("/domains", axum::routing::post(handlers::domains::create))
            .route("/domains/:id", axum::routing::get(handlers::domains::show))
            .route("/domains/:id/edit", axum::routing::get(handlers::domains::edit))
            .route("/domains/:id", axum::routing::put(handlers::domains::update))
            .route("/domains/:id", axum::routing::delete(handlers::domains::delete))
            .route("/domains/:id/toggle", axum::routing::post(handlers::domains::toggle_active))
            .route("/users", axum::routing::get(handlers::users::list))
            .route("/users", axum::routing::post(handlers::users::create))
            .route("/users/:id", axum::routing::get(handlers::users::show))
            .route("/users/:id/edit", axum::routing::get(handlers::users::edit))
            .route("/users/:id", axum::routing::put(handlers::users::update))
            .route("/users/:id", axum::routing::delete(handlers::users::delete))
            .route("/users/:id/toggle", axum::routing::post(handlers::users::toggle_active))
            .route("/aliases", axum::routing::get(handlers::aliases::list))
            .route("/aliases", axum::routing::post(handlers::aliases::create))
            .route("/aliases/:id", axum::routing::get(handlers::aliases::show))
            .route("/aliases/:id/edit", axum::routing::get(handlers::aliases::edit))
            .route("/aliases/:id", axum::routing::put(handlers::aliases::update))
            .route("/aliases/:id", axum::routing::delete(handlers::aliases::delete))
            .route("/aliases/:id/toggle-list", axum::routing::post(handlers::aliases::toggle_active))
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
        let new_domain = crate::models::NewDomain {
            domain: "list-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(Request::builder().uri("/domains").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(body_str.contains("list-test.com"));
        assert!(body_str.contains("Test domain"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_create_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        let form_data = "domain=create-test.com&description=Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost&backupmx=on&active=on";
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify domain was created
        let domains = crate::db::get_domains(&state.pool).unwrap();
        assert!(!domains.is_empty());
        assert!(domains.iter().any(|d| d.domain == "create-test.com"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_show_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let new_domain = crate::models::NewDomain {
            domain: "show-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}", domain.id))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(body_str.contains("show-test.com"));
        assert!(body_str.contains("Test domain"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_edit_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let new_domain = crate::models::NewDomain {
            domain: "edit-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}/edit", domain.id))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(body_str.contains("edit-test.com"));
        assert!(body_str.contains("Test domain"));
        assert!(body_str.contains("form")); // Should contain edit form

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_update_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let new_domain = crate::models::NewDomain {
            domain: "update-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = "domain=updated-update.com&description=Updated+Domain&aliases=20&maxquota=2000000&quota=1000000&transport=smtp%3Aupdated&backupmx=on&active=off";
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/domains/{}", domain.id))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify domain was updated
        let updated_domain = crate::db::get_domain(&state.pool, domain.id).unwrap();
        assert_eq!(updated_domain.domain, "updated-update.com");
        assert_eq!(updated_domain.aliases, 20);
        assert_eq!(updated_domain.active, false);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_domains_toggle_active_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain
        let new_domain = crate::models::NewDomain {
            domain: "toggle-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", domain.id))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify domain was toggled
        let toggled_domain = crate::db::get_domain(&state.pool, domain.id).unwrap();
        assert_eq!(toggled_domain.active, false);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_list_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain and user
        let new_domain = crate::models::NewDomain {
            domain: "users-list-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let user_form = crate::models::UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            active: true,
        };
        let _user = crate::db::create_user(&state.pool, user_form).unwrap();

        let response = app
            .oneshot(Request::builder().uri("/users").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(body_str.contains("testuser"));
        assert!(body_str.contains("Test User"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_users_create_handler() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Create test domain first
        let new_domain = crate::models::NewDomain {
            domain: "users-create-test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = "username=testuser&password=password123&name=Test+User&domain=users-create-test.com&quota=100000&active=on";
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify user was created
        let users = crate::db::get_users(&state.pool).unwrap();
        assert!(!users.is_empty());
        assert!(users.iter().any(|u| u.username == "testuser"));

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
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let alias_form = crate::models::AliasForm {
            mail: "test@aliases-list-test.com".to_string(),
            destination: "user@aliases-list-test.com".to_string(),
            domain: domain.domain.clone(),
            active: true,
        };
        let _alias = crate::db::create_alias(&state.pool, alias_form).unwrap();

        let response = app
            .oneshot(Request::builder().uri("/aliases").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
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
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let _domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let form_data = "mail=test%40aliases-create-test.com&destination=user%40aliases-create-test.com&domain=aliases-create-test.com&active=on";
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/aliases")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify alias was created
        let aliases = crate::db::get_aliases(&state.pool).unwrap();
        assert!(!aliases.is_empty());
        assert!(aliases.iter().any(|a| a.mail == "test@aliases-create-test.com"));

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
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };
        let domain = crate::db::create_domain(&state.pool, new_domain).unwrap();

        let user_form = crate::models::UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            active: true,
        };
        let _user = crate::db::create_user(&state.pool, user_form).unwrap();

        let alias_form = crate::models::AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            active: true,
        };
        let _alias = crate::db::create_alias(&state.pool, alias_form).unwrap();

        let response = app
            .oneshot(Request::builder().uri("/stats").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Should contain statistics information
        assert!(body_str.contains("Statistics") || body_str.contains("stats"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_dashboard_handler() {
        let (app, state) = create_test_app().await;

        let response = app
            .oneshot(Request::builder().uri("/dashboard").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Should contain dashboard content
        assert!(body_str.contains("Dashboard") || body_str.contains("dashboard"));

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_not_found_handler() {
        let (app, state) = create_test_app().await;

        let response = app
            .oneshot(Request::builder().uri("/nonexistent").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        cleanup_test_db(&state.pool);
    }
} 

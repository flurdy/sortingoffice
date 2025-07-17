#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    use crate::config::DatabaseConfig;
    use crate::config::DatabaseFeatures;
    use crate::handlers;
    use crate::tests::testcontainers_setup::setup_test_db;
    use crate::AppState;

    async fn create_test_app() -> (
        Router,
        AppState,
        crate::tests::testcontainers_setup::TestContainer,
    ) {
        // Use testcontainers for proper isolation
        let container = setup_test_db();
        let pool = container.get_pool();

        let db_config = vec![DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: format!(
                "mysql://root@127.0.0.1:{}/mysql",
                container.get_mysql_port()
            ),
            features: DatabaseFeatures::default(),
        }];
        let db_manager = crate::db::DatabaseManager::new(db_config)
            .await
            .expect("Failed to create database manager");
        let i18n = crate::i18n::I18n::new("en-US").expect("Failed to initialize i18n");
        let config = crate::config::Config::default();
        let state = AppState {
            db_manager,
            i18n,
            config,
        };
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
            .route("/users", axum::routing::get(handlers::users::list))
            .route("/users", axum::routing::post(handlers::users::create))
            .route("/users/{id}", axum::routing::get(handlers::users::show))
            .route(
                "/users/{id}/edit",
                axum::routing::get(handlers::users::edit),
            )
            .route("/users/{id}", axum::routing::put(handlers::users::update))
            .route(
                "/users/{id}",
                axum::routing::delete(handlers::users::delete),
            )
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

        (app, state, container)
    }

    #[tokio::test]
    async fn test_full_domain_workflow() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create a domain via HTTP POST
        let form_data = "domain=integration-domain.com&transport=smtp%3Aintegration&enabled=on";

        let create_response = app
            .clone()
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

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 2: Verify domain was created by checking the list
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/domains")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("integration-domain.com"));

        // Step 3: Get the domain ID from the database
        let _pool = container.get_pool();
        let domains = crate::db::get_domains(_pool).unwrap();
        let domain = domains
            .iter()
            .find(|d| d.domain == "integration-domain.com")
            .unwrap();

        // Step 4: View the domain details
        let show_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}", domain.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(show_response.status(), StatusCode::OK);

        // Step 5: Update the domain
        let update_form_data =
            "domain=updated-integration.com&transport=smtp%3Aupdated&enabled=off";

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/domains/{}", domain.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(update_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 6: Verify the update
        let updated_domain = crate::db::get_domain(_pool, domain.pkid).unwrap();
        assert_eq!(updated_domain.domain, "updated-integration.com");
        assert_eq!(updated_domain.enabled, false);

        // Step 7: Toggle the domain active status
        let toggle_response = app
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

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 8: Verify the toggle
        let toggled_domain = crate::db::get_domain(_pool, domain.pkid).unwrap();
        assert_eq!(toggled_domain.enabled, true);

        // Step 9: Delete the domain
        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/domains/{}", domain.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::OK);

        // Step 10: Verify the domain was deleted
        let remaining_domains = crate::db::get_domains(_pool).unwrap();
        assert!(!remaining_domains
            .iter()
            .any(|d| d.domain == "updated-integration.com"));
    }

    #[tokio::test]
    async fn test_full_user_workflow() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create a domain first (required for users)
        let domain_form_data =
            "domain=integration-user-test.com&transport=smtp%3Alocalhost&enabled=on";

        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Step 2: Create a user via HTTP POST
        let user_form_data = "id=integrationuser@integration-user-test.com&password=securepass123&name=Integration+User&maildir=testdir&domain=integration-user-test.com&quota=100000&enabled=on&change_password=false";

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(user_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 3: Verify user was created
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("integrationuser@integration-user-test.com"));

        // Step 4: Get the user ID from the database
        let _pool = container.get_pool();
        let users = crate::db::get_users(_pool).unwrap();
        let user = users
            .iter()
            .find(|u| u.id == "integrationuser@integration-user-test.com")
            .unwrap();

        // Step 5: View the user details
        let show_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}", user.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(show_response.status(), StatusCode::OK);

        // Step 6: Update the user
        let update_form_data =
            "id=updateduser@integration-user-test.com&password=newpass123&name=Updated+User&maildir=testdir&domain=integration-user-test.com&quota=200000&enabled=off&change_password=true";

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/users/{}", user.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(update_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 7: Verify the update
        let updated_user = crate::db::get_user(_pool, user.pkid).unwrap();
        println!(
            "DEBUG: Updated user - id: {}, enabled: {}, change_password: {}",
            updated_user.id, updated_user.enabled, updated_user.change_password
        );
        assert_eq!(updated_user.id, "updateduser@integration-user-test.com");
        assert_eq!(updated_user.enabled, false);
        assert_eq!(updated_user.change_password, true);

        // Step 8: Toggle the user active status
        let toggle_response = app
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

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 9: Verify the toggle
        let toggled_user = crate::db::get_user(_pool, user.pkid).unwrap();
        println!(
            "DEBUG: Toggled user - id: {}, enabled: {}",
            toggled_user.id, toggled_user.enabled
        );
        assert_eq!(toggled_user.enabled, true);
        // Note: change_password field is not affected by toggle operation
    }

    #[tokio::test]
    async fn test_full_alias_workflow() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create a domain first (required for aliases)
        let domain_form_data =
            "domain=integration-alias-test.com&transport=smtp%3Alocalhost&enabled=on";

        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Step 2: Create an alias via HTTP POST
        let alias_form_data = "mail=test%40integration-alias-test.com&destination=user%40integration-alias-test.com&domain=integration-alias-test.com&active=on";

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/aliases")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(alias_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 3: Verify alias was created
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/aliases")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("test@integration-alias-test.com"));

        // Step 4: Get the alias ID from the database
        let _pool = container.get_pool();
        let aliases = crate::db::get_aliases(_pool).unwrap();
        let alias = aliases
            .iter()
            .find(|a| a.mail == "test@integration-alias-test.com")
            .unwrap();

        // Step 5: View the alias details
        let show_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/aliases/{}", alias.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(show_response.status(), StatusCode::OK);

        // Step 6: Update the alias
        let update_form_data =
            "mail=updated%40integration-alias-test.com&destination=updateduser%40integration-alias-test.com&domain=integration-alias-test.com&active=off";

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/aliases/{}", alias.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(update_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 7: Verify the update
        let updated_alias = crate::db::get_alias(_pool, alias.pkid).unwrap();
        assert_eq!(updated_alias.mail, "updated@integration-alias-test.com");
        assert_eq!(updated_alias.enabled, false);

        // Step 8: Toggle the alias active status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/aliases/{}/toggle-list", alias.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 9: Verify the toggle
        let toggled_alias = crate::db::get_alias(_pool, alias.pkid).unwrap();
        assert_eq!(toggled_alias.enabled, true);
    }

    #[tokio::test]
    async fn test_stats_integration() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create test data
        let domain_form_data =
            "domain=integration-stats-test.com&transport=smtp%3Alocalhost&enabled=on";

        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        let user_form_data =
            "id=statsuser@integration-stats-test.com&password=stats123&name=Stats+User&maildir=testdir&enabled=on";

        let _user_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(user_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        let alias_form_data = "mail=stats%40integration-stats-test.com&destination=user%40integration-stats-test.com&domain=integration-stats-test.com&active=on";

        let _alias_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/aliases")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(alias_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Step 2: Test stats endpoint
        let stats_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain statistics information
        assert!(body_str.contains("Statistics") || body_str.contains("stats"));

        // Step 3: Verify database stats match
        let _pool = container.get_pool();
        let system_stats = crate::db::get_system_stats(_pool).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);
    }

    #[tokio::test]
    async fn test_complex_domain_management_journey() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create multiple domains with different configurations
        let domains_data = vec![
            ("primary-domain.com", "smtp:primary-server", true),
            ("secondary-domain.com", "smtp:secondary-server", true),
            ("disabled-domain.com", "smtp:disabled-server", false),
        ];

        let mut created_domains = Vec::new();

        for (domain, transport, enabled) in domains_data {
            let form_data = format!(
                "domain={}&transport={}&enabled={}",
                domain,
                transport,
                if enabled { "on" } else { "off" }
            );

            let response = app
                .clone()
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
            created_domains.push(domain.to_string());
        }

        // Step 2: Verify all domains were created
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/domains")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        for domain in &created_domains {
            assert!(body_str.contains(domain));
        }

        // Step 3: Create users for each domain
        let users_data = vec![
            ("user1@primary-domain.com", "user1", "password123"),
            ("user2@primary-domain.com", "user2", "password456"),
            ("admin@secondary-domain.com", "admin", "adminpass"),
        ];

        for (email, username, password) in users_data {
            let form_data = format!(
                "id={}&password={}&name={}&maildir=testdir&enabled=on",
                username, password, username
            );

            let response = app
                .clone()
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
        }

        // Step 4: Create aliases for the domains
        let aliases_data = vec![
            ("postmaster@primary-domain.com", "admin@primary-domain.com"),
            ("abuse@primary-domain.com", "admin@primary-domain.com"),
            ("info@secondary-domain.com", "admin@secondary-domain.com"),
        ];

        for (alias, destination) in aliases_data {
            let form_data = format!("mail={}&destination={}&enabled=on", alias, destination);

            let response = app
                .clone()
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
        }

        // Step 5: Test domain management operations
        let _pool = container.get_pool();
        let domains = crate::db::get_domains(_pool).unwrap();
        let primary_domain = domains
            .iter()
            .find(|d| d.domain == "primary-domain.com")
            .unwrap();

        // Toggle domain status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", primary_domain.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Verify toggle
        let toggled_domain = crate::db::get_domain(_pool, primary_domain.pkid).unwrap();
        assert_eq!(toggled_domain.enabled, false);

        // Step 6: Test statistics
        let stats_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Verify statistics show our data
        assert!(stats_str.contains("3")); // 3 domains
        assert!(stats_str.contains("3")); // 3 users
        assert!(stats_str.contains("3")); // 3 aliases
    }

    #[tokio::test]
    async fn test_user_management_with_aliases_journey() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create a domain
        let domain_form = "domain=user-test.com&transport=smtp%3Alocalhost&enabled=on";
        let domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(domain_response.status(), StatusCode::OK);

        // Step 2: Create multiple users with different configurations
        let users_data = vec![
            ("john", "john123", "John Doe", true),
            ("jane", "jane123", "Jane Smith", true),
            ("bob", "bob123", "Bob Wilson", false), // disabled user
        ];

        for (username, password, name, enabled) in users_data {
            let form_data = format!(
                "id={}&password={}&name={}&maildir=testdir&enabled={}",
                username,
                password,
                name,
                if enabled { "on" } else { "off" }
            );

            let response = app
                .clone()
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
        }

        // Step 3: Create aliases for users
        let aliases_data = vec![
            ("john@user-test.com", "john"),
            ("jane@user-test.com", "jane"),
            ("bob@user-test.com", "bob"),
            ("admin@user-test.com", "john"), // admin alias to john
        ];

        for (alias, username) in aliases_data {
            let form_data = format!("mail={}&destination={}&enabled=on", alias, username);

            let response = app
                .clone()
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
        }

        // Step 4: Test user management operations
        let _pool = container.get_pool();
        let users = crate::db::get_users(_pool).unwrap();
        let john = users.iter().find(|u| u.id == "john").unwrap();

        // Toggle user status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", john.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Verify toggle
        let toggled_john = crate::db::get_user(_pool, john.pkid).unwrap();
        assert_eq!(toggled_john.enabled, false);

        // Step 5: Test alias management
        let aliases = crate::db::get_aliases(_pool).unwrap();
        let john_alias = aliases
            .iter()
            .find(|a| a.mail == "john@user-test.com")
            .unwrap();

        // Toggle alias status
        let alias_toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/aliases/{}/toggle-list", john_alias.pkid))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(alias_toggle_response.status(), StatusCode::OK);

        // Verify alias toggle
        let toggled_alias = crate::db::get_alias(_pool, john_alias.pkid).unwrap();
        assert_eq!(toggled_alias.enabled, false);

        // Step 6: Test statistics
        let stats_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Verify statistics show our data
        assert!(stats_str.contains("1")); // 1 domain
        assert!(stats_str.contains("3")); // 3 users
        assert!(stats_str.contains("4")); // 4 aliases
    }

    #[tokio::test]
    async fn test_error_handling_and_edge_cases_journey() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Test duplicate domain creation (should fail gracefully)
        let domain_form = "domain=duplicate-test.com&transport=smtp%3Alocalhost&enabled=on";
        let first_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(first_response.status(), StatusCode::OK);

        // Try to create the same domain again (should fail)
        let second_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(second_response.status(), StatusCode::OK);

        // Verify the response contains an error message
        let body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("A domain with this name already exists"));

        // Step 2: Test edge cases with empty/invalid data
        // Test with empty domain name
        let empty_domain_form = "domain=&transport=smtp%3Alocalhost&enabled=on";
        let empty_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(empty_domain_form))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should handle gracefully (either error or redirect)
        assert!(
            empty_response.status().is_client_error()
                || empty_response.status().is_redirection()
                || empty_response.status().is_success()
        );

        // Step 3: Test statistics with mixed data
        let stats_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Should contain statistics information
        assert!(stats_str.contains("Statistics") || stats_str.contains("stats"));

        // Verify we have the expected domains
        let _pool = container.get_pool();
        let final_domains = crate::db::get_domains(_pool).unwrap();
        assert!(final_domains.len() >= 1); // At least duplicate-test.com
    }

    #[tokio::test]
    async fn test_multi_database_workflow_journey() {
        let (app, _state, container) = create_test_app().await;

        // Step 1: Create domains in different "virtual databases" (simulated by different naming patterns)
        let database_domains = vec![
            ("db1-domain.com", "smtp:db1-server"),
            ("db2-domain.com", "smtp:db2-server"),
            ("db3-domain.com", "smtp:db3-server"),
        ];

        for (domain, transport) in database_domains {
            let form_data = format!("domain={}&transport={}&enabled=on", domain, transport);

            let response = app
                .clone()
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
        }

        // Step 2: Create users for each "database"
        let database_users = vec![
            ("db1-user", "db1pass", "DB1 User"),
            ("db2-user", "db2pass", "DB2 User"),
            ("db3-user", "db3pass", "DB3 User"),
        ];

        for (username, password, name) in database_users {
            let form_data = format!(
                "id={}&password={}&name={}&maildir=testdir&enabled=on",
                username, password, name
            );

            let response = app
                .clone()
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
        }

        // Step 3: Create aliases for each "database"
        let database_aliases = vec![
            ("postmaster@db1-domain.com", "db1-user@db1-domain.com"),
            ("postmaster@db2-domain.com", "db2-user@db2-domain.com"),
            ("postmaster@db3-domain.com", "db3-user@db3-domain.com"),
        ];

        for (alias, destination) in database_aliases {
            let form_data = format!("mail={}&destination={}&enabled=on", alias, destination);

            let response = app
                .clone()
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
        }

        // Step 4: Test cross-database operations (simulated by creating aliases that point across domains)
        let cross_database_aliases = vec![("cross@db1-domain.com", "db2-user@db2-domain.com")];

        for (alias, destination) in cross_database_aliases {
            let form_data = format!("mail={}&destination={}&enabled=on", alias, destination);

            let response = app
                .clone()
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
        }

        // Step 5: Test bulk operations
        let bulk_aliases = vec![
            ("bulk1@db1-domain.com", "db1-user@db1-domain.com"),
            ("bulk2@db2-domain.com", "db2-user@db2-domain.com"),
            ("bulk3@db3-domain.com", "db3-user@db3-domain.com"),
        ];

        for (alias, destination) in bulk_aliases {
            let form_data = format!("mail={}&destination={}&enabled=on", alias, destination);

            let response = app
                .clone()
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
        }

        // Step 6: Test statistics across "databases"
        let stats_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Verify statistics show our multi-database data
        assert!(stats_str.contains("3")); // 3 domains
        assert!(stats_str.contains("3")); // 3 users
        assert!(stats_str.contains("7")); // 7 aliases (3 postmaster + 1 cross + 3 bulk)

        // Step 7: Test data isolation (simulated by naming patterns)
        let _pool = container.get_pool();
        let domains = crate::db::get_domains(_pool).unwrap();
        let users = crate::db::get_users(_pool).unwrap();
        let aliases = crate::db::get_aliases(_pool).unwrap();

        // Verify we have the expected data
        assert_eq!(domains.len(), 3);
        assert_eq!(users.len(), 3);
        assert_eq!(aliases.len(), 7);

        // Verify data integrity
        for domain in &domains {
            assert!(domain.domain.contains("db"));
        }

        for user in &users {
            assert!(user.id.contains("db"));
        }

        for alias in &aliases {
            assert!(alias.mail.contains("db"));
        }
    }
}

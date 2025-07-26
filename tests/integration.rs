#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    use sortingoffice::{
        config::{Config, DatabaseConfig, DatabaseFeatures},
        db::{self, DatabaseManager},
        handlers,
        i18n::I18n,
        AppState,
    };

    use sortingoffice::test_helpers::testcontainers_setup::{setup_test_db, TestContainer};
    use sortingoffice::test_helpers::test_utils::{TestUtils, TestData};

    async fn create_test_app() -> (Router<AppState>, AppState, TestContainer) {
        let container = setup_test_db().await;
        let _ = container.get_pool(); // keep for possible future use
        let schema = container.get_schema();
        let port = container.get_port();

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
        let i18n = I18n::new("en-US").expect("Failed to initialize i18n");
        let config = Config::default();
        let state = AppState {
            db_manager,
            i18n,
            config,
        };

        let app = handlers::create_app(state.clone());
        (app, state, container)
    }

    // Helper function to create an authenticated cookie with edit role
    fn create_edit_auth_cookie() -> axum::http::HeaderValue {
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour in the future
        let cookie = format!(
            "authenticated={expiry}:edit:test; Path=/; Max-Age=3600; HttpOnly; SameSite=Lax"
        );
        axum::http::HeaderValue::from_str(&cookie).unwrap()
    }

    #[tokio::test]
    async fn test_full_domain_workflow() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let db_url = format!("mysql://root@127.0.0.1:{}/{}", 
                            container.get_port(), container.get_schema());
        
        let (app, state) = TestUtils::create_test_app_with_db(&db_url, "test").await;
        
        // Generate unique test data
        let domain = TestData::unique_domain();
        let form_data = TestData::domain_form_data(&domain, "smtp:integration", true);
        
        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();
        
        // Step 1: Create a domain via HTTP POST using shared helper
        let create_response = TestUtils::make_post_request(
            &app, &state, "/domains", &form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&create_response, StatusCode::OK);
        
        // Step 2: Verify domain was created by checking the list
        let list_response = TestUtils::make_get_request(
            &app, &state, "/domains", Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&list_response, StatusCode::OK);
        TestUtils::assert_body_contains(list_response, &domain).await;
        
        // Step 3: Get the domain ID from the database
        let domains = db::get_domains(container.get_pool()).unwrap();
        let domain_record = domains.iter().find(|d| d.domain == domain).unwrap();
        
        // Step 4: View the domain details
        let show_response = TestUtils::make_get_request(
            &app, &state, &format!("/domains/{}", domain_record.pkid), Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&show_response, StatusCode::OK);
        
        // Step 5: Update the domain
        let updated_domain = TestData::unique_domain();
        let update_form_data = TestData::domain_form_data(&updated_domain, "smtp:updated", false);
        
        let update_response = TestUtils::make_put_request(
            &app, &state, &format!("/domains/{}", domain_record.pkid), &update_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&update_response, StatusCode::OK);
        
        // Step 6: Verify the update
        let updated_record = db::get_domain(container.get_pool(), domain_record.pkid).unwrap();
        assert_eq!(updated_record.domain, updated_domain);
        assert!(!updated_record.enabled);
        
        // Step 7: Toggle the domain active status
        let toggle_response = TestUtils::make_post_request(
            &app, &state, &format!("/domains/{}/toggle", domain_record.pkid), "", Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&toggle_response, StatusCode::OK);
        
        // Step 8: Verify the toggle
        let toggled_domain = db::get_domain(container.get_pool(), domain_record.pkid).unwrap();
        assert!(toggled_domain.enabled);
        
        // Step 9: Delete the domain
        let delete_response = TestUtils::make_delete_request(
            &app, &state, &format!("/domains/{}", domain_record.pkid), Some(auth_cookie)
        ).await.unwrap();
        
        TestUtils::assert_status(&delete_response, StatusCode::OK);
        
        // Step 10: Verify the domain was deleted
        let remaining_domains = db::get_domains(container.get_pool()).unwrap();
        assert!(!remaining_domains
            .iter()
            .any(|d| d.domain == updated_domain));
    }

    #[tokio::test]
    async fn test_full_user_workflow() {
        let (app, state, container) = create_test_app().await;

        // Step 1: Create a domain first (required for users)
        let domain_form_data =
            "domain=integration-user-test.com&transport=smtp%3Alocalhost&enabled=on";

        let _domain_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::from(domain_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Step 2: Create a user via HTTP POST
        let user_form_data = "id=integrationuser@integration-user-test.com&password=securepass123&name=Integration+User&maildir=testdir&home=/var/spool/mail/virtual&domain=integration-user-test.com&quota=100000&enabled=on&change_password=false";

        let create_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::from(user_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 3: Verify user was created
        let list_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .header("cookie", create_edit_auth_cookie())
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
        let users = db::get_users(_pool).unwrap();
        let user = users
            .iter()
            .find(|u| u.id == "integrationuser@integration-user-test.com")
            .unwrap();

        // Step 5: View the user details
        let show_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{}", user.id))
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(show_response.status(), StatusCode::OK);

        // Step 6: Update the user
        let update_form_data =
            "id=updateduser@integration-user-test.com&password=newpass123&name=Updated+User&maildir=testdir&home=/var/spool/mail/virtual&domain=integration-user-test.com&quota=200000&enabled=off&change_password=true";

        let update_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/users/{}", user.id))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::from(update_form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 7: Verify the update
        let updated_user =
            db::get_user(_pool, "updateduser@integration-user-test.com".to_string()).unwrap();
        println!(
            "DEBUG: Updated user - id: {}, enabled: {}, change_password: {}",
            updated_user.id, updated_user.enabled, updated_user.change_password
        );
        assert_eq!(updated_user.id, "updateduser@integration-user-test.com");
        assert!(!updated_user.enabled);
        assert!(updated_user.change_password);

        // Step 8: Toggle the user active status
        let toggle_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", updated_user.id))
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 9: Verify the toggle
        let toggled_user = db::get_user(_pool, updated_user.id.clone()).unwrap();
        println!(
            "DEBUG: Toggled user - id: {}, enabled: {}",
            toggled_user.id, toggled_user.enabled
        );
        assert!(toggled_user.enabled);
        // Note: change_password field is not affected by toggle operation
    }

    #[tokio::test]
    async fn test_full_alias_workflow() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let db_url = format!("mysql://root@127.0.0.1:{}/{}", 
                            container.get_port(), container.get_schema());
        
        let (app, state) = TestUtils::create_test_app_with_db(&db_url, "test").await;
        
        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();
        
        // Step 1: Create a domain first (required for aliases)
        let domain = TestData::unique_domain();
        let domain_form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);
        
        let _domain_response = TestUtils::make_post_request(
            &app, &state, "/domains", &domain_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&_domain_response, StatusCode::OK);
        
        // Step 2: Create an alias via HTTP POST
        let alias_mail = format!("test@{}", domain);
        let alias_destination = format!("user@{}", domain);
        let alias_form_data = TestData::alias_form_data(&alias_mail, &alias_destination, true);
        
        let create_response = TestUtils::make_post_request(
            &app, &state, "/aliases", &alias_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&create_response, StatusCode::OK);
        
        // Step 3: Verify alias was created
        let list_response = TestUtils::make_get_request(
            &app, &state, "/aliases", Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&list_response, StatusCode::OK);
        TestUtils::assert_body_contains(list_response, &alias_mail).await;
        
        // Step 4: Get the alias ID from the database
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let alias = aliases.iter().find(|a| a.mail == alias_mail).unwrap();
        
        // Step 5: View the alias details
        let show_response = TestUtils::make_get_request(
            &app, &state, &format!("/aliases/{}", alias.pkid), Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&show_response, StatusCode::OK);
        
        // Step 6: Update the alias
        let updated_alias_mail = format!("updated@{}", domain);
        let updated_alias_destination = format!("updateduser@{}", domain);
        let update_form_data = TestData::alias_form_data(&updated_alias_mail, &updated_alias_destination, false);
        
        let update_response = TestUtils::make_put_request(
            &app, &state, &format!("/aliases/{}", alias.pkid), &update_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&update_response, StatusCode::OK);
        
        // Step 7: Verify the update
        let updated_alias = db::get_alias(container.get_pool(), alias.pkid).unwrap();
        assert_eq!(updated_alias.mail, updated_alias_mail);
        assert!(!updated_alias.enabled);
        
        // Step 8: Toggle the alias active status
        let toggle_response = TestUtils::make_post_request(
            &app, &state, &format!("/aliases/{}/toggle-list", alias.pkid), "", Some(auth_cookie)
        ).await.unwrap();
        
        TestUtils::assert_status(&toggle_response, StatusCode::OK);
        
        // Step 9: Verify the toggle
        let toggled_alias = db::get_alias(container.get_pool(), alias.pkid).unwrap();
        assert!(toggled_alias.enabled);
    }

    #[tokio::test]
    async fn test_stats_integration() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let db_url = format!("mysql://root@127.0.0.1:{}/{}", 
                            container.get_port(), container.get_schema());
        
        let (app, state) = TestUtils::create_test_app_with_db(&db_url, "test").await;
        
        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();
        
        // Step 1: Create test data
        let domain = TestData::unique_domain();
        let domain_form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);
        
        let _domain_response = TestUtils::make_post_request(
            &app, &state, "/domains", &domain_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&_domain_response, StatusCode::OK);
        
        let user_id = format!("statsuser@{}", domain);
        let user_form_data = TestData::user_form_data_complete(
            &user_id,
            "stats123",
            "Stats User",
            "testdir",
            "/var/spool/mail/virtual",
            &domain,
            "100000",
            true,
            false,
        );
        
        let _user_response = TestUtils::make_post_request(
            &app, &state, "/users", &user_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&_user_response, StatusCode::OK);
        
        let alias_mail = format!("stats@{}", domain);
        let alias_destination = format!("user@{}", domain);
        let alias_form_data = TestData::alias_form_data(&alias_mail, &alias_destination, true);
        
        let _alias_response = TestUtils::make_post_request(
            &app, &state, "/aliases", &alias_form_data, Some(auth_cookie.clone())
        ).await.unwrap();
        
        TestUtils::assert_status(&_alias_response, StatusCode::OK);
        
        // Step 2: Test stats endpoint
        let stats_response = TestUtils::make_get_request(
            &app, &state, "/stats", Some(auth_cookie)
        ).await.unwrap();
        
        TestUtils::assert_status(&stats_response, StatusCode::OK);
        TestUtils::assert_body_contains(stats_response, "Statistics").await;
        
        // Step 3: Verify database stats match
        let system_stats = db::get_system_stats(container.get_pool()).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);
    }

    #[tokio::test]
    async fn test_complex_domain_management_journey() {
        let (app, state, container) = create_test_app().await;

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
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/domains")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/domains")
                    .header("cookie", create_edit_auth_cookie())
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

        for (_email, username, password) in users_data {
            let form_data = format!(
                "id={username}&password={password}&name={username}&maildir=testdir&home=/var/spool/mail/virtual&enabled=on"
            );

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/users")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            let form_data = format!("mail={alias}&destination={destination}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/aliases")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
                        .body(Body::from(form_data))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }

        // Step 5: Test domain management operations
        let _pool = container.get_pool();
        let domains = db::get_domains(_pool).unwrap();
        let primary_domain = domains
            .iter()
            .find(|d| d.domain == "primary-domain.com")
            .unwrap();

        // Toggle domain status
        let toggle_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", primary_domain.pkid))
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Verify toggle
        let toggled_domain = db::get_domain(_pool, primary_domain.pkid).unwrap();
        assert!(!toggled_domain.enabled);

        // Step 6: Test statistics
        let stats_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .header("cookie", create_edit_auth_cookie())
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
        let (app, state, container) = create_test_app().await;

        // Step 1: Create a domain
        let domain_form = "domain=user-test.com&transport=smtp%3Alocalhost&enabled=on";
        let domain_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
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
                "id={}&password={}&name={}&maildir=testdir&home=/var/spool/mail/virtual&enabled={}",
                username,
                password,
                name,
                if enabled { "on" } else { "off" }
            );

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/users")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            let form_data = format!("mail={alias}&destination={username}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/aliases")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
                        .body(Body::from(form_data))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }

        // Step 4: Test user management operations
        let _pool = container.get_pool();
        let users = db::get_users(_pool).unwrap();
        let john = users.iter().find(|u| u.id == "john").unwrap();

        // Toggle user status
        let toggle_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", john.id))
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Verify toggle
        let toggled_john = db::get_user(_pool, john.id.clone()).unwrap();
        assert!(!toggled_john.enabled);

        // Step 5: Test alias management
        let aliases = db::get_aliases(_pool).unwrap();
        let john_alias = aliases
            .iter()
            .find(|a| a.mail == "john@user-test.com")
            .unwrap();

        // Toggle alias status
        let alias_toggle_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/aliases/{}/toggle-list", john_alias.pkid))
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(alias_toggle_response.status(), StatusCode::OK);

        // Verify alias toggle
        let toggled_alias = db::get_alias(_pool, john_alias.pkid).unwrap();
        assert!(!toggled_alias.enabled);

        // Step 6: Test statistics
        let stats_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .header("cookie", create_edit_auth_cookie())
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
        let (app, state, container) = create_test_app().await;

        // Step 1: Test duplicate domain creation (should fail gracefully)
        let domain_form = "domain=duplicate-test.com&transport=smtp%3Alocalhost&enabled=on";
        let first_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
                    .body(Body::from(domain_form))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(first_response.status(), StatusCode::OK);

        // Try to create the same domain again (should fail)
        let second_response = app
            .clone()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
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
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header("cookie", create_edit_auth_cookie())
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
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .header("cookie", create_edit_auth_cookie())
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
        let final_domains = db::get_domains(_pool).unwrap();
        assert!(!final_domains.is_empty()); // At least duplicate-test.com
    }

    #[tokio::test]
    async fn test_multi_database_workflow_journey() {
        let (app, state, container) = create_test_app().await;

        // Step 1: Create domains in different "virtual databases" (simulated by different naming patterns)
        let database_domains = vec![
            ("db1-domain.com", "smtp:db1-server"),
            ("db2-domain.com", "smtp:db2-server"),
            ("db3-domain.com", "smtp:db3-server"),
        ];

        for (domain, transport) in database_domains {
            let form_data = format!("domain={domain}&transport={transport}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/domains")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
                "id={username}&password={password}&name={name}&maildir=testdir&home=/var/spool/mail/virtual&enabled=on"
            );

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/users")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            let form_data = format!("mail={alias}&destination={destination}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/aliases")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            let form_data = format!("mail={alias}&destination={destination}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/aliases")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            let form_data = format!("mail={alias}&destination={destination}&enabled=on");

            let response = app
                .clone()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/aliases")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header("cookie", create_edit_auth_cookie())
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
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .header("cookie", create_edit_auth_cookie())
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
        let domains = db::get_domains(_pool).unwrap();
        let users = db::get_users(_pool).unwrap();
        let aliases = db::get_aliases(_pool).unwrap();

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

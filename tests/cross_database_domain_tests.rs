mod common;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use sortingoffice::config::{
        AdminCredentials, AdminRole, ConnectionPoolConfig, DatabaseConfig, DatabaseFeatures,
    };
    use sortingoffice::test_helpers::test_utils::{TestData, TestUtils};
    use sortingoffice::test_helpers::testcontainers_setup::{setup_test_db, TestContainer};
    use sortingoffice::AppState;
    use std::collections::HashMap;

    /// Setup multiple test databases for cross-database testing
    async fn setup_multiple_test_dbs() -> (TestContainer, TestContainer, TestContainer) {
        // Create three separate MySQL containers
        let db1 = setup_test_db().await;
        let db2 = setup_test_db().await;
        let db3 = setup_test_db().await;

        (db1, db2, db3)
    }

    /// Helper function to switch database and get updated cookie
    async fn switch_database_and_get_cookie(
        app: &axum::Router<AppState>,
        state: &AppState,
        database_id: &str,
        auth_cookie: axum::http::HeaderValue,
    ) -> axum::http::HeaderValue {
        let switch_response = TestUtils::make_post_request(
            app,
            state,
            "/database/select",
            &format!("database_id={}", database_id),
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();
        TestUtils::assert_status(&switch_response, StatusCode::FOUND);

        // Extract the updated cookie from the switch response
        switch_response
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .map(|s| axum::http::HeaderValue::from_str(s).unwrap())
            .unwrap_or(auth_cookie)
    }

    /// Create database configurations for all three databases
    fn create_database_configs(
        db1: &TestContainer,
        db2: &TestContainer,
        db3: &TestContainer,
    ) -> Vec<DatabaseConfig> {
        vec![
            // Database 1 (primary)
            DatabaseConfig {
                id: "db1".to_string(),
                label: "Test Database 1".to_string(),
                url: db1.get_db_url(),
                features: DatabaseFeatures {
                    read_only: false,
                    no_new_users: false,
                    no_new_domains: false,
                    no_password_updates: false,
                    no_clients: false,
                    no_migrations: false,
                    no_relays: false,
                    no_relocated: false,
                    no_seeding: false,
                    disabled: false,
                },
                field_map: HashMap::new(),
                connection_pool: ConnectionPoolConfig {
                    max_size: 10,
                    min_idle: 1,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 1800,
                    test_on_acquire: false,
                    test_on_return: false,
                },
            },
            // Database 2 (backup1)
            DatabaseConfig {
                id: "db2".to_string(),
                label: "Test Database 2".to_string(),
                url: db2.get_db_url(),
                features: DatabaseFeatures {
                    read_only: false,
                    no_new_users: false,
                    no_new_domains: false,
                    no_password_updates: false,
                    no_clients: false,
                    no_migrations: false,
                    no_relays: false,
                    no_relocated: false,
                    no_seeding: false,
                    disabled: false,
                },
                field_map: HashMap::new(),
                connection_pool: ConnectionPoolConfig {
                    max_size: 10,
                    min_idle: 1,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 1800,
                    test_on_acquire: false,
                    test_on_return: false,
                },
            },
            // Database 3 (backup2 - read-only)
            DatabaseConfig {
                id: "db3".to_string(),
                label: "Test Database 3".to_string(),
                url: db3.get_db_url(),
                features: DatabaseFeatures {
                    read_only: true, // This one is read-only
                    no_new_users: false,
                    no_new_domains: false,
                    no_password_updates: false,
                    no_clients: false,
                    no_migrations: false,
                    no_relays: false,
                    no_relocated: false,
                    no_seeding: false,
                    disabled: false,
                },
                field_map: HashMap::new(),
                connection_pool: ConnectionPoolConfig {
                    max_size: 10,
                    min_idle: 1,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 1800,
                    test_on_acquire: false,
                    test_on_return: false,
                },
            },
        ]
    }

    #[tokio::test]
    async fn test_cross_database_domain_scenarios() {
        // Add a timeout to prevent hanging tests
        let test_future = async {
            // Setup three test databases
            let (db1, db2, db3) = setup_multiple_test_dbs().await;
            let db_configs = create_database_configs(&db1, &db2, &db3);
            // Create a custom config that includes the database features
            let config = sortingoffice::config::Config {
                admins: vec![AdminCredentials {
                    username: "admin".to_string(),
                    password_hash: "admin123".to_string(), // In real usage this would be hashed
                    role: AdminRole::Edit,
                }],
                contact: None,
                databases: db_configs.clone(),
                ..Default::default()
            };
            let (app, state) = TestUtils::create_test_app_with_config(db_configs, config).await;

            // Create authentication cookie for db1 (default)
            let auth_cookie = TestUtils::create_edit_auth_cookie_with_db("db1");

            // Test scenario 1: Create domain1 in db1
            println!("=== Test 1: Create domain1 in db1 ===");
            let domain1 = "test1.example.com";
            let form_data = TestData::domain_form_data(domain1, "virtual", true);

            // Switch to db1
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db1", auth_cookie).await;

            // Create domain1 in db1
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Verify domain1 is listed in db1
            let list_response =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, domain1).await;

            // Test scenario 2: Create domain1 in db2 (same domain name, different database)
            println!("=== Test 2: Create domain1 in db2 ===");

            // Switch to db2
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db2", auth_cookie).await;

            // Create domain1 in db2
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Verify domain1 is listed in db2
            let list_response =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, domain1).await;

            // Test scenario 3: Create domain1 as backup domain in db3
            println!("=== Test 3: Create domain1 as backup domain in db3 ===");

            // Switch to db3
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db3", auth_cookie).await;
            println!("Switched to db3, cookie: {:?}", auth_cookie);

            // Try to create domain1 in db3 (should fail - read-only)
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            println!("Create response status: {:?}", create_response.status());
            let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            println!("Create response body: {}", body_str);
            // Should fail with permission error - check for the error message in the HTML
            assert!(body_str.contains("This operation is not allowed on the current database due to restrictions."), 
                "Expected body to contain 'This operation is not allowed on the current database due to restrictions.', but got: {}", body_str);

            // Test scenario 4: Create domain2 in db2
            println!("=== Test 4: Create domain2 in db2 ===");

            // Switch back to db2
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db2", auth_cookie).await;

            let domain2 = "test2.example.com";
            let form_data2 = TestData::domain_form_data(domain2, "virtual", true);

            // Create domain2 in db2
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data2,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Test scenario 5: Create domain3 in db3 (should fail - read-only)
            println!("=== Test 5: Create domain3 in db3 (should fail) ===");

            // Switch to db3
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db3", auth_cookie).await;

            let domain3 = "test3.example.com";
            let form_data3 = TestData::domain_form_data(domain3, "virtual", true);

            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data3,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            // Should fail with permission error
            TestUtils::assert_body_contains(
                create_response,
                "This operation is not allowed on the current database due to restrictions.",
            )
            .await;

            // Test scenario 6: Create domain4 as backup domain in db1
            println!("=== Test 6: Create domain4 as backup domain in db1 ===");

            // Switch back to db1
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db1", auth_cookie).await;

            let domain4 = "test4.example.com";
            let form_data4 = TestData::domain_form_data(domain4, "virtual", true);

            // Create domain4 in db1
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data4,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Test scenario 7: Create domain4 as domain in db2
            println!("=== Test 7: Create domain4 as domain in db2 ===");

            // Switch to db2
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db2", auth_cookie).await;

            // Create domain4 in db2
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data4,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Test scenario 8: Verify cross-database domain matrix report
            println!("=== Test 8: Verify cross-database domain matrix report ===");

            let report_response = TestUtils::make_get_request(
                &app,
                &state,
                "/reports/cross-database-matrix",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_status(&report_response, StatusCode::OK);

            // Verify all domains are listed in the report
            TestUtils::assert_body_contains(report_response, domain1).await;

            // Make separate requests for each domain check
            let report_response2 = TestUtils::make_get_request(
                &app,
                &state,
                "/reports/cross-database-matrix",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_body_contains(report_response2, domain2).await;

            let report_response3 = TestUtils::make_get_request(
                &app,
                &state,
                "/reports/cross-database-matrix",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();
            TestUtils::assert_body_contains(report_response3, domain4).await;

            // Test scenario 9: Verify domain isolation
            println!("=== Test 9: Verify domain isolation ===");

            // Switch to db1 and verify only db1 domains are listed
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db1", auth_cookie).await;

            let list_response =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, domain1).await;

            // Make separate requests for each domain check
            let list_response2 =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_body_contains(list_response2, domain4).await;

            let list_response3 =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            // domain2 should not be visible in db1
            TestUtils::assert_body_not_contains(list_response3, domain2).await;

            // Switch to db2 and verify only db2 domains are listed
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db2", auth_cookie).await;

            let list_response =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, domain1).await;

            // Make separate requests for each domain check
            let list_response2 =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_body_contains(list_response2, domain2).await;

            let list_response3 =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();
            TestUtils::assert_body_contains(list_response3, domain4).await;

            println!("=== All cross-database domain tests passed! ===");
        };

        // Run the test with a timeout to prevent hanging
        tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            test_future,
        )
        .await
        .expect("Test timed out after 5 minutes");
    }

    #[tokio::test]
    async fn test_domain_creation_error_handling() {
        // Add a timeout to prevent hanging tests
        let test_future = async {
            // Setup three test databases
            let (db1, db2, db3) = setup_multiple_test_dbs().await;
            let db_configs = create_database_configs(&db1, &db2, &db3);

            // Create a custom config with the database configurations
            let config = sortingoffice::config::Config {
                required_aliases: vec!["postmaster".to_string(), "abuse".to_string()],
                common_aliases: vec!["admin".to_string(), "webmaster".to_string()],
                global_features: sortingoffice::config::GlobalFeatures {
                    read_only: false,
                    no_new_users: false,
                    no_new_domains: false,
                    no_password_updates: false,
                },
                databases: db_configs.clone(),
                admins: vec![sortingoffice::config::AdminCredentials {
                    username: "admin".to_string(),
                    password_hash: "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO"
                        .to_string(),
                    role: sortingoffice::config::AdminRole::Edit,
                }],
                admin: None,
                contact: Some(sortingoffice::config::ContactInfo {
                    name: "Test Admin".to_string(),
                    email: None,
                    contact_form: None,
                    role: Some("Test Administrator".to_string()),
                }),
            };

            let (app, state) = TestUtils::create_test_app_with_config(db_configs, config).await;

            // Create authentication cookie for db1 (default)
            let auth_cookie = TestUtils::create_edit_auth_cookie_with_db("db1");

            // Test 1: Try to create domain in read-only database (db3)
            println!("=== Test 1: Try to create domain in read-only database ===");

            // Create authentication cookie directly for db3 (read-only)
            let db3_auth_cookie = TestUtils::create_edit_auth_cookie_with_db("db3");

            let domain = "test-readonly.example.com";
            let form_data = TestData::domain_form_data(domain, "virtual", true);

            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(db3_auth_cookie),
            )
            .await
            .unwrap();

            // Should fail with specific error message
            TestUtils::assert_body_contains(create_response, "This operation is not allowed").await;

            // Test 2: Try to create domain with invalid data
            println!("=== Test 2: Try to create domain with invalid data ===");

            // Switch to db1 (writable)
            let auth_cookie =
                switch_database_and_get_cookie(&app, &state, "db1", auth_cookie).await;

            // Try to create domain with empty name
            let invalid_form_data = "domain=&transport=virtual&enabled=on";

            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                invalid_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            // Should fail with validation error
            TestUtils::assert_body_contains(create_response, "Domain name is required").await;

            println!("=== Domain creation error handling tests passed! ===");
        };

        // Run the test with a timeout to prevent hanging
        tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            test_future,
        )
        .await
        .expect("Test timed out after 5 minutes");
    }
}

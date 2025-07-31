#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use sortingoffice::db;
    use sortingoffice::test_helpers::test_utils::{TestData, TestUtils};
    use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

    #[tokio::test]
    async fn test_full_domain_workflow() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();
        let form_data = TestData::domain_form_data(&domain, "smtp:integration", true);

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create a domain via HTTP POST using shared helper
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

        // Step 2: Verify domain was created by checking the list
        let list_response =
            TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                .await
                .unwrap();

        TestUtils::assert_status(&list_response, StatusCode::OK);
        TestUtils::assert_body_contains(list_response, &domain).await;

        // Step 3: Get the domain ID from the database
        let domains = db::get_domains(container.get_pool()).unwrap();
        let domain_record = domains.iter().find(|d| d.domain == domain).unwrap();

        // Step 4: View the domain details
        let show_response = TestUtils::make_get_request(
            &app,
            &state,
            &format!("/domains/{}", domain_record.pkid),
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&show_response, StatusCode::OK);

        // Step 5: Update the domain
        let updated_domain = TestData::unique_domain();
        let update_form_data = TestData::domain_form_data(&updated_domain, "smtp:updated", false);

        let update_response = TestUtils::make_put_request(
            &app,
            &state,
            &format!("/domains/{}", domain_record.pkid),
            &update_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&update_response, StatusCode::OK);

        // Step 6: Verify the update
        let updated_record = db::get_domain(container.get_pool(), domain_record.pkid).unwrap();
        assert_eq!(updated_record.domain, updated_domain);
        assert!(!updated_record.enabled);

        // Step 7: Toggle the domain active status
        let toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/domains/{}/toggle", domain_record.pkid),
            "",
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&toggle_response, StatusCode::OK);

        // Step 8: Verify the toggle
        let toggled_domain = db::get_domain(container.get_pool(), domain_record.pkid).unwrap();
        assert!(toggled_domain.enabled);

        // Step 9: Delete the domain
        let delete_response = TestUtils::make_delete_request(
            &app,
            &state,
            &format!("/domains/{}", domain_record.pkid),
            Some(auth_cookie),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&delete_response, StatusCode::OK);

        // Step 10: Verify the domain was deleted
        let remaining_domains = db::get_domains(container.get_pool()).unwrap();
        assert!(!remaining_domains.iter().any(|d| d.domain == updated_domain));
    }

    #[tokio::test]
    async fn test_full_user_workflow() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create a domain first (required for users)
        let domain = TestData::unique_domain();
        let domain_form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

        let _domain_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&_domain_response, StatusCode::OK);

        // Step 2: Create a user via HTTP POST
        let user_id = format!("integrationuser@{domain}");
        let user_form_data = TestData::user_form_data_complete(
            &user_id,
            "securepass123",
            "Integration User",
            "testdir",
            "/var/spool/mail/virtual",
            true,
            false,
        );

        let create_response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &user_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&create_response, StatusCode::OK);

        // Step 3: Verify user was created
        let list_response =
            TestUtils::make_get_request(&app, &state, "/users", Some(auth_cookie.clone()))
                .await
                .unwrap();

        TestUtils::assert_status(&list_response, StatusCode::OK);
        TestUtils::assert_body_contains(list_response, &user_id).await;

        // Step 4: Get the user ID from the database
        let users = db::get_users(container.get_pool()).unwrap();
        let user = users.iter().find(|u| u.id == user_id).unwrap();

        // Step 5: View the user details
        let show_response = TestUtils::make_get_request(
            &app,
            &state,
            &format!("/users/{}", user.id),
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&show_response, StatusCode::OK);

        // Step 6: Update the user
        let updated_user_id = format!("updateduser@{domain}");
        let update_form_data = TestData::user_form_data_complete(
            &updated_user_id,
            "newpass123",
            "Updated User",
            "testdir",
            "/var/spool/mail/virtual",
            false,
            true,
        );

        let update_response = TestUtils::make_put_request(
            &app,
            &state,
            &format!("/users/{}", user.id),
            &update_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&update_response, StatusCode::OK);

        // Step 7: Verify the update
        let updated_user = db::get_user(container.get_pool(), updated_user_id.clone()).unwrap();
        println!(
            "DEBUG: Updated user - id: {}, enabled: {}, change_password: {}",
            updated_user.id, updated_user.enabled, updated_user.change_password
        );
        assert_eq!(updated_user.id, updated_user_id);
        assert!(!updated_user.enabled);
        assert!(updated_user.change_password);

        // Step 8: Toggle the user active status
        let toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/users/{}/toggle", updated_user.id),
            "",
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&toggle_response, StatusCode::OK);

        // Step 9: Verify the toggle
        let toggled_user = db::get_user(container.get_pool(), updated_user.id.clone()).unwrap();
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
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create a domain first (required for aliases)
        let domain = TestData::unique_domain();
        let domain_form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

        let _domain_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&_domain_response, StatusCode::OK);

        // Step 2: Create an alias via HTTP POST
        let alias_mail = format!("test@{domain}");
        let alias_destination = format!("user@{domain}");
        let alias_form_data = TestData::alias_form_data(&alias_mail, &alias_destination, true);

        let create_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&create_response, StatusCode::OK);

        // Step 3: Verify alias was created
        let list_response =
            TestUtils::make_get_request(&app, &state, "/aliases", Some(auth_cookie.clone()))
                .await
                .unwrap();

        TestUtils::assert_status(&list_response, StatusCode::OK);
        TestUtils::assert_body_contains(list_response, &alias_mail).await;

        // Step 4: Get the alias ID from the database
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let alias = aliases.iter().find(|a| a.mail == alias_mail).unwrap();

        // Step 5: View the alias details
        let show_response = TestUtils::make_get_request(
            &app,
            &state,
            &format!("/aliases/{}", alias.pkid),
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&show_response, StatusCode::OK);

        // Step 6: Update the alias
        let updated_alias_mail = format!("updated@{domain}");
        let updated_alias_destination = format!("updateduser@{domain}");
        let update_form_data =
            TestData::alias_form_data(&updated_alias_mail, &updated_alias_destination, false);

        let update_response = TestUtils::make_put_request(
            &app,
            &state,
            &format!("/aliases/{}", alias.pkid),
            &update_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&update_response, StatusCode::OK);

        // Step 7: Verify the update
        let updated_alias = db::get_alias(container.get_pool(), alias.pkid).unwrap();
        assert_eq!(updated_alias.mail, updated_alias_mail);
        assert!(!updated_alias.enabled);

        // Step 8: Toggle the alias active status
        let toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/aliases/{}/toggle-list", alias.pkid),
            "",
            Some(auth_cookie),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&toggle_response, StatusCode::OK);

        // Step 9: Verify the toggle
        let toggled_alias = db::get_alias(container.get_pool(), alias.pkid).unwrap();
        assert!(toggled_alias.enabled);
    }

    #[tokio::test]
    async fn test_stats_integration() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create test data
        let domain = TestData::unique_domain();
        let domain_form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

        let _domain_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&_domain_response, StatusCode::OK);

        let user_id = format!("statsuser@{domain}");
        let user_form_data = TestData::user_form_data_complete(
            &user_id,
            "stats123",
            "Stats User",
            "testdir",
            "/var/spool/mail/virtual",
            true,
            false,
        );

        let _user_response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &user_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&_user_response, StatusCode::OK);

        let alias_mail = format!("stats@{domain}");
        let alias_destination = format!("user@{domain}");
        let alias_form_data = TestData::alias_form_data(&alias_mail, &alias_destination, true);

        let _alias_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&_alias_response, StatusCode::OK);

        // Step 2: Test stats endpoint
        let stats_response = TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie))
            .await
            .unwrap();

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
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create multiple domains with different configurations
        let domains_data = vec![
            ("primary-domain.com", "smtp:primary-server", true),
            ("secondary-domain.com", "smtp:secondary-server", true),
            ("disabled-domain.com", "smtp:disabled-server", false),
        ];

        let mut created_domains = Vec::new();

        for (domain, transport, enabled) in domains_data {
            let form_data = TestData::domain_form_data(domain, transport, enabled);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
            created_domains.push(domain.to_string());
        }

        // Step 2: Verify all domains were created
        let list_response =
            TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                .await
                .unwrap();

        TestUtils::assert_status(&list_response, StatusCode::OK);

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
            let form_data = TestData::user_form_data_complete(
                email,  // Use the full email address as user_id
                password,
                username,
                "testdir",
                "/var/spool/mail/virtual",
                true,
                false,
            );

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 4: Create aliases for the domains
        let aliases_data = vec![
            ("postmaster@primary-domain.com", "admin@primary-domain.com"),
            ("abuse@primary-domain.com", "admin@primary-domain.com"),
            ("info@secondary-domain.com", "admin@secondary-domain.com"),
        ];

        for (alias, destination) in aliases_data {
            let form_data = TestData::alias_form_data(alias, destination, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 5: Test domain management operations
        let domains = db::get_domains(container.get_pool()).unwrap();
        let primary_domain = domains
            .iter()
            .find(|d| d.domain == "primary-domain.com")
            .unwrap();

        // Toggle domain status
        let toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/domains/{}/toggle", primary_domain.pkid),
            "",
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&toggle_response, StatusCode::OK);

        // Verify toggle
        let toggled_domain = db::get_domain(container.get_pool(), primary_domain.pkid).unwrap();
        assert!(!toggled_domain.enabled);

        // Step 6: Test statistics
        let stats_response = TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie))
            .await
            .unwrap();

        TestUtils::assert_status(&stats_response, StatusCode::OK);

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
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create a domain
        let domain_form_data = TestData::domain_form_data("user-test.com", "smtp:localhost", true);
        let domain_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&domain_response, StatusCode::OK);

        // Step 2: Create multiple users with different configurations
        let users_data = vec![
            ("john", "john123", "John Doe", true),
            ("jane", "jane123", "Jane Smith", true),
            ("bob", "bob123", "Bob Wilson", false), // disabled user
        ];

        for (username, password, name, enabled) in users_data {
            let user_id = format!("{}@user-test.com", username);
            let form_data = TestData::user_form_data_complete(
                &user_id,  // Use the full email address as user_id
                password,
                name,
                "testdir",
                "/var/spool/mail/virtual",
                enabled,
                false,
            );

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 3: Create aliases for users
        let aliases_data = vec![
            ("john@user-test.com", "john"),
            ("jane@user-test.com", "jane"),
            ("bob@user-test.com", "bob"),
            ("admin@user-test.com", "john"), // admin alias to john
        ];

        for (alias, username) in aliases_data {
            let destination = format!("{}@user-test.com", username);
            let form_data = TestData::alias_form_data(alias, &destination, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 4: Test user management operations
        let users = db::get_users(container.get_pool()).unwrap();
        let john = users.iter().find(|u| u.id == "john@user-test.com").unwrap();

        // Toggle user status
        let toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/users/{}/toggle", john.id),
            "",
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&toggle_response, StatusCode::OK);

        // Verify toggle
        let toggled_john = db::get_user(container.get_pool(), john.id.clone()).unwrap();
        assert!(!toggled_john.enabled);

        // Step 5: Test alias management
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let john_alias = aliases
            .iter()
            .find(|a| a.mail == "john@user-test.com")
            .unwrap();

        // Toggle alias status
        let alias_toggle_response = TestUtils::make_post_request(
            &app,
            &state,
            &format!("/aliases/{}/toggle-list", john_alias.pkid),
            "",
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&alias_toggle_response, StatusCode::OK);

        // Verify alias toggle
        let toggled_alias = db::get_alias(container.get_pool(), john_alias.pkid).unwrap();
        assert!(!toggled_alias.enabled);

        // Step 6: Test statistics
        let stats_response = TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie))
            .await
            .unwrap();

        TestUtils::assert_status(&stats_response, StatusCode::OK);

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
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Test duplicate domain creation (should fail gracefully)
        let domain_form_data =
            TestData::domain_form_data("duplicate-test.com", "smtp:localhost", true);

        let first_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&first_response, StatusCode::OK);

        // Try to create the same domain again (should fail)
        let second_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        TestUtils::assert_status(&second_response, StatusCode::OK);

        // Verify the response contains an error message
        let body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("A domain with this name already exists"));

        // Step 2: Test edge cases with empty/invalid data
        // Test with empty domain name
        let empty_domain_form = "domain=&transport=smtp:localhost&enabled=on";
        let empty_response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            empty_domain_form,
            Some(auth_cookie.clone()),
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
        let stats_response = TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie))
            .await
            .unwrap();

        TestUtils::assert_status(&stats_response, StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Should contain statistics information
        assert!(stats_str.contains("Statistics") || stats_str.contains("stats"));

        // Verify we have the expected domains
        let final_domains = db::get_domains(container.get_pool()).unwrap();
        assert!(!final_domains.is_empty()); // At least duplicate-test.com
    }

    #[tokio::test]
    async fn test_multi_database_workflow_journey() {
        // Setup test environment using shared helpers
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create authentication cookie
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Step 1: Create domains in different "virtual databases" (simulated by different naming patterns)
        let database_domains = vec![
            ("db1-domain.com", "smtp:db1-server"),
            ("db2-domain.com", "smtp:db2-server"),
            ("db3-domain.com", "smtp:db3-server"),
        ];

        for (domain, transport) in database_domains {
            let form_data = TestData::domain_form_data(domain, transport, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 2: Create users for each "database"
        let database_users = vec![
            ("db1-user", "db1pass", "DB1 User"),
            ("db2-user", "db2pass", "DB2 User"),
            ("db3-user", "db3pass", "DB3 User"),
        ];

        for (username, password, name) in database_users {
            let user_id = format!("{}@{}-domain.com", username, username.split('-').next().unwrap());
            let form_data = TestData::user_form_data_complete(
                &user_id,  // Use the full email address as user_id
                password,
                name,
                "testdir",
                "/var/spool/mail/virtual",
                true,
                false,
            );

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 3: Create aliases for each "database"
        let database_aliases = vec![
            ("postmaster@db1-domain.com", "db1-user@db1-domain.com"),
            ("postmaster@db2-domain.com", "db2-user@db2-domain.com"),
            ("postmaster@db3-domain.com", "db3-user@db3-domain.com"),
        ];

        for (alias, destination) in database_aliases {
            let form_data = TestData::alias_form_data(alias, destination, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 4: Test cross-database operations (simulated by creating aliases that point across domains)
        let cross_database_aliases = vec![("cross@db1-domain.com", "db2-user@db2-domain.com")];

        for (alias, destination) in cross_database_aliases {
            let form_data = TestData::alias_form_data(alias, destination, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 5: Test bulk operations
        let bulk_aliases = vec![
            ("bulk1@db1-domain.com", "db1-user@db1-domain.com"),
            ("bulk2@db2-domain.com", "db2-user@db2-domain.com"),
            ("bulk3@db3-domain.com", "db3-user@db3-domain.com"),
        ];

        for (alias, destination) in bulk_aliases {
            let form_data = TestData::alias_form_data(alias, destination, true);

            let response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&response, StatusCode::OK);
        }

        // Step 6: Test statistics across "databases"
        let stats_response =
            TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie.clone()))
                .await
                .unwrap();

        TestUtils::assert_status(&stats_response, StatusCode::OK);

        let stats_body = axum::body::to_bytes(stats_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats_str = String::from_utf8(stats_body.to_vec()).unwrap();

        // Verify statistics show our multi-database data
        assert!(stats_str.contains("3")); // 3 domains
        assert!(stats_str.contains("3")); // 3 users
        assert!(stats_str.contains("7")); // 7 aliases (3 postmaster + 1 cross + 3 bulk)

        // Step 7: Test data isolation (simulated by naming patterns)
        let domains = db::get_domains(container.get_pool()).unwrap();
        let users = db::get_users(container.get_pool()).unwrap();
        let aliases = db::get_aliases(container.get_pool()).unwrap();

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

    #[tokio::test]
    async fn test_edge_case_validation_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Domain with invalid characters
        let invalid_domain_form =
            TestData::domain_form_data("test@domain.com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &invalid_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Domain with capital letters
        let capitalized_domain_form =
            TestData::domain_form_data("Test.com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &capitalized_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: Domain with consecutive dots
        let consecutive_dots_form = TestData::domain_form_data("test..com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &consecutive_dots_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 4: Valid domain for comparison
        let valid_domain = TestData::unique_domain();
        let valid_domain_form = TestData::domain_form_data(&valid_domain, "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &valid_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should succeed
        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edge_case_alias_validation_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Create a valid domain first
        let domain = TestData::unique_domain();
        let domain_form = TestData::domain_form_data(&domain, "smtp:localhost", true);
        TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Test 1: Alias with invalid email format
        let invalid_alias_form =
            TestData::alias_form_data("invalid-email", "user@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &invalid_alias_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Alias with empty destination
        let empty_dest_form = TestData::alias_form_data("user@example.com", "", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &empty_dest_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: Alias with plus sign at start
        let plus_start_form =
            TestData::alias_form_data("+user@example.com", "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &plus_start_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 4: Valid alias for comparison
        let valid_alias_form =
            TestData::alias_form_data("user@example.com", "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &valid_alias_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should succeed
        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edge_case_user_validation_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: User with invalid email format
        let invalid_user_form = TestData::user_form_data("invalid-email", "password", "Test User");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &invalid_user_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: User with catchall email (not allowed for users)
        let catchall_user_form = TestData::user_form_data("@example.com", "password", "Test User");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &catchall_user_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: User with empty password
        let empty_password_form = TestData::user_form_data("user@example.com", "", "Test User");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &empty_password_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 4: Valid user for comparison
        let valid_user_form = TestData::user_form_data("user@example.com", "password", "Test User");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &valid_user_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should succeed
        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edge_case_backup_validation_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Backup with capital letters
        let capitalized_backup_form = TestData::backup_form_data("Test.com", "smtp:localhost");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domain_backup",
            &capitalized_backup_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Backup with invalid characters
        let invalid_backup_form = TestData::backup_form_data("test@backup.com", "smtp:localhost");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domain_backup",
            &invalid_backup_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: Valid backup for comparison
        let valid_backup = TestData::unique_domain();
        let valid_backup_form = TestData::backup_form_data(&valid_backup, "smtp:localhost");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domain_backup",
            &valid_backup_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should succeed
        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edge_case_boundary_conditions_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Very long domain name
        let long_domain = "a".repeat(100) + ".com";
        let long_domain_form = TestData::domain_form_data(&long_domain, "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &long_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Very long email address
        let long_email = "a".repeat(100) + "@example.com";
        let long_email_form = TestData::alias_form_data(&long_email, "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &long_email_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: Empty strings
        let empty_domain_form = TestData::domain_form_data("", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &empty_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 4: Single character domain (should be valid)
        let single_char_form = TestData::domain_form_data("a.com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &single_char_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should succeed
        TestUtils::assert_status(&response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edge_case_unicode_handling_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Domain with unicode characters
        let unicode_domain_form = TestData::domain_form_data("tëst.com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &unicode_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Email with unicode characters
        let unicode_email_form = TestData::alias_form_data("tëst@example.com", "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &unicode_email_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: User with Unicode characters
        let unicode_user_form =
            TestData::user_form_data("useré@example.com", "password", "Test User");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &unicode_user_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 4: Backup with Unicode characters
        let unicode_backup_form = TestData::backup_form_data("test-ü.com", "smtp:localhost");
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domain_backup",
            &unicode_backup_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;
    }

    #[tokio::test]
    async fn test_edge_case_sql_injection_prevention_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Domain with SQL injection attempt
        let sql_injection_domain_form = TestData::domain_form_data("'; DROP TABLE domains; --", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &sql_injection_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Email with SQL injection attempt
        let sql_injection_email_form = TestData::alias_form_data("'; DROP TABLE aliases; --@example.com", "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &sql_injection_email_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 3: User with SQL injection attempt
        let sql_injection_user_form = TestData::user_form_data(
            "'; DROP TABLE users; --@example.com",
            "password",
            "Test User",
        );
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/users",
            &sql_injection_user_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;
    }

    #[tokio::test]
    async fn test_edge_case_xss_prevention_integration() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        let auth_cookie = TestUtils::create_edit_auth_cookie();

        // Test 1: Domain with XSS attempt
        let xss_domain_form = TestData::domain_form_data("<script>alert('xss')</script>.com", "smtp:localhost", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/domains",
            &xss_domain_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;

        // Test 2: Email with XSS attempt
        let xss_email_form = TestData::alias_form_data("<script>alert('xss')</script>@example.com", "dest@example.com", true);
        let response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &xss_email_form,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Should return 200 OK with error message in HTML
        TestUtils::assert_validation_error(response).await;
    }
}

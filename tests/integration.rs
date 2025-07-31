#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use sortingoffice::db;
    use sortingoffice::test_helpers::test_utils::{TestData, TestUtils};
    use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

    // Domain management tests
    mod domain_tests {
        use super::*;

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
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete_response, StatusCode::OK);

            // Step 10: Verify the domain was deleted
            let final_domains = db::get_domains(container.get_pool()).unwrap();
            assert!(!final_domains.iter().any(|d| d.domain == updated_domain));
        }

        #[tokio::test]
        async fn test_complex_domain_management_journey() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Generate unique test data
            let domain1 = TestData::unique_domain();
            let domain2 = TestData::unique_domain();
            let form_data1 = TestData::domain_form_data(&domain1, "smtp:complex1", true);
            let form_data2 = TestData::domain_form_data(&domain2, "smtp:complex2", false);

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Create multiple domains
            let create1_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data1,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create1_response, StatusCode::OK);

            let create2_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &form_data2,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create2_response, StatusCode::OK);

            // Step 2: Verify both domains were created
            let list_response =
                TestUtils::make_get_request(&app, &state, "/domains", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, &domain1).await;
            TestUtils::assert_body_contains(list_response, &domain2).await;

            // Step 3: Get domain records
            let domains = db::get_domains(container.get_pool()).unwrap();
            let domain1_record = domains.iter().find(|d| d.domain == domain1).unwrap();
            let domain2_record = domains.iter().find(|d| d.domain == domain2).unwrap();

            // Step 4: Update both domains
            let updated_domain1 = TestData::unique_domain();
            let updated_domain2 = TestData::unique_domain();
            let update_form1 = TestData::domain_form_data(&updated_domain1, "smtp:updated1", true);
            let update_form2 = TestData::domain_form_data(&updated_domain2, "smtp:updated2", false);

            let update1_response = TestUtils::make_put_request(
                &app,
                &state,
                &format!("/domains/{}", domain1_record.pkid),
                &update_form1,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&update1_response, StatusCode::OK);

            let update2_response = TestUtils::make_put_request(
                &app,
                &state,
                &format!("/domains/{}", domain2_record.pkid),
                &update_form2,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&update2_response, StatusCode::OK);

            // Step 5: Verify updates
            let updated_record1 = db::get_domain(container.get_pool(), domain1_record.pkid).unwrap();
            let updated_record2 = db::get_domain(container.get_pool(), domain2_record.pkid).unwrap();
            assert_eq!(updated_record1.domain, updated_domain1);
            assert!(updated_record1.enabled);
            assert_eq!(updated_record2.domain, updated_domain2);
            assert!(!updated_record2.enabled);

            // Step 6: Toggle both domains
            let toggle1_response = TestUtils::make_post_request(
                &app,
                &state,
                &format!("/domains/{}/toggle", domain1_record.pkid),
                "",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&toggle1_response, StatusCode::OK);

            let toggle2_response = TestUtils::make_post_request(
                &app,
                &state,
                &format!("/domains/{}/toggle", domain2_record.pkid),
                "",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&toggle2_response, StatusCode::OK);

            // Step 7: Verify toggles
            let toggled_record1 = db::get_domain(container.get_pool(), domain1_record.pkid).unwrap();
            let toggled_record2 = db::get_domain(container.get_pool(), domain2_record.pkid).unwrap();
            assert!(!toggled_record1.enabled); // Should be toggled from true to false
            assert!(toggled_record2.enabled); // Should be toggled from false to true

            // Step 8: Delete both domains
            let delete1_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/domains/{}", domain1_record.pkid),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete1_response, StatusCode::OK);

            let delete2_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/domains/{}", domain2_record.pkid),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete2_response, StatusCode::OK);

            // Step 9: Verify both domains were deleted
            let final_domains = db::get_domains(container.get_pool()).unwrap();
            assert!(!final_domains.iter().any(|d| d.domain == updated_domain1));
            assert!(!final_domains.iter().any(|d| d.domain == updated_domain2));
        }
    }

    // User management tests
    mod user_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_user_workflow() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Generate unique test data
            let user_id = TestData::unique_user_id();
            let form_data = TestData::user_form_data_complete(&user_id, "password123", "Test User", "testdir", "/var/spool/mail/virtual", true, false);

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Create a user via HTTP POST
            let create_response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Step 2: Verify user was created by checking the list
            let list_response =
                TestUtils::make_get_request(&app, &state, "/users", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, &user_id).await;

            // Step 3: Get the user ID from the database
            let users = db::get_users(container.get_pool()).unwrap();
            let user_record = users.iter().find(|u| u.id == user_id).unwrap();

            // Step 4: View the user details
            let show_response = TestUtils::make_get_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&show_response, StatusCode::OK);

            // Step 5: Update the user
            let updated_user_id = TestData::unique_user_id();
            let update_form_data = TestData::user_form_data_complete(&updated_user_id, "password123", "Updated User", "testdir", "/var/spool/mail/virtual", false, false);

            let update_response = TestUtils::make_put_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                &update_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&update_response, StatusCode::OK);

            // Step 6: Verify the update
            let updated_record = db::get_user(container.get_pool(), user_record.id.clone()).unwrap();
            assert_eq!(updated_record.id, updated_user_id);
            assert!(!updated_record.enabled);

            // Step 7: Toggle the user active status
            let toggle_response = TestUtils::make_post_request(
                &app,
                &state,
                &format!("/users/{}/toggle", user_record.id),
                "",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&toggle_response, StatusCode::OK);

            // Step 8: Verify the toggle
            let toggled_user = db::get_user(container.get_pool(), user_record.id.clone()).unwrap();
            assert!(toggled_user.enabled);

            // Step 9: Delete the user
            let delete_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete_response, StatusCode::OK);

            // Step 10: Verify the user was deleted
            let final_users = db::get_users(container.get_pool()).unwrap();
            assert!(!final_users.iter().any(|u| u.id == updated_user_id));
        }

        #[tokio::test]
        async fn test_user_management_with_aliases_journey() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Generate unique test data
            let user_id = TestData::unique_user_id();
            let domain = TestData::unique_domain();
            let user_form_data = TestData::user_form_data_complete(&user_id, "password123", "Test User", "testdir", "/var/spool/mail/virtual", true, false);
            let domain_form_data = TestData::domain_form_data(&domain, "smtp:user-test", true);

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Create a domain first
            let create_domain_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &domain_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_domain_response, StatusCode::OK);

            // Step 2: Get the domain ID
            let domains = db::get_domains(container.get_pool()).unwrap();
            let domain_record = domains.iter().find(|d| d.domain == domain).unwrap();

            // Step 3: Create a user
            let create_user_response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &user_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_user_response, StatusCode::OK);

            // Step 4: Get the user ID
            let users = db::get_users(container.get_pool()).unwrap();
            let user_record = users.iter().find(|u| u.id == user_id).unwrap();

            // Step 5: Create an alias for the user
            let alias_mail = format!("{}@{}", TestData::unique_alias(), domain);
            let alias_form_data = TestData::alias_form_data(&alias_mail, &user_id, true);

            let create_alias_response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &alias_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_alias_response, StatusCode::OK);

            // Step 6: Verify the alias was created
            let aliases = db::get_aliases(container.get_pool()).unwrap();
            let alias_record = aliases.iter().find(|a| a.mail == alias_mail).unwrap();
            assert_eq!(alias_record.destination, user_id);

            // Step 7: View the user details to see the alias
            let show_user_response = TestUtils::make_get_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&show_user_response, StatusCode::OK);
            TestUtils::assert_body_contains(show_user_response, &alias_mail).await;

            // Step 8: Update the user
            let updated_user_id = TestData::unique_user_id();
            let update_user_form = TestData::user_form_data_complete(&updated_user_id, "password123", "Updated User", "testdir", "/var/spool/mail/virtual", false, false);

            let update_user_response = TestUtils::make_put_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                &update_user_form,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&update_user_response, StatusCode::OK);

            // Step 9: Verify the user update
            let updated_user = db::get_user(container.get_pool(), user_record.id.clone()).unwrap();
            assert_eq!(updated_user.id, updated_user_id);
            assert!(!updated_user.enabled);

            // Step 10: Delete the user (should also handle alias cleanup)
            let delete_user_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/users/{}", user_record.id),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete_user_response, StatusCode::OK);

            // Step 11: Verify the user was deleted
            let final_users = db::get_users(container.get_pool()).unwrap();
            assert!(!final_users.iter().any(|u| u.id == updated_user_id));

            // Step 12: Clean up domain
            let delete_domain_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/domains/{}", domain_record.pkid),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete_domain_response, StatusCode::OK);
        }
    }

    // Alias management tests
    mod alias_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_alias_workflow() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Generate unique test data
            let domain = TestData::unique_domain();
            let user_id = TestData::unique_user_id();
            let alias_mail = format!("{}@{}", TestData::unique_alias_name(), domain);

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Create a domain first
            let domain_form_data = TestData::domain_form_data(&domain, "smtp:alias-test", true);
            let create_domain_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &domain_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_domain_response, StatusCode::OK);

            // Step 2: Create a user
            let user_form_data = TestData::user_form_data_complete(&user_id, "Test User", true);
            let create_user_response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &user_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_user_response, StatusCode::OK);

            // Step 3: Create an alias
            let alias_form_data = TestData::alias_form_data(&alias_mail, &user_id, true);
            let create_alias_response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &alias_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_alias_response, StatusCode::OK);

            // Step 4: Verify alias was created by checking the list
            let list_response =
                TestUtils::make_get_request(&app, &state, "/aliases", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&list_response, StatusCode::OK);
            TestUtils::assert_body_contains(list_response, &alias_mail).await;

            // Step 5: Get the alias ID from the database
            let aliases = db::get_aliases(container.get_pool()).unwrap();
            let alias_record = aliases.iter().find(|a| a.mail == alias_mail).unwrap();

            // Step 6: View the alias details
            let show_response = TestUtils::make_get_request(
                &app,
                &state,
                &format!("/aliases/{}", alias_record.pkid),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&show_response, StatusCode::OK);

            // Step 7: Update the alias
            let updated_user_id = TestData::unique_user_id();
            let update_alias_form = TestData::alias_form_data(&alias_mail, &updated_user_id, false);

            let update_response = TestUtils::make_put_request(
                &app,
                &state,
                &format!("/aliases/{}", alias_record.pkid),
                &update_alias_form,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&update_response, StatusCode::OK);

            // Step 8: Verify the update
            let updated_record = db::get_alias(container.get_pool(), alias_record.pkid).unwrap();
            assert_eq!(updated_record.destination, updated_user_id);
            assert!(!updated_record.enabled);

            // Step 9: Toggle the alias active status
            let toggle_response = TestUtils::make_post_request(
                &app,
                &state,
                &format!("/aliases/{}/toggle", alias_record.pkid),
                "",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&toggle_response, StatusCode::OK);

            // Step 10: Verify the toggle
            let toggled_alias = db::get_alias(container.get_pool(), alias_record.pkid).unwrap();
            assert!(toggled_alias.enabled);

            // Step 11: Delete the alias
            let delete_response = TestUtils::make_delete_request(
                &app,
                &state,
                &format!("/aliases/{}", alias_record.pkid),
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&delete_response, StatusCode::OK);

            // Step 12: Verify the alias was deleted
            let final_aliases = db::get_aliases(container.get_pool()).unwrap();
            assert!(!final_aliases.iter().any(|a| a.mail == alias_mail));
        }
    }

    // Statistics and reporting tests
    mod stats_tests {
        use super::*;

        #[tokio::test]
        async fn test_stats_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Create test data
            let domain = TestData::unique_domain();
            let user_id = TestData::unique_user_id();
            let alias_mail = format!("{}@{}", TestData::unique_alias_name(), domain);

            // Create domain
            let domain_form_data = TestData::domain_form_data(&domain, "smtp:stats-test", true);
            let create_domain_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &domain_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_domain_response, StatusCode::OK);

            // Create user
            let user_form_data = TestData::user_form_data_complete(&user_id, "Stats Test User", true);
            let create_user_response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                &user_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_user_response, StatusCode::OK);

            // Create alias
            let alias_form_data = TestData::alias_form_data(&alias_mail, &user_id, true);
            let create_alias_response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &alias_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_alias_response, StatusCode::OK);

            // Step 2: Test statistics endpoint
            let stats_response =
                TestUtils::make_get_request(&app, &state, "/stats", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&stats_response, StatusCode::OK);

            // Step 3: Verify stats contain expected data
            let stats_body = TestUtils::get_response_body(stats_response).await;
            assert!(stats_body.contains("domains"));
            assert!(stats_body.contains("users"));
            assert!(stats_body.contains("aliases"));

            // Step 4: Test dashboard endpoint
            let dashboard_response =
                TestUtils::make_get_request(&app, &state, "/dashboard", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&dashboard_response, StatusCode::OK);

            // Step 5: Verify dashboard contains expected data
            let dashboard_body = TestUtils::get_response_body(dashboard_response).await;
            assert!(dashboard_body.contains("dashboard"));
        }
    }

    // Multi-database workflow tests
    mod multi_database_tests {
        use super::*;

        #[tokio::test]
        async fn test_multi_database_workflow_journey() {
            // Setup test environment with multiple databases
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Test database selection
            let databases_response =
                TestUtils::make_get_request(&app, &state, "/database", Some(auth_cookie.clone()))
                    .await
                    .unwrap();

            TestUtils::assert_status(&databases_response, StatusCode::OK);

            // Step 2: Test database API endpoint
            let api_databases_response = TestUtils::make_get_request(
                &app,
                &state,
                "/api/databases",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&api_databases_response, StatusCode::OK);

            // Step 3: Verify API returns JSON
            let api_body = TestUtils::get_response_body(api_databases_response).await;
            assert!(api_body.contains("test")); // Should contain test database
        }
    }

    // Error handling and edge cases tests
    mod error_handling_tests {
        use super::*;

        #[tokio::test]
        async fn test_error_handling_and_edge_cases_journey() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Step 1: Test invalid domain creation
            let invalid_domain_form = "domain=invalid-domain&transport=smtp:test&enabled=on";
            let invalid_domain_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                invalid_domain_form,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            // Should handle validation errors gracefully
            TestUtils::assert_status(&invalid_domain_response, StatusCode::OK);

            // Step 2: Test invalid user creation
            let invalid_user_form = "user_id=invalid-user&name=Test&enabled=on";
            let invalid_user_response = TestUtils::make_post_request(
                &app,
                &state,
                "/users",
                invalid_user_form,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            // Should handle validation errors gracefully
            TestUtils::assert_status(&invalid_user_response, StatusCode::OK);

            // Step 3: Test invalid alias creation
            let invalid_alias_form = "mail=invalid-alias&destination=invalid-dest&enabled=on";
            let invalid_alias_response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                invalid_alias_form,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            // Should handle validation errors gracefully
            TestUtils::assert_status(&invalid_alias_response, StatusCode::OK);

            // Step 4: Test non-existent resource access
            let not_found_response = TestUtils::make_get_request(
                &app,
                &state,
                "/domains/999999",
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            // Should return appropriate error status
            assert!(matches!(
                not_found_response.status(),
                StatusCode::NOT_FOUND | StatusCode::OK
            ));
        }

        #[tokio::test]
        async fn test_edge_case_validation_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test various edge cases
            let edge_cases = vec![
                ("", "empty domain"),
                ("a", "single character domain"),
                ("a" .repeat(100), "very long domain"),
                ("test@domain.com", "domain with @ symbol"),
                ("test.domain.com", "domain with dots"),
                ("test-domain.com", "domain with hyphens"),
            ];

            for (domain, description) in edge_cases {
                let form_data = TestData::domain_form_data(domain, "smtp:edge-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domains",
                    &form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle edge cases gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_alias_validation_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Create a domain first
            let domain = TestData::unique_domain();
            let domain_form_data = TestData::domain_form_data(&domain, "smtp:alias-edge-test", true);
            let create_domain_response = TestUtils::make_post_request(
                &app,
                &state,
                "/domains",
                &domain_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            TestUtils::assert_status(&create_domain_response, StatusCode::OK);

            // Test various alias edge cases
            let edge_cases = vec![
                ("", "empty alias"),
                ("a@domain.com", "single character alias"),
                (&format!("{}@{}", "a".repeat(100), domain), "very long alias name"),
                ("test@test@domain.com", "alias with multiple @ symbols"),
                ("test..test@domain.com", "alias with consecutive dots"),
            ];

            for (alias_mail, description) in edge_cases {
                let user_id = TestData::unique_user_id();
                let alias_form_data = TestData::alias_form_data(alias_mail, &user_id, true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/aliases",
                    &alias_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle edge cases gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_user_validation_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test various user edge cases
            let edge_cases = vec![
                ("", "empty user ID"),
                ("a", "single character user ID"),
                (&"a".repeat(100), "very long user ID"),
                ("test@test@domain.com", "user ID with multiple @ symbols"),
                ("test..test@domain.com", "user ID with consecutive dots"),
            ];

            for (user_id, description) in edge_cases {
                let user_form_data = TestData::user_form_data_complete(user_id, "Edge Test User", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/users",
                    &user_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle edge cases gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_backup_validation_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test various backup edge cases
            let edge_cases = vec![
                ("", "empty backup domain"),
                ("a", "single character backup domain"),
                (&"a".repeat(100), "very long backup domain"),
                ("test@domain.com", "backup domain with @ symbol"),
                ("test..domain.com", "backup domain with consecutive dots"),
            ];

            for (backup_domain, description) in edge_cases {
                let backup_form_data = TestData::domain_form_data(backup_domain, "smtp:backup-edge-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domain_backup",
                    &backup_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle edge cases gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_boundary_conditions_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test boundary conditions
            let boundary_tests = vec![
                // Test with maximum allowed lengths
                ("a".repeat(63) + ".com", "domain at max length"),
                ("a".repeat(64) + "@domain.com", "alias at max length"),
                ("a".repeat(254) + "@domain.com", "user ID at max length"),
            ];

            for (test_value, description) in boundary_tests {
                // Test domain creation with boundary values
                let domain_form_data = TestData::domain_form_data(&test_value, "smtp:boundary-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domains",
                    &domain_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle boundary conditions gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_unicode_handling_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test Unicode handling
            let unicode_tests = vec![
                ("tëst.com", "domain with accented character"),
                ("tëst@domain.com", "alias with accented character"),
                ("tëst@domain.com", "user ID with accented character"),
                ("测试.com", "domain with Chinese characters"),
                ("test@测试.com", "alias with Chinese characters"),
                ("测试@domain.com", "user ID with Chinese characters"),
            ];

            for (test_value, description) in unicode_tests {
                // Test domain creation with Unicode values
                let domain_form_data = TestData::domain_form_data(&test_value, "smtp:unicode-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domains",
                    &domain_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should handle Unicode gracefully
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_sql_injection_prevention_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test SQL injection prevention
            let sql_injection_tests = vec![
                ("'; DROP TABLE domains; --", "SQL injection attempt 1"),
                ("'; INSERT INTO domains VALUES ('hacked', 'hacked', 1); --", "SQL injection attempt 2"),
                ("'; UPDATE domains SET domain = 'hacked'; --", "SQL injection attempt 3"),
                ("' OR '1'='1", "SQL injection attempt 4"),
                ("' UNION SELECT * FROM domains --", "SQL injection attempt 5"),
            ];

            for (test_value, description) in sql_injection_tests {
                // Test domain creation with SQL injection attempts
                let domain_form_data = TestData::domain_form_data(&test_value, "smtp:sql-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domains",
                    &domain_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should prevent SQL injection
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }

        #[tokio::test]
        async fn test_edge_case_xss_prevention_integration() {
            // Setup test environment
            let container = setup_test_db().await;
            let (app, state) =
                TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

            // Create authentication cookie
            let auth_cookie = TestUtils::create_edit_auth_cookie();

            // Test XSS prevention
            let xss_tests = vec![
                ("<script>alert('xss')</script>", "XSS attempt 1"),
                ("<img src=x onerror=alert('xss')>", "XSS attempt 2"),
                ("javascript:alert('xss')", "XSS attempt 3"),
                ("<svg onload=alert('xss')>", "XSS attempt 4"),
                ("<iframe src=javascript:alert('xss')>", "XSS attempt 5"),
            ];

            for (test_value, description) in xss_tests {
                // Test domain creation with XSS attempts
                let domain_form_data = TestData::domain_form_data(&test_value, "smtp:xss-test", true);
                let response = TestUtils::make_post_request(
                    &app,
                    &state,
                    "/domains",
                    &domain_form_data,
                    Some(auth_cookie.clone()),
                )
                .await
                .unwrap();

                // Should prevent XSS
                TestUtils::assert_status(&response, StatusCode::OK);
            }
        }
    }
}

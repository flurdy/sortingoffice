use axum::http::StatusCode;
use sortingoffice::db;
use sortingoffice::test_helpers::test_utils::TestData;
use sortingoffice::test_helpers::test_utils::TestUtils;
use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

/// Integration tests for alias destinations with multiple comma-separated emails
#[cfg(test)]
mod multi_email_alias_tests {
    use super::*;

    #[tokio::test]
    async fn test_alias_destination_multiple_emails_comma_separated() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();
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

        // Step 2: Test alias with multiple comma-separated email destinations
        let multiple_destinations = "user1@example.com,user2@example.com,user3@example.com";

        // Create form data with multiple destinations
        let alias_form_data = format!(
            "mail={}&destination={}&enabled=on",
            alias_mail, multiple_destinations
        );

        println!(
            "Testing alias with multiple destinations: {}",
            multiple_destinations
        );

        let create_alias_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        // Check the response status
        let status = create_alias_response.status();
        println!("Response status: {}", status);

        // Multiple comma-separated emails should be valid and create the alias successfully
        assert!(
            status == StatusCode::OK || status == StatusCode::FOUND,
            "Expected OK or FOUND for valid multiple emails, got: {}",
            status
        );

        // Verify the alias was created successfully
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let alias_record = aliases.iter().find(|a| a.mail == alias_mail);

        assert!(alias_record.is_some(), "Alias should have been created");
        if let Some(alias) = alias_record {
            assert_eq!(alias.destination, multiple_destinations);
            println!(
                "✅ Alias created successfully with multiple destinations: {}",
                alias.destination
            );
        }
    }

    #[tokio::test]
    async fn test_alias_destination_with_spaces_around_commas() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();
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

        // Step 2: Test alias with spaces around commas
        let destinations_with_spaces = "user1@example.com, user2@example.com , user3@example.com";

        let alias_form_data = format!(
            "mail={}&destination={}&enabled=on",
            alias_mail, destinations_with_spaces
        );

        println!(
            "Testing alias with destinations containing spaces: {}",
            destinations_with_spaces
        );

        let create_alias_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        let status = create_alias_response.status();
        println!("Response status: {}", status);

        // Emails with spaces around commas should be valid and create the alias successfully
        assert!(
            status == StatusCode::OK || status == StatusCode::FOUND,
            "Expected OK or FOUND for valid emails with spaces, got: {}",
            status
        );

        // Verify the alias was created successfully
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let alias_record = aliases.iter().find(|a| a.mail == alias_mail);

        assert!(alias_record.is_some(), "Alias should have been created");
        if let Some(alias) = alias_record {
            assert_eq!(alias.destination, destinations_with_spaces);
            println!(
                "✅ Alias created successfully with destinations containing spaces: {}",
                alias.destination
            );
        }
    }

    #[tokio::test]
    async fn test_alias_destination_mixed_valid_invalid_emails() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();
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

        // Step 2: Test alias with mixed valid and invalid emails
        let mixed_destinations = "valid@example.com,invalid-email,another@valid.com";

        let alias_form_data = format!(
            "mail={}&destination={}&enabled=on",
            alias_mail, mixed_destinations
        );

        println!(
            "Testing alias with mixed valid/invalid destinations: {}",
            mixed_destinations
        );

        let create_alias_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        let status = create_alias_response.status();
        println!("Response status: {}", status);

        // This should be rejected due to invalid email format
        // When validation fails, the form is returned with validation errors (200 OK)
        assert_eq!(
            status,
            StatusCode::OK,
            "Expected OK with validation errors for mixed valid/invalid emails, got: {}",
            status
        );

        // Check that the response contains validation error
        let body = axum::body::to_bytes(create_alias_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("validation-alias-destination-invalid")
                || body_str.contains("Invalid alias destination")
                || body_str.contains("error")
                || body_str.contains("invalid"),
            "Response should contain validation error message for invalid email"
        );

        println!("✅ Alias creation correctly rejected with validation error for invalid email (expected behavior)");
    }

    #[tokio::test]
    async fn test_alias_destination_edge_cases() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();

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

        // Test various edge cases
        let very_long_email = format!("user{}@example.com", "1".repeat(100));
        let test_cases = vec![
            // Empty destination
            ("", "Empty destination"),
            // Single comma
            (",", "Single comma"),
            // Multiple commas
            (",,,", "Multiple commas"),
            // Comma at start
            (",user@example.com", "Comma at start"),
            // Comma at end
            ("user@example.com,", "Comma at end"),
            // Only commas and spaces
            (" , , ", "Only commas and spaces"),
            // Very long list
            (&very_long_email, "Very long email"),
        ];

        for (i, (destinations, description)) in test_cases.iter().enumerate() {
            let alias_mail = format!("{}@{}", TestData::unique_alias_name(), domain);

            let alias_form_data = format!(
                "mail={}&destination={}&enabled=on",
                alias_mail, destinations
            );

            println!("Test case {}: {} - '{}'", i + 1, description, destinations);

            let create_alias_response = TestUtils::make_post_request(
                &app,
                &state,
                "/aliases",
                &alias_form_data,
                Some(auth_cookie.clone()),
            )
            .await
            .unwrap();

            let status = create_alias_response.status();
            println!("  Status: {}", status);

            // Most edge cases should result in validation errors, but some might succeed
            if status == StatusCode::OK {
                // Check if this is a validation error (contains error in body) or success
                let body = axum::body::to_bytes(create_alias_response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let body_str = String::from_utf8_lossy(&body);

                if body_str.contains("validation-alias-destination-invalid")
                    || body_str.contains("Invalid alias destination")
                    || body_str.contains("error")
                    || body_str.contains("invalid")
                {
                    println!("  Validation error (expected for: {})", description);
                } else {
                    println!("  Success (unexpected for: {})", description);
                    // For edge cases that shouldn't succeed, we might want to assert this
                    if *destinations == "" || *destinations == "," || *destinations == ",,," {
                        panic!("Edge case '{}' should have failed validation", description);
                    }
                }
            } else if status == StatusCode::FOUND {
                println!("  Success (redirect)");
            } else {
                panic!("Test case '{}': Unexpected status {}", description, status);
            }
        }
    }

    #[tokio::test]
    async fn test_alias_destination_single_valid_email() {
        // Setup test environment
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Generate unique test data
        let domain = TestData::unique_domain();
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

        // Step 2: Test alias with a single valid email (should succeed)
        let single_destination = "user1@example.com";

        let alias_form_data = format!(
            "mail={}&destination={}&enabled=on",
            alias_mail, single_destination
        );

        println!(
            "Testing alias with single valid destination: {}",
            single_destination
        );

        let create_alias_response = TestUtils::make_post_request(
            &app,
            &state,
            "/aliases",
            &alias_form_data,
            Some(auth_cookie.clone()),
        )
        .await
        .unwrap();

        let status = create_alias_response.status();
        println!("Response status: {}", status);

        // Should succeed (redirect to list or show success)
        assert!(
            status == StatusCode::OK || status == StatusCode::FOUND,
            "Expected OK or FOUND for valid single email, got: {}",
            status
        );

        // Verify the alias was created successfully
        let aliases = db::get_aliases(container.get_pool()).unwrap();
        let alias_record = aliases.iter().find(|a| a.mail == alias_mail);

        assert!(alias_record.is_some(), "Alias should have been created");
        if let Some(alias) = alias_record {
            assert_eq!(alias.destination, single_destination);
            println!(
                "✅ Alias created successfully with single destination: {}",
                alias.destination
            );
        }
    }
}

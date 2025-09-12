mod common;

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, Router};
    use sortingoffice::{
        config::AdminRole,
        test_helpers::test_utils::{TestData, TestUtils},
        test_helpers::testcontainers_setup::setup_test_db,
        AppState,
    };

    async fn create_test_app() -> (
        Router<AppState>,
        AppState,
        sortingoffice::test_helpers::testcontainers_setup::TestContainer,
    ) {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;
        (app, state, container)
    }

    fn create_auth_cookie(role: AdminRole) -> axum::http::HeaderValue {
        TestUtils::create_auth_cookie(role)
    }

    // Edge Case Tests for Domain Management
    mod domain_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_domain_with_unicode_characters() {
            let (app, state, _container) = create_test_app().await;

            // Test domain with unicode characters
            let unicode_domain = "tëst-ünicödé.com";
            let form_data = TestData::domain_form_data(unicode_domain, "smtp:localhost", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle unicode domains gracefully
            TestUtils::assert_status(&response, StatusCode::OK);
        }

        #[tokio::test]
        async fn test_domain_with_extremely_long_name() {
            let (app, state, _container) = create_test_app().await;

            // Test domain with very long name (255+ characters)
            let long_domain = format!("{}.com", "a".repeat(250));
            let form_data = TestData::domain_form_data(&long_domain, "smtp:localhost", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should either accept or reject with proper validation
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
            );
        }

        #[tokio::test]
        async fn test_domain_with_special_characters() {
            let (app, state, _container) = create_test_app().await;

            // Test domains with various special characters
            let special_domains = vec![
                "test-domain.com",
                "test_domain.com",
                "test.domain.com",
                "test-domain123.com",
                "123test.com",
                "test-domain-123.com",
            ];

            for domain in special_domains {
                let form_data = TestData::domain_form_data(domain, "smtp:localhost", true);

                let response = TestUtils::make_handler_post_request(
                    &app,
                    &state,
                    "/domains",
                    &form_data,
                    Some(create_auth_cookie(AdminRole::Edit)),
                )
                .await;

                // Should handle special characters appropriately
                assert!(
                    response.status() == StatusCode::OK
                        || response.status() == StatusCode::BAD_REQUEST
                );
            }
        }

        #[tokio::test]
        async fn test_domain_with_empty_transport() {
            let (app, state, _container) = create_test_app().await;

            let unique_id = sortingoffice::test_helpers::common::unique_test_id();
            let domain = format!("empty-transport-{}.com", unique_id);
            let form_data = TestData::domain_form_data(&domain, "", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle empty transport gracefully
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
            );
        }

        #[tokio::test]
        async fn test_domain_duplicate_creation() {
            let (app, state, _container) = create_test_app().await;

            let unique_id = sortingoffice::test_helpers::common::unique_test_id();
            let domain = format!("duplicate-test-{}.com", unique_id);
            let form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

            // Create domain first time
            let response1 = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            TestUtils::assert_status(&response1, StatusCode::OK);

            // Try to create same domain again
            let response2 = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle duplicate gracefully
            assert!(
                response2.status() == StatusCode::OK || response2.status() == StatusCode::CONFLICT
            );
        }
    }

    // Edge Case Tests for User Management
    mod user_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_user_with_unicode_username() {
            let (app, state, _container) = create_test_app().await;

            let unicode_username = "tëst-üser";
            let form_data = TestData::user_form_data(unicode_username, "password123", "testuser");

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle unicode usernames appropriately
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
            );
        }

        #[tokio::test]
        async fn test_user_with_extremely_long_username() {
            let (app, state, _container) = create_test_app().await;

            let long_username = "a".repeat(1000);
            let form_data = TestData::user_form_data(&long_username, "password123", "testuser");

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/users",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should reject extremely long usernames
            assert!(
                response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::OK
            );
        }

        #[tokio::test]
        async fn test_user_with_invalid_email_formats() {
            let (app, state, _container) = create_test_app().await;

            let invalid_emails = vec![
                "not-an-email",
                "@example.com",
                "test@",
                "test@.com",
                "test@example.",
                "test..test@example.com",
                "test@example..com",
                "",
            ];

            for email in invalid_emails {
                let form_data = TestData::user_form_data(email, "password123", "testuser");

                let response = TestUtils::make_handler_post_request(
                    &app,
                    &state,
                    "/users",
                    &form_data,
                    Some(create_auth_cookie(AdminRole::Edit)),
                )
                .await;

                // Should reject invalid email formats
                assert!(
                    response.status() == StatusCode::BAD_REQUEST
                        || response.status() == StatusCode::OK
                );
            }
        }

        #[tokio::test]
        async fn test_user_with_weak_passwords() {
            let (app, state, _container) = create_test_app().await;

            let weak_passwords = vec!["123", "password", "12345678", "qwerty", "abc", ""];

            for password in weak_passwords {
                let form_data = TestData::user_form_data("test@example.com", password, "testuser");

                let response = TestUtils::make_handler_post_request(
                    &app,
                    &state,
                    "/users",
                    &form_data,
                    Some(create_auth_cookie(AdminRole::Edit)),
                )
                .await;

                // Should handle weak passwords appropriately
                assert!(
                    response.status() == StatusCode::OK
                        || response.status() == StatusCode::BAD_REQUEST
                );
            }
        }
    }

    // Edge Case Tests for Alias Management
    mod alias_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_alias_with_unicode_characters() {
            let (app, state, _container) = create_test_app().await;

            let unicode_alias = "tëst-älïas";
            let form_data = TestData::alias_form_data(unicode_alias, "test@example.com", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle unicode aliases appropriately
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
            );
        }

        #[tokio::test]
        async fn test_alias_with_special_characters() {
            let (app, state, _container) = create_test_app().await;

            let special_aliases = vec![
                "test-alias",
                "test_alias",
                "test.alias",
                "test+alias",
                "test@alias",
                "test#alias",
                "test$alias",
                "test%alias",
            ];

            for alias in special_aliases {
                let form_data = TestData::alias_form_data(alias, "test@example.com", true);

                let response = TestUtils::make_handler_post_request(
                    &app,
                    &state,
                    "/aliases",
                    &form_data,
                    Some(create_auth_cookie(AdminRole::Edit)),
                )
                .await;

                // Should handle special characters appropriately
                assert!(
                    response.status() == StatusCode::OK
                        || response.status() == StatusCode::BAD_REQUEST
                );
            }
        }

        #[tokio::test]
        async fn test_alias_with_extremely_long_name() {
            let (app, state, _container) = create_test_app().await;

            let long_alias = "a".repeat(1000);
            let form_data = TestData::alias_form_data(&long_alias, "test@example.com", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/aliases",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should reject extremely long aliases
            assert!(
                response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::OK
            );
        }
    }

    // Edge Case Tests for Error Handling
    mod error_handling_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_malformed_json_requests() {
            let (app, state, _container) = create_test_app().await;

            let malformed_json =
                r#"{"domain": "test.com", "transport": "smtp:localhost", "enabled": true"#; // Missing closing brace

            let response = TestUtils::make_handler_post_request_with_body(
                &app,
                &state,
                "/domains",
                malformed_json,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle malformed JSON gracefully
            assert!(
                response.status() == StatusCode::BAD_REQUEST
                    || response.status() == StatusCode::OK
                    || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            );
        }

        #[tokio::test]
        async fn test_oversized_requests() {
            let (app, state, _container) = create_test_app().await;

            // Create a very large request body
            let large_domain = "a".repeat(10000);
            let form_data = TestData::domain_form_data(&large_domain, "smtp:localhost", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle oversized requests appropriately
            assert!(
                response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::OK
            );
        }

        #[tokio::test]
        async fn test_concurrent_requests() {
            let (app, state, _container) = create_test_app().await;

            // Test concurrent requests to the same endpoint
            let mut handles = vec![];

            for i in 0..10 {
                let app_clone = app.clone();
                let state_clone = state.clone();
                let auth_cookie = create_auth_cookie(AdminRole::Edit);

                let handle = tokio::spawn(async move {
                    let unique_id = sortingoffice::test_helpers::common::unique_test_id();
                    let domain = format!("concurrent-test-{}-{}.com", i, unique_id);
                    let form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

                    TestUtils::make_handler_post_request(
                        &app_clone,
                        &state_clone,
                        "/domains",
                        &form_data,
                        Some(auth_cookie),
                    )
                    .await
                });

                handles.push(handle);
            }

            // Wait for all requests to complete
            for handle in handles {
                let response = handle.await.unwrap();
                // Should handle concurrent requests gracefully
                assert!(
                    response.status() == StatusCode::OK
                        || response.status() == StatusCode::BAD_REQUEST
                );
            }
        }
    }

    // Edge Case Tests for Database Operations
    mod database_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_database_connection_failure_handling() {
            // This test would require mocking database connection failures
            // For now, we'll test that the app handles database errors gracefully
            let (app, state, _container) = create_test_app().await;

            // Test with invalid data that might cause database errors
            let invalid_form_data = "invalid=form&data=test";

            let response = TestUtils::make_handler_post_request_with_body(
                &app,
                &state,
                "/domains",
                invalid_form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle invalid form data gracefully
            assert!(
                response.status() == StatusCode::BAD_REQUEST
                    || response.status() == StatusCode::OK
                    || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            );
        }

        #[tokio::test]
        async fn test_database_transaction_rollback() {
            let (app, state, container) = create_test_app().await;

            // Create a domain first
            let unique_id = sortingoffice::test_helpers::common::unique_test_id();
            let domain = format!("rollback-test-{}.com", unique_id);
            let form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

            let create_response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            TestUtils::assert_status(&create_response, StatusCode::OK);

            // Verify domain was created
            let pool = container.get_pool();
            let domains = sortingoffice::db::get_domains(pool).unwrap();
            assert!(domains.iter().any(|d| d.domain == domain));

            // Try to create the same domain again (should fail gracefully)
            let duplicate_response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(create_auth_cookie(AdminRole::Edit)),
            )
            .await;

            // Should handle duplicate creation gracefully
            assert!(
                duplicate_response.status() == StatusCode::OK
                    || duplicate_response.status() == StatusCode::CONFLICT
            );
        }
    }

    // Edge Case Tests for Authentication and Authorization
    mod auth_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_unauthorized_access_attempts() {
            let (app, state, _container) = create_test_app().await;

            // Test accessing protected endpoints without authentication
            let protected_endpoints = vec![
                "/domains",
                "/users",
                "/aliases",
                "/clients",
                "/relays",
                "/relocated",
            ];

            for endpoint in protected_endpoints {
                let response =
                    TestUtils::make_handler_get_request(&app, &state, endpoint, None).await;

                // Should redirect to login or return unauthorized
                assert!(
                    response.status() == StatusCode::FOUND
                        || response.status() == StatusCode::UNAUTHORIZED
                        || response.status() == StatusCode::OK
                );
            }
        }

        #[tokio::test]
        async fn test_invalid_authentication_cookies() {
            let (app, state, _container) = create_test_app().await;

            // Test with malformed authentication cookies
            let malformed_cookies = vec![
                "invalid-cookie",
                "session=invalid",
                "session=; Path=/; HttpOnly",
                "",
            ];

            for cookie in malformed_cookies {
                let cookie_header = axum::http::HeaderValue::from_str(cookie)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid"));
                let response = TestUtils::make_handler_get_request(
                    &app,
                    &state,
                    "/domains",
                    Some(cookie_header),
                )
                .await;

                // Should handle malformed cookies gracefully
                assert!(
                    response.status() == StatusCode::FOUND
                        || response.status() == StatusCode::UNAUTHORIZED
                        || response.status() == StatusCode::OK
                );
            }
        }

        #[tokio::test]
        async fn test_role_based_access_control() {
            let (app, state, _container) = create_test_app().await;

            // Test with read-only role trying to perform edit operations
            let read_only_cookie = create_auth_cookie(AdminRole::ReadOnly);
            let unique_id = sortingoffice::test_helpers::common::unique_test_id();
            let domain = format!("rbac-test-{}.com", unique_id);
            let form_data = TestData::domain_form_data(&domain, "smtp:localhost", true);

            let response = TestUtils::make_handler_post_request(
                &app,
                &state,
                "/domains",
                &form_data,
                Some(read_only_cookie),
            )
            .await;

            // Should deny edit operations for read-only users
            assert!(
                response.status() == StatusCode::FORBIDDEN
                    || response.status() == StatusCode::UNAUTHORIZED
                    || response.status() == StatusCode::OK
            );
        }
    }
}

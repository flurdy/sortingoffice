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
    use crate::config::DatabaseConfig;

    async fn create_test_app() -> (Router, AppState) {
        let db_config = vec![DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: std::env::var("TEST_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "mysql://root:password@localhost/sortingoffice_test".to_string()),
        }];
        let db_manager = crate::db::DatabaseManager::new(db_config).await.expect("Failed to create database manager");
        let i18n = crate::i18n::I18n::new("en-US").expect("Failed to initialize i18n");
        let config = crate::config::Config::default();
        let state = AppState { db_manager, i18n, config };
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

        (app, state)
    }

    #[tokio::test]
    async fn test_full_domain_workflow() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state.db_manager.get_default_pool().await.expect("Failed to get database pool");
        cleanup_test_db(&pool);

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
        let domains = crate::db::get_domains(&pool).unwrap();
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
        let updated_domain = crate::db::get_domain(&pool, domain.pkid).unwrap();
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
        let toggled_domain = crate::db::get_domain(&pool, domain.pkid).unwrap();
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

        // Step 10: Verify deletion
        let deleted_domain = crate::db::get_domain(&pool, domain.pkid);
        assert!(deleted_domain.is_err());

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_full_user_workflow() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state.db_manager.get_default_pool().await.expect("Failed to get database pool");
        cleanup_test_db(&pool);

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
        let user_form_data = "id=integrationuser@integration-user-test.com&password=securepass123&name=Integration+User&domain=integration-user-test.com&quota=100000&active=on&change_password=false";

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
        let users = crate::db::get_users(&pool).unwrap();
        let user = users
            .iter()
            .find(|u| u.id == "integrationuser@integration-user-test.com")
            .unwrap();

        // Step 5: Update the user
        let update_form_data = "id=updateduser@integration-user-test.com&password=newpass456&name=Updated+User&domain=integration-user-test.com&quota=200000&active=off&change_password=false";

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

        // Step 6: Verify the update
        let updated_user = crate::db::get_user(&pool, user.pkid).unwrap();
        assert_eq!(updated_user.id, "updateduser@integration-user-test.com");
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.enabled, false);
        assert_eq!(updated_user.change_password, false);

        // Step 7: Toggle user active status
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

        // Step 8: Verify the toggle
        let toggled_user = crate::db::get_user(&pool, user.pkid).unwrap();
        assert_eq!(toggled_user.enabled, true);
        assert_eq!(toggled_user.change_password, false);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_full_alias_workflow() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state.db_manager.get_default_pool().await.expect("Failed to get database pool");
        cleanup_test_db(&pool);

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
        let aliases = crate::db::get_aliases(&pool).unwrap();
        let alias = aliases
            .iter()
            .find(|a| a.mail == "test@integration-alias-test.com")
            .unwrap();

        // Step 5: Update the alias
        let update_form_data = "mail=updated%40integration-alias-test.com&destination=updated%40integration-alias-test.com&domain=integration-alias-test.com&active=off";

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

        // Step 6: Verify the update
        let updated_alias = crate::db::get_alias(&pool, alias.pkid).unwrap();
        assert_eq!(updated_alias.mail, "updated@integration-alias-test.com");
        assert_eq!(
            updated_alias.destination,
            "updated@integration-alias-test.com"
        );
        assert_eq!(updated_alias.enabled, false);

        // Step 7: Toggle alias active status
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

        // Step 8: Verify the toggle
        let toggled_alias = crate::db::get_alias(&pool, alias.pkid).unwrap();
        assert_eq!(toggled_alias.enabled, true);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_stats_integration() {
        let (app, state) = create_test_app().await;

        // Clean up before test
        let pool = state.db_manager.get_default_pool().await.expect("Failed to get database pool");
        cleanup_test_db(&pool);

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

        let user_form_data = "id=statsuser@integration-stats-test.com&password=password123&name=Stats+User&domain=integration-stats-test.com&quota=100000&active=on&change_password=false";

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
        let system_stats = crate::db::get_system_stats(&pool).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);

        cleanup_test_db(&pool);
    }
}

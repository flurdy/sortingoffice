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
    async fn test_full_domain_workflow() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Step 1: Create a domain via HTTP POST
        let form_data = "domain=integration-domain.com&description=Integration+Test+Domain&aliases=15&maxquota=2000000&quota=1000000&transport=smtp%3Aintegration&backupmx=on&active=on";
        
        let create_response = app
            .clone()
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

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 2: Verify domain was created by checking the list
        let list_response = app
            .clone()
            .oneshot(Request::builder().uri("/domains").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("integration-domain.com"));

        // Step 3: Get the domain ID from the database
        let domains = crate::db::get_domains(&state.pool).unwrap();
        let domain = domains.iter().find(|d| d.domain == "integration-domain.com").unwrap();

        // Step 4: View the domain details
        let show_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/domains/{}", domain.pkid))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(show_response.status(), StatusCode::OK);

        // Step 5: Update the domain
        let update_form_data = "domain=updated-integration.com&description=Updated+Integration+Domain&aliases=25&maxquota=3000000&quota=1500000&transport=smtp%3Aupdated&backupmx=off&active=off";
        
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/domains/{}", domain.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(update_form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 6: Verify the update
        let updated_domain = crate::db::get_domain(&state.pool, domain.pkid).unwrap();
        assert_eq!(updated_domain.domain, "updated-integration.com");
        assert_eq!(updated_domain.aliases, 25);
        assert_eq!(updated_domain.active, false);

        // Step 7: Toggle the domain active status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/domains/{}/toggle", domain.pkid))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 8: Verify the toggle
        let toggled_domain = crate::db::get_domain(&state.pool, domain.pkid).unwrap();
        assert_eq!(toggled_domain.active, true);

        // Step 9: Delete the domain
        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/domains/{}", domain.pkid))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::OK);

        // Step 10: Verify deletion
        let deleted_domain = crate::db::get_domain(&state.pool, domain.pkid);
        assert!(deleted_domain.is_err());

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_full_user_workflow() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Step 1: Create a domain first (required for users)
        let domain_form_data = "domain=integration-user-test.com&description=User+Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost&backupmx=off&active=on";
        
        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        // Step 2: Create a user via HTTP POST
        let user_form_data = "username=integrationuser&password=securepass123&name=Integration+User&domain=integration-user-test.com&quota=100000&active=on";
        
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(user_form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 3: Verify user was created
        let list_response = app
            .clone()
            .oneshot(Request::builder().uri("/users").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("integrationuser"));

        // Step 4: Get the user ID from the database
        let users = crate::db::get_users(&state.pool).unwrap();
        let user = users.iter().find(|u| u.username == "integrationuser").unwrap();

        // Step 5: Update the user
        let update_form_data = "username=updateduser&password=newpass456&name=Updated+User&domain=integration-user-test.com&quota=200000&active=off";
        
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/users/{}", user.pkid))
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(update_form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 6: Verify the update
        let updated_user = crate::db::get_user(&state.pool, user.pkid).unwrap();
        assert_eq!(updated_user.username, "updateduser");
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.active, false);

        // Step 7: Toggle user active status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/users/{}/toggle", user.pkid))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 8: Verify the toggle
        let toggled_user = crate::db::get_user(&state.pool, user.pkid).unwrap();
        assert_eq!(toggled_user.active, true);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_full_alias_workflow() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Step 1: Create a domain first (required for aliases)
        let domain_form_data = "domain=integration-alias-test.com&description=Alias+Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost&backupmx=off&active=on";
        
        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap()
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
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);

        // Step 3: Verify alias was created
        let list_response = app
            .clone()
            .oneshot(Request::builder().uri("/aliases").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("test@integration-alias-test.com"));

        // Step 4: Get the alias ID from the database
        let aliases = crate::db::get_aliases(&state.pool).unwrap();
        let alias = aliases.iter().find(|a| a.mail == "test@integration-alias-test.com").unwrap();

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
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Step 6: Verify the update
        let updated_alias = crate::db::get_alias(&state.pool, alias.pkid).unwrap();
        assert_eq!(updated_alias.mail, "updated@integration-alias-test.com");
        assert_eq!(updated_alias.destination, "updated@integration-alias-test.com");
        assert_eq!(updated_alias.active, false);

        // Step 7: Toggle alias active status
        let toggle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/aliases/{}/toggle-list", alias.pkid))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(toggle_response.status(), StatusCode::OK);

        // Step 8: Verify the toggle
        let toggled_alias = crate::db::get_alias(&state.pool, alias.pkid).unwrap();
        assert_eq!(toggled_alias.active, true);

        cleanup_test_db(&state.pool);
    }

    #[tokio::test]
    async fn test_stats_integration() {
        let (app, state) = create_test_app().await;
        
        // Clean up before test
        cleanup_test_db(&state.pool);

        // Step 1: Create test data
        let domain_form_data = "domain=integration-stats-test.com&description=Stats+Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost&backupmx=off&active=on";
        
        let _domain_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/domains")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(domain_form_data))
                    .unwrap()
            )
            .await
            .unwrap();

        let user_form_data = "username=statsuser&password=password123&name=Stats+User&domain=integration-stats-test.com&quota=100000&active=on";
        
        let _user_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(user_form_data))
                    .unwrap()
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
                    .unwrap()
            )
            .await
            .unwrap();

        // Step 2: Test stats endpoint
        let stats_response = app
            .clone()
            .oneshot(Request::builder().uri("/stats").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(stats_response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Should contain statistics information
        assert!(body_str.contains("Statistics") || body_str.contains("stats"));

        // Step 3: Verify database stats match
        let system_stats = crate::db::get_system_stats(&state.pool).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);

        cleanup_test_db(&state.pool);
    }
} 

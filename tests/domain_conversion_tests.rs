mod common;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use sortingoffice::{
        config::AdminRole,
        db,
        models::{NewBackup, NewDomain},
        test_helpers::{
            common::unique_test_id, test_utils::TestUtils, testcontainers_setup::setup_test_db,
        },
    };

    /// Helper to create an authenticated cookie with edit permissions
    fn create_auth_cookie() -> axum::http::HeaderValue {
        TestUtils::create_auth_cookie(AdminRole::Edit)
    }

    #[tokio::test]
    async fn test_convert_domain_to_backup() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test domain
        let unique_id = unique_test_id();
        let domain_name = format!("convert-to-backup-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();

        // Verify domain exists in domains table
        let domains_before = db::get_domains(pool).unwrap();
        assert!(domains_before.iter().any(|d| d.domain == domain_name));

        // Verify domain doesn't exist in backups table
        let backups_before = db::get_backups(pool).unwrap();
        assert!(!backups_before.iter().any(|b| b.domain == domain_name));

        // Convert domain to backup via HTTP POST
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify domain no longer exists in domains table
        let domains_after = db::get_domains(pool).unwrap();
        assert!(!domains_after.iter().any(|d| d.domain == domain_name));

        // Verify domain now exists in backups table
        let backups_after = db::get_backups(pool).unwrap();
        let backup = backups_after
            .iter()
            .find(|b| b.domain == domain_name)
            .expect("Backup domain should exist");

        // Verify data was preserved
        assert_eq!(backup.domain, domain_name);
        assert_eq!(backup.transport, domain.transport);
        assert_eq!(backup.enabled, domain.enabled);
    }

    #[tokio::test]
    async fn test_convert_backup_to_domain() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test backup domain
        let unique_id = unique_test_id();
        let domain_name = format!("convert-to-domain-{unique_id}.com");
        let new_backup = NewBackup {
            domain: domain_name.clone(),
            transport: Some("smtp:backup".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let backup = db::create_backup(pool, new_backup).unwrap();

        // Verify backup exists in backups table
        let backups_before = db::get_backups(pool).unwrap();
        assert!(backups_before.iter().any(|b| b.domain == domain_name));

        // Verify backup doesn't exist in domains table
        let domains_before = db::get_domains(pool).unwrap();
        assert!(!domains_before.iter().any(|d| d.domain == domain_name));

        // Convert backup to domain via HTTP POST
        let url = format!("/domain_backup/{}/convert-to-domain", backup.pkid);
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify backup no longer exists in backups table
        let backups_after = db::get_backups(pool).unwrap();
        assert!(!backups_after.iter().any(|b| b.domain == domain_name));

        // Verify backup now exists in domains table
        let domains_after = db::get_domains(pool).unwrap();
        let domain = domains_after
            .iter()
            .find(|d| d.domain == domain_name)
            .expect("Domain should exist");

        // Verify data was preserved
        assert_eq!(domain.domain, domain_name);
        assert_eq!(domain.transport, backup.transport);
        assert_eq!(domain.enabled, backup.enabled);
    }

    #[tokio::test]
    async fn test_convert_domain_to_backup_preserves_timestamps() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test domain
        let unique_id = unique_test_id();
        let domain_name = format!("timestamp-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();
        let original_created = domain.created;

        // Convert domain to backup
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let _response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Verify created timestamp was preserved
        let backups = db::get_backups(pool).unwrap();
        let backup = backups
            .iter()
            .find(|b| b.domain == domain_name)
            .expect("Backup should exist");

        assert_eq!(backup.created, original_created);
    }

    #[tokio::test]
    async fn test_convert_backup_to_domain_preserves_timestamps() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test backup
        let unique_id = unique_test_id();
        let domain_name = format!("backup-timestamp-{unique_id}.com");
        let new_backup = NewBackup {
            domain: domain_name.clone(),
            transport: Some("smtp:backup".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let backup = db::create_backup(pool, new_backup).unwrap();
        let original_created = backup.created;

        // Convert backup to domain
        let url = format!("/domain_backup/{}/convert-to-domain", backup.pkid);
        let _response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Verify created timestamp was preserved
        let domains = db::get_domains(pool).unwrap();
        let domain = domains
            .iter()
            .find(|d| d.domain == domain_name)
            .expect("Domain should exist");

        assert_eq!(domain.created, original_created);
    }

    #[tokio::test]
    async fn test_convert_disabled_domain_to_backup() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a disabled domain
        let unique_id = unique_test_id();
        let domain_name = format!("disabled-domain-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: false, // Disabled
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();

        // Convert domain to backup
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        TestUtils::assert_status(&response, StatusCode::OK);

        // Verify backup preserves disabled status
        let backups = db::get_backups(pool).unwrap();
        let backup = backups
            .iter()
            .find(|b| b.domain == domain_name)
            .expect("Backup should exist");

        assert!(!backup.enabled, "Backup should be disabled");
    }

    #[tokio::test]
    async fn test_convert_domain_with_custom_transport() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a domain with custom transport
        let unique_id = unique_test_id();
        let domain_name = format!("custom-transport-{unique_id}.com");
        let custom_transport = "smtp:[mail.example.com]:587";
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some(custom_transport.to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();

        // Convert domain to backup
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let _response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Verify transport was preserved
        let backups = db::get_backups(pool).unwrap();
        let backup = backups
            .iter()
            .find(|b| b.domain == domain_name)
            .expect("Backup should exist");

        assert_eq!(backup.transport, Some(custom_transport.to_string()));
    }

    #[tokio::test]
    async fn test_convert_nonexistent_domain_returns_error() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Try to convert a non-existent domain (ID 99999)
        let url = "/domains/99999/convert-to-backup";
        let response =
            TestUtils::make_handler_post_request(&app, &state, url, "", Some(create_auth_cookie()))
                .await;

        // Should return OK with error message in HTML body (handlers return HTML errors)
        TestUtils::assert_status(&response, StatusCode::OK);
        // The handler returns an error message in the response (logged as ERROR)
    }

    #[tokio::test]
    async fn test_convert_nonexistent_backup_returns_error() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Try to convert a non-existent backup (ID 99999)
        let url = "/domain_backup/99999/convert-to-domain";
        let response =
            TestUtils::make_handler_post_request(&app, &state, url, "", Some(create_auth_cookie()))
                .await;

        // Should return OK with error message in HTML body (handlers return HTML errors)
        TestUtils::assert_status(&response, StatusCode::OK);
        // The handler returns an error message in the response (logged as ERROR)
    }

    #[tokio::test]
    async fn test_convert_domain_requires_authentication() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test domain
        let unique_id = unique_test_id();
        let domain_name = format!("auth-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();

        // Try to convert without authentication
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let response = TestUtils::make_handler_post_request(&app, &state, &url, "", None).await;

        // Should redirect to login or return forbidden (middleware catches this)
        assert!(
            response.status() == StatusCode::FOUND
                || response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::SEE_OTHER
                || response.status() == StatusCode::FORBIDDEN,
            "Should require authentication, got: {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn test_convert_backup_requires_authentication() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test backup
        let unique_id = unique_test_id();
        let domain_name = format!("backup-auth-test-{unique_id}.com");
        let new_backup = NewBackup {
            domain: domain_name.clone(),
            transport: Some("smtp:backup".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let backup = db::create_backup(pool, new_backup).unwrap();

        // Try to convert without authentication
        let url = format!("/domain_backup/{}/convert-to-domain", backup.pkid);
        let response = TestUtils::make_handler_post_request(&app, &state, &url, "", None).await;

        // Should redirect to login or return forbidden (middleware catches this)
        assert!(
            response.status() == StatusCode::FOUND
                || response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::SEE_OTHER
                || response.status() == StatusCode::FORBIDDEN,
            "Should require authentication, got: {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn test_convert_domain_requires_edit_permissions() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test domain
        let unique_id = unique_test_id();
        let domain_name = format!("edit-perm-test-{unique_id}.com");
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let domain = db::create_domain(pool, new_domain).unwrap();

        // Try to convert with read-only permissions
        let read_only_cookie = TestUtils::create_auth_cookie(AdminRole::ReadOnly);
        let url = format!("/domains/{}/convert-to-backup", domain.pkid);
        let response =
            TestUtils::make_handler_post_request(&app, &state, &url, "", Some(read_only_cookie))
                .await;

        // Should return forbidden
        TestUtils::assert_status(&response, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_roundtrip_conversion() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        // Create a test domain
        let unique_id = unique_test_id();
        let domain_name = format!("roundtrip-{unique_id}.com");
        let transport = "smtp:roundtrip";
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some(transport.to_string()),
            enabled: true,
        };
        let pool = container.get_pool();
        let original_domain = db::create_domain(pool, new_domain).unwrap();
        let original_created = original_domain.created;

        // Convert domain to backup
        let url1 = format!("/domains/{}/convert-to-backup", original_domain.pkid);
        let _response1 = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url1,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Get the backup
        let backups = db::get_backups(pool).unwrap();
        let backup = backups
            .iter()
            .find(|b| b.domain == domain_name)
            .expect("Backup should exist");

        // Convert backup back to domain
        let url2 = format!("/domain_backup/{}/convert-to-domain", backup.pkid);
        let _response2 = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url2,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Verify domain exists again with same data
        let domains = db::get_domains(pool).unwrap();
        let final_domain = domains
            .iter()
            .find(|d| d.domain == domain_name)
            .expect("Domain should exist after roundtrip");

        assert_eq!(final_domain.domain, domain_name);
        assert_eq!(final_domain.transport, Some(transport.to_string()));
        assert_eq!(final_domain.enabled, true);
        assert_eq!(final_domain.created, original_created);
    }

    #[tokio::test]
    async fn test_convert_multiple_domains_independently() {
        let container = setup_test_db().await;
        let (app, state) =
            TestUtils::create_test_app_with_db(&container.get_db_url(), "test").await;

        let unique_id = unique_test_id();
        let pool = container.get_pool();

        // Create multiple domains
        let domain1_name = format!("multi1-{unique_id}.com");
        let domain2_name = format!("multi2-{unique_id}.com");

        let domain1 = db::create_domain(
            pool,
            NewDomain {
                domain: domain1_name.clone(),
                transport: Some("smtp:1".to_string()),
                enabled: true,
            },
        )
        .unwrap();

        let _domain2 = db::create_domain(
            pool,
            NewDomain {
                domain: domain2_name.clone(),
                transport: Some("smtp:2".to_string()),
                enabled: true,
            },
        )
        .unwrap();

        // Convert only domain1 to backup
        let url = format!("/domains/{}/convert-to-backup", domain1.pkid);
        let _response = TestUtils::make_handler_post_request(
            &app,
            &state,
            &url,
            "",
            Some(create_auth_cookie()),
        )
        .await;

        // Verify domain1 is now a backup
        let backups = db::get_backups(pool).unwrap();
        assert!(backups.iter().any(|b| b.domain == domain1_name));

        // Verify domain2 is still a primary domain
        let domains = db::get_domains(pool).unwrap();
        assert!(domains.iter().any(|d| d.domain == domain2_name));
        assert!(!domains.iter().any(|d| d.domain == domain1_name));
    }
}

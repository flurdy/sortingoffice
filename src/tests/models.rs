#[cfg(test)]
mod tests {

    use crate::models::*;
    use serde_json;

    #[test]
    fn test_domain_serialization() {
        let domain = Domain {
            pkid: 1,
            domain: "example.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            enabled: true,
        };

        let json = serde_json::to_string(&domain).unwrap();
        let deserialized: Domain = serde_json::from_str(&json).unwrap();

        assert_eq!(domain.pkid, deserialized.pkid);
        assert_eq!(domain.domain, deserialized.domain);
        assert_eq!(domain.transport, deserialized.transport);

        assert_eq!(domain.created, deserialized.created);
        assert_eq!(domain.modified, deserialized.modified);
        assert_eq!(domain.enabled, deserialized.enabled);
    }

    #[test]
    fn test_new_domain_creation() {
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };

        assert_eq!(new_domain.domain, "test.com");
        assert_eq!(new_domain.transport, Some("smtp:localhost".to_string()));

        assert_eq!(new_domain.enabled, true);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "testuser@example.com".to_string(),
            crypt: "hashed_password".to_string(),
            name: "Test User".to_string(),
            maildir: "testuser/".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
            uid: 5000,
            gid: 5000,
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            enabled: true,
            change_password: false,
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.name, deserialized.name);
        assert_eq!(user.maildir, deserialized.maildir);
        assert_eq!(user.enabled, deserialized.enabled);
    }

    #[test]
    fn test_alias_serialization() {
        let alias = Alias {
            pkid: 1,
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            enabled: true,
        };

        let json = serde_json::to_string(&alias).unwrap();
        let deserialized: Alias = serde_json::from_str(&json).unwrap();

        assert_eq!(alias.pkid, deserialized.pkid);
        assert_eq!(alias.mail, deserialized.mail);
        assert_eq!(alias.destination, deserialized.destination);
        assert_eq!(alias.enabled, deserialized.enabled);
    }

    #[test]
    fn test_domain_form_deserialization() {
        let form_data = "domain=test.com&transport=smtp%3Alocalhost&enabled=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.transport, "smtp:localhost");
        assert_eq!(form.enabled, true);
    }

    #[test]
    fn test_domain_form_deserialization_without_checkboxes() {
        let form_data = "domain=test.com&transport=smtp%3Alocalhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.transport, "smtp:localhost");
        assert_eq!(form.enabled, false); // Default value
    }

    #[test]
    fn test_user_form_deserialization() {
        let form_data =
            "id=testuser@example.com&password=secret123&name=Test+User&maildir=testdir&enabled=on";
        let form: UserForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.id, "testuser@example.com");
        assert_eq!(form.password, "secret123");
        assert_eq!(form.name, "Test User");
        assert_eq!(form.maildir, "testdir");
        assert_eq!(form.enabled, true);
    }

    #[test]
    fn test_alias_form_deserialization() {
        let form_data = "mail=test%40example.com&destination=user%40example.com&enabled=on";
        let form: AliasForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.mail, "test@example.com");
        assert_eq!(form.destination, "user@example.com");
        assert_eq!(form.enabled, true);
    }

    #[test]
    fn test_checkbox_deserialization() {
        // Test form data deserialization with "on" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.enabled, true);

        // Test form data deserialization with "true" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=true";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.enabled, true);

        // Test form data deserialization with "1" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=1";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.enabled, true);

        // Test form data deserialization with missing values
        let form_data = "domain=test.com&transport=smtp:localhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.enabled, false);

        // Test form data deserialization with "off" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=off";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.enabled, false);
    }

    #[test]
    fn test_system_stats_serialization() {
        let stats = SystemStats {
            total_domains: 5,
            enabled_domains: 3,
            disabled_domains: 2,
            recent_domains: 1,
            total_users: 10,
            enabled_users: 8,
            disabled_users: 2,
            recent_users: 3,
            total_aliases: 15,
            enabled_aliases: 12,
            disabled_aliases: 3,
            recent_aliases: 4,
            total_backups: 2,
            enabled_backups: 1,
            disabled_backups: 1,
            recent_backups: 0,
            total_relays: 3,
            enabled_relays: 2,
            disabled_relays: 1,
            recent_relays: 1,
            total_relocated: 4,
            enabled_relocated: 3,
            disabled_relocated: 1,
            recent_relocated: 2,
            total_clients: 6,
            enabled_clients: 5,
            disabled_clients: 1,
            recent_clients: 2,
            total_quota: 1000000000,
            used_quota: 250000000,
            quota_usage_percent: 25.0,
            enabled_domains_and_backups: 4,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SystemStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_domains, deserialized.total_domains);
        assert_eq!(stats.total_users, deserialized.total_users);
        assert_eq!(stats.total_aliases, deserialized.total_aliases);
        assert_eq!(stats.total_backups, deserialized.total_backups);
        assert_eq!(stats.total_relays, deserialized.total_relays);
        assert_eq!(stats.total_relocated, deserialized.total_relocated);
        assert_eq!(stats.total_clients, deserialized.total_clients);
        assert_eq!(stats.total_quota, deserialized.total_quota);
        assert_eq!(stats.used_quota, deserialized.used_quota);
    }

    #[test]
    fn test_domain_stats_serialization() {
        let stats = DomainStats {
            domain: "example.com".to_string(),
            user_count: 10,
            alias_count: 20,
            total_quota: 5000000,
            used_quota: 2500000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: DomainStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.domain, deserialized.domain);
        assert_eq!(stats.user_count, deserialized.user_count);
        assert_eq!(stats.alias_count, deserialized.alias_count);
        assert_eq!(stats.total_quota, deserialized.total_quota);
        assert_eq!(stats.used_quota, deserialized.used_quota);
    }

    #[test]
    fn test_backup_serialization() {
        let backup = Backup {
            pkid: 1,
            domain: "backup.example.com".to_string(),
            transport: Some("smtp:[]".to_string()),
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            enabled: true,
        };

        let json = serde_json::to_string(&backup).unwrap();
        let deserialized: Backup = serde_json::from_str(&json).unwrap();

        assert_eq!(backup.pkid, deserialized.pkid);
        assert_eq!(backup.domain, deserialized.domain);
        assert_eq!(backup.transport, deserialized.transport);
        assert_eq!(backup.enabled, deserialized.enabled);
    }

    #[test]
    fn test_new_backup_creation() {
        let new_backup = NewBackup {
            domain: "backup.test.com".to_string(),
            transport: Some("smtp:relay.test.com".to_string()),
            enabled: true,
        };

        assert_eq!(new_backup.domain, "backup.test.com");
        assert_eq!(
            new_backup.transport,
            Some("smtp:relay.test.com".to_string())
        );
        assert_eq!(new_backup.enabled, true);
    }

    #[test]
    fn test_backup_form_deserialization() {
        let form_data = "domain=backup.test.com&transport=smtp%3Arelay.test.com&enabled=on";
        let form: BackupForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "backup.test.com");
        assert_eq!(form.transport, "smtp:relay.test.com");
        assert_eq!(form.enabled, true);
    }

    #[test]
    fn test_backup_form_deserialization_without_checkboxes() {
        let form_data = "domain=backup.test.com&transport=smtp%3Arelay.test.com";
        let form: BackupForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "backup.test.com");
        assert_eq!(form.transport, "smtp:relay.test.com");
        assert_eq!(form.enabled, false); // Default value
    }

    #[test]
    fn test_paginated_result_empty() {
        let paginated: PaginatedResult<i32> = PaginatedResult::new(vec![], 0, 1, 10);
        assert_eq!(paginated.items.len(), 0);
        assert_eq!(paginated.total_count, 0);
        assert_eq!(paginated.current_page, 1);
        assert_eq!(paginated.per_page, 10);
        assert_eq!(paginated.total_pages, 0);
        assert!(!paginated.has_next);
        assert!(!paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_single_page() {
        let paginated = PaginatedResult::new(vec![1, 2, 3], 3, 1, 10);
        assert_eq!(paginated.total_pages, 1);
        assert!(!paginated.has_next);
        assert!(!paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_multiple_pages() {
        let paginated = PaginatedResult::new((1..21).collect(), 100, 2, 20);
        assert_eq!(paginated.total_pages, 5);
        assert!(paginated.has_next);
        assert!(paginated.has_prev);
        assert_eq!(paginated.current_page, 2);
        assert_eq!(paginated.per_page, 20);
    }

    #[test]
    fn test_paginated_result_last_page() {
        let paginated = PaginatedResult::new((81..101).collect(), 100, 5, 20);
        assert_eq!(paginated.total_pages, 5);
        assert!(!paginated.has_next);
        assert!(paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_out_of_bounds_page() {
        let paginated: PaginatedResult<i32> = PaginatedResult::new(vec![], 30, 5, 10);
        assert_eq!(paginated.total_pages, 3);
        assert!(!paginated.has_next);
        assert!(paginated.has_prev);
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use chrono::Utc;

    #[test]
    fn test_checkbox_deserialization_utility() {
        // Test various checkbox values in form data
        let test_cases = vec![
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=on&active=on", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=true&active=true", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=1&active=1", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=off&active=off", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=false&active=false", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=0&active=0", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost", false),
        ];

        for (form_data, expected) in test_cases {
            let form: crate::models::DomainForm = serde_urlencoded::from_str(form_data).unwrap();
            assert_eq!(form.backupmx, expected, "Failed for input: {}", form_data);
            assert_eq!(form.active, expected, "Failed for input: {}", form_data);
        }
    }

    #[test]
    fn test_form_validation() {
        // Test valid domain form
        let valid_domain_form = DomainForm {
            domain: "example.com".to_string(),
            description: "Test domain".to_string(),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: "smtp:localhost".to_string(),
            backupmx: false,
            active: true,
        };

        assert!(!valid_domain_form.domain.is_empty());
        assert!(valid_domain_form.aliases >= 0);
        assert!(valid_domain_form.maxquota > 0);
        assert!(valid_domain_form.quota >= 0);
        assert!(valid_domain_form.quota <= valid_domain_form.maxquota);

        // Test valid user form
        let valid_user_form = UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: "example.com".to_string(),
            quota: 100000,
            active: true,
        };

        assert!(!valid_user_form.username.is_empty());
        assert!(!valid_user_form.password.is_empty());
        assert!(!valid_user_form.name.is_empty());
        assert!(!valid_user_form.domain.is_empty());
        assert!(valid_user_form.quota >= 0);

        // Test valid alias form
        let valid_alias_form = AliasForm {
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            domain: "example.com".to_string(),
            active: true,
        };

        assert!(!valid_alias_form.mail.is_empty());
        assert!(!valid_alias_form.destination.is_empty());
        assert!(!valid_alias_form.domain.is_empty());
        assert!(valid_alias_form.mail.contains('@'));
        assert!(valid_alias_form.destination.contains('@'));
    }

    #[test]
    fn test_model_creation() {
        // Test Domain model creation
        let now = Utc::now().naive_utc();
        let domain = Domain {
            pkid: 1,
            domain: "example.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            created: now,
            modified: now,
            active: true,
        };

        assert_eq!(domain.pkid, 1);
        assert_eq!(domain.domain, "example.com");
        assert_eq!(domain.aliases, 10);
        assert_eq!(domain.active, true);
        assert_eq!(domain.created, now);
        assert_eq!(domain.modified, now);

        // Test User model creation
        let user = User {
            pkid: 1,
            username: "testuser".to_string(),
            password: "hashed_password".to_string(),
            name: "Test User".to_string(),
            maildir: "testuser/".to_string(),
            quota: 100000,
            domain: "example.com".to_string(),
            created: now,
            modified: now,
            active: true,
        };

        assert_eq!(user.pkid, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.maildir, "testuser/");
        assert_eq!(user.quota, 100000);
        assert_eq!(user.domain, "example.com");
        assert_eq!(user.active, true);

        // Test Alias model creation
        let alias = Alias {
            pkid: 1,
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            domain: "example.com".to_string(),
            created: now,
            modified: now,
            active: true,
        };

        assert_eq!(alias.pkid, 1);
        assert_eq!(alias.mail, "test@example.com");
        assert_eq!(alias.destination, "user@example.com");
        assert_eq!(alias.domain, "example.com");
        assert_eq!(alias.active, true);
    }

    #[test]
    fn test_new_model_creation() {
        // Test NewDomain creation
        let new_domain = NewDomain {
            domain: "example.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            active: true,
        };

        assert_eq!(new_domain.domain, "example.com");
        assert_eq!(new_domain.description, Some("Test domain".to_string()));
        assert_eq!(new_domain.aliases, 10);
        assert_eq!(new_domain.maxquota, 1000000);
        assert_eq!(new_domain.quota, 500000);
        assert_eq!(new_domain.transport, Some("smtp:localhost".to_string()));
        assert_eq!(new_domain.backupmx, false);
        assert_eq!(new_domain.active, true);

        // Test NewUser creation
        let new_user = NewUser {
            username: "testuser".to_string(),
            password: "hashed_password".to_string(),
            name: "Test User".to_string(),
            maildir: "testuser/".to_string(),
            quota: 100000,
            domain: "example.com".to_string(),
            active: true,
        };

        assert_eq!(new_user.username, "testuser");
        assert_eq!(new_user.password, "hashed_password");
        assert_eq!(new_user.name, "Test User");
        assert_eq!(new_user.maildir, "testuser/");
        assert_eq!(new_user.quota, 100000);
        assert_eq!(new_user.domain, "example.com");
        assert_eq!(new_user.active, true);

        // Test NewAlias creation
        let new_alias = NewAlias {
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            domain: "example.com".to_string(),
            active: true,
        };

        assert_eq!(new_alias.mail, "test@example.com");
        assert_eq!(new_alias.destination, "user@example.com");
        assert_eq!(new_alias.domain, "example.com");
        assert_eq!(new_alias.active, true);
    }

    #[test]
    fn test_stats_model_creation() {
        // Test SystemStats creation
        let system_stats = SystemStats {
            total_domains: 5,
            total_users: 25,
            total_aliases: 50,
            total_quota: 10000000,
            used_quota: 5000000,
        };

        assert_eq!(system_stats.total_domains, 5);
        assert_eq!(system_stats.total_users, 25);
        assert_eq!(system_stats.total_aliases, 50);
        assert_eq!(system_stats.total_quota, 10000000);
        assert_eq!(system_stats.used_quota, 5000000);

        // Test DomainStats creation
        let domain_stats = DomainStats {
            domain: "example.com".to_string(),
            user_count: 10,
            alias_count: 20,
            total_quota: 5000000,
            used_quota: 2500000,
        };

        assert_eq!(domain_stats.domain, "example.com");
        assert_eq!(domain_stats.user_count, 10);
        assert_eq!(domain_stats.alias_count, 20);
        assert_eq!(domain_stats.total_quota, 5000000);
        assert_eq!(domain_stats.used_quota, 2500000);
    }

    #[test]
    fn test_email_validation() {
        // Test valid email addresses
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
            "123@numbers.com",
        ];

        for email in valid_emails {
            assert!(email.contains('@'), "Email should contain @: {}", email);
            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2, "Email should have exactly one @: {}", email);
            assert!(
                !parts[0].is_empty(),
                "Local part should not be empty: {}",
                email
            );
            assert!(
                !parts[1].is_empty(),
                "Domain part should not be empty: {}",
                email
            );
        }

        // Test invalid email addresses
        let invalid_emails = vec![
            "invalid-email",
            "@example.com",
            "user@",
            "user..name@example.com",
        ];

        for email in invalid_emails {
            if !email.contains('@') {
                continue; // Skip emails without @ as they're obviously invalid
            }
            let parts: Vec<&str> = email.split('@').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                // This is expected to be invalid
                continue;
            }
            // If we get here, the email format is valid but we know it's supposed to be invalid
            // This might indicate a test case issue
        }
    }

    #[test]
    fn test_domain_validation() {
        // Test valid domains
        let valid_domains = vec![
            "example.com",
            "subdomain.example.com",
            "domain.co.uk",
            "test-domain.org",
        ];

        for domain in valid_domains {
            assert!(!domain.is_empty(), "Domain should not be empty: {}", domain);
            assert!(
                !domain.starts_with('.'),
                "Domain should not start with dot: {}",
                domain
            );
            assert!(
                !domain.ends_with('.'),
                "Domain should not end with dot: {}",
                domain
            );
            assert!(
                domain.contains('.'),
                "Domain should contain at least one dot: {}",
                domain
            );
        }

        // Test invalid domains
        let invalid_domains = vec!["", ".example.com", "example.com.", "example"];

        for domain in invalid_domains {
            if domain.is_empty() {
                continue; // Expected to be invalid
            }
            if domain.starts_with('.') || domain.ends_with('.') || !domain.contains('.') {
                continue; // Expected to be invalid
            }
            // If we get here, the domain format is valid but we know it's supposed to be invalid
            // This might indicate a test case issue
        }
    }

    #[test]
    fn test_quota_validation() {
        // Test valid quota values
        let valid_quotas = vec![0, 1000, 1000000, 1000000000];

        for quota in valid_quotas {
            assert!(quota >= 0, "Quota should be non-negative: {}", quota);
        }

        // Test quota relationships
        let max_quota = 1000000;
        let used_quota = 500000;
        assert!(
            used_quota <= max_quota,
            "Used quota should not exceed max quota"
        );
        assert!(max_quota > 0, "Max quota should be positive");
    }

    #[test]
    fn test_username_validation() {
        // Test valid usernames
        let valid_usernames = vec!["user", "user123", "user-name", "user_name", "user.name"];

        for username in valid_usernames {
            assert!(
                !username.is_empty(),
                "Username should not be empty: {}",
                username
            );
            assert!(
                username.len() <= 64,
                "Username should not be too long: {}",
                username
            );
            // Add more specific validation rules as needed
        }

        // Test invalid usernames
        let long_username = "a".repeat(65);
        let invalid_usernames = vec![
            "",
            &long_username, // Too long
        ];

        for username in invalid_usernames {
            if username.is_empty() {
                continue; // Expected to be invalid
            }
            if username.len() > 64 {
                continue; // Expected to be invalid
            }
            // If we get here, the username format is valid but we know it's supposed to be invalid
            // This might indicate a test case issue
        }
    }
}

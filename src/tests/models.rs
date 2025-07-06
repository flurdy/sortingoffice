#[cfg(test)]
mod tests {

    use crate::models::*;
    use serde_json;

    #[test]
    fn test_domain_serialization() {
        let domain = Domain {
            pkid: 1,
            domain: "example.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            active: true,
        };

        let json = serde_json::to_string(&domain).unwrap();
        let deserialized: Domain = serde_json::from_str(&json).unwrap();

        assert_eq!(domain.pkid, deserialized.pkid);
        assert_eq!(domain.domain, deserialized.domain);
        assert_eq!(domain.description, deserialized.description);
        assert_eq!(domain.aliases, deserialized.aliases);
        assert_eq!(domain.maxquota, deserialized.maxquota);
        assert_eq!(domain.quota, deserialized.quota);
        assert_eq!(domain.transport, deserialized.transport);
        assert_eq!(domain.backupmx, deserialized.backupmx);
        assert_eq!(domain.active, deserialized.active);
    }

    #[test]
    fn test_new_domain_creation() {
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 5,
            maxquota: 500000,
            quota: 250000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: true,
            active: true,
        };

        assert_eq!(new_domain.domain, "test.com");
        assert_eq!(new_domain.aliases, 5);
        assert_eq!(new_domain.backupmx, true);
        assert_eq!(new_domain.active, true);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            pkid: 1,
            username: "testuser".to_string(),
            password: "hashed_password".to_string(),
            name: "Test User".to_string(),
            maildir: "testuser/".to_string(),
            quota: 100000,
            domain: "example.com".to_string(),
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            active: true,
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user.pkid, deserialized.pkid);
        assert_eq!(user.username, deserialized.username);
        assert_eq!(user.name, deserialized.name);
        assert_eq!(user.maildir, deserialized.maildir);
        assert_eq!(user.quota, deserialized.quota);
        assert_eq!(user.domain, deserialized.domain);
        assert_eq!(user.active, deserialized.active);
    }

    #[test]
    fn test_alias_serialization() {
        let alias = Alias {
            pkid: 1,
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            domain: "example.com".to_string(),
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
            active: true,
        };

        let json = serde_json::to_string(&alias).unwrap();
        let deserialized: Alias = serde_json::from_str(&json).unwrap();

        assert_eq!(alias.pkid, deserialized.pkid);
        assert_eq!(alias.mail, deserialized.mail);
        assert_eq!(alias.destination, deserialized.destination);
        assert_eq!(alias.domain, deserialized.domain);
        assert_eq!(alias.active, deserialized.active);
    }

    #[test]
    fn test_domain_form_deserialization() {
        let form_data = "domain=test.com&description=Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost&backupmx=on&active=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.description, "Test Domain");
        assert_eq!(form.aliases, 10);
        assert_eq!(form.maxquota, 1000000);
        assert_eq!(form.quota, 500000);
        assert_eq!(form.transport, "smtp:localhost");
        assert_eq!(form.backupmx, true);
        assert_eq!(form.active, true);
    }

    #[test]
    fn test_domain_form_deserialization_without_checkboxes() {
        let form_data = "domain=test.com&description=Test+Domain&aliases=10&maxquota=1000000&quota=500000&transport=smtp%3Alocalhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.description, "Test Domain");
        assert_eq!(form.aliases, 10);
        assert_eq!(form.maxquota, 1000000);
        assert_eq!(form.quota, 500000);
        assert_eq!(form.transport, "smtp:localhost");
        assert_eq!(form.backupmx, false); // Default value
        assert_eq!(form.active, false); // Default value
    }

    #[test]
    fn test_user_form_deserialization() {
        let form_data = "username=testuser&password=secret123&name=Test+User&domain=example.com&quota=100000&active=on";
        let form: UserForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.username, "testuser");
        assert_eq!(form.password, "secret123");
        assert_eq!(form.name, "Test User");
        assert_eq!(form.domain, "example.com");
        assert_eq!(form.quota, 100000);
        assert_eq!(form.active, true);
    }

    #[test]
    fn test_alias_form_deserialization() {
        let form_data =
            "mail=test%40example.com&destination=user%40example.com&domain=example.com&active=on";
        let form: AliasForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.mail, "test@example.com");
        assert_eq!(form.destination, "user@example.com");
        assert_eq!(form.domain, "example.com");
        assert_eq!(form.active, true);
    }

    #[test]
    fn test_checkbox_deserialization() {
        // Test form data deserialization with "on" value
        let form_data = "domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=on&active=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.backupmx, true);
        assert_eq!(form.active, true);

        // Test form data deserialization with "true" value
        let form_data = "domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=true&active=true";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.backupmx, true);
        assert_eq!(form.active, true);

        // Test form data deserialization with "1" value
        let form_data = "domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=1&active=1";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.backupmx, true);
        assert_eq!(form.active, true);

        // Test form data deserialization with missing values
        let form_data = "domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.backupmx, false);
        assert_eq!(form.active, false);

        // Test form data deserialization with "off" value
        let form_data = "domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&backupmx=off&active=off";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert_eq!(form.backupmx, false);
        assert_eq!(form.active, false);
    }

    #[test]
    fn test_system_stats_serialization() {
        let stats = SystemStats {
            total_domains: 5,
            total_users: 25,
            total_aliases: 50,
            total_quota: 10000000,
            used_quota: 5000000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SystemStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_domains, deserialized.total_domains);
        assert_eq!(stats.total_users, deserialized.total_users);
        assert_eq!(stats.total_aliases, deserialized.total_aliases);
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
}

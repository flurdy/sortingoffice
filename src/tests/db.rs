#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::*;
    use crate::tests::common::{cleanup_test_db, setup_test_db};

    #[tokio::test]
    async fn test_domain_crud_operations() {
        let pool = setup_test_db();

        // Test create domain
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            enabled: true,
        };

        let created_domain = db::create_domain(&pool, new_domain).unwrap();
        assert_eq!(created_domain.domain, "test.com");
        assert_eq!(created_domain.aliases, 10);
        assert_eq!(created_domain.enabled, true);

        // Test get domain
        let retrieved_domain = db::get_domain(&pool, created_domain.pkid).unwrap();
        assert_eq!(retrieved_domain.pkid, created_domain.pkid);
        assert_eq!(retrieved_domain.domain, "test.com");

        // Test get domain by name
        let domain_by_name = db::get_domain_by_name(&pool, "test.com").unwrap();
        assert_eq!(domain_by_name.pkid, created_domain.pkid);

        // Test update domain
        let form_data = DomainForm {
            domain: "updated.com".to_string(),
            description: "Updated description".to_string(),
            aliases: 20,
            maxquota: 2000000,
            quota: 1000000,
            transport: "smtp:updated".to_string(),
            backupmx: true,
            enabled: false,
        };

        let updated_domain = db::update_domain(&pool, created_domain.pkid, form_data).unwrap();
        assert_eq!(updated_domain.domain, "updated.com");
        assert_eq!(updated_domain.aliases, 20);
        assert_eq!(updated_domain.enabled, false);

        // Test get all domains
        let all_domains = db::get_domains(&pool).unwrap();
        assert!(!all_domains.is_empty());
        assert!(all_domains.iter().any(|d| d.pkid == created_domain.pkid));

        // Test delete domain
        let deleted_count = db::delete_domain(&pool, created_domain.pkid).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify domain is deleted
        let deleted_domain = db::get_domain(&pool, created_domain.pkid);
        assert!(deleted_domain.is_err());

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_user_crud_operations() {
        let pool = setup_test_db();

        // Create a domain first
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create user
        let user_form = UserForm {
            id: "testuser@test.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            enabled: true,
        };

        let created_user = db::create_user(&pool, user_form).unwrap();
        assert_eq!(created_user.id, "testuser@test.com");
        assert_eq!(created_user.name, "Test User");
        assert_eq!(created_user.domain, domain.domain);
        assert_eq!(created_user.enabled, true);
        assert!(created_user.crypt.starts_with("$2b$")); // bcrypt hash

        // Test get user
        let retrieved_user = db::get_user(&pool, created_user.pkid).unwrap();
        assert_eq!(retrieved_user.pkid, created_user.pkid);
        assert_eq!(retrieved_user.id, "testuser@test.com");

        // Test get user by id
        let user_by_id = db::get_user_by_id(&pool, "testuser@test.com").unwrap();
        assert_eq!(user_by_id.pkid, created_user.pkid);

        // Test update user with new password
        let update_form = UserForm {
            id: "updateduser@test.com".to_string(),
            password: "newpassword123".to_string(),
            name: "Updated User".to_string(),
            domain: domain.domain.clone(),
            quota: 200000,
            enabled: false,
        };

        let updated_user = db::update_user(&pool, created_user.pkid, update_form).unwrap();
        assert_eq!(updated_user.id, "updateduser@test.com");
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.enabled, false);

        // Test update user without password
        let update_form_no_password = UserForm {
            id: "finaluser@test.com".to_string(),
            password: "".to_string(), // Empty password
            name: "Final User".to_string(),
            domain: domain.domain.clone(),
            quota: 300000,
            enabled: true,
        };

        let final_user =
            db::update_user(&pool, created_user.pkid, update_form_no_password).unwrap();
        assert_eq!(final_user.id, "finaluser@test.com");
        assert_eq!(final_user.name, "Final User");
        assert_eq!(final_user.enabled, true);

        // Test get all users
        let all_users = db::get_users(&pool).unwrap();
        assert!(!all_users.is_empty());
        assert!(all_users.iter().any(|u| u.pkid == created_user.pkid));

        // Test delete user
        let deleted_count = db::delete_user(&pool, created_user.pkid).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify user is deleted
        let deleted_user = db::get_user(&pool, created_user.pkid);
        assert!(deleted_user.is_err());

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_alias_crud_operations() {
        let pool = setup_test_db();

        // Create a domain first
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create alias
        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };

        let created_alias = db::create_alias(&pool, alias_form).unwrap();
        assert_eq!(created_alias.mail, "test@test.com");
        assert_eq!(created_alias.destination, "user@test.com");
        assert_eq!(created_alias.domain, domain.domain);
        assert_eq!(created_alias.enabled, true);

        // Test get alias
        let retrieved_alias = db::get_alias(&pool, created_alias.pkid).unwrap();
        assert_eq!(retrieved_alias.pkid, created_alias.pkid);
        assert_eq!(retrieved_alias.mail, "test@test.com");

        // Test update alias
        let update_form = AliasForm {
            mail: "updated@test.com".to_string(),
            destination: "updated@test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: false,
        };

        let updated_alias = db::update_alias(&pool, created_alias.pkid, update_form).unwrap();
        assert_eq!(updated_alias.mail, "updated@test.com");
        assert_eq!(updated_alias.destination, "updated@test.com");
        assert_eq!(updated_alias.enabled, false);

        // Test get all aliases
        let all_aliases = db::get_aliases(&pool).unwrap();
        assert!(!all_aliases.is_empty());
        assert!(all_aliases.iter().any(|a| a.pkid == created_alias.pkid));

        // Test delete alias
        let deleted_count = db::delete_alias(&pool, created_alias.pkid).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify alias is deleted
        let deleted_alias = db::get_alias(&pool, created_alias.pkid);
        assert!(deleted_alias.is_err());

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_toggle_operations() {
        let pool = setup_test_db();

        // Create test data
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_form = UserForm {
            id: "testuser@test.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            enabled: true,
        };
        let user = db::create_user(&pool, user_form).unwrap();

        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let alias = db::create_alias(&pool, alias_form).unwrap();

        // Test toggle domain active
        let toggled_domain = db::toggle_domain_enabled(&pool, domain.pkid).unwrap();
        assert_eq!(toggled_domain.enabled, false);

        let toggled_domain_again = db::toggle_domain_enabled(&pool, domain.pkid).unwrap();
        assert_eq!(toggled_domain_again.enabled, true);

        // Test toggle user enabled
        let toggled_user = db::toggle_user_enabled(&pool, user.pkid).unwrap();
        assert_eq!(toggled_user.enabled, false);

        let toggled_user_again = db::toggle_user_enabled(&pool, user.pkid).unwrap();
        assert_eq!(toggled_user_again.enabled, true);

        // Test toggle alias enabled
        let toggled_alias = db::toggle_alias_enabled(&pool, alias.pkid).unwrap();
        assert_eq!(toggled_alias.enabled, false);

        let toggled_alias_again = db::toggle_alias_enabled(&pool, alias.pkid).unwrap();
        assert_eq!(toggled_alias_again.enabled, true);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_stats_operations() {
        let pool = setup_test_db();

        // Create test data
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            description: Some("Test domain".to_string()),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: Some("smtp:localhost".to_string()),
            backupmx: false,
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_form = UserForm {
            id: "testuser@test.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            enabled: true,
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            enabled: true,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        // Test system stats
        let system_stats = db::get_system_stats(&pool).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);
        assert!(system_stats.total_quota > 0);
        assert!(system_stats.used_quota >= 0);

        // Test domain stats
        let domain_stats = db::get_domain_stats(&pool).unwrap();
        assert!(!domain_stats.is_empty());
        let test_domain_stats = domain_stats
            .iter()
            .find(|s| s.domain == "test.com")
            .unwrap();
        assert_eq!(test_domain_stats.user_count, 1);
        assert_eq!(test_domain_stats.alias_count, 1);
        assert!(test_domain_stats.total_quota > 0);
        assert!(test_domain_stats.used_quota >= 0);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = setup_test_db();

        // Test getting non-existent domain
        let non_existent_domain = db::get_domain(&pool, 999);
        assert!(non_existent_domain.is_err());

        // Test getting non-existent user
        let non_existent_user = db::get_user(&pool, 999);
        assert!(non_existent_user.is_err());

        // Test getting non-existent alias
        let non_existent_alias = db::get_alias(&pool, 999);
        assert!(non_existent_alias.is_err());

        // Test updating non-existent domain
        let form_data = DomainForm {
            domain: "test.com".to_string(),
            description: "Test".to_string(),
            aliases: 10,
            maxquota: 1000000,
            quota: 500000,
            transport: "smtp:localhost".to_string(),
            backupmx: false,
            enabled: true,
        };
        let update_result = db::update_domain(&pool, 999, form_data);
        assert!(update_result.is_err());

        cleanup_test_db(&pool);
    }
}

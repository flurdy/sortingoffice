#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::db;
    use crate::tests::common::{setup_test_db, cleanup_test_db};

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
            active: true,
        };

        let created_domain = db::create_domain(&pool, new_domain).unwrap();
        assert_eq!(created_domain.domain, "test.com");
        assert_eq!(created_domain.aliases, 10);
        assert_eq!(created_domain.active, true);

        // Test get domain
        let retrieved_domain = db::get_domain(&pool, created_domain.id).unwrap();
        assert_eq!(retrieved_domain.id, created_domain.id);
        assert_eq!(retrieved_domain.domain, "test.com");

        // Test get domain by name
        let domain_by_name = db::get_domain_by_name(&pool, "test.com").unwrap();
        assert_eq!(domain_by_name.id, created_domain.id);

        // Test update domain
        let form_data = DomainForm {
            domain: "updated.com".to_string(),
            description: "Updated description".to_string(),
            aliases: 20,
            maxquota: 2000000,
            quota: 1000000,
            transport: "smtp:updated".to_string(),
            backupmx: true,
            active: false,
        };

        let updated_domain = db::update_domain(&pool, created_domain.id, form_data).unwrap();
        assert_eq!(updated_domain.domain, "updated.com");
        assert_eq!(updated_domain.aliases, 20);
        assert_eq!(updated_domain.active, false);

        // Test get all domains
        let all_domains = db::get_domains(&pool).unwrap();
        assert!(!all_domains.is_empty());
        assert!(all_domains.iter().any(|d| d.id == created_domain.id));

        // Test delete domain
        let deleted_count = db::delete_domain(&pool, created_domain.id).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify domain is deleted
        let deleted_domain = db::get_domain(&pool, created_domain.id);
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
            active: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create user
        let user_form = UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            active: true,
        };

        let created_user = db::create_user(&pool, user_form).unwrap();
        assert_eq!(created_user.username, "testuser");
        assert_eq!(created_user.name, "Test User");
        assert_eq!(created_user.domain, domain.domain);
        assert_eq!(created_user.active, true);
        assert!(created_user.password.starts_with("$2b$")); // bcrypt hash

        // Test get user
        let retrieved_user = db::get_user(&pool, created_user.id).unwrap();
        assert_eq!(retrieved_user.id, created_user.id);
        assert_eq!(retrieved_user.username, "testuser");

        // Test get user by username
        let user_by_username = db::get_user_by_username(&pool, "testuser").unwrap();
        assert_eq!(user_by_username.id, created_user.id);

        // Test update user with new password
        let update_form = UserForm {
            username: "updateduser".to_string(),
            password: "newpassword123".to_string(),
            name: "Updated User".to_string(),
            domain: domain.domain.clone(),
            quota: 200000,
            active: false,
        };

        let updated_user = db::update_user(&pool, created_user.id, update_form).unwrap();
        assert_eq!(updated_user.username, "updateduser");
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.active, false);

        // Test update user without password
        let update_form_no_password = UserForm {
            username: "finaluser".to_string(),
            password: "".to_string(), // Empty password
            name: "Final User".to_string(),
            domain: domain.domain.clone(),
            quota: 300000,
            active: true,
        };

        let final_user = db::update_user(&pool, created_user.id, update_form_no_password).unwrap();
        assert_eq!(final_user.username, "finaluser");
        assert_eq!(final_user.name, "Final User");
        assert_eq!(final_user.active, true);

        // Test get all users
        let all_users = db::get_users(&pool).unwrap();
        assert!(!all_users.is_empty());
        assert!(all_users.iter().any(|u| u.id == created_user.id));

        // Test delete user
        let deleted_count = db::delete_user(&pool, created_user.id).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify user is deleted
        let deleted_user = db::get_user(&pool, created_user.id);
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
            active: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create alias
        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            active: true,
        };

        let created_alias = db::create_alias(&pool, alias_form).unwrap();
        assert_eq!(created_alias.mail, "test@test.com");
        assert_eq!(created_alias.destination, "user@test.com");
        assert_eq!(created_alias.domain, domain.domain);
        assert_eq!(created_alias.active, true);

        // Test get alias
        let retrieved_alias = db::get_alias(&pool, created_alias.id).unwrap();
        assert_eq!(retrieved_alias.id, created_alias.id);
        assert_eq!(retrieved_alias.mail, "test@test.com");

        // Test update alias
        let update_form = AliasForm {
            mail: "updated@test.com".to_string(),
            destination: "updated@test.com".to_string(),
            domain: domain.domain.clone(),
            active: false,
        };

        let updated_alias = db::update_alias(&pool, created_alias.id, update_form).unwrap();
        assert_eq!(updated_alias.mail, "updated@test.com");
        assert_eq!(updated_alias.destination, "updated@test.com");
        assert_eq!(updated_alias.active, false);

        // Test get all aliases
        let all_aliases = db::get_aliases(&pool).unwrap();
        assert!(!all_aliases.is_empty());
        assert!(all_aliases.iter().any(|a| a.id == created_alias.id));

        // Test delete alias
        let deleted_count = db::delete_alias(&pool, created_alias.id).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify alias is deleted
        let deleted_alias = db::get_alias(&pool, created_alias.id);
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
            active: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_form = UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            active: true,
        };
        let user = db::create_user(&pool, user_form).unwrap();

        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            active: true,
        };
        let alias = db::create_alias(&pool, alias_form).unwrap();

        // Test toggle domain active
        let toggled_domain = db::toggle_domain_active(&pool, domain.id).unwrap();
        assert_eq!(toggled_domain.active, false);

        let toggled_domain_again = db::toggle_domain_active(&pool, domain.id).unwrap();
        assert_eq!(toggled_domain_again.active, true);

        // Test toggle user active
        let toggled_user = db::toggle_user_active(&pool, user.id).unwrap();
        assert_eq!(toggled_user.active, false);

        let toggled_user_again = db::toggle_user_active(&pool, user.id).unwrap();
        assert_eq!(toggled_user_again.active, true);

        // Test toggle alias active
        let toggled_alias = db::toggle_alias_active(&pool, alias.id).unwrap();
        assert_eq!(toggled_alias.active, false);

        let toggled_alias_again = db::toggle_alias_active(&pool, alias.id).unwrap();
        assert_eq!(toggled_alias_again.active, true);

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
            active: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_form = UserForm {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            domain: domain.domain.clone(),
            quota: 100000,
            active: true,
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let alias_form = AliasForm {
            mail: "test@test.com".to_string(),
            destination: "user@test.com".to_string(),
            domain: domain.domain.clone(),
            active: true,
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
        let test_domain_stats = domain_stats.iter().find(|s| s.domain == "test.com").unwrap();
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
            active: true,
        };
        let update_result = db::update_domain(&pool, 999, form_data);
        assert!(update_result.is_err());

        cleanup_test_db(&pool);
    }
} 

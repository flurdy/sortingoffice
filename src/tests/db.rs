#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::*;
    use crate::tests::common::{cleanup_test_db, setup_test_db};
    use diesel::prelude::*;
    use crate::schema::{aliases, domains};

    #[tokio::test]
    async fn test_domain_crud_operations() {
        let pool = setup_test_db();

        // Test create domain
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };

        let created_domain = db::create_domain(&pool, new_domain).unwrap();
        assert_eq!(created_domain.domain, domain_name);
        assert_eq!(created_domain.enabled, true);

        // Test get domain
        let retrieved_domain = db::get_domain(&pool, created_domain.pkid).unwrap();
        assert_eq!(retrieved_domain.pkid, created_domain.pkid);
        assert_eq!(retrieved_domain.domain, domain_name);

        // Test get domain by name
        let domain_by_name = db::get_domain_by_name(&pool, &domain_name).unwrap();
        assert_eq!(domain_by_name.pkid, created_domain.pkid);

        // Test update domain
        let updated_domain_name = format!("updated-{}.com", unique_id);
        let form_data = DomainForm {
            domain: updated_domain_name.clone(),
            transport: "smtp:updated".to_string(),
            enabled: false,
        };

        let updated_domain = db::update_domain(&pool, created_domain.pkid, form_data).unwrap();
        assert_eq!(updated_domain.domain, updated_domain_name);
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
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create user
        let user_id = format!("testuser@{}", domain_name);
        let user_form = UserForm {
            id: user_id.clone(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
        };

        let created_user = db::create_user(&pool, user_form).unwrap();
        assert_eq!(created_user.id, user_id);
        assert_eq!(created_user.name, "Test User");
        assert_eq!(created_user.enabled, true);
        assert!(created_user.crypt.starts_with("$2b$")); // bcrypt hash

        // Test get user
        let retrieved_user = db::get_user(&pool, created_user.pkid).unwrap();
        assert_eq!(retrieved_user.pkid, created_user.pkid);
        assert_eq!(retrieved_user.id, user_id);

        // Test get user by id
        let user_by_id = db::get_user_by_id(&pool, &user_id).unwrap();
        assert_eq!(user_by_id.pkid, created_user.pkid);

        // Test update user with new password
        let updated_user_id = format!("updateduser@{}", domain_name);
        let update_form = UserForm {
            id: updated_user_id.clone(),
            password: "newpassword123".to_string(),
            name: "Updated User".to_string(),
            enabled: false,
        };

        let updated_user = db::update_user(&pool, created_user.pkid, update_form).unwrap();
        assert_eq!(updated_user.id, updated_user_id);
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.enabled, false);

        // Test update user without password
        let final_user_id = format!("finaluser@{}", domain_name);
        let update_form_no_password = UserForm {
            id: final_user_id.clone(),
            password: "".to_string(), // Empty password
            name: "Final User".to_string(),
            enabled: true,
        };

        let final_user =
            db::update_user(&pool, created_user.pkid, update_form_no_password).unwrap();
        assert_eq!(final_user.id, final_user_id);
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
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        // Test create alias
        let alias_mail = format!("test@{}", domain_name);
        let alias_destination = format!("user@{}", domain_name);
        let alias_form = AliasForm {
            mail: alias_mail.clone(),
            destination: alias_destination.clone(),
            enabled: true,
        };

        let created_alias = db::create_alias(&pool, alias_form).unwrap();
        assert_eq!(created_alias.mail, alias_mail);
        assert_eq!(created_alias.destination, alias_destination);
        assert_eq!(created_alias.enabled, true);

        // Test get alias
        let retrieved_alias = db::get_alias(&pool, created_alias.pkid).unwrap();
        assert_eq!(retrieved_alias.pkid, created_alias.pkid);
        assert_eq!(retrieved_alias.mail, alias_mail);

        // Test update alias
        let updated_mail = format!("updated@{}", domain_name);
        let updated_destination = format!("updated@{}", domain_name);
        let update_form = AliasForm {
            mail: updated_mail.clone(),
            destination: updated_destination.clone(),
            enabled: false,
        };

        let updated_alias = db::update_alias(&pool, created_alias.pkid, update_form).unwrap();
        assert_eq!(updated_alias.mail, updated_mail);
        assert_eq!(updated_alias.destination, updated_destination);
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
    async fn test_backup_crud_operations() {
        let pool = setup_test_db();

        // Test create backup
        let unique_id = crate::tests::common::unique_test_id();
        let backup_domain = format!("backup-{}.com", unique_id);
        let new_backup = NewBackup {
            domain: backup_domain.clone(),
            transport: Some("smtp:relay.test.com".to_string()),
            enabled: true,
        };

        let created_backup = db::create_backup(&pool, new_backup).unwrap();
        assert_eq!(created_backup.domain, backup_domain);
        assert_eq!(created_backup.transport, Some("smtp:relay.test.com".to_string()));
        assert_eq!(created_backup.enabled, true);

        // Test get backup
        let retrieved_backup = db::get_backup(&pool, created_backup.pkid).unwrap();
        assert_eq!(retrieved_backup.pkid, created_backup.pkid);
        assert_eq!(retrieved_backup.domain, backup_domain);

        // Test update backup
        let updated_backup_domain = format!("updated-backup-{}.com", unique_id);
        let update_form = BackupForm {
            domain: updated_backup_domain.clone(),
            transport: "smtp:updated-relay.test.com".to_string(),
            enabled: false,
        };

        let updated_backup = db::update_backup(&pool, created_backup.pkid, update_form).unwrap();
        assert_eq!(updated_backup.domain, updated_backup_domain);
        assert_eq!(updated_backup.transport, Some("smtp:updated-relay.test.com".to_string()));
        assert_eq!(updated_backup.enabled, false);

        // Test get all backups
        let all_backups = db::get_backups(&pool).unwrap();
        assert!(!all_backups.is_empty());
        assert!(all_backups.iter().any(|b| b.pkid == created_backup.pkid));

        // Test delete backup
        let deleted_count = db::delete_backup(&pool, created_backup.pkid).unwrap();
        assert_eq!(deleted_count, 1);

        // Verify backup is deleted
        let deleted_backup = db::get_backup(&pool, created_backup.pkid);
        assert!(deleted_backup.is_err());

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_toggle_operations() {
        let pool = setup_test_db();

        // Create test data
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_id = format!("testuser@{}", domain_name);
        let user_form = UserForm {
            id: user_id,
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
        };
        let user = db::create_user(&pool, user_form).unwrap();

        let alias_mail = format!("test@{}", domain_name);
        let alias_destination = format!("user@{}", domain_name);
        let alias_form = AliasForm {
            mail: alias_mail,
            destination: alias_destination,
            enabled: true,
        };
        let alias = db::create_alias(&pool, alias_form).unwrap();

        let backup_domain = format!("backup-{}.com", unique_id);
        let new_backup = NewBackup {
            domain: backup_domain,
            transport: Some("smtp:relay.test.com".to_string()),
            enabled: true,
        };
        let backup = db::create_backup(&pool, new_backup).unwrap();

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

        // Test toggle backup enabled
        let toggled_backup = db::toggle_backup_enabled(&pool, backup.pkid).unwrap();
        assert_eq!(toggled_backup.enabled, false);

        let toggled_backup_again = db::toggle_backup_enabled(&pool, backup.pkid).unwrap();
        assert_eq!(toggled_backup_again.enabled, true);

        cleanup_test_db(&pool);
    }

    #[tokio::test]
    async fn test_stats_operations() {
        let pool = setup_test_db();

        // Create test data
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();

        let user_id = format!("testuser@{}", domain_name);
        let user_form = UserForm {
            id: user_id,
            password: "password123".to_string(),
            name: "Test User".to_string(),
            enabled: true,
        };
        let _user = db::create_user(&pool, user_form).unwrap();

        let alias_mail = format!("test@{}", domain_name);
        let alias_destination = format!("user@{}", domain_name);
        let alias_form = AliasForm {
            mail: alias_mail,
            destination: alias_destination,
            enabled: true,
        };
        let _alias = db::create_alias(&pool, alias_form).unwrap();

        let backup_domain = format!("backup-{}.com", unique_id);
        let new_backup = NewBackup {
            domain: backup_domain,
            transport: Some("smtp:relay.test.com".to_string()),
            enabled: true,
        };
        let _backup = db::create_backup(&pool, new_backup).unwrap();

        // Test system stats
        let system_stats = db::get_system_stats(&pool).unwrap();
        assert_eq!(system_stats.total_domains, 1);
        assert_eq!(system_stats.total_users, 1);
        assert_eq!(system_stats.total_aliases, 1);
        assert_eq!(system_stats.total_backups, 1);

        // Test domain stats
        let domain_stats = db::get_domain_stats(&pool).unwrap();
        assert!(!domain_stats.is_empty());
        let test_domain_stats = domain_stats
            .iter()
            .find(|s| s.domain == domain_name)
            .unwrap();
        assert_eq!(test_domain_stats.user_count, 1);
        assert_eq!(test_domain_stats.alias_count, 1);

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

        // Test getting non-existent backup
        let non_existent_backup = db::get_backup(&pool, 999);
        assert!(non_existent_backup.is_err());

        // Test updating non-existent domain
        let unique_id = crate::tests::common::unique_test_id();
        let test_domain = format!("test-{}.com", unique_id);
        let form_data = DomainForm {
            domain: test_domain,
            transport: "smtp:localhost".to_string(),
            enabled: true,
        };
        let update_result = db::update_domain(&pool, 999, form_data);
        assert!(update_result.is_err());

        // Test updating non-existent backup
        let backup_domain = format!("backup-{}.com", unique_id);
        let backup_form_data = BackupForm {
            domain: backup_domain,
            transport: "smtp:relay.test.com".to_string(),
            enabled: true,
        };
        let backup_update_result = db::update_backup(&pool, 999, backup_form_data);
        assert!(backup_update_result.is_err());

        cleanup_test_db(&pool);
    }

    #[test]
    fn test_catch_all_report() {
        let pool = setup_test_db();
        
        // Clean up any existing test data
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
        
        // Create a test domain
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("virtual".to_string()),
            enabled: true,
        };
        let domain = db::create_domain(&pool, new_domain).unwrap();
        
        // Create a catch-all alias
        let catch_all_mail = format!("@{}", domain_name);
        let admin_email = format!("admin@{}", domain_name);
        let catch_all_alias = NewAlias {
            mail: catch_all_mail.clone(),
            destination: admin_email.clone(),
            enabled: true,
        };
        let _ = db::create_alias(&pool, AliasForm {
            mail: catch_all_alias.mail.clone(),
            destination: catch_all_alias.destination.clone(),
            enabled: catch_all_alias.enabled,
        });
        
        // Create some required aliases
        let required_aliases = vec![
            (format!("postmaster@{}", domain_name), admin_email.clone()),
            (format!("abuse@{}", domain_name), admin_email.clone()),
            (format!("webmaster@{}", domain_name), admin_email.clone()),
        ];
        
        for (mail, destination) in required_aliases {
            let _ = db::create_alias(&pool, AliasForm {
                mail: mail.to_string(),
                destination: destination.to_string(),
                enabled: true,
            });
        }
        
        // Test the catch-all report
        let reports = db::get_catch_all_report(&pool).unwrap();
        assert_eq!(reports.len(), 1);
        
        let report = &reports[0];
        assert_eq!(report.domain, domain_name);
        assert_eq!(report.catch_all_alias, catch_all_mail);
        assert_eq!(report.catch_all_destination, admin_email);
        assert_eq!(report.required_aliases.len(), 3);
        
        // Clean up
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
    }

    #[test]
    fn test_alias_report() {
        let pool = setup_test_db();
        
        // Clean up any existing test data
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
        
        // Create two test domains
        let unique_id = crate::tests::common::unique_test_id();
        let domain1_name = format!("catchall-{}.com", unique_id);
        let domain2_name = format!("missing-{}.com", unique_id);
        
        let domain1 = db::create_domain(&pool, NewDomain {
            domain: domain1_name.clone(),
            transport: Some("virtual".to_string()),
            enabled: true,
        }).unwrap();
        
        let domain2 = db::create_domain(&pool, NewDomain {
            domain: domain2_name.clone(),
            transport: Some("virtual".to_string()),
            enabled: true,
        }).unwrap();
        
        // Create a catch-all alias for domain1
        let _ = db::create_alias(&pool, AliasForm {
            mail: format!("@{}", domain1_name),
            destination: format!("admin@{}", domain1_name),
            enabled: true,
        });
        
        // Create some required aliases for domain1
        let _ = db::create_alias(&pool, AliasForm {
            mail: format!("postmaster@{}", domain1_name),
            destination: format!("admin@{}", domain1_name),
            enabled: true,
        });
        
        // Create only some required aliases for domain2 (missing some)
        let _ = db::create_alias(&pool, AliasForm {
            mail: format!("postmaster@{}", domain2_name),
            destination: format!("admin@{}", domain2_name),
            enabled: true,
        });
        
        // Test the enhanced alias report
        let report = db::get_alias_report(&pool).unwrap();
        
        // Check domains with catch-all
        assert_eq!(report.domains_with_catch_all.len(), 1);
        let catch_all_domain = &report.domains_with_catch_all[0];
        assert_eq!(catch_all_domain.domain, domain1_name);
        assert!(catch_all_domain.has_catch_all);
        assert_eq!(catch_all_domain.catch_all_alias, Some(format!("@{}", domain1_name)));
        assert_eq!(catch_all_domain.catch_all_destination, Some(format!("admin@{}", domain1_name)));
        assert_eq!(catch_all_domain.required_aliases.len(), 2); // catch-all + postmaster
        assert!(catch_all_domain.missing_required_aliases.len() > 0); // Should be missing some required aliases
        
        // Check domains without catch-all
        assert_eq!(report.domains_without_catch_all.len(), 1);
        let missing_domain = &report.domains_without_catch_all[0];
        assert_eq!(missing_domain.domain, domain2_name);
        assert!(!missing_domain.has_catch_all);
        assert_eq!(missing_domain.catch_all_alias, None);
        assert_eq!(missing_domain.catch_all_destination, None);
        assert_eq!(missing_domain.required_aliases.len(), 1); // Only postmaster
        assert!(missing_domain.missing_required_aliases.len() > 0); // Should be missing some required aliases
        
        // Clean up
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
    }

    #[test]
    fn test_configurable_required_aliases() {
        let pool = setup_test_db();
        
        // Clean up any existing test data
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
        
        // Create a test domain
        let unique_id = crate::tests::common::unique_test_id();
        let domain_name = format!("test-{}.com", unique_id);
        let new_domain = NewDomain {
            domain: domain_name.clone(),
            transport: Some("virtual".to_string()),
            enabled: true,
        };
        let _domain = db::create_domain(&pool, new_domain).unwrap();
        
        // Test with default configuration
        let report = db::get_alias_report(&pool).unwrap();
        assert_eq!(report.domains_without_catch_all.len(), 1);
        
        let domain_report = &report.domains_without_catch_all[0];
        assert_eq!(domain_report.domain, domain_name);
        assert!(!domain_report.has_catch_all);
        
        // The default config should include standard aliases like postmaster, abuse, etc.
        assert!(domain_report.missing_required_aliases.contains(&"postmaster".to_string()));
        assert!(domain_report.missing_required_aliases.contains(&"abuse".to_string()));
        assert!(domain_report.missing_required_aliases.contains(&"hostmaster".to_string()));
        
        // Test RequiredAliasConfig methods
        let mut config = RequiredAliasConfig::default();
        assert!(config.get_required_aliases().contains(&"postmaster".to_string()));
        assert!(config.get_common_aliases().contains(&"admin".to_string()));
        
        // Test adding and removing aliases
        config.add_required_alias("custom".to_string());
        assert!(config.get_required_aliases().contains(&"custom".to_string()));
        
        config.remove_required_alias("postmaster");
        assert!(!config.get_required_aliases().contains(&"postmaster".to_string()));
        
        // Test promote/demote functionality
        config.promote_to_required("admin");
        assert!(config.get_required_aliases().contains(&"admin".to_string()));
        assert!(!config.get_common_aliases().contains(&"admin".to_string()));
        
        config.demote_to_common("admin");
        assert!(!config.get_required_aliases().contains(&"admin".to_string()));
        assert!(config.get_common_aliases().contains(&"admin".to_string()));
        
        // Test from_strings method
        let config_from_strings = RequiredAliasConfig::from_strings("test1,test2", "test3,test4");
        assert_eq!(config_from_strings.get_required_aliases().len(), 2);
        assert_eq!(config_from_strings.get_common_aliases().len(), 2);
        assert!(config_from_strings.get_required_aliases().contains(&"test1".to_string()));
        assert!(config_from_strings.get_common_aliases().contains(&"test3".to_string()));
        
        // Clean up
        let _ = diesel::delete(aliases::table).execute(&mut pool.get().unwrap());
        let _ = diesel::delete(domains::table).execute(&mut pool.get().unwrap());
    }
}

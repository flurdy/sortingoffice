#[cfg(test)]
mod tests {
    use sortingoffice::test_helpers::common::TestDataManager;
    use sortingoffice::test_helpers::test_utils::TestData;
    use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

    #[tokio::test]
    async fn test_enhanced_test_data_generation() {
        // Test unique ID generation
        let domain1 = TestData::unique_domain();
        let domain2 = TestData::unique_domain();
        assert_ne!(domain1, domain2);
        assert!(domain1.contains("test-"));
        assert!(domain1.ends_with(".example.com"));

        let user1 = TestData::unique_user_id();
        let user2 = TestData::unique_user_id();
        assert_ne!(user1, user2);
        assert!(user1.contains("user-"));
        assert!(user1.contains("@example.com"));

        let alias1 = TestData::unique_alias();
        let alias2 = TestData::unique_alias();
        assert_ne!(alias1, alias2);
        assert!(alias1.contains("alias-"));
        assert!(alias1.contains("@example.com"));
    }

    #[tokio::test]
    async fn test_random_data_generation() {
        // Test random domain generation
        let random_domain1 = TestData::random_domain();
        let random_domain2 = TestData::random_domain();
        assert_ne!(random_domain1, random_domain2);
        assert!(random_domain1.contains("."));

        // Test random user generation
        let random_user1 = TestData::random_user();
        let random_user2 = TestData::random_user();
        assert_ne!(random_user1, random_user2);
        assert!(random_user1.contains("@"));

        // Test random alias generation
        let random_alias1 = TestData::random_alias();
        let random_alias2 = TestData::random_alias();
        assert_ne!(random_alias1, random_alias2);
        assert!(random_alias1.contains("@"));

        // Test random transport generation
        let transport1 = TestData::random_transport();
        let transport2 = TestData::random_transport();
        assert!(transport1.starts_with("smtp:"));
        assert!(transport2.starts_with("smtp:"));

        // Test random password generation
        let password1 = TestData::random_password();
        let password2 = TestData::random_password();
        assert!(!password1.is_empty());
        assert!(!password2.is_empty());

        // Test random name generation
        let name1 = TestData::random_name();
        let name2 = TestData::random_name();
        assert!(!name1.is_empty());
        assert!(!name2.is_empty());
    }

    #[tokio::test]
    async fn test_specific_data_generation() {
        // Test domain with specific suffix
        let domain = TestData::domain_with_suffix("test.org");
        assert!(domain.ends_with(".test.org"));

        // Test domain with specific length
        let short_domain = TestData::domain_with_length(5);
        assert!(short_domain.contains(".example.com"));
        let long_domain = TestData::domain_with_length(20);
        assert!(long_domain.contains(".example.com"));

        // Test user for specific domain
        let test_domain = "example.com";
        let user = TestData::user_for_domain(test_domain);
        assert!(user.ends_with("@example.com"));

        // Test alias for specific domain
        let alias = TestData::alias_for_domain(test_domain);
        assert!(alias.ends_with("@example.com"));

        // Test alias with specific name and domain
        let alias = TestData::alias_with_name("postmaster", test_domain);
        assert_eq!(alias, "postmaster@example.com");

        // Test transport with specific server
        let transport = TestData::transport_with_server("mail.example.com");
        assert_eq!(transport, "smtp:mail.example.com");

        // Test password with pattern
        let password = TestData::password_with_pattern("test");
        assert!(password.starts_with("test"));

        // Test name with pattern
        let name = TestData::name_with_pattern("Test");
        assert!(name.starts_with("Test "));
    }

    #[tokio::test]
    async fn test_form_data_generation() {
        // Test random form data generation
        let domain_form = TestData::random_domain_form_data();
        assert!(domain_form.contains("domain="));
        assert!(domain_form.contains("transport="));

        let user_form = TestData::random_user_form_data();
        assert!(user_form.contains("id="));
        assert!(user_form.contains("name="));
        assert!(user_form.contains("password="));

        let user_complete_form = TestData::random_user_form_data_complete();
        assert!(user_complete_form.contains("id="));
        assert!(user_complete_form.contains("password="));
        assert!(user_complete_form.contains("name="));
        assert!(user_complete_form.contains("maildir="));
        assert!(user_complete_form.contains("home="));
        assert!(user_complete_form.contains("enabled="));
        assert!(user_complete_form.contains("change_password="));

        let alias_form = TestData::random_alias_form_data();
        assert!(alias_form.contains("mail="));
        assert!(alias_form.contains("destination="));
        assert!(alias_form.contains("enabled="));

        let backup_form = TestData::random_backup_form_data();
        assert!(backup_form.contains("domain="));
        assert!(backup_form.contains("transport="));

        let relay_form = TestData::random_relay_form_data();
        assert!(relay_form.contains("recipient="));
        assert!(relay_form.contains("status="));
        assert!(relay_form.contains("enabled="));

        let relocated_form = TestData::random_relocated_form_data();
        assert!(relocated_form.contains("old_address="));
        assert!(relocated_form.contains("new_address="));
        assert!(relocated_form.contains("enabled="));

        let client_form = TestData::random_client_form_data();
        assert!(client_form.contains("client="));
        assert!(client_form.contains("status="));
        assert!(client_form.contains("enabled="));
    }

    #[tokio::test]
    async fn test_test_datasets() {
        // Test complete test dataset
        let (domain, user, alias) = TestData::complete_test_dataset();
        assert!(!domain.is_empty());
        assert!(!user.is_empty());
        assert!(!alias.is_empty());
        assert!(user.contains("@"));
        assert!(alias.contains("@"));

        // Test multiple test datasets
        let datasets = TestData::multiple_test_datasets(3);
        assert_eq!(datasets.len(), 3);
        for (domain, user, alias) in &datasets {
            assert!(!domain.is_empty());
            assert!(!user.is_empty());
            assert!(!alias.is_empty());
        }

        // Test edge case test data
        let edge_cases = TestData::edge_case_test_data();
        assert!(!edge_cases.is_empty());
        for (domain, user, alias) in &edge_cases {
            assert!(!domain.is_empty());
            assert!(!user.is_empty());
            assert!(!alias.is_empty());
        }

        // Test security test data
        let security_cases = TestData::security_test_data();
        assert!(!security_cases.is_empty());
        for (domain, user, alias) in &security_cases {
            assert!(!domain.is_empty());
            assert!(!user.is_empty());
            assert!(!alias.is_empty());
        }

        // Test Unicode test data
        let unicode_cases = TestData::unicode_test_data();
        assert!(!unicode_cases.is_empty());
        for (domain, user, alias) in &unicode_cases {
            assert!(!domain.is_empty());
            assert!(!user.is_empty());
            assert!(!alias.is_empty());
        }
    }

    #[tokio::test]
    async fn test_enhanced_cleanup_utilities() {
        let container = setup_test_db().await;
        let pool = container.get_pool();

        // Test specific table cleanup
        sortingoffice::test_helpers::common::cleanup_specific_tables(pool, &["domains", "users"]);

        // Test cleanup with verification
        let cleanup_result = sortingoffice::test_helpers::common::cleanup_with_verification(pool);
        assert!(cleanup_result.is_ok());
    }

    #[tokio::test]
    async fn test_test_data_manager() {
        let container = setup_test_db().await;
        let pool = container.get_pool();

        // Test creating test data with cleanup
        let result = TestDataManager::create_test_data_with_cleanup(pool, |_pool| {
            Ok::<String, Box<dyn std::error::Error>>("test-data".to_string())
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-data");

        // Test creating multiple datasets
        let datasets = TestDataManager::create_multiple_datasets(pool, 3, |_pool, index| {
            Ok::<String, Box<dyn std::error::Error>>(format!("dataset-{index}"))
        });
        assert!(datasets.is_ok());
        let datasets = datasets.unwrap();
        assert_eq!(datasets.len(), 3);
        assert_eq!(datasets[0], "dataset-0");
        assert_eq!(datasets[1], "dataset-1");
        assert_eq!(datasets[2], "dataset-2");

        // Test with automatic cleanup
        let result = TestDataManager::with_test_data(pool, |_pool| {
            Ok::<String, Box<dyn std::error::Error>>("auto-cleanup-test".to_string())
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "auto-cleanup-test");
    }

    #[tokio::test]
    async fn test_unique_id_variations() {
        // Test unique ID with prefix
        let prefixed_id = sortingoffice::test_helpers::common::unique_test_id_with_prefix("custom");
        assert!(prefixed_id.starts_with("custom-test-"));

        // Test unique ID with timestamp
        let timestamp_id = sortingoffice::test_helpers::common::unique_test_id_with_timestamp();
        assert!(timestamp_id.starts_with("test-"));

        // Test unique ID with random component
        let random_id = sortingoffice::test_helpers::common::unique_test_id_with_random();
        assert!(random_id.starts_with("test-"));
        assert!(random_id.contains("-"));
    }

    #[tokio::test]
    async fn test_form_data_validation() {
        // Test domain form data validation
        let domain_form = TestData::domain_form_data("test.com", "smtp:localhost", true);
        assert!(domain_form.contains("domain=test.com"));
        assert!(domain_form.contains("transport=smtp:localhost"));
        assert!(domain_form.contains("enabled=on"));

        let domain_form_disabled = TestData::domain_form_data("test.com", "smtp:localhost", false);
        assert!(domain_form_disabled.contains("domain=test.com"));
        assert!(domain_form_disabled.contains("transport=smtp:localhost"));
        assert!(!domain_form_disabled.contains("enabled=on"));

        // Test user form data validation
        let user_form = TestData::user_form_data("user@test.com", "password123", "Test User");
        assert!(user_form.contains("id=user@test.com"));
        assert!(user_form.contains("name=Test User"));
        assert!(user_form.contains("password=password123"));
        assert!(user_form.contains("maildir=testdir"));
        assert!(user_form.contains("home=/var/spool/mail/virtual"));

        // Test alias form data validation
        let alias_form = TestData::alias_form_data("alias@test.com", "user@test.com", true);
        assert!(alias_form.contains("mail=alias@test.com"));
        assert!(alias_form.contains("destination=user@test.com"));
        assert!(alias_form.contains("enabled=on"));

        let alias_form_disabled =
            TestData::alias_form_data("alias@test.com", "user@test.com", false);
        assert!(alias_form_disabled.contains("mail=alias@test.com"));
        assert!(alias_form_disabled.contains("destination=user@test.com"));
        assert!(!alias_form_disabled.contains("enabled=on"));

        // Test backup form data validation
        let backup_form = TestData::backup_form_data("backup.com", "smtp:backup");
        assert!(backup_form.contains("domain=backup.com"));
        assert!(backup_form.contains("transport=smtp:backup"));
    }
}

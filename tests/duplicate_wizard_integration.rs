use sortingoffice::models::DuplicateDomainForm;
use sortingoffice::validation::{self, ValidationError};

#[tokio::test]
async fn test_duplicate_wizard_form_submission() -> Result<(), Box<dyn std::error::Error>> {
    // This test verifies that the duplicate wizard form can be submitted without errors
    // We'll test the form deserialization directly

    // Test form data that should work
    let form_data = "source_domain=example.com&new_domain=new-example.com&transport=virtual&enabled=on&duplicate_aliases=on&duplicate_relays=on";

    // Parse the form data
    let form: DuplicateDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify all fields are correctly parsed
    assert_eq!(form.source_domain, "example.com");
    assert_eq!(form.new_domain, "new-example.com");
    assert_eq!(form.transport, "virtual:");
    assert_eq!(form.enabled, true);
    assert_eq!(form.duplicate_aliases, true);
    assert_eq!(form.duplicate_relays, true);
    assert_eq!(form.confirmed, false); // Default value

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_without_optional_fields(
) -> Result<(), Box<dyn std::error::Error>> {
    // Test form data without optional checkbox fields
    let form_data = "source_domain=example.com&new_domain=new-example.com&transport=virtual";

    // Parse the form data
    let form: DuplicateDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify required fields are correctly parsed
    assert_eq!(form.source_domain, "example.com");
    assert_eq!(form.new_domain, "new-example.com");
    assert_eq!(form.transport, "virtual:");

    // Verify optional fields have default values
    assert_eq!(form.enabled, false);
    assert_eq!(form.duplicate_aliases, false);
    assert_eq!(form.duplicate_relays, false);
    assert_eq!(form.confirmed, false);

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_with_smtp_transport() -> Result<(), Box<dyn std::error::Error>>
{
    // Test form data with SMTP transport
    let form_data = "source_domain=example.com&new_domain=new-example.com&transport=smtp:mail.example.com&enabled=on&duplicate_aliases=on";

    // Parse the form data
    let form: DuplicateDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify all fields are correctly parsed
    assert_eq!(form.source_domain, "example.com");
    assert_eq!(form.new_domain, "new-example.com");
    assert_eq!(form.transport, "smtp:mail.example.com");
    assert_eq!(form.enabled, true);
    assert_eq!(form.duplicate_aliases, true);
    assert_eq!(form.duplicate_relays, false); // Not specified
    assert_eq!(form.confirmed, false);

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    // Test form data for the confirmation step
    let form_data = "confirmed=on";

    // Parse the form data
    let form: DuplicateDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify confirmation field is correctly parsed
    assert_eq!(form.confirmed, true);

    // Other fields should have default values
    assert_eq!(form.source_domain, "");
    assert_eq!(form.new_domain, "");
    assert_eq!(form.transport, "");
    assert_eq!(form.enabled, false);
    assert_eq!(form.duplicate_aliases, false);
    assert_eq!(form.duplicate_relays, false);

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_translations() {
    use sortingoffice::config::ConnectionPoolConfig;
    use sortingoffice::config::DatabaseConfig;
    use sortingoffice::config::DatabaseFeatures;
    use sortingoffice::i18n;
    use sortingoffice::AppState;
    use std::collections::HashMap;

    // Create a minimal AppState for testing
    let configs = vec![DatabaseConfig {
        id: "primary".to_string(),
        label: "Primary".to_string(),
        url: "mysql://test".to_string(),
        features: DatabaseFeatures::default(),
        field_map: HashMap::new(),
        connection_pool: ConnectionPoolConfig::default(),
    }];

    let db_manager = sortingoffice::db::DatabaseManager::new(configs)
        .await
        .expect("Failed to create database manager");

    let i18n = sortingoffice::I18n::new("en-US").expect("Failed to initialize i18n");
    let config = sortingoffice::Config::default();

    let state = AppState {
        db_manager,
        i18n,
        config,
    };

    let locale = "en-US";

    // Test some key translations
    let title = i18n::get_translation(&state, locale, "duplicate-wizard-title").await;
    assert_eq!(title, "Duplicate Domain Wizard");

    let description = i18n::get_translation(&state, locale, "duplicate-wizard-description").await;
    assert_eq!(
        description,
        "Select a domain to duplicate and configure the new domain settings."
    );

    let source_label =
        i18n::get_translation(&state, locale, "duplicate-wizard-source-domain-label").await;
    assert_eq!(source_label, "Source Domain");

    let next_button = i18n::get_translation(&state, locale, "wizard-next").await;
    assert_eq!(next_button, "Next");

    println!("✅ All duplicate wizard translations are working correctly!");
}

#[tokio::test]
async fn test_duplicate_wizard_form_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test form validation with various input scenarios
    
    // Test valid form data
    let valid_form_data = "source_domain=example.com&new_domain=test-example.com&transport=virtual&enabled=on&duplicate_aliases=on&duplicate_relays=on";
    let valid_form: DuplicateDomainForm = serde_urlencoded::from_str(valid_form_data)?;
    
    assert_eq!(valid_form.source_domain, "example.com");
    assert_eq!(valid_form.new_domain, "test-example.com");
    assert_eq!(valid_form.transport, "virtual:");
    assert_eq!(valid_form.enabled, true);
    assert_eq!(valid_form.duplicate_aliases, true);
    assert_eq!(valid_form.duplicate_relays, true);

    // Test form with SMTP transport
    let smtp_form_data = "source_domain=example.com&new_domain=test-example.com&transport=smtp:mail.example.com&enabled=on&duplicate_aliases=on&duplicate_relays=on";
    let smtp_form: DuplicateDomainForm = serde_urlencoded::from_str(smtp_form_data)?;
    
    assert_eq!(smtp_form.transport, "smtp:mail.example.com");

    // Test form with empty optional fields
    let minimal_form_data = "source_domain=example.com&new_domain=test-example.com&transport=virtual";
    let minimal_form: DuplicateDomainForm = serde_urlencoded::from_str(minimal_form_data)?;
    
    assert_eq!(minimal_form.source_domain, "example.com");
    assert_eq!(minimal_form.new_domain, "test-example.com");
    assert_eq!(minimal_form.enabled, false); // Default value
    assert_eq!(minimal_form.duplicate_aliases, false); // Default value
    assert_eq!(minimal_form.duplicate_relays, false); // Default value

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test edge cases and boundary conditions
    
    // Test form with very long domain names
    let long_domain = "a".repeat(100);
    let long_form_data = format!("source_domain=example.com&new_domain={}.com&transport=virtual", long_domain);
    let long_form: DuplicateDomainForm = serde_urlencoded::from_str(&long_form_data)?;
    
    assert_eq!(long_form.new_domain, format!("{}.com", long_domain));

    // Test form with special characters in transport
    let special_transport_form_data = "source_domain=example.com&new_domain=test.com&transport=smtp:[2001:db8::1]:587&enabled=on";
    let special_form: DuplicateDomainForm = serde_urlencoded::from_str(special_transport_form_data)?;
    
    assert_eq!(special_form.transport, "smtp:[2001:db8::1]:587");

    // Test form with unicode characters
    let unicode_form_data = "source_domain=example.com&new_domain=测试.com&transport=virtual";
    let unicode_form: DuplicateDomainForm = serde_urlencoded::from_str(unicode_form_data)?;
    
    assert_eq!(unicode_form.new_domain, "测试.com");

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_confirmation_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    // Test different confirmation scenarios
    
    // Test confirmation form
    let confirm_form_data = "source_domain=example.com&new_domain=test.com&transport=virtual&confirmed=true";
    let confirm_form: DuplicateDomainForm = serde_urlencoded::from_str(confirm_form_data)?;
    
    assert_eq!(confirm_form.confirmed, true);
    assert_eq!(confirm_form.source_domain, "example.com");
    assert_eq!(confirm_form.new_domain, "test.com");

    // Test cancellation form
    let cancel_form_data = "source_domain=example.com&new_domain=test.com&transport=virtual&confirmed=false";
    let cancel_form: DuplicateDomainForm = serde_urlencoded::from_str(cancel_form_data)?;
    
    assert_eq!(cancel_form.confirmed, false);

    // Test form without confirmation (default)
    let no_confirm_form_data = "source_domain=example.com&new_domain=test.com&transport=virtual";
    let no_confirm_form: DuplicateDomainForm = serde_urlencoded::from_str(no_confirm_form_data)?;
    
    assert_eq!(no_confirm_form.confirmed, false); // Default value

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_form_boolean_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test boolean field handling in forms
    
    // Test all boolean fields enabled
    let all_enabled_data = "source_domain=example.com&new_domain=test.com&transport=virtual&enabled=on&duplicate_aliases=on&duplicate_relays=on&confirmed=true";
    let all_enabled_form: DuplicateDomainForm = serde_urlencoded::from_str(all_enabled_data)?;
    
    assert_eq!(all_enabled_form.enabled, true);
    assert_eq!(all_enabled_form.duplicate_aliases, true);
    assert_eq!(all_enabled_form.duplicate_relays, true);
    assert_eq!(all_enabled_form.confirmed, true);

    // Test all boolean fields disabled
    let all_disabled_data = "source_domain=example.com&new_domain=test.com&transport=virtual";
    let all_disabled_form: DuplicateDomainForm = serde_urlencoded::from_str(all_disabled_data)?;
    
    assert_eq!(all_disabled_form.enabled, false);
    assert_eq!(all_disabled_form.duplicate_aliases, false);
    assert_eq!(all_disabled_form.duplicate_relays, false);
    assert_eq!(all_disabled_form.confirmed, false);

    // Test mixed boolean fields
    let mixed_data = "source_domain=example.com&new_domain=test.com&transport=virtual&enabled=on&duplicate_relays=on";
    let mixed_form: DuplicateDomainForm = serde_urlencoded::from_str(mixed_data)?;
    
    assert_eq!(mixed_form.enabled, true);
    assert_eq!(mixed_form.duplicate_aliases, false); // Not provided
    assert_eq!(mixed_form.duplicate_relays, true);
    assert_eq!(mixed_form.confirmed, false); // Not provided

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_domain_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test domain validation with various invalid domain names
    
    // Test uppercase letters (should fail)
    let uppercase_form_data = "source_domain=example.com&new_domain=INVALID-DOMAIN.com&transport=virtual";
    let uppercase_form: DuplicateDomainForm = serde_urlencoded::from_str(uppercase_form_data)?;
    
    // Validate the domain
    let validation_result = validation::validate_domain(&uppercase_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("uppercase"));
    }
    
    // Test consecutive dots (should fail)
    let consecutive_dots_form_data = "source_domain=example.com&new_domain=test..domain.com&transport=virtual";
    let consecutive_dots_form: DuplicateDomainForm = serde_urlencoded::from_str(consecutive_dots_form_data)?;
    
    let validation_result = validation::validate_domain(&consecutive_dots_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("consecutive"));
    }
    
    // Test invalid characters (should fail)
    let invalid_chars_form_data = "source_domain=example.com&new_domain=test@domain.com&transport=virtual";
    let invalid_chars_form: DuplicateDomainForm = serde_urlencoded::from_str(invalid_chars_form_data)?;
    
    let validation_result = validation::validate_domain(&invalid_chars_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("lowercase letters, numbers, dots, and hyphens"));
    }
    
    // Test valid domain (should pass)
    let valid_form_data = "source_domain=example.com&new_domain=valid-domain.com&transport=virtual";
    let valid_form: DuplicateDomainForm = serde_urlencoded::from_str(valid_form_data)?;
    
    let validation_result = validation::validate_domain(&valid_form.new_domain);
    assert!(validation_result.is_ok());
    
    // Test domain starting with hyphen (should fail)
    let start_hyphen_form_data = "source_domain=example.com&new_domain=-invalid.com&transport=virtual";
    let start_hyphen_form: DuplicateDomainForm = serde_urlencoded::from_str(start_hyphen_form_data)?;
    
    let validation_result = validation::validate_domain(&start_hyphen_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("start with dot or hyphen"));
    }
    
    // Test domain ending with hyphen (should fail)
    let end_hyphen_form_data = "source_domain=example.com&new_domain=invalid-&transport=virtual";
    let end_hyphen_form: DuplicateDomainForm = serde_urlencoded::from_str(end_hyphen_form_data)?;
    
    let validation_result = validation::validate_domain(&end_hyphen_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("end with dot or hyphen"));
    }
    
    // Test empty domain (should fail)
    let empty_form_data = "source_domain=example.com&new_domain=&transport=virtual";
    let empty_form: DuplicateDomainForm = serde_urlencoded::from_str(empty_form_data)?;
    
    let validation_result = validation::validate_domain(&empty_form.new_domain);
    assert!(validation_result.is_err());
    if let Err(ValidationError::DomainInvalid(msg)) = validation_result {
        assert!(msg.contains("empty"));
    }
    
    Ok(())
}

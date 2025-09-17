use sortingoffice::models::DuplicateDomainForm;

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
    assert_eq!(form.transport, "virtual");
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
    assert_eq!(form.transport, "virtual");

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

    let source_label = i18n::get_translation(&state, locale, "duplicate-wizard-source-domain-label").await;
    assert_eq!(source_label, "Source Domain");

    let next_button = i18n::get_translation(&state, locale, "wizard-next").await;
    assert_eq!(next_button, "Next");

    println!("✅ All duplicate wizard translations are working correctly!");
}

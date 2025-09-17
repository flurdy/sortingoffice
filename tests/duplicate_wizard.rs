use sortingoffice::models::{DuplicateDomainSession, DuplicateWizardStep};

#[tokio::test]
async fn test_duplicate_domain_session_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Test session creation with different configurations
    let session1 = DuplicateDomainSession {
        step: DuplicateWizardStep::DomainSelection,
        source_domain: None,
        source_is_backup: false,
        new_domain: "test.com".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: None,
    };

    // Test basic session properties
    assert_eq!(session1.new_domain, "test.com");
    assert_eq!(session1.duplicate_aliases, true);
    assert_eq!(session1.duplicate_relays, true);
    assert_eq!(session1.enabled, true);
    assert_eq!(session1.transport, "virtual");

    // Test session with different settings
    let session2 = DuplicateDomainSession {
        step: DuplicateWizardStep::Review,
        source_domain: None,
        source_is_backup: false,
        new_domain: "another.com".to_string(),
        transport: "smtp:mail.another.com".to_string(),
        enabled: false,
        duplicate_aliases: false,
        duplicate_relays: false,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: None,
    };

    assert_eq!(session2.new_domain, "another.com");
    assert_eq!(session2.duplicate_aliases, false);
    assert_eq!(session2.duplicate_relays, false);
    assert_eq!(session2.enabled, false);
    assert_eq!(session2.transport, "smtp:mail.another.com");

    Ok(())
}

#[tokio::test]
async fn test_duplicate_wizard_step_enum() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can create different wizard steps
    let steps = vec![
        DuplicateWizardStep::DomainSelection,
        DuplicateWizardStep::Configuration,
        DuplicateWizardStep::Review,
        DuplicateWizardStep::Executing,
        DuplicateWizardStep::Complete,
    ];

    // Verify we can create all steps
    assert_eq!(steps.len(), 5);

    // Test pattern matching
    match steps[0] {
        DuplicateWizardStep::DomainSelection => {}
        _ => panic!("Expected DomainSelection step"),
    }

    match steps[1] {
        DuplicateWizardStep::Configuration => {}
        _ => panic!("Expected Configuration step"),
    }

    match steps[2] {
        DuplicateWizardStep::Review => {}
        _ => panic!("Expected Review step"),
    }

    match steps[3] {
        DuplicateWizardStep::Executing => {}
        _ => panic!("Expected Executing step"),
    }

    match steps[4] {
        DuplicateWizardStep::Complete => {}
        _ => panic!("Expected Complete step"),
    }

    Ok(())
}

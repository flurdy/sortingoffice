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

#[tokio::test]
async fn test_backup_domain_session_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test session creation for backup domain duplication
    let backup_session = DuplicateDomainSession {
        step: DuplicateWizardStep::Configuration,
        source_domain: None,
        source_is_backup: true, // Source is a backup domain
        new_domain: "new-backup.com".to_string(),
        transport: "smtp:mail.backup.com".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: Some(true), // Target should also be backup
    };

    // Test backup domain properties
    assert_eq!(backup_session.source_is_backup, true);
    assert_eq!(backup_session.target_is_backup, Some(true));
    assert_eq!(backup_session.new_domain, "new-backup.com");
    assert_eq!(backup_session.transport, "smtp:mail.backup.com");

    // Test session with backup source but normal target
    let mixed_session = DuplicateDomainSession {
        step: DuplicateWizardStep::Configuration,
        source_domain: None,
        source_is_backup: true, // Source is backup
        new_domain: "new-normal.com".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: Some(false), // Target is normal
    };

    assert_eq!(mixed_session.source_is_backup, true);
    assert_eq!(mixed_session.target_is_backup, Some(false));

    // Test session with normal source but backup target
    let reverse_mixed_session = DuplicateDomainSession {
        step: DuplicateWizardStep::Configuration,
        source_domain: None,
        source_is_backup: false, // Source is normal
        new_domain: "new-backup-from-normal.com".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: Some(true), // Target is backup
    };

    assert_eq!(reverse_mixed_session.source_is_backup, false);
    assert_eq!(reverse_mixed_session.target_is_backup, Some(true));

    Ok(())
}

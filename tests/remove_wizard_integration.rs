use sortingoffice::models::{
    DeletedResourceCount, DisabledResourceCount, RemoveDomainForm, RemoveWizardStep,
};

#[tokio::test]
async fn test_remove_wizard_form_submission() -> Result<(), Box<dyn std::error::Error>> {
    // Test form data for domain selection step
    let form_data = "domain_name=example.com&confirmed=on";

    // Parse the form data
    let form: RemoveDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify all fields are correctly parsed
    assert_eq!(form.domain_name, "example.com");
    assert_eq!(form.confirmed, true);

    Ok(())
}

#[tokio::test]
async fn test_remove_wizard_form_without_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    // Test form data without confirmation
    let form_data = "domain_name=example.com";

    // Parse the form data
    let form: RemoveDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify required field is parsed
    assert_eq!(form.domain_name, "example.com");

    // Verify confirmation defaults to false
    assert_eq!(form.confirmed, false);

    Ok(())
}

#[tokio::test]
async fn test_remove_wizard_form_empty_domain() -> Result<(), Box<dyn std::error::Error>> {
    // Test form data with empty domain
    let form_data = "domain_name=";

    // Parse the form data
    let form: RemoveDomainForm = serde_urlencoded::from_str(form_data)?;

    // Verify domain is empty string (validation should happen in handler)
    assert_eq!(form.domain_name, "");
    assert_eq!(form.confirmed, false);

    Ok(())
}

#[tokio::test]
async fn test_remove_wizard_step_progression() -> Result<(), Box<dyn std::error::Error>> {
    // Test that wizard steps are in logical order
    let steps = vec![
        RemoveWizardStep::DomainSelection,
        RemoveWizardStep::ReviewAffected,
        RemoveWizardStep::DisableResources,
        RemoveWizardStep::ReviewDisabled,
        RemoveWizardStep::ConfirmDelete,
        RemoveWizardStep::DeletingResources,
        RemoveWizardStep::Complete,
    ];

    // Verify we have 7 steps
    assert_eq!(steps.len(), 7);

    // Verify all steps are unique
    for (i, step) in steps.iter().enumerate() {
        for (j, other_step) in steps.iter().enumerate() {
            if i == j {
                assert_eq!(step, other_step);
            } else {
                assert_ne!(step, other_step);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_disabled_count_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = DisabledResourceCount::default();

    // Verify default is all zeros
    assert_eq!(count.domain, 0);
    assert_eq!(count.aliases, 0);
    assert_eq!(count.users, 0);
    assert_eq!(count.relays, 0);
    assert_eq!(count.relocated, 0);

    // Simulate disabling resources
    count.domain = 1;
    count.aliases = 5;
    count.users = 3;
    count.relays = 2;
    count.relocated = 1;

    // Verify counts are updated
    assert_eq!(count.domain, 1);
    assert_eq!(count.aliases, 5);
    assert_eq!(count.users, 3);
    assert_eq!(count.relays, 2);
    assert_eq!(count.relocated, 1);

    Ok(())
}

#[tokio::test]
async fn test_deleted_count_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = DeletedResourceCount::default();

    // Verify default is all zeros
    assert_eq!(count.domain, 0);
    assert_eq!(count.aliases, 0);
    assert_eq!(count.users, 0);
    assert_eq!(count.relays, 0);
    assert_eq!(count.relocated, 0);

    // Simulate deleting resources in correct order
    count.relocated = 1;
    count.relays = 2;
    count.users = 3;
    count.aliases = 5;
    count.domain = 1;

    // Verify counts are updated
    assert_eq!(count.relocated, 1);
    assert_eq!(count.relays, 2);
    assert_eq!(count.users, 3);
    assert_eq!(count.aliases, 5);
    assert_eq!(count.domain, 1);

    Ok(())
}

#[tokio::test]
async fn test_resource_count_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test DisabledResourceCount serialization
    let disabled_count = DisabledResourceCount {
        domain: 1,
        aliases: 5,
        users: 3,
        relays: 2,
        relocated: 1,
    };

    let serialized = serde_json::to_string(&disabled_count)?;
    let deserialized: DisabledResourceCount = serde_json::from_str(&serialized)?;

    assert_eq!(deserialized.domain, 1);
    assert_eq!(deserialized.aliases, 5);
    assert_eq!(deserialized.users, 3);
    assert_eq!(deserialized.relays, 2);
    assert_eq!(deserialized.relocated, 1);

    // Test DeletedResourceCount serialization
    let deleted_count = DeletedResourceCount {
        domain: 1,
        aliases: 5,
        users: 3,
        relays: 2,
        relocated: 1,
    };

    let serialized = serde_json::to_string(&deleted_count)?;
    let deserialized: DeletedResourceCount = serde_json::from_str(&serialized)?;

    assert_eq!(deserialized.domain, 1);
    assert_eq!(deserialized.aliases, 5);
    assert_eq!(deserialized.users, 3);
    assert_eq!(deserialized.relays, 2);
    assert_eq!(deserialized.relocated, 1);

    Ok(())
}

#[tokio::test]
async fn test_remove_wizard_step_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test that wizard steps can be serialized and deserialized
    let test_cases = vec![
        RemoveWizardStep::DomainSelection,
        RemoveWizardStep::ReviewAffected,
        RemoveWizardStep::DisableResources,
        RemoveWizardStep::ReviewDisabled,
        RemoveWizardStep::ConfirmDelete,
        RemoveWizardStep::DeletingResources,
        RemoveWizardStep::Complete,
    ];

    for step in test_cases {
        let serialized = serde_json::to_string(&step)?;
        let deserialized: RemoveWizardStep = serde_json::from_str(&serialized)?;
        assert_eq!(step, deserialized);
    }

    Ok(())
}

#[tokio::test]
async fn test_remove_wizard_form_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test various form inputs
    let valid_form = "domain_name=test.com&confirmed=on";
    let form: RemoveDomainForm = serde_urlencoded::from_str(valid_form)?;
    assert!(!form.domain_name.is_empty());
    assert!(form.confirmed);

    // Test with special characters in domain (should be accepted by form parser)
    let special_form = "domain_name=sub.domain.test.com&confirmed=on";
    let form: RemoveDomainForm = serde_urlencoded::from_str(special_form)?;
    assert_eq!(form.domain_name, "sub.domain.test.com");

    Ok(())
}

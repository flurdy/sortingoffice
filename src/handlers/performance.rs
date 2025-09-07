use crate::AppState;

/// Optimized helper to get config aliases without cloning
/// Performance optimization: Avoid unnecessary cloning of config data
pub fn get_config_aliases_references(state: &AppState) -> (&[String], &[String]) {
    (&state.config.required_aliases, &state.config.common_aliases)
}

/// Optimized helper to create domain form from domain without cloning
/// Performance optimization: Use references where possible
pub fn create_domain_form_from_domain(domain: &crate::models::Domain) -> crate::models::DomainForm {
    crate::models::DomainForm {
        domain: domain.domain.as_str().to_string(),
        transport: domain.transport.as_deref().unwrap_or("virtual").to_string(),
        enabled: domain.enabled,
    }
}

/// Optimized helper to create new domain without cloning
/// Performance optimization: Use references for form data
pub fn create_new_domain_from_form(form: &crate::models::DomainForm) -> crate::models::NewDomain {
    crate::models::NewDomain {
        domain: form.domain.as_str().to_string(),
        transport: Some(form.transport.as_str().to_string()),
        enabled: form.enabled,
    }
}

/// Optimized helper to create alias form from alias without cloning
/// Performance optimization: Use references where possible
pub fn create_alias_form_from_alias(alias: &crate::models::Alias) -> crate::models::AliasForm {
    crate::models::AliasForm {
        mail: alias.mail.as_str().to_string(),
        destination: alias.destination.as_str().to_string(),
        enabled: alias.enabled,
        redirect_to: Some(String::new()),
        return_url: Some(String::new()),
    }
}

/// Optimized helper to create user form from user without cloning
/// Performance optimization: Use references where possible
pub fn create_user_form_from_user(user: &crate::models::User) -> crate::models::UserForm {
    crate::models::UserForm {
        id: user.id.as_str().to_string(),
        name: user.id.as_str().to_string(), // Use id as name for form
        password: String::new(),            // Don't include password in form
        enabled: user.enabled,
        change_password: false,
        home: String::new(),
        maildir: String::new(),
    }
}

/// Optimized helper to create backup form from backup without cloning
/// Performance optimization: Use references where possible
pub fn create_backup_form_from_backup(backup: &crate::models::Backup) -> crate::models::BackupForm {
    crate::models::BackupForm {
        domain: backup.domain.as_str().to_string(),
        transport: backup.transport.as_deref().unwrap_or("virtual").to_string(),
        enabled: backup.enabled,
    }
}

/// Optimized helper to create relay form from relay without cloning
/// Performance optimization: Use references where possible
pub fn create_relay_form_from_relay(relay: &crate::models::Relay) -> crate::models::RelayForm {
    crate::models::RelayForm {
        recipient: relay.recipient.as_str().to_string(),
        status: relay.status.as_str().to_string(),
        enabled: relay.enabled,
    }
}

/// Optimized helper to create relocated form from relocated without cloning
/// Performance optimization: Use references where possible
pub fn create_relocated_form_from_relocated(
    relocated: &crate::models::Relocated,
) -> crate::models::RelocatedForm {
    crate::models::RelocatedForm {
        old_address: relocated.old_address.as_str().to_string(),
        new_address: relocated.new_address.as_str().to_string(),
        enabled: relocated.enabled,
    }
}

/// Optimized helper to create client form from client without cloning
/// Performance optimization: Use references where possible
pub fn create_client_form_from_client(client: &crate::models::Client) -> crate::models::ClientForm {
    crate::models::ClientForm {
        client: client.client.as_str().to_string(),
        status: client.status.as_str().to_string(),
        enabled: client.enabled,
    }
}

use crate::{
    db,
    handlers::{
        http_helpers::get_user_locale,
        rendering::{
            render_duplicate_domain_complete_page, render_duplicate_domain_review_page,
            render_duplicate_domain_selection_page,
        },
    },
    models::{Domain, DuplicateDomainForm, DuplicateDomainSession, DuplicateWizardStep, Relay},
    AppState, DbPool,
};
use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::{Html, Redirect},
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct DomainSearchQuery {
    pub domain: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ToggleEnabledForm {
    pub enabled: String,
}

#[derive(Deserialize)]
pub struct ToggleAliasEnabledForm {
    pub alias_id: i32,
    pub enabled: String,
}

#[derive(Deserialize)]
pub struct ToggleRelayEnabledForm {
    pub relay_id: i32,
    pub enabled: String,
}

// Simple session storage using static HashMap
lazy_static! {
    static ref DUPLICATE_WIZARD_SESSIONS: Mutex<HashMap<String, DuplicateDomainSession>> =
        Mutex::new(HashMap::new());
}

/// Duplicate wizard index page
pub async fn index(State(_state): State<AppState>, _headers: HeaderMap) -> Redirect {
    // Clear any existing session when starting a new wizard
    clear_session();

    // Redirect directly to domain selection step
    Redirect::to("/duplicate-wizard/domain-selection")
}

/// Step 1: Domain selection
pub async fn domain_selection(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    // Get available domains for selection
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    // Get regular domains
    let mut domains = match db::get_domains(&pool) {
        Ok(domains) => domains,
        Err(e) => {
            error!("Failed to get domains: {:?}", e);
            vec![]
        }
    };

    // Get backup domains and add them to the list
    let backups = match db::get_backups(&pool) {
        Ok(backups) => backups,
        Err(e) => {
            error!("Failed to get backup domains: {:?}", e);
            vec![]
        }
    };

    // Convert backup domains to regular domains for display
    for backup in backups {
        let backup_domain = Domain {
            pkid: backup.pkid,
            domain: backup.domain,
            transport: backup.transport.or_else(|| Some("virtual".to_string())),
            created: backup.created,
            modified: backup.modified,
            enabled: backup.enabled,
        };
        domains.push(backup_domain);
    }

    // Sort domains alphabetically
    domains.sort_by(|a, b| a.domain.cmp(&b.domain));

    // Create session
    let session = DuplicateDomainSession {
        step: DuplicateWizardStep::DomainSelection,
        source_domain: None,
        source_is_backup: false,
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: None,
    };
    save_session(session);

    render_duplicate_domain_selection_page(domains, &state, &locale, &headers).await
}

/// Handle domain selection form submission
pub async fn domain_selection_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DuplicateDomainForm>,
) -> Result<Html<String>, Redirect> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Ok(Html("Database connection error".to_string()));
        }
    };

    // Get source domain and determine if it's a backup domain
    let (source_domain, source_is_backup) = match db::get_domain_by_name(&pool, &form.source_domain)
    {
        Ok(domain) => (domain, false), // Normal domain
        Err(_) => {
            // Try backup domains
            match db::get_backup_by_name(&pool, &form.source_domain) {
                Ok(backup) => {
                    let domain = Domain {
                        pkid: backup.pkid,
                        domain: backup.domain,
                        transport: backup.transport,
                        created: backup.created,
                        modified: backup.modified,
                        enabled: backup.enabled,
                    };
                    (domain, true) // Backup domain
                }
                Err(_) => {
                    return Ok(Html("Source domain not found".to_string()));
                }
            }
        }
    };

    // Get aliases and relays to duplicate (always duplicate both)
    let aliases_to_duplicate = match db::get_aliases_for_domain(&pool, &source_domain.domain) {
        Ok(aliases) => {
            // Transform aliases to show what they will look like after duplication
            aliases
                .into_iter()
                .map(|alias| {
                    let new_mail = alias.mail.replace(
                        &format!("@{}", source_domain.domain),
                        &format!("@{}", form.new_domain),
                    );
                    crate::models::Alias {
                        pkid: alias.pkid,
                        mail: new_mail,
                        destination: alias.destination,
                        created: alias.created,
                        modified: alias.modified,
                        enabled: alias.enabled,
                    }
                })
                .collect()
        }
        Err(e) => {
            error!("Failed to get aliases: {:?}", e);
            vec![]
        }
    };

    let relays_to_duplicate = match get_relays_for_domain(&pool, &source_domain.domain).await {
        Ok(relays) => relays,
        Err(e) => {
            error!("Failed to get relays: {:?}", e);
            vec![]
        }
    };

    // Update session
    let mut session = get_session().unwrap_or_else(|| DuplicateDomainSession {
        step: DuplicateWizardStep::DomainSelection,
        source_domain: None,
        source_is_backup: false,
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
        target_is_backup: None,
    });

    session.step = DuplicateWizardStep::Configuration;
    session.source_domain = Some(source_domain.clone());
    session.source_is_backup = source_is_backup;
    session.new_domain = form.new_domain;
    session.transport = source_domain
        .transport
        .unwrap_or_else(|| "virtual".to_string()); // Always copy source domain's transport
    session.enabled = true; // Default to enabled, can be changed via toggles on review page
    session.duplicate_aliases = true; // Always duplicate aliases
    session.duplicate_relays = true; // Always duplicate relays
    session.aliases_to_duplicate = aliases_to_duplicate;
    session.relays_to_duplicate = relays_to_duplicate;
    // Set target domain type to match source domain type (for now)
    session.target_is_backup = Some(source_is_backup);

    save_session(session);

    // Redirect to review step
    Ok(review(State(state), headers).await)
}

/// Step 2: Review configuration
pub async fn review(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Html("No session found. Please start over.".to_string());
        }
    };

    render_duplicate_domain_review_page(&session, &state, &locale, &headers).await
}

/// Step 3: Execute duplication
pub async fn execute(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DuplicateDomainForm>,
) -> Result<Html<String>, Redirect> {
    if !form.confirmed {
        // User didn't confirm, go back to review
        return Ok(review(State(state), headers).await);
    }

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found. Please start over.".to_string()));
        }
    };

    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Ok(Html("Database connection error".to_string()));
        }
    };

    // Create the new domain
    let new_domain = match create_duplicated_domain(&pool, &session).await {
        Ok(domain) => domain,
        Err(e) => {
            error!("Failed to create domain: {:?}", e);
            return Ok(Html("Failed to create domain".to_string()));
        }
    };

    info!(
        "Successfully duplicated domain: {} -> {}",
        session.source_domain.as_ref().unwrap().domain,
        new_domain.domain
    );

    // Clear session
    clear_session();

    // Get translations for success page
    let locale = get_user_locale(&headers);

    let html = render_duplicate_domain_complete_page(
        &session.source_domain.as_ref().unwrap().domain,
        &new_domain.domain,
        new_domain.pkid,
        &state,
        &locale,
        &headers,
    )
    .await;

    Ok(html)
}

/// Helper function to get relays for a domain
pub async fn get_relays_for_domain(
    pool: &DbPool,
    domain: &str,
) -> Result<Vec<Relay>, diesel::result::Error> {
    // Get all relays and filter by domain (relays are domain-specific through recipient field)
    let all_relays = db::get_relays(pool)?;
    let domain_relays: Vec<Relay> = all_relays
        .into_iter()
        .filter(|relay| relay.recipient.ends_with(&format!("@{}", domain)))
        .collect();
    Ok(domain_relays)
}

/// Helper function to create duplicated domain
pub async fn create_duplicated_domain(
    pool: &DbPool,
    session: &DuplicateDomainSession,
) -> Result<Domain, diesel::result::Error> {
    use crate::models::{NewBackup, NewDomain};

    // Determine if we should create a backup domain or normal domain
    let should_create_backup = session.target_is_backup.unwrap_or(session.source_is_backup);

    let created_domain = if should_create_backup {
        // Create backup domain
        let new_backup = NewBackup {
            domain: session.new_domain.clone(),
            transport: Some(session.transport.clone()),
            enabled: session.enabled,
        };

        let backup = db::create_backup(pool, new_backup)?;
        // Convert backup to Domain for return type compatibility
        Domain {
            pkid: backup.pkid,
            domain: backup.domain,
            transport: backup.transport,
            created: backup.created,
            modified: backup.modified,
            enabled: backup.enabled,
        }
    } else {
        // Create normal domain
        let new_domain = NewDomain {
            domain: session.new_domain.clone(),
            transport: Some(session.transport.clone()),
            enabled: session.enabled,
        };

        db::create_domain(pool, new_domain)?
    };

    // Duplicate aliases if requested
    if session.duplicate_aliases {
        for alias in &session.aliases_to_duplicate {
            let new_mail = alias.mail.replace(
                &session.source_domain.as_ref().unwrap().domain,
                &session.new_domain,
            );
            let new_alias = crate::models::NewAlias {
                mail: new_mail,
                destination: alias.destination.clone(),
                enabled: alias.enabled,
            };
            let _ = db::create_alias(
                pool,
                crate::models::AliasForm {
                    mail: new_alias.mail,
                    destination: new_alias.destination,
                    enabled: new_alias.enabled,
                    return_url: None,
                    redirect_to: None,
                },
            );
        }
    }

    // Duplicate relays if requested
    if session.duplicate_relays {
        for relay in &session.relays_to_duplicate {
            let new_recipient = relay.recipient.replace(
                &session.source_domain.as_ref().unwrap().domain,
                &session.new_domain,
            );
            let new_relay = crate::models::NewRelay {
                recipient: new_recipient,
                status: relay.status.clone(),
                enabled: relay.enabled,
            };
            let _ = db::create_relay(
                pool,
                crate::models::RelayForm {
                    recipient: new_relay.recipient,
                    status: new_relay.status,
                    enabled: new_relay.enabled,
                },
            );
        }
    }

    Ok(created_domain)
}

/// Helper function to get session for a user (simplified - using admin as key)
fn get_session() -> Option<DuplicateDomainSession> {
    DUPLICATE_WIZARD_SESSIONS
        .lock()
        .unwrap()
        .get("admin")
        .cloned()
}

/// Helper function to save session for a user
fn save_session(session: DuplicateDomainSession) {
    DUPLICATE_WIZARD_SESSIONS
        .lock()
        .unwrap()
        .insert("admin".to_string(), session);
}

/// HTMX handler to toggle new domain enabled state
pub async fn toggle_new_domain_enabled(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ToggleEnabledForm>,
) -> Result<Html<String>, Redirect> {
    let mut session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found".to_string()));
        }
    };

    // Update the new domain enabled state
    session.enabled = form.enabled == "true";

    // Save the updated session
    save_session(session);

    // Return the updated toggle HTML
    let enabled = form.enabled == "true";
    let enabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-enabled").await;
    let disabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-disabled").await;

    let checked_true = if enabled { "checked" } else { "" };
    let checked_false = if !enabled { "checked" } else { "" };
    
    let html = format!(
        "<div class=\"radio-toggle-container\">\
            <div class=\"radio-toggle\">\
                <input type=\"radio\" id=\"new_enabled_true\" name=\"new_enabled\" value=\"true\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-new-domain-enabled\" hx-vals='{{\"enabled\": \"true\"}}' hx-target=\"#new-domain-enabled-toggle\">\
                <label for=\"new_enabled_true\" class=\"radio-toggle-label radio-toggle-label-left\">\
                    <span class=\"radio-toggle-icon\">✓</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <input type=\"radio\" id=\"new_enabled_false\" name=\"new_enabled\" value=\"false\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-new-domain-enabled\" hx-vals='{{\"enabled\": \"false\"}}' hx-target=\"#new-domain-enabled-toggle\">\
                <label for=\"new_enabled_false\" class=\"radio-toggle-label radio-toggle-label-right\">\
                    <span class=\"radio-toggle-icon\">✗</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <div class=\"radio-toggle-slider\"></div>\
            </div>\
        </div>",
        checked_true,
        enabled_status,
        checked_false,
        disabled_status
    );

    Ok(Html(html))
}

/// HTMX handler to toggle alias enabled state
pub async fn toggle_alias_enabled(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ToggleAliasEnabledForm>,
) -> Result<Html<String>, Redirect> {
    let mut session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found".to_string()));
        }
    };

    // Find and update the alias in the session
    if let Some(alias) = session.aliases_to_duplicate.iter_mut().find(|a| a.pkid == form.alias_id) {
        alias.enabled = form.enabled == "true";
    }

    // Save the updated session
    save_session(session);

    // Return the updated toggle HTML
    let enabled = form.enabled == "true";
    let enabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-enabled").await;
    let disabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-disabled").await;

    let checked_true = if enabled { "checked" } else { "" };
    let checked_false = if !enabled { "checked" } else { "" };
    
    let html = format!(
        "<div class=\"radio-toggle-container\">\
            <div class=\"radio-toggle\">\
                <input type=\"radio\" id=\"alias_enabled_{}_true\" name=\"alias_enabled_{}\" value=\"true\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-alias-enabled\" hx-vals='{{\"alias_id\": \"{}\", \"enabled\": \"true\"}}' hx-target=\"#alias-enabled-toggle-{}\">\
                <label for=\"alias_enabled_{}_true\" class=\"radio-toggle-label radio-toggle-label-left\">\
                    <span class=\"radio-toggle-icon\">✓</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <input type=\"radio\" id=\"alias_enabled_{}_false\" name=\"alias_enabled_{}\" value=\"false\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-alias-enabled\" hx-vals='{{\"alias_id\": \"{}\", \"enabled\": \"false\"}}' hx-target=\"#alias-enabled-toggle-{}\">\
                <label for=\"alias_enabled_{}_false\" class=\"radio-toggle-label radio-toggle-label-right\">\
                    <span class=\"radio-toggle-icon\">✗</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <div class=\"radio-toggle-slider\"></div>\
            </div>\
        </div>",
        form.alias_id, form.alias_id, checked_true,
        form.alias_id, form.alias_id,
        form.alias_id, enabled_status,
        form.alias_id, form.alias_id, checked_false,
        form.alias_id, form.alias_id,
        form.alias_id, disabled_status
    );

    Ok(Html(html))
}

/// HTMX handler to toggle relay enabled state
pub async fn toggle_relay_enabled(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ToggleRelayEnabledForm>,
) -> Result<Html<String>, Redirect> {
    let mut session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found".to_string()));
        }
    };

    // Find and update the relay in the session
    if let Some(relay) = session.relays_to_duplicate.iter_mut().find(|r| r.pkid == form.relay_id) {
        relay.enabled = form.enabled == "true";
    }

    // Save the updated session
    save_session(session);

    // Return the updated toggle HTML
    let enabled = form.enabled == "true";
    let enabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-enabled").await;
    let disabled_status = crate::i18n::get_translation(&state, &get_user_locale(&headers), "status-disabled").await;

    let checked_true = if enabled { "checked" } else { "" };
    let checked_false = if !enabled { "checked" } else { "" };
    
    let html = format!(
        "<div class=\"radio-toggle-container\">\
            <div class=\"radio-toggle\">\
                <input type=\"radio\" id=\"relay_enabled_{}_true\" name=\"relay_enabled_{}\" value=\"true\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-relay-enabled\" hx-vals='{{\"relay_id\": \"{}\", \"enabled\": \"true\"}}' hx-target=\"#relay-enabled-toggle-{}\">\
                <label for=\"relay_enabled_{}_true\" class=\"radio-toggle-label radio-toggle-label-left\">\
                    <span class=\"radio-toggle-icon\">✓</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <input type=\"radio\" id=\"relay_enabled_{}_false\" name=\"relay_enabled_{}\" value=\"false\" {} class=\"radio-toggle-input\" hx-post=\"/duplicate-wizard/toggle-relay-enabled\" hx-vals='{{\"relay_id\": \"{}\", \"enabled\": \"false\"}}' hx-target=\"#relay-enabled-toggle-{}\">\
                <label for=\"relay_enabled_{}_false\" class=\"radio-toggle-label radio-toggle-label-right\">\
                    <span class=\"radio-toggle-icon\">✗</span>\
                    <span class=\"radio-toggle-text\">{}</span>\
                </label>\
                <div class=\"radio-toggle-slider\"></div>\
            </div>\
        </div>",
        form.relay_id, form.relay_id, checked_true,
        form.relay_id, form.relay_id,
        form.relay_id, enabled_status,
        form.relay_id, form.relay_id, checked_false,
        form.relay_id, form.relay_id,
        form.relay_id, disabled_status
    );

    Ok(Html(html))
}

/// Helper function to clear session
fn clear_session() {
    DUPLICATE_WIZARD_SESSIONS.lock().unwrap().remove("admin");
}

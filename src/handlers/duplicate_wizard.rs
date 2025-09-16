use crate::{
    db,
    handlers::http_helpers::get_user_locale,
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
    let _locale = get_user_locale(&headers);
    let _translations = get_duplicate_wizard_translations(&state, &_locale).await;

    // Get available domains for selection
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let domains = match db::get_domains(&pool) {
        Ok(domains) => domains,
        Err(e) => {
            error!("Failed to get domains: {:?}", e);
            vec![]
        }
    };

    let _backups = match db::get_backups(&pool) {
        Ok(backups) => backups,
        Err(e) => {
            error!("Failed to get backups: {:?}", e);
            vec![]
        }
    };

    // Create session
    let session = DuplicateDomainSession {
        step: DuplicateWizardStep::DomainSelection,
        source_domain: None,
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
    };
    save_session(session);

    // For now, return a simple HTML form
    // TODO: Create proper template
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Duplicate Domain Wizard</title>
        </head>
        <body>
            <h1>Duplicate Domain Wizard</h1>
            <form method="post" action="/duplicate-wizard/domain-selection">
                <h2>Select Source Domain</h2>
                <select name="source_domain" required>
                    <option value="">Choose a domain to duplicate...</option>
                    {}
                </select>
                
                <h2>New Domain Configuration</h2>
                <label>
                    New Domain Name:
                    <input type="text" name="new_domain" required placeholder="new-domain.com" />
                </label>
                
                <label>
                    Transport:
                    <select name="transport">
                        <option value="virtual">Virtual</option>
                        <option value="smtp:mail.example.com">SMTP</option>
                    </select>
                </label>
                
                <label>
                    <input type="checkbox" name="enabled" checked />
                    Enable domain
                </label>
                
                <h2>What to Duplicate</h2>
                <label>
                    <input type="checkbox" name="duplicate_aliases" checked />
                    Duplicate aliases and destinations
                </label>
                
                <label>
                    <input type="checkbox" name="duplicate_relays" checked />
                    Duplicate relays
                </label>
                
                <button type="submit">Next: Review</button>
            </form>
        </body>
        </html>
        "#,
        domains
            .iter()
            .map(|d| format!("<option value=\"{}\">{}</option>", d.domain, d.domain))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Html(html)
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

    // Get source domain
    let source_domain = match db::get_domain_by_name(&pool, &form.source_domain) {
        Ok(domain) => domain,
        Err(_) => {
            // Try backup domains
            match db::get_backup_by_name(&pool, &form.source_domain) {
                Ok(backup) => Domain {
                    pkid: backup.pkid,
                    domain: backup.domain,
                    transport: backup.transport,
                    created: backup.created,
                    modified: backup.modified,
                    enabled: backup.enabled,
                },
                Err(_) => {
                    return Ok(Html("Source domain not found".to_string()));
                }
            }
        }
    };

    // Get aliases and relays to duplicate
    let aliases_to_duplicate = if form.duplicate_aliases {
        match db::get_aliases_for_domain(&pool, &source_domain.domain) {
            Ok(aliases) => aliases,
            Err(e) => {
                error!("Failed to get aliases: {:?}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    let relays_to_duplicate = if form.duplicate_relays {
        match get_relays_for_domain(&pool, &source_domain.domain).await {
            Ok(relays) => relays,
            Err(e) => {
                error!("Failed to get relays: {:?}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    // Update session
    let mut session = get_session().unwrap_or_else(|| DuplicateDomainSession {
        step: DuplicateWizardStep::DomainSelection,
        source_domain: None,
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
    });

    session.step = DuplicateWizardStep::Configuration;
    session.source_domain = Some(source_domain);
    session.new_domain = form.new_domain;
    session.transport = form.transport;
    session.enabled = form.enabled;
    session.duplicate_aliases = form.duplicate_aliases;
    session.duplicate_relays = form.duplicate_relays;
    session.aliases_to_duplicate = aliases_to_duplicate;
    session.relays_to_duplicate = relays_to_duplicate;

    save_session(session);

    // Redirect to review step
    Ok(review(State(state), headers).await)
}

/// Step 2: Review configuration
pub async fn review(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let _translations = get_duplicate_wizard_translations(&state, &locale).await;

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Html("No session found. Please start over.".to_string());
        }
    };

    // For now, return a simple HTML review page
    // TODO: Create proper template
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Review Duplication</title>
        </head>
        <body>
            <h1>Review Domain Duplication</h1>
            
            <h2>Source Domain</h2>
            <p><strong>Domain:</strong> {}</p>
            <p><strong>Transport:</strong> {}</p>
            <p><strong>Enabled:</strong> {}</p>
            
            <h2>New Domain Configuration</h2>
            <p><strong>Domain:</strong> {}</p>
            <p><strong>Transport:</strong> {}</p>
            <p><strong>Enabled:</strong> {}</p>
            
            <h2>Items to Duplicate</h2>
            <p><strong>Aliases:</strong> {} ({} items)</p>
            <p><strong>Relays:</strong> {} ({} items)</p>
            
            <form method="post" action="/duplicate-wizard/execute">
                <button type="submit" name="confirmed" value="true">Confirm and Duplicate</button>
                <button type="submit" name="confirmed" value="false">Cancel</button>
            </form>
        </body>
        </html>
        "#,
        session
            .source_domain
            .as_ref()
            .map(|d| d.domain.as_str())
            .unwrap_or("None"),
        session
            .source_domain
            .as_ref()
            .map(|d| d.transport_display())
            .unwrap_or("None".to_string()),
        session
            .source_domain
            .as_ref()
            .map(|d| d.enabled)
            .unwrap_or(false),
        session.new_domain,
        session.transport,
        session.enabled,
        session.duplicate_aliases,
        session.aliases_to_duplicate.len(),
        session.duplicate_relays,
        session.relays_to_duplicate.len(),
    );

    Html(html)
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

    // Return success page
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Duplication Complete</title>
        </head>
        <body>
            <h1>Domain Duplication Complete!</h1>
            <p>Successfully duplicated domain <strong>{}</strong> to <strong>{}</strong></p>
            <p><a href="/domains/{}">View New Domain</a></p>
            <p><a href="/domains">Back to Domains</a></p>
        </body>
        </html>
        "#,
        session.source_domain.as_ref().unwrap().domain,
        new_domain.domain,
        new_domain.pkid
    );

    Ok(Html(html))
}

/// Helper function to get relays for a domain
pub async fn get_relays_for_domain(
    pool: &DbPool,
    _domain: &str,
) -> Result<Vec<Relay>, diesel::result::Error> {
    // This is a simplified implementation
    // In practice, you'd need to determine how relays are associated with domains
    // For now, return all relays
    db::get_relays(pool)
}

/// Helper function to create duplicated domain
pub async fn create_duplicated_domain(
    pool: &DbPool,
    session: &DuplicateDomainSession,
) -> Result<Domain, diesel::result::Error> {
    use crate::models::NewDomain;

    // Create the new domain
    let new_domain = NewDomain {
        domain: session.new_domain.clone(),
        transport: Some(session.transport.clone()),
        enabled: session.enabled,
    };

    let created_domain = db::create_domain(pool, new_domain)?;

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

/// Helper function to get duplicate wizard translations
async fn get_duplicate_wizard_translations(
    _state: &AppState,
    _locale: &str,
) -> HashMap<String, String> {
    // TODO: Implement proper translation system
    let mut translations = HashMap::new();
    translations.insert(
        "duplicate-wizard-title".to_string(),
        "Duplicate Domain Wizard".to_string(),
    );
    translations.insert(
        "select-source-domain".to_string(),
        "Select Source Domain".to_string(),
    );
    translations.insert(
        "new-domain-configuration".to_string(),
        "New Domain Configuration".to_string(),
    );
    translations.insert(
        "what-to-duplicate".to_string(),
        "What to Duplicate".to_string(),
    );
    translations.insert(
        "review-duplication".to_string(),
        "Review Duplication".to_string(),
    );
    translations
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

/// Helper function to clear session
fn clear_session() {
    DUPLICATE_WIZARD_SESSIONS.lock().unwrap().remove("admin");
}

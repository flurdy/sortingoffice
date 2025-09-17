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
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
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
        new_domain: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        duplicate_aliases: true,
        duplicate_relays: true,
        aliases_to_duplicate: vec![],
        relays_to_duplicate: vec![],
    });

    session.step = DuplicateWizardStep::Configuration;
    session.source_domain = Some(source_domain.clone());
    session.new_domain = form.new_domain;
    session.transport = source_domain
        .transport
        .unwrap_or_else(|| "virtual".to_string()); // Always copy source domain's transport
    session.enabled = form.enabled;
    session.duplicate_aliases = true; // Always duplicate aliases
    session.duplicate_relays = true; // Always duplicate relays
    session.aliases_to_duplicate = aliases_to_duplicate;
    session.relays_to_duplicate = relays_to_duplicate;

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

use crate::{
    analytics::{find_database_common_aliases, find_most_common_destination},
    db,
    handlers::utils::get_user_locale,
    models::{
        AliasConfigForm, DomainConfigForm, DomainWizardData, DomainWizardSession,
        WizardConfirmForm, WizardStep, WizardSummary,
    },
    render_template_with_title,
    templates::wizard::{
        WizardAliasConfigTemplate, WizardCompleteTemplate, WizardDomainConfigTemplate,
        WizardReviewTemplate,
    },
    AppState,
};
use askama::Template;
use axum::{
    extract::{Form, Query, State},
    http::HeaderMap,
    response::Html,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::error;

#[derive(Deserialize)]
pub struct WizardDestinationSearchQuery {
    pub destination: Option<String>,
    pub limit: Option<i64>,
}

// Simple session storage using static HashMap
lazy_static! {
    static ref WIZARD_SESSIONS: Mutex<HashMap<String, DomainWizardSession>> =
        Mutex::new(HashMap::new());
}

// Helper function to get wizard translations
async fn get_wizard_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    crate::handlers::utils::get_translations_batch(
        state,
        locale,
        &[
            "wizard-title",
            "wizard-description",
            "wizard-start",
            "wizard-next",
            "wizard-back",
            "wizard-cancel",
            "wizard-confirm",
            "wizard-complete",
            "wizard-step-1-title",
            "wizard-step-1-description",
            "wizard-domains-label",
            "wizard-domains-placeholder",
            "wizard-transport-label",
            "wizard-transport-placeholder",
            "wizard-enabled-label",
            "wizard-step-2-title",
            "wizard-step-2-description",
            "wizard-required-aliases",
            "wizard-common-aliases",
            "wizard-custom-aliases",
            "wizard-custom-aliases-placeholder",
            "wizard-destination-label",
            "wizard-destination-placeholder",
            "wizard-step-3-title",
            "wizard-step-3-description",
            "wizard-summary-domains",
            "wizard-summary-aliases",
            "wizard-summary-total",
            "wizard-summary-destination",
            "wizard-step-4-title",
            "wizard-step-4-description",
            "wizard-progress-domains",
            "wizard-progress-aliases",
            "wizard-progress-complete",
            "wizard-step-5-title",
            "wizard-step-5-description",
            "wizard-results-success",
            "wizard-results-failed",
            "wizard-view-domains",
            "wizard-new-wizard",
            // Additional wizard translation keys
            "wizard-step-1-box-title",
            "wizard-step-1-box-description",
            "wizard-step-2-box-title",
            "wizard-step-2-box-description",
            "wizard-step-3-box-title",
            "wizard-step-3-box-description",
            "wizard-domains-to-configure",
            "wizard-custom-aliases-description",
            "wizard-catchall-title",
            "wizard-catchall-description",
            "wizard-destination-title",
            "wizard-destination-description",
            "wizard-setup-results",
            "wizard-domains-created",
            "wizard-aliases-created",
            "wizard-errors-title",
            "wizard-errors-description",
            "wizard-new-badge",
            // Additional domain config translations
            "wizard-domains-label",
            "wizard-domains-description",
            "wizard-transport-label",
            "wizard-transport-description",
            "wizard-enabled-description",
            "wizard-domain-status-label",
            "wizard-enabled-label",
            "wizard-disabled-label",
            // Additional review translations
            "wizard-configuration-summary-title",
            "wizard-domains-plural",
            "wizard-aliases-plural",
            "wizard-created-domains-title",
            // Additional executing translations
            "wizard-creating-domains-text",
            "wizard-creating-aliases-text",
            // Additional analytics-driven common aliases translations
            "wizard-analytics-common-aliases",
            "wizard-config-common-aliases",
        ],
    )
    .await
}

// Helper function to generate aliases from config
#[allow(dead_code)]
fn generate_aliases_from_config(config: &crate::Config) -> Vec<String> {
    let mut aliases = Vec::new();

    // Add required aliases
    aliases.extend(config.required_aliases.clone());

    // Add common aliases
    aliases.extend(config.common_aliases.clone());

    aliases
}

// Helper function to create a new wizard session
fn create_wizard_session() -> DomainWizardSession {
    DomainWizardSession {
        step: WizardStep::DomainConfig,
        domains: Vec::new(),
        common_aliases: Vec::new(),
        custom_aliases: Vec::new(),
        common_destination: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
        catchall_enabled: false,
    }
}

// Helper function to get session for a user (simplified - using admin as key)
fn get_session() -> Option<DomainWizardSession> {
    WIZARD_SESSIONS.lock().unwrap().get("admin").cloned()
}

// Helper function to save session for a user
fn save_session(session: DomainWizardSession) {
    WIZARD_SESSIONS
        .lock()
        .unwrap()
        .insert("admin".to_string(), session);
}

// Helper function to clear session
fn clear_session() {
    WIZARD_SESSIONS.lock().unwrap().remove("admin");
}

// Wizard index page
pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    // Clear any existing session when starting a new wizard
    clear_session();

    // Redirect directly to domain config step
    domain_config(State(state), headers).await
}

// Step 1: Domain configuration
pub async fn domain_config(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get existing session to restore form data
    let session = get_session();
    let form = if let Some(session) = session {
        // Restore domains from session
        let domains_str = session
            .domains
            .iter()
            .map(|d| d.domain.clone())
            .collect::<Vec<String>>()
            .join(", ");

        println!(
            "[WIZARD DEBUG] Restoring session with domains: {:?}",
            session.domains
        );
        println!("[WIZARD DEBUG] Restored domains string: '{}'", domains_str);

        DomainConfigForm {
            domains: domains_str,
            transport: session.transport.clone(),
            enabled: session.enabled,
        }
    } else {
        println!("[WIZARD DEBUG] No session found, using default form");
        DomainConfigForm {
            domains: String::new(),
            transport: "virtual:".to_string(),
            enabled: true,
        }
    };

    let content_template = WizardDomainConfigTemplate {
        title: &translations["wizard-step-1-title"],
        description: &translations["wizard-step-1-description"],
        form: &form,
        error: "",
        domains_label: &translations["wizard-domains-label"],
        domains_description: &translations["wizard-domains-description"],
        domains_placeholder: &translations["wizard-domains-placeholder"],
        transport_label: &translations["wizard-transport-label"],
        transport_description: &translations["wizard-transport-description"],
        transport_placeholder: &translations["wizard-transport-placeholder"],
        enabled_description: &translations["wizard-enabled-description"],
        domain_status_label: &translations["wizard-domain-status-label"],
        enabled_label: &translations["wizard-enabled-label"],
        disabled_label: &translations["wizard-disabled-label"],
        next_button: &translations["wizard-next"],
        cancel_button: &translations["wizard-cancel"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

// Handle domain configuration form submission
pub async fn domain_config_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DomainConfigForm>,
) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Parse domains from comma-separated string
    let domains: Vec<String> = form
        .domains
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if domains.is_empty() {
        let error_msg = "Please enter at least one domain";
        let content_template = WizardDomainConfigTemplate {
            title: &translations["wizard-step-1-title"],
            description: &translations["wizard-step-1-description"],
            form: &form,
            error: &error_msg,
            domains_label: &translations["wizard-domains-label"],
            domains_description: &translations["wizard-domains-description"],
            domains_placeholder: &translations["wizard-domains-placeholder"],
            transport_label: &translations["wizard-transport-label"],
            transport_description: &translations["wizard-transport-description"],
            transport_placeholder: &translations["wizard-transport-placeholder"],
            enabled_description: &translations["wizard-enabled-description"],
            domain_status_label: &translations["wizard-domain-status-label"],
            enabled_label: &translations["wizard-enabled-label"],
            disabled_label: &translations["wizard-disabled-label"],
            next_button: &translations["wizard-next"],
            cancel_button: &translations["wizard-cancel"],
        };

        return render_template_with_title!(
            content_template,
            content_template.title,
            &state,
            &locale,
            &headers
        );
    }

    // Comprehensive domain validation
    for domain in &domains {
        match crate::validation::validate_domain(domain) {
            Ok(_) => {}
            Err(e) => {
                let error_msg = format!("Invalid domain '{}': {}", domain, e);
                let content_template = WizardDomainConfigTemplate {
                    title: &translations["wizard-step-1-title"],
                    description: &translations["wizard-step-1-description"],
                    form: &form,
                    error: &error_msg,
                    domains_label: &translations["wizard-domains-label"],
                    domains_description: &translations["wizard-domains-description"],
                    domains_placeholder: &translations["wizard-domains-placeholder"],
                    transport_label: &translations["wizard-transport-label"],
                    transport_description: &translations["wizard-transport-description"],
                    transport_placeholder: &translations["wizard-transport-placeholder"],
                    enabled_description: &translations["wizard-enabled-description"],
                    domain_status_label: &translations["wizard-domain-status-label"],
                    enabled_label: &translations["wizard-enabled-label"],
                    disabled_label: &translations["wizard-disabled-label"],
                    next_button: &translations["wizard-next"],
                    cancel_button: &translations["wizard-cancel"],
                };

                return render_template_with_title!(
                    content_template,
                    content_template.title,
                    &state,
                    &locale,
                    &headers
                );
            }
        }
    }

    // Create wizard session with domain data
    let mut session = create_wizard_session();
    session.step = WizardStep::AliasConfig;
    session.transport = form.transport;
    session.enabled = form.enabled;

    // Add domains to session
    for domain in domains {
        session.domains.push(DomainWizardData {
            domain,
            transport: None, // Use common transport
            enabled: session.enabled,
            aliases: Vec::new(),
        });
    }

    // Don't pre-populate common aliases - let user choose in alias config step
    session.common_aliases = Vec::new();

    // Find the most common destination from existing aliases
    session.common_destination = find_most_common_destination(&state, &headers).await;

    // Save session
    save_session(session);

    // Redirect to alias configuration
    alias_config(State(state), headers).await
}

// Step 2: Alias configuration
pub async fn alias_config(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get session or create default
    let session = if let Some(session) = get_session() {
        session
    } else {
        let mut default_session = create_wizard_session();
        // Don't add a default domain - let user enter their own
        // Find the most common destination for new sessions
        let common_destination = find_most_common_destination(&state, &headers).await;
        default_session.common_destination = common_destination;
        default_session
    };

    let domains: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();
    let required_aliases = state.config.required_aliases.clone();

    // Get database-specific common aliases from analytics
    let analytics_common_aliases = find_database_common_aliases(&state, &headers, 10, 3).await;

    // Separate config common aliases from analytics-driven ones
    let config_common_aliases = state.config.common_aliases.clone();

    // Filter out analytics aliases that are already in required or config common aliases
    let filtered_analytics_aliases: Vec<String> = analytics_common_aliases
        .iter()
        .filter(|alias| !required_aliases.contains(alias) && !config_common_aliases.contains(alias))
        .cloned()
        .collect();

    // Combine all common aliases for the form
    let mut common_aliases = config_common_aliases.clone();
    for alias in &filtered_analytics_aliases {
        if !common_aliases.contains(alias) {
            common_aliases.push(alias.clone());
        }
    }

    // Restore form data from session if available
    println!(
        "[WIZARD DEBUG] Session restoration - common_destination: {:?}",
        session.common_destination
    );
    println!(
        "[WIZARD DEBUG] Session restoration - common_aliases: {:?}",
        session.common_aliases
    );

    let form = AliasConfigForm {
        required_aliases: if session.common_aliases.is_empty() {
            required_aliases.clone()
        } else {
            // Extract required aliases from session (they're mixed with common aliases)
            let config_required = state.config.required_aliases.clone();
            session
                .common_aliases
                .iter()
                .filter(|alias| config_required.contains(alias))
                .cloned()
                .collect()
        },
        common_aliases: if session.common_aliases.is_empty() {
            // Default: no common aliases are checked
            Vec::new()
        } else {
            // Extract ALL non-required aliases from session (including analytics-driven ones)
            let config_required = state.config.required_aliases.clone();
            session
                .common_aliases
                .iter()
                .filter(|alias| !config_required.contains(alias))
                .cloned()
                .collect()
        },
        custom_aliases: session.custom_aliases.clone(),
        common_destination: session.common_destination.clone(),
        alias_destinations: HashMap::new(),
        catchall_enabled: session.catchall_enabled,
    };

    let content_template = WizardAliasConfigTemplate {
        title: &translations["wizard-step-2-title"],
        description: &translations["wizard-step-2-description"],
        domains: &domains,
        form: &form,
        error: "",
        required_aliases: &required_aliases,
        common_aliases: &common_aliases,
        analytics_common_aliases: &filtered_analytics_aliases,
        config_common_aliases: &config_common_aliases,
        required_aliases_label: &translations["wizard-required-aliases"],
        common_aliases_label: &translations["wizard-common-aliases"],
        analytics_common_aliases_label: &translations["wizard-analytics-common-aliases"],
        config_common_aliases_label: &translations["wizard-config-common-aliases"],
        custom_aliases_label: &translations["wizard-custom-aliases"],
        custom_aliases_placeholder: &translations["wizard-custom-aliases-placeholder"],
        custom_aliases_description: &translations["wizard-custom-aliases-description"],
        catchall_title: &translations["wizard-catchall-title"],
        catchall_description: &translations["wizard-catchall-description"],
        destination_title: &translations["wizard-destination-title"],
        destination_description: &translations["wizard-destination-description"],
        destination_placeholder: &translations["wizard-destination-placeholder"],
        domains_to_configure_label: &translations["wizard-domains-to-configure"],
        next_button: &translations["wizard-next"],
        back_button: &translations["wizard-back"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

// Step 2: Alias configuration POST handler
pub async fn alias_config_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
) -> Html<String> {
    // Parse form data manually to handle duplicate field names
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    let body_string = String::from_utf8_lossy(&body_bytes);
    println!("[WIZARD DEBUG] Raw form data: {}", body_string);

    // Parse form data manually
    let mut required_aliases = Vec::new();
    let mut common_aliases = Vec::new();
    let mut custom_aliases = Vec::new();
    let mut common_destination = String::new();
    let mut catchall_enabled = false;

    for pair in body_string.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // Use a simpler URL decoding approach
            let decoded_value = value
                .replace("%40", "@")
                .replace("%20", " ")
                .replace("+", " ");

            match key {
                "required_aliases" => required_aliases.push(decoded_value),
                "common_aliases" => common_aliases.push(decoded_value),
                "custom_aliases" => {
                    // Only add non-empty custom aliases
                    if !decoded_value.is_empty() {
                        custom_aliases.push(decoded_value);
                    }
                }
                "common_destination" => common_destination = decoded_value,
                "catchall_enabled" => catchall_enabled = decoded_value == "on",
                _ => {}
            }
        }
    }

    let form = AliasConfigForm {
        required_aliases,
        common_aliases,
        custom_aliases,
        common_destination,
        alias_destinations: HashMap::new(),
        catchall_enabled,
    };

    // Debug logging
    println!("[WIZARD DEBUG] Parsed form: {:?}", form);

    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get current session
    let mut session = get_session().unwrap_or_else(|| create_wizard_session());

    // Validate form data
    if form.common_destination.is_empty() {
        let error_msg = "Please enter a valid destination";
        let domains: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();
        // Get analytics-driven common aliases for error template
        let analytics_common_aliases = find_database_common_aliases(&state, &headers, 10, 3).await;
        let config_common_aliases = state.config.common_aliases.clone();

        // Filter out analytics aliases that are already in required or config common aliases
        let filtered_analytics_aliases: Vec<String> = analytics_common_aliases
            .iter()
            .filter(|alias| {
                !state.config.required_aliases.contains(alias)
                    && !config_common_aliases.contains(alias)
            })
            .cloned()
            .collect();

        let content_template = WizardAliasConfigTemplate {
            title: &translations["wizard-step-2-title"],
            description: &translations["wizard-step-2-description"],
            domains: &domains,
            form: &form,
            error: error_msg,
            required_aliases: &state.config.required_aliases,
            common_aliases: &state.config.common_aliases,
            analytics_common_aliases: &filtered_analytics_aliases,
            config_common_aliases: &config_common_aliases,
            required_aliases_label: &translations["wizard-required-aliases"],
            common_aliases_label: &translations["wizard-common-aliases"],
            analytics_common_aliases_label: &translations["wizard-analytics-common-aliases"],
            config_common_aliases_label: &translations["wizard-config-common-aliases"],
            custom_aliases_label: &translations["wizard-custom-aliases"],
            custom_aliases_placeholder: &translations["wizard-custom-aliases-placeholder"],
            custom_aliases_description: &translations["wizard-custom-aliases-description"],
            catchall_title: &translations["wizard-catchall-title"],
            catchall_description: &translations["wizard-catchall-description"],
            destination_title: &translations["wizard-destination-title"],
            destination_description: &translations["wizard-destination-description"],
            destination_placeholder: &translations["wizard-destination-placeholder"],
            domains_to_configure_label: &translations["wizard-domains-to-configure"],
            next_button: &translations["wizard-next"],
            back_button: &translations["wizard-back"],
        };

        return render_template_with_title!(
            content_template,
            content_template.title,
            &state,
            &locale,
            &headers
        );
    }

    // Update session with form data
    session.common_destination = form.common_destination;
    session.common_aliases = form
        .required_aliases
        .iter()
        .chain(form.common_aliases.iter())
        .cloned()
        .collect();
    session.custom_aliases = form.custom_aliases.clone();
    session.catchall_enabled = form.catchall_enabled;
    session.step = WizardStep::Review;
    save_session(session);

    // Redirect to review page
    review(State(state), headers).await
}

// Step 3: Review and confirmation
pub async fn review(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get session
    let session = match get_session() {
        Some(session) => session,
        None => {
            // No session found - this shouldn't happen in normal wizard flow
            // Redirect to wizard start
            return index(State(state), headers).await;
        }
    };

    let domains_list: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();

    // Generate aliases list from session data, grouped by domain and sorted
    let mut aliases_list = Vec::new();

    for domain in &session.domains {
        // Add required and common aliases
        for alias in &session.common_aliases {
            aliases_list.push(format!("{}@{}", alias, domain.domain));
        }
        // Add custom aliases
        for alias in &session.custom_aliases {
            if !alias.is_empty() {
                aliases_list.push(format!("{}@{}", alias, domain.domain));
            }
        }
        // Add catchall alias if enabled
        if session.catchall_enabled {
            aliases_list.push(format!("@{}", domain.domain)); // Just @domain for catchall
        }
    }

    // Sort aliases alphabetically
    aliases_list.sort();

    let summary = WizardSummary {
        total_domains: domains_list.len() as i32,
        total_aliases: aliases_list.len() as i32,
        domains_list,
        aliases_list,
    };

    let content_template = WizardReviewTemplate {
        title: &translations["wizard-step-3-title"],
        description: &translations["wizard-step-3-description"],
        session: &session,
        summary: &summary,
        configuration_summary_title: &translations["wizard-configuration-summary-title"],
        summary_domains_label: &translations["wizard-summary-domains"],
        summary_aliases_label: &translations["wizard-summary-aliases"],
        summary_total_label: &translations["wizard-summary-total"],
        destination_label: &translations["wizard-summary-destination"],
        domains_plural: &translations["wizard-domains-plural"],
        aliases_plural: &translations["wizard-aliases-plural"],
        new_badge: &translations["wizard-new-badge"],
        confirm_button: &translations["wizard-confirm"],
        back_button: &translations["wizard-back"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

// Step 4: Execute wizard
pub async fn execute(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<WizardConfirmForm>,
) -> Html<String> {
    let locale = get_user_locale(&headers);
    let _translations = get_wizard_translations(&state, &locale).await;

    if !form.confirmed {
        // User didn't confirm, go back to review
        return review(State(state), headers).await;
    }

    // Get session
    let mut session = match get_session() {
        Some(session) => session,
        None => {
            // No session found - this shouldn't happen in normal wizard flow
            // Return an error page or redirect to start
            return review(State(state), headers).await;
        }
    };

    // Update session step
    session.step = WizardStep::Executing;
    save_session(session.clone());

    // Actually create domains and aliases in database
    let mut _domains_created = 0;
    let mut _aliases_created = 0;
    let mut successfully_created_domains = Vec::new();

    for domain_data in &session.domains {
        // Create domain
        let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("[WIZARD DEBUG] Failed to get database pool: {:?}", e);
                continue;
            }
        };

        let new_domain = crate::models::NewDomain {
            domain: domain_data.domain.clone(),
            transport: Some(session.transport.clone()),
            enabled: session.enabled,
        };

        match db::create_domain(&pool, new_domain) {
            Ok(_) => {
                _domains_created += 1;
                successfully_created_domains.push(domain_data.domain.clone());
                println!("[WIZARD DEBUG] Created domain: {}", domain_data.domain);
            }
            Err(e) => {
                println!(
                    "[WIZARD DEBUG] Failed to create domain {}: {:?}",
                    domain_data.domain, e
                );
            }
        }

        // Create aliases for this domain
        let mut aliases_to_create = Vec::new();

        // Add all selected aliases from session (including analytics-driven ones)
        for alias in &session.common_aliases {
            aliases_to_create.push(format!("{}@{}", alias, domain_data.domain));
        }

        // Add custom aliases
        for alias in &session.custom_aliases {
            if !alias.is_empty() {
                aliases_to_create.push(format!("{}@{}", alias, domain_data.domain));
            }
        }

        // Add catchall alias if enabled
        if session.catchall_enabled {
            aliases_to_create.push(format!("@{}", domain_data.domain));
        }

        // Remove duplicates
        aliases_to_create.sort();
        aliases_to_create.dedup();

        // Create each alias
        for alias in aliases_to_create {
            let alias_form = crate::models::AliasForm {
                mail: alias.clone(),
                destination: session.common_destination.clone(),
                enabled: true,
                return_url: None,
            };

            match db::create_alias(&pool, alias_form) {
                Ok(_) => {
                    _aliases_created += 1;
                    println!(
                        "[WIZARD DEBUG] Created alias: {} -> {}",
                        alias, session.common_destination
                    );
                }
                Err(e) => {
                    println!("[WIZARD DEBUG] Failed to create alias {}: {:?}", alias, e);
                }
            }
        }
    }

    // Update session to only contain successfully created domains
    session.step = WizardStep::Complete;
    session.domains = successfully_created_domains
        .into_iter()
        .map(|domain| DomainWizardData {
            domain,
            transport: Some(session.transport.clone()),
            enabled: session.enabled,
            aliases: Vec::new(),
        })
        .collect();
    save_session(session);

    // Redirect to complete page
    complete(State(state), headers).await
}

// Step 5: Complete
pub async fn complete(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get session
    let session = match get_session() {
        Some(session) => session,
        None => {
            // No session found - this shouldn't happen in normal wizard flow
            // Redirect to wizard start
            return index(State(state), headers).await;
        }
    };

    // Calculate total aliases that should have been created
    let mut total_aliases = 0;
    for _domain_data in &session.domains {
        // Count ALL selected aliases from session (including analytics-driven ones)
        total_aliases += session.common_aliases.len() as i32;

        // Count custom aliases
        for alias in &session.custom_aliases {
            if !alias.is_empty() {
                total_aliases += 1;
            }
        }
        // Count catchall if enabled
        if session.catchall_enabled {
            total_aliases += 1;
        }
    }

    // Extract domain names from session
    let created_domains: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();

    // Look up domain IDs by name
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let mut created_domain_ids = Vec::new();
    for domain_name in &created_domains {
        match db::get_domain_by_name(&pool, domain_name) {
            Ok(domain) => created_domain_ids.push(domain.pkid),
            Err(e) => {
                error!("Failed to get domain ID for {}: {:?}", domain_name, e);
                // If we can't find the domain, skip it
                continue;
            }
        }
    }

    let content_template = WizardCompleteTemplate {
        title: &translations["wizard-step-5-title"],
        description: &translations["wizard-step-5-description"],
        domains_created: session.domains.len() as i32,
        aliases_created: total_aliases,
        has_errors: false,
        created_domains: &created_domains,
        created_domain_ids: &created_domain_ids,
        setup_results_title: &translations["wizard-setup-results"],
        domains_created_label: &translations["wizard-domains-created"],
        aliases_created_label: &translations["wizard-aliases-created"],
        domains_plural: &translations["wizard-domains-plural"],
        created_domains_title: &translations["wizard-created-domains-title"],
        errors_title: &translations["wizard-errors-title"],
        errors_description: &translations["wizard-errors-description"],
        view_domains_button: &translations["wizard-view-domains"],
        new_wizard_button: &translations["wizard-new-wizard"],
    };

    // Clear session after completion
    clear_session();

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

// Wizard destination search endpoint
pub async fn destination_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WizardDestinationSearchQuery>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    // Get the query string
    let query_string = query.destination.unwrap_or_default();

    // Handle empty or missing query
    if query_string.len() < 2 {
        let locale = crate::handlers::utils::get_user_locale(&headers);
        let translations = crate::handlers::utils::get_translations_batch(
            &state,
            &locale,
            &["aliases-search-no-results", "aliases-search-select"],
        )
        .await;
        let html = format!(
            "<ul><li class=\"text-gray-400\">{}</li></ul>",
            translations["aliases-search-no-results"]
        );
        return Html(html);
    }

    let limit = query.limit.unwrap_or(10);

    // --- Collect all matching values from aliases and users ---
    let mut values = std::collections::HashSet::new();

    // 1. Alias mail and destination
    if let Ok(aliases) = db::search_aliases(&pool, &query_string, limit * 2) {
        for alias in aliases {
            if alias.mail.contains(&query_string) {
                values.insert(alias.mail);
            }
            if alias.destination.contains(&query_string) {
                values.insert(alias.destination);
            }
        }
    }

    // 2. User ids
    use diesel::prelude::*;
    if let Ok(mut conn) = pool.get() {
        let search_pattern = format!("%{query_string}%");
        let user_ids: Vec<String> = crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::id.like(&search_pattern))
            .select(crate::schema::users::dsl::id)
            .limit(limit * 2)
            .load::<String>(&mut conn)
            .unwrap_or_default();
        for user_id in user_ids {
            values.insert(user_id);
        }
    }

    // 3. Sort and limit
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort_by_key(|a| a.to_lowercase());
    values.truncate(limit as usize);

    // 4. Render as a flat list of suggestions
    let html = if values.is_empty() {
        let locale = crate::handlers::utils::get_user_locale(&headers);
        let translations = crate::handlers::utils::get_translations_batch(
            &state,
            &locale,
            &["aliases-search-no-results", "aliases-search-select"],
        )
        .await;
        format!(
            "<ul><li class=\"text-gray-400\">{}</li></ul>",
            translations["aliases-search-no-results"]
        )
    } else {
        let items: String = values
            .into_iter()
            .map(|v| format!("<li class=\"cursor-pointer\">{v}</li>"))
            .collect();
        format!("<ul>{items}</ul>")
    };

    Html(html)
}

// Helper function to execute wizard creation

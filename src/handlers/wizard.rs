use crate::{
    handlers::utils::get_user_locale,
    models::{
        AliasConfigForm, DomainConfigForm, DomainWizardData, DomainWizardSession,
        WizardConfirmForm, WizardStep, WizardSummary,
    },
    render_template_with_title,
    templates::wizard::{
        WizardAliasConfigTemplate, WizardCompleteTemplate, WizardDomainConfigTemplate,
        WizardIndexTemplate, WizardReviewTemplate,
    },
    AppState,
};
use askama::Template;
use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::Html,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

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
            "wizard-destination-label",
            "wizard-destination-placeholder",
            "wizard-step-3-title",
            "wizard-step-3-description",
            "wizard-summary-domains",
            "wizard-summary-aliases",
            "wizard-summary-total",
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
        ],
    )
    .await
}



// Helper function to generate aliases from config
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
        common_destination: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
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
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    let content_template = WizardIndexTemplate {
        title: &translations["wizard-title"],
        description: &translations["wizard-description"],
        start_button: &translations["wizard-start"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

// Step 1: Domain configuration
pub async fn domain_config(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    let form = DomainConfigForm {
        domains: String::new(),
        transport: "virtual".to_string(),
        enabled: true,
    };

    let content_template = WizardDomainConfigTemplate {
        title: &translations["wizard-step-1-title"],
        description: &translations["wizard-step-1-description"],
        form: &form,
        error: "",
        domains_placeholder: &translations["wizard-domains-placeholder"],
        transport_placeholder: &translations["wizard-transport-placeholder"],
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
    let domains: Vec<String> = form.domains
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
            domains_placeholder: &translations["wizard-domains-placeholder"],
            transport_placeholder: &translations["wizard-transport-placeholder"],
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

    // Basic domain validation
    for domain in &domains {
        if !domain.contains('.') {
            let error_msg = format!("Invalid domain format: {}", domain);
            let content_template = WizardDomainConfigTemplate {
                title: &translations["wizard-step-1-title"],
                description: &translations["wizard-step-1-description"],
                form: &form,
                error: &error_msg,
                domains_placeholder: &translations["wizard-domains-placeholder"],
                transport_placeholder: &translations["wizard-transport-placeholder"],
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

    // Generate common aliases from config
    session.common_aliases = generate_aliases_from_config(&state.config);
    session.common_destination = "admin@example.com".to_string(); // Default

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
    let session = get_session().unwrap_or_else(|| {
        let mut default_session = create_wizard_session();
        default_session.domains.push(DomainWizardData {
            domain: "example.com".to_string(),
            transport: None,
            enabled: true,
            aliases: Vec::new(),
        });
        default_session
    });

    let domains: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();
    let required_aliases = state.config.required_aliases.clone();
    let common_aliases = state.config.common_aliases.clone();

    let form = AliasConfigForm {
        required_aliases: required_aliases.clone(),
        common_aliases: common_aliases.clone(),
        custom_aliases: Vec::new(),
        common_destination: session.common_destination.clone(),
        alias_destinations: HashMap::new(),
        catchall_enabled: false,
    };

    let content_template = WizardAliasConfigTemplate {
        title: &translations["wizard-step-2-title"],
        description: &translations["wizard-step-2-description"],
        domains: &domains,
        form: &form,
        error: "",
        required_aliases: &required_aliases,
        common_aliases: &common_aliases,
        required_aliases_label: &translations["wizard-required-aliases"],
        common_aliases_label: &translations["wizard-common-aliases"],
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
    Form(form): Form<AliasConfigForm>,
) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get current session
    let mut session = get_session().unwrap_or_else(|| {
        let mut default_session = create_wizard_session();
        default_session.domains.push(DomainWizardData {
            domain: "example.com".to_string(),
            transport: None,
            enabled: true,
            aliases: Vec::new(),
        });
        default_session
    });

    // Validate form data
    if form.common_destination.is_empty() {
        let error_msg = "Please enter a valid destination";
        let domains: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();
        let content_template = WizardAliasConfigTemplate {
            title: &translations["wizard-step-2-title"],
            description: &translations["wizard-step-2-description"],
            domains: &domains,
            form: &form,
            error: error_msg,
            required_aliases: &state.config.required_aliases,
            common_aliases: &state.config.common_aliases,
            required_aliases_label: &translations["wizard-required-aliases"],
            common_aliases_label: &translations["wizard-common-aliases"],
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
    let session = get_session().unwrap_or_else(|| {
        let mut default_session = create_wizard_session();
        default_session.domains.push(DomainWizardData {
            domain: "example.com".to_string(),
            transport: None,
            enabled: true,
            aliases: Vec::new(),
        });
        default_session
    });

    let domains_list: Vec<String> = session.domains.iter().map(|d| d.domain.clone()).collect();
    let aliases_list = vec![
        "postmaster@example.com".to_string(),
        "admin@example.com".to_string(),
        "webmaster@example.com".to_string(),
    ];

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
        summary_domains_label: &translations["wizard-summary-domains"],
        summary_aliases_label: &translations["wizard-summary-aliases"],
        summary_total_label: &translations["wizard-summary-total"],
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
    let mut session = get_session().unwrap_or_else(|| {
        let mut default_session = create_wizard_session();
        default_session.domains.push(DomainWizardData {
            domain: "example.com".to_string(),
            transport: None,
            enabled: true,
            aliases: Vec::new(),
        });
        default_session
    });

    // Update session step
    session.step = WizardStep::Executing;
    save_session(session.clone());

    // TODO: Actually create domains and aliases in database
    // For now, just simulate success

    // Update session to complete
    session.step = WizardStep::Complete;
    save_session(session);

    // Redirect to complete page
    complete(State(state), headers).await
}

// Step 5: Complete
pub async fn complete(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);
    let translations = get_wizard_translations(&state, &locale).await;

    // Get session
    let session = get_session().unwrap_or_else(|| {
        let mut default_session = create_wizard_session();
        default_session.domains.push(DomainWizardData {
            domain: "example.com".to_string(),
            transport: None,
            enabled: true,
            aliases: Vec::new(),
        });
        default_session
    });

    let content_template = WizardCompleteTemplate {
        title: &translations["wizard-step-5-title"],
        description: &translations["wizard-step-5-description"],
        domains_created: session.domains.len() as i32,
        aliases_created: 5, // TODO: Calculate actual aliases created
        has_errors: false,
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

// Helper function to execute wizard creation

use crate::templates::contact::ContactTemplate;
use crate::templates::layout::BaseTemplate;
use crate::AppState;

use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::translations::get_translations_batch(
        &state,
        &locale,
        &[
            "contact-title",
            "contact-subtitle",
            "contact-app-maintainer-title",
            "contact-app-maintainer-desc",
            "contact-project-maintainer-title",
            "contact-project-maintainer-desc",
            "contact-name",
            "contact-email",
            "contact-role",
            "contact-github-issues",
            "contact-github-issues-desc",
            "contact-security-advisories",
            "contact-security-advisories-desc",
            "contact-contact-form",
            "contact-contact-form-desc",
            "contact-pgp-keys",
            "contact-pgp-keys-desc",
            "contact-documentation",
            "contact-documentation-desc",
            "contact-sponsorship",
            "contact-sponsorship-desc",
        ],
    )
    .await;

    let content_template = ContactTemplate {
        title: &form_translations["contact-title"],
        subtitle: &form_translations["contact-subtitle"],
        app_maintainer_title: &form_translations["contact-app-maintainer-title"],
        app_maintainer_desc: &form_translations["contact-app-maintainer-desc"],
        project_maintainer_title: &form_translations["contact-project-maintainer-title"],
        project_maintainer_desc: &form_translations["contact-project-maintainer-desc"],
        contact_name: &form_translations["contact-name"],
        contact_email: &form_translations["contact-email"],
        contact_role: &form_translations["contact-role"],
        github_issues: &form_translations["contact-github-issues"],
        github_issues_desc: &form_translations["contact-github-issues-desc"],
        security_advisories: &form_translations["contact-security-advisories"],
        security_advisories_desc: &form_translations["contact-security-advisories-desc"],
        contact_form: &form_translations["contact-contact-form"],
        contact_form_desc: &form_translations["contact-contact-form-desc"],
        pgp_keys: &form_translations["contact-pgp-keys"],
        pgp_keys_desc: &form_translations["contact-pgp-keys-desc"],
        documentation: &form_translations["contact-documentation"],
        documentation_desc: &form_translations["contact-documentation-desc"],
        sponsorship: &form_translations["contact-sponsorship"],
        sponsorship_desc: &form_translations["contact-sponsorship-desc"],
        app_contact: state.config.contact.clone(),
    };
    let content = match crate::handlers::templates::render_template_safely(content_template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    };

    // Get current database id from session/cookie or default
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    // Get current database label from db_manager
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    let template = BaseTemplate::with_i18n(
        form_translations["contact-title"].clone(),
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    .unwrap();

    match crate::handlers::templates::render_template_safely(template) {
        Ok(content) => Html(content),
        Err(_) => crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

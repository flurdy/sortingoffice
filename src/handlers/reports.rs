use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};
use crate::{AppState, db, i18n::get_translation};
use crate::templates::layout::BaseTemplate;
use crate::templates::reports::ReportsTemplate;
use askama::Template;

pub async fn catch_all_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    
    // Get translations
    let title = get_translation(&state, &locale, "reports-catch-all-title").await;
    let description = get_translation(&state, &locale, "reports-catch-all-description").await;
    let domain_header = get_translation(&state, &locale, "reports-domain-header").await;
    let catch_all_header = get_translation(&state, &locale, "reports-catch-all-header").await;
    let destination_header = get_translation(&state, &locale, "reports-destination-header").await;
    let required_aliases_header = get_translation(&state, &locale, "reports-required-aliases-header").await;
    let missing_aliases_header = get_translation(&state, &locale, "reports-missing-aliases-header").await;
    let missing_required_aliases_header = get_translation(&state, &locale, "reports-missing-required-aliases-header").await;
    let missing_common_aliases_header = get_translation(&state, &locale, "reports-missing-common-aliases-header").await;
    let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
    let status_header = get_translation(&state, &locale, "reports-status-header").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let no_catch_all_domains = get_translation(&state, &locale, "reports-no-catch-all-domains").await;
    let no_catch_all_domains_description = get_translation(&state, &locale, "reports-no-catch-all-domains-description").await;
    let no_required_aliases = get_translation(&state, &locale, "reports-no-required-aliases").await;
    let no_missing_aliases = get_translation(&state, &locale, "reports-no-missing-aliases").await;
    let domains_with_catch_all_header = get_translation(&state, &locale, "reports-domains-with-catch-all-header").await;
    let domains_without_catch_all_header = get_translation(&state, &locale, "reports-domains-without-catch-all-header").await;

    // Get enhanced alias report data
    let report = match db::get_alias_report(&state.pool) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error getting alias report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the reports template
    let content_template = ReportsTemplate {
        title: &title,
        description: &description,
        domain_header: &domain_header,
        catch_all_header: &catch_all_header,
        destination_header: &destination_header,
        required_aliases_header: &required_aliases_header,
        missing_aliases_header: &missing_aliases_header,
        missing_required_aliases_header: &missing_required_aliases_header,
        missing_common_aliases_header: &missing_common_aliases_header,
        mail_header: &mail_header,
        status_header: &status_header,
        status_active: &status_active,
        status_inactive: &status_inactive,
        no_catch_all_domains: &no_catch_all_domains,
        no_catch_all_domains_description: &no_catch_all_domains_description,
        no_required_aliases: &no_required_aliases,
        no_missing_aliases: &no_missing_aliases,
        domains_with_catch_all_header: &domains_with_catch_all_header,
        domains_without_catch_all_header: &domains_without_catch_all_header,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering reports template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let template = match BaseTemplate::with_i18n(title, content, &state, &locale).await {
        Ok(template) => template,
        Err(e) => {
            tracing::error!("Error creating base template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match template.render() {
        Ok(content) => Ok(Html(content)),
        Err(e) => {
            tracing::error!("Error rendering final template: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

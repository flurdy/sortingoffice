use crate::templates::about::AboutTemplate;
use crate::templates::layout::BaseTemplate;
use crate::AppState;

use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "about-title",
            "about-subtitle",
            "about-what-is-title",
            "about-what-is-p1",
            "about-what-is-p2",
            "about-features-title",
            "about-feature-domain-management",
            "about-feature-domain-management-desc",
            "about-feature-user-management",
            "about-feature-user-management-desc",
            "about-feature-alias-management",
            "about-feature-alias-management-desc",
            "about-feature-backup-configuration",
            "about-feature-backup-configuration-desc",
            "about-feature-statistics-dashboard",
            "about-feature-statistics-dashboard-desc",
            "about-feature-dark-mode-support",
            "about-feature-dark-mode-support-desc",
            "about-technology-stack-title",
            "about-backend",
            "about-backend-desc",
            "about-database",
            "about-database-desc",
            "about-frontend",
            "about-frontend-desc",
            "about-templating",
            "about-templating-desc",
            "about-mail-server",
            "about-mail-server-desc",
            "about-deployment",
            "about-deployment-desc",
            "about-based-on-flurdy-title",
            "about-based-on-flurdy-desc",
            "about-read-guide",
            "about-github-project-title",
            "about-open-source",
            "about-open-source-desc",
            "about-view-repository",
            "about-view-repository-desc",
            "about-report-issues",
            "about-report-issues-desc",
            "about-pull-requests",
            "about-pull-requests-desc",
            "about-readme",
            "about-readme-desc",
            "about-version-information",
            "about-project-details",
            "about-version",
            "about-license",
            "about-maintainer",
        ],
    )
    .await;

    let content_template = AboutTemplate {
        title: &form_translations["about-title"],
        subtitle: &form_translations["about-subtitle"],
        what_is_title: &form_translations["about-what-is-title"],
        what_is_p1: &form_translations["about-what-is-p1"],
        what_is_p2: &form_translations["about-what-is-p2"],
        features_title: &form_translations["about-features-title"],
        feature_domain_management: &form_translations["about-feature-domain-management"],
        feature_domain_management_desc: &form_translations["about-feature-domain-management-desc"],
        feature_user_management: &form_translations["about-feature-user-management"],
        feature_user_management_desc: &form_translations["about-feature-user-management-desc"],
        feature_alias_management: &form_translations["about-feature-alias-management"],
        feature_alias_management_desc: &form_translations["about-feature-alias-management-desc"],
        feature_backup_configuration: &form_translations["about-feature-backup-configuration"],
        feature_backup_configuration_desc: &form_translations
            ["about-feature-backup-configuration-desc"],
        feature_statistics_dashboard: &form_translations["about-feature-statistics-dashboard"],
        feature_statistics_dashboard_desc: &form_translations
            ["about-feature-statistics-dashboard-desc"],
        feature_dark_mode_support: &form_translations["about-feature-dark-mode-support"],
        feature_dark_mode_support_desc: &form_translations["about-feature-dark-mode-support-desc"],
        technology_stack_title: &form_translations["about-technology-stack-title"],
        backend: &form_translations["about-backend"],
        backend_desc: &form_translations["about-backend-desc"],
        database: &form_translations["about-database"],
        database_desc: &form_translations["about-database-desc"],
        frontend: &form_translations["about-frontend"],
        frontend_desc: &form_translations["about-frontend-desc"],
        templating: &form_translations["about-templating"],
        templating_desc: &form_translations["about-templating-desc"],
        mail_server: &form_translations["about-mail-server"],
        mail_server_desc: &form_translations["about-mail-server-desc"],
        deployment: &form_translations["about-deployment"],
        deployment_desc: &form_translations["about-deployment-desc"],
        based_on_flurdy_title: &form_translations["about-based-on-flurdy-title"],
        based_on_flurdy_desc: &form_translations["about-based-on-flurdy-desc"],
        read_guide: &form_translations["about-read-guide"],
        github_project_title: &form_translations["about-github-project-title"],
        open_source: &form_translations["about-open-source"],
        open_source_desc: &form_translations["about-open-source-desc"],
        view_repository: &form_translations["about-view-repository"],
        view_repository_desc: &form_translations["about-view-repository-desc"],
        report_issues: &form_translations["about-report-issues"],
        report_issues_desc: &form_translations["about-report-issues-desc"],
        pull_requests: &form_translations["about-pull-requests"],
        pull_requests_desc: &form_translations["about-pull-requests-desc"],
        readme: &form_translations["about-readme"],
        readme_desc: &form_translations["about-readme-desc"],
        version_information: &form_translations["about-version-information"],
        project_details: &form_translations["about-project-details"],
        version: &form_translations["about-version"],
        license: &form_translations["about-license"],
        maintainer: &form_translations["about-maintainer"],
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
        form_translations["about-title"].clone(),
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

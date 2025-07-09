use crate::templates::about::AboutTemplate;
use crate::templates::layout::BaseTemplate;
use crate::{i18n::get_translation, AppState};
use askama::Template;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    let content_template = AboutTemplate {
        title: &get_translation(&state, &locale, "about-title").await,
        subtitle: &get_translation(&state, &locale, "about-subtitle").await,
        what_is_title: &get_translation(&state, &locale, "about-what-is-title").await,
        what_is_p1: &get_translation(&state, &locale, "about-what-is-p1").await,
        what_is_p2: &get_translation(&state, &locale, "about-what-is-p2").await,
        features_title: &get_translation(&state, &locale, "about-features-title").await,
        feature_domain_management: &get_translation(
            &state,
            &locale,
            "about-feature-domain-management",
        )
        .await,
        feature_domain_management_desc: &get_translation(
            &state,
            &locale,
            "about-feature-domain-management-desc",
        )
        .await,
        feature_user_management: &get_translation(&state, &locale, "about-feature-user-management")
            .await,
        feature_user_management_desc: &get_translation(
            &state,
            &locale,
            "about-feature-user-management-desc",
        )
        .await,
        feature_alias_management: &get_translation(
            &state,
            &locale,
            "about-feature-alias-management",
        )
        .await,
        feature_alias_management_desc: &get_translation(
            &state,
            &locale,
            "about-feature-alias-management-desc",
        )
        .await,
        feature_backup_configuration: &get_translation(
            &state,
            &locale,
            "about-feature-backup-configuration",
        )
        .await,
        feature_backup_configuration_desc: &get_translation(
            &state,
            &locale,
            "about-feature-backup-configuration-desc",
        )
        .await,
        feature_statistics_dashboard: &get_translation(
            &state,
            &locale,
            "about-feature-statistics-dashboard",
        )
        .await,
        feature_statistics_dashboard_desc: &get_translation(
            &state,
            &locale,
            "about-feature-statistics-dashboard-desc",
        )
        .await,
        feature_dark_mode_support: &get_translation(
            &state,
            &locale,
            "about-feature-dark-mode-support",
        )
        .await,
        feature_dark_mode_support_desc: &get_translation(
            &state,
            &locale,
            "about-feature-dark-mode-support-desc",
        )
        .await,
        technology_stack_title: &get_translation(&state, &locale, "about-technology-stack-title")
            .await,
        backend: &get_translation(&state, &locale, "about-backend").await,
        backend_desc: &get_translation(&state, &locale, "about-backend-desc").await,
        database: &get_translation(&state, &locale, "about-database").await,
        database_desc: &get_translation(&state, &locale, "about-database-desc").await,
        frontend: &get_translation(&state, &locale, "about-frontend").await,
        frontend_desc: &get_translation(&state, &locale, "about-frontend-desc").await,
        templating: &get_translation(&state, &locale, "about-templating").await,
        templating_desc: &get_translation(&state, &locale, "about-templating-desc").await,
        mail_server: &get_translation(&state, &locale, "about-mail-server").await,
        mail_server_desc: &get_translation(&state, &locale, "about-mail-server-desc").await,
        deployment: &get_translation(&state, &locale, "about-deployment").await,
        deployment_desc: &get_translation(&state, &locale, "about-deployment-desc").await,
        based_on_flurdy_title: &get_translation(&state, &locale, "about-based-on-flurdy-title")
            .await,
        based_on_flurdy_desc: &get_translation(&state, &locale, "about-based-on-flurdy-desc").await,
        read_guide: &get_translation(&state, &locale, "about-read-guide").await,
        github_project_title: &get_translation(&state, &locale, "about-github-project-title").await,
        open_source: &get_translation(&state, &locale, "about-open-source").await,
        open_source_desc: &get_translation(&state, &locale, "about-open-source-desc").await,
        view_repository: &get_translation(&state, &locale, "about-view-repository").await,
        view_repository_desc: &get_translation(&state, &locale, "about-view-repository-desc").await,
        report_issues: &get_translation(&state, &locale, "about-report-issues").await,
        report_issues_desc: &get_translation(&state, &locale, "about-report-issues-desc").await,
        pull_requests: &get_translation(&state, &locale, "about-pull-requests").await,
        pull_requests_desc: &get_translation(&state, &locale, "about-pull-requests-desc").await,
        readme: &get_translation(&state, &locale, "about-readme").await,
        readme_desc: &get_translation(&state, &locale, "about-readme-desc").await,
        version_information: &get_translation(&state, &locale, "about-version-information").await,
        project_details: &get_translation(&state, &locale, "about-project-details").await,
        version: &get_translation(&state, &locale, "about-version").await,
        license: &get_translation(&state, &locale, "about-license").await,
        maintainer: &get_translation(&state, &locale, "about-maintainer").await,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "about-title").await,
        content,
        &state,
        &locale,
    )
    .await
    .unwrap();

    Html(template.render().unwrap())
}

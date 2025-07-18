use crate::templates::layout::BaseTemplate;
use crate::templates::reports::{
    AliasCrossDomainReportTemplate, CrossDatabaseMatrixReportTemplate,
    ExternalForwarderReportTemplate, MatrixReportTemplate, OrphanedReportTemplate,
    ReportsListTemplate,
};
use crate::{db, i18n::get_translation, AppState};
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};

pub async fn matrix_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-matrix-title").await;
    let description = get_translation(&state, &locale, "reports-matrix-description").await;
    let domain_header = get_translation(&state, &locale, "reports-domain-header").await;
    let catch_all_header = get_translation(&state, &locale, "reports-catch-all-header").await;
    let required_aliases_header =
        get_translation(&state, &locale, "reports-required-aliases-header").await;
    let status_present = get_translation(&state, &locale, "reports-status-present").await;
    let status_missing = get_translation(&state, &locale, "reports-status-missing").await;
    let status_disabled = get_translation(&state, &locale, "reports-status-disabled").await;
    let legend_title = get_translation(&state, &locale, "reports-legend-title").await;
    let no_domains = get_translation(&state, &locale, "reports-no-domains").await;
    let no_domains_description =
        get_translation(&state, &locale, "reports-no-domains-description").await;

    // Get matrix report data
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let report = match db::get_domain_alias_matrix_report(&pool) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating matrix report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the matrix report template
    let content_template = MatrixReportTemplate {
        title: &title,
        description: &description,
        domain_header: &domain_header,
        catch_all_header: &catch_all_header,
        required_aliases_header: &required_aliases_header,
        status_present: &status_present,
        status_missing: &status_missing,
        status_disabled: &status_disabled,
        legend_title: &legend_title,
        no_domains: &no_domains,
        no_domains_description: &no_domains_description,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering matrix report template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

pub async fn cross_database_domain_matrix_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-cross-db-matrix-title").await;
    let description = get_translation(&state, &locale, "reports-cross-db-matrix-description").await;
    let domain_header = get_translation(&state, &locale, "reports-domain-header").await;
    let database_header = get_translation(&state, &locale, "reports-database-header").await;
    let primary_domain = get_translation(&state, &locale, "reports-primary-domain").await;
    let backup_domain = get_translation(&state, &locale, "reports-backup-domain").await;
    let not_present = get_translation(&state, &locale, "reports-not-present").await;
    let legend_title = get_translation(&state, &locale, "reports-legend-title").await;
    let no_domains = get_translation(&state, &locale, "reports-no-domains").await;
    let no_domains_description =
        get_translation(&state, &locale, "reports-no-domains-description").await;

    // Get cross-database domain matrix report data
    let report = match db::get_cross_database_domain_matrix_report(&state.db_manager).await {
        Ok(report) => report,
        Err(e) => {
            tracing::error!(
                "Error generating cross-database domain matrix report: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the cross-database matrix report template
    let content_template = CrossDatabaseMatrixReportTemplate {
        title: &title,
        description: &description,
        domain_header: &domain_header,
        database_header: &database_header,
        primary_domain: &primary_domain,
        backup_domain: &backup_domain,
        not_present: &not_present,
        legend_title: &legend_title,
        no_domains: &no_domains,
        no_domains_description: &no_domains_description,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering cross-database matrix report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

pub async fn reports_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-list-title").await;
    let description = get_translation(&state, &locale, "reports-list-description").await;
    let matrix_report_title = get_translation(&state, &locale, "reports-matrix-title").await;
    let matrix_report_description =
        get_translation(&state, &locale, "reports-matrix-description").await;
    let orphaned_aliases_report_title =
        get_translation(&state, &locale, "reports-orphaned-aliases-title").await;
    let orphaned_aliases_report_description =
        get_translation(&state, &locale, "reports-orphaned-aliases-description").await;
    let external_forwarders_report_title =
        get_translation(&state, &locale, "reports-external-forwarders-title").await;
    let external_forwarders_report_description =
        get_translation(&state, &locale, "reports-external-forwarders-description").await;
    let alias_cross_domain_report_title =
        get_translation(&state, &locale, "reports-alias-cross-domain-title").await;
    let alias_cross_domain_report_description =
        get_translation(&state, &locale, "reports-alias-cross-domain-description").await;
    let cross_database_matrix_report_title =
        get_translation(&state, &locale, "reports-cross-db-matrix-title").await;
    let cross_database_matrix_report_description =
        get_translation(&state, &locale, "reports-cross-db-matrix-description").await;
    let view_report = get_translation(&state, &locale, "reports-view-report").await;

    // Create the reports list template
    let content_template = ReportsListTemplate {
        title: &title,
        description: &description,
        matrix_report_title: &matrix_report_title,
        matrix_report_description: &matrix_report_description,
        orphaned_aliases_report_title: &orphaned_aliases_report_title,
        orphaned_aliases_report_description: &orphaned_aliases_report_description,
        external_forwarders_report_title: &external_forwarders_report_title,
        external_forwarders_report_description: &external_forwarders_report_description,
        alias_cross_domain_report_title: &alias_cross_domain_report_title,
        alias_cross_domain_report_description: &alias_cross_domain_report_description,
        cross_database_matrix_report_title: &cross_database_matrix_report_title,
        cross_database_matrix_report_description: &cross_database_matrix_report_description,
        view_report: &view_report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering reports list template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

// Orphaned aliases/users report
pub async fn orphaned_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let report = match db::get_orphaned_aliases_report(&pool) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating orphaned report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let content_template = OrphanedReportTemplate {
        title: "Orphaned Aliases & Users",
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering orphaned report template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        "Orphaned Aliases & Users".to_string(),
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

// External forwarders report
pub async fn external_forwarders_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let report = match db::get_external_forwarders_report(&pool) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating external forwarders report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let content_template = ExternalForwarderReportTemplate {
        title: "External Forwarders",
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering external forwarders report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        "External Forwarders".to_string(),
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

// Alias cross-domain search report
pub async fn alias_cross_domain_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let alias = params.get("alias").cloned().unwrap_or_default();
    let report = match db::get_alias_cross_domain_report(&pool, &alias) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating alias cross-domain report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let content_template = AliasCrossDomainReportTemplate {
        title: &format!("Alias '{}' Across Domains", alias),
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering alias cross-domain report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
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

    let template = match BaseTemplate::with_i18n(
        format!("Alias '{}' Across Domains", alias),
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
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

use crate::templates::layout::BaseTemplate;
use crate::templates::reports::{
    AliasCrossDomainReportTemplate, CrossDatabaseFeatureToggleReportTemplate,
    CrossDatabaseMatrixReportTemplate, CrossDatabaseMigrationReportTemplate,
    CrossDatabaseUserDistributionReportTemplate, DomainStatisticsReportTemplate,
    ExternalForwarderReportTemplate, MatrixReportTemplate, MxServersReportTemplate,
    OrphanedReportTemplate, ReportsListTemplate,
};
use crate::{db, i18n::get_translation, models::OrphanedReportParams, AppState};
use askama::Template;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
};

pub async fn matrix_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::translations::get_translations_batch(
        &state,
        &locale,
        &[
            "reports-matrix-title",
            "reports-matrix-description",
            "reports-domain-header",
            "reports-catch-all-header",
            "reports-required-aliases-header",
            "reports-status-present",
            "reports-status-missing",
            "reports-status-disabled",
            "reports-legend-title",
            "reports-no-domains",
            "reports-no-domains-description",
            "reports-back-to-reports",
        ],
    )
    .await;

    // Get matrix report data
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let report = match state
        .db_manager
        .get_domain_alias_matrix_report_cached(&pool)
        .await
    {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating matrix report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the matrix report template
    let content_template = MatrixReportTemplate {
        title: &form_translations["reports-matrix-title"],
        description: &form_translations["reports-matrix-description"],
        domain_header: &form_translations["reports-domain-header"],
        catch_all_header: &form_translations["reports-catch-all-header"],
        required_aliases_header: &form_translations["reports-required-aliases-header"],
        status_present: &form_translations["reports-status-present"],
        status_missing: &form_translations["reports-status-missing"],
        status_disabled: &form_translations["reports-status-disabled"],
        legend_title: &form_translations["reports-legend-title"],
        no_domains: &form_translations["reports-no-domains"],
        no_domains_description: &form_translations["reports-no-domains-description"],
        back_to_reports: &form_translations["reports-back-to-reports"],
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
        form_translations["reports-matrix-title"].clone(),
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

    // Get all reports translations using consolidated helper function
    let translations =
        crate::handlers::translations::get_reports_translations(&state, &locale).await;

    // Get additional translations not in the consolidated helper
    let matrix_enabled = get_translation(&state, &locale, "reports-matrix-enabled").await;
    let matrix_disabled = get_translation(&state, &locale, "reports-matrix-disabled").await;
    let back_to_reports = get_translation(&state, &locale, "reports-back-to-reports").await;

    // Get current database id from session/cookie or default
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Get cross-database domain matrix report data
    let report =
        match db::get_cross_database_domain_matrix_report(&state.db_manager, Some(&current_db_id))
            .await
        {
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
        title: &translations["reports-cross-db-matrix-title"],
        description: &translations["reports-cross-db-matrix-description"],
        domain_header: &translations["reports-domain-header"],
        database_header: &translations["reports-database-header"],
        primary_domain: &translations["reports-primary-domain"],
        backup_domain: &translations["reports-backup-domain"],
        not_present: &translations["reports-not-present"],
        legend_title: &translations["reports-legend-title"],
        no_domains: &translations["reports-no-domains"],
        no_domains_description: &translations["reports-no-domains-description"],
        matrix_enabled: &matrix_enabled,
        matrix_disabled: &matrix_disabled,
        back_to_reports: &back_to_reports,
        current_db_id: &current_db_id,
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
        translations["reports-cross-db-matrix-title"].clone(),
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
        get_translation(&state, &locale, "reports-alias-cross-domain-list-title").await;
    let alias_cross_domain_report_description =
        get_translation(&state, &locale, "reports-alias-cross-domain-description").await;
    let cross_database_matrix_report_title =
        get_translation(&state, &locale, "reports-cross-db-matrix-title").await;
    let cross_database_matrix_report_description =
        get_translation(&state, &locale, "reports-cross-db-matrix-description").await;
    let cross_database_user_distribution_report_title =
        get_translation(&state, &locale, "reports-cross-db-user-distribution-title").await;
    let cross_database_user_distribution_report_description = get_translation(
        &state,
        &locale,
        "reports-cross-db-user-distribution-description",
    )
    .await;
    let cross_database_feature_toggle_report_title =
        get_translation(&state, &locale, "reports-cross-db-feature-toggle-title").await;
    let cross_database_feature_toggle_report_description = get_translation(
        &state,
        &locale,
        "reports-cross-db-feature-toggle-description",
    )
    .await;
    let cross_database_migration_report_title =
        get_translation(&state, &locale, "reports-cross-db-migration-title").await;
    let cross_database_migration_report_description =
        get_translation(&state, &locale, "reports-cross-db-migration-description").await;
    let domain_statistics_report_title =
        get_translation(&state, &locale, "reports-domain-statistics-title").await;
    let domain_statistics_report_description =
        get_translation(&state, &locale, "reports-domain-statistics-description").await;
    let recent_changes_report_title =
        get_translation(&state, &locale, "reports-recent-changes-title").await;
    let recent_changes_report_description =
        get_translation(&state, &locale, "reports-recent-changes-description").await;
    let mx_servers_report_title =
        get_translation(&state, &locale, "reports-mx-servers-title").await;
    let mx_servers_report_description =
        get_translation(&state, &locale, "reports-mx-servers-description").await;
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
        cross_database_user_distribution_report_title:
            &cross_database_user_distribution_report_title,
        cross_database_user_distribution_report_description:
            &cross_database_user_distribution_report_description,
        cross_database_feature_toggle_report_title: &cross_database_feature_toggle_report_title,
        cross_database_feature_toggle_report_description:
            &cross_database_feature_toggle_report_description,
        cross_database_migration_report_title: &cross_database_migration_report_title,
        cross_database_migration_report_description: &cross_database_migration_report_description,
        domain_statistics_report_title: &domain_statistics_report_title,
        domain_statistics_report_description: &domain_statistics_report_description,
        recent_changes_report_title: &recent_changes_report_title,
        recent_changes_report_description: &recent_changes_report_description,
        mx_servers_report_title: &mx_servers_report_title,
        mx_servers_report_description: &mx_servers_report_description,
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
    Query(params): Query<OrphanedReportParams>,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let mut report = match state
        .db_manager
        .get_orphaned_aliases_report_cached(&pool)
        .await
    {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating orphaned report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Apply filtering if requested
    let hide_disabled = params.hide_disabled.unwrap_or(false);
    if hide_disabled {
        report.orphaned_aliases.retain(|alias| alias.enabled);
        report.orphaned_users.retain(|user| user.enabled);
        report.users_without_aliases.retain(|user| user.enabled);
    }

    // Get translations
    let title = get_translation(&state, &locale, "reports-orphaned-aliases-title").await;
    let hide_disabled_label = get_translation(&state, &locale, "reports-hide-disabled").await;
    let show_disabled_label = get_translation(&state, &locale, "reports-show-disabled").await;

    let content_template = OrphanedReportTemplate {
        title: &title,
        report: &report,
        hide_disabled,
        hide_disabled_label: &hide_disabled_label,
        show_disabled_label: &show_disabled_label,
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
        title.clone(),
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let report = match state
        .db_manager
        .get_external_forwarders_report_cached(&pool)
        .await
    {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating external forwarders report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get translations
    let title = get_translation(&state, &locale, "reports-external-forwarders-title").await;

    let content_template = ExternalForwarderReportTemplate {
        title: &title,
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
        title.clone(),
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let alias = params.get("alias").cloned().unwrap_or_default();
    let report = match db::get_alias_cross_domain_report(&pool, &alias) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating alias cross-domain report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get translations
    let title_template = get_translation(&state, &locale, "reports-alias-cross-domain-title").await;
    let title = title_template.replace("{alias}", &alias);
    let alias_placeholder = get_translation(&state, &locale, "reports-alias-placeholder").await;

    let content_template = AliasCrossDomainReportTemplate {
        title: &title,
        alias_placeholder: &alias_placeholder,
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
        title.clone(),
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

// Cross-database User Distribution Report
pub async fn cross_database_user_distribution_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-cross-db-user-distribution-title").await;
    let description = get_translation(
        &state,
        &locale,
        "reports-cross-db-user-distribution-description",
    )
    .await;
    let user_header = get_translation(&state, &locale, "reports-user-header").await;
    let database_header = get_translation(&state, &locale, "reports-database-header").await;
    let present = get_translation(&state, &locale, "reports-present").await;
    let not_present = get_translation(&state, &locale, "reports-not-present").await;
    let legend_title = get_translation(&state, &locale, "reports-legend-title").await;
    let no_users = get_translation(&state, &locale, "reports-no-users").await;
    let no_users_description =
        get_translation(&state, &locale, "reports-no-users-description").await;
    let disabled = get_translation(&state, &locale, "reports-disabled").await;
    let total_users = get_translation(&state, &locale, "reports-total-users").await;
    let in_multiple_dbs = get_translation(&state, &locale, "reports-in-multiple-dbs").await;
    let in_single_db = get_translation(&state, &locale, "reports-in-single-db").await;
    let enabled = get_translation(&state, &locale, "reports-enabled").await;
    let back_to_reports = get_translation(&state, &locale, "reports-back-to-reports").await;

    // Get cross-database user distribution report data
    let report = match db::get_cross_database_user_distribution_report(&state.db_manager).await {
        Ok(report) => report,
        Err(e) => {
            tracing::error!(
                "Error generating cross-database user distribution report: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get current database ID
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Create the cross-database user distribution report template
    let content_template = CrossDatabaseUserDistributionReportTemplate {
        title: &title,
        description: &description,
        user_header: &user_header,
        database_header: &database_header,
        present: &present,
        not_present: &not_present,
        legend_title: &legend_title,
        no_users: &no_users,
        no_users_description: &no_users_description,
        disabled: &disabled,
        total_users: &total_users,
        in_multiple_dbs: &in_multiple_dbs,
        in_single_db: &in_single_db,
        enabled: &enabled,
        back_to_reports: &back_to_reports,
        current_db_id: &current_db_id,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering cross-database user distribution report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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

// Cross-database Feature Toggle Compliance Report
pub async fn cross_database_feature_toggle_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-cross-db-feature-toggle-title").await;
    let description = get_translation(
        &state,
        &locale,
        "reports-cross-db-feature-toggle-description",
    )
    .await;
    let database_header = get_translation(&state, &locale, "reports-database-header").await;
    let database_status_header =
        get_translation(&state, &locale, "reports-database-status-header").await;
    let read_only = get_translation(&state, &locale, "reports-read-only").await;
    let no_new_users = get_translation(&state, &locale, "reports-no-new-users").await;
    let no_new_domains = get_translation(&state, &locale, "reports-no-new-domains").await;
    let no_password_updates = get_translation(&state, &locale, "reports-no-password-updates").await;
    let enabled = get_translation(&state, &locale, "reports-enabled").await;
    let disabled = get_translation(&state, &locale, "reports-disabled").await;
    let total_databases = get_translation(&state, &locale, "reports-total-databases").await;
    let fully_restricted = get_translation(&state, &locale, "reports-fully-restricted").await;
    let feature_toggle_legend =
        get_translation(&state, &locale, "reports-feature-toggle-legend").await;
    let feature_enabled = get_translation(&state, &locale, "reports-feature-enabled").await;
    let feature_disabled = get_translation(&state, &locale, "reports-feature-disabled").await;
    let back_to_reports = get_translation(&state, &locale, "reports-back-to-reports").await;

    // Get cross-database feature toggle report data
    let report = match db::get_cross_database_feature_toggle_report(&state.db_manager).await {
        Ok(report) => report,
        Err(e) => {
            tracing::error!(
                "Error generating cross-database feature toggle report: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the cross-database feature toggle report template
    let content_template = CrossDatabaseFeatureToggleReportTemplate {
        title: &title,
        description: &description,
        database_header: &database_header,
        database_status_header: &database_status_header,
        read_only: &read_only,
        no_new_users: &no_new_users,
        no_new_domains: &no_new_domains,
        no_password_updates: &no_password_updates,
        enabled: &enabled,
        disabled: &disabled,
        total_databases: &total_databases,
        fully_restricted: &fully_restricted,
        feature_toggle_legend: &feature_toggle_legend,
        feature_enabled: &feature_enabled,
        feature_disabled: &feature_disabled,
        back_to_reports: &back_to_reports,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering cross-database feature toggle report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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

// Cross-database Migration Status Report
pub async fn cross_database_migration_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "reports-cross-db-migration-title").await;
    let description =
        get_translation(&state, &locale, "reports-cross-db-migration-description").await;
    let database_header = get_translation(&state, &locale, "reports-database-header").await;
    let status_header = get_translation(&state, &locale, "reports-status-header").await;
    let last_migration_header =
        get_translation(&state, &locale, "reports-last-migration-header").await;
    let migration_count_header =
        get_translation(&state, &locale, "reports-migration-count-header").await;
    let total_databases = get_translation(&state, &locale, "reports-total-databases").await;
    let up_to_date = get_translation(&state, &locale, "reports-up-to-date").await;
    let behind = get_translation(&state, &locale, "reports-behind").await;
    let errors = get_translation(&state, &locale, "reports-errors").await;
    let unknown = get_translation(&state, &locale, "reports-unknown").await;
    let latest_migration = get_translation(&state, &locale, "reports-latest-migration").await;
    let migration_status_legend =
        get_translation(&state, &locale, "reports-migration-status-legend").await;
    let behind_on_migrations =
        get_translation(&state, &locale, "reports-behind-on-migrations").await;
    let migration_error = get_translation(&state, &locale, "reports-migration-error").await;
    let unknown_status = get_translation(&state, &locale, "reports-unknown-status").await;
    let back_to_reports = get_translation(&state, &locale, "reports-back-to-reports").await;

    // Get cross-database migration report data
    let report = match db::get_cross_database_migration_report(&state.db_manager).await {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating cross-database migration report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the cross-database migration report template
    let content_template = CrossDatabaseMigrationReportTemplate {
        title: &title,
        description: &description,
        database_header: &database_header,
        status_header: &status_header,
        last_migration_header: &last_migration_header,
        migration_count_header: &migration_count_header,
        total_databases: &total_databases,
        up_to_date: &up_to_date,
        behind: &behind,
        errors: &errors,
        unknown: &unknown,
        latest_migration: &latest_migration,
        migration_status_legend: &migration_status_legend,
        behind_on_migrations: &behind_on_migrations,
        migration_error: &migration_error,
        unknown_status: &unknown_status,
        back_to_reports: &back_to_reports,
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "Error rendering cross-database migration report template: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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

pub async fn domain_statistics_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get domain statistics data
    let domain_stats = match db::get_domain_stats(&pool) {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("Error generating domain statistics report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get translations
    let title = get_translation(&state, &locale, "reports-domain-statistics-title").await;
    let description =
        get_translation(&state, &locale, "reports-domain-statistics-description").await;
    let domain_statistics = get_translation(&state, &locale, "stats-domain-statistics").await;
    let table_header_domain = get_translation(&state, &locale, "stats-table-header-domain").await;
    let table_header_users = get_translation(&state, &locale, "stats-table-header-users").await;
    let table_header_aliases = get_translation(&state, &locale, "stats-table-header-aliases").await;
    let table_header_relays = get_translation(&state, &locale, "stats-table-header-relays").await;
    let table_header_relocated =
        get_translation(&state, &locale, "stats-table-header-relocated").await;
    let empty_title =
        get_translation(&state, &locale, "reports-domain-statistics-empty-title").await;
    let empty_description = get_translation(
        &state,
        &locale,
        "reports-domain-statistics-empty-description",
    )
    .await;

    let content_template = DomainStatisticsReportTemplate {
        title: &title,
        description: &description,
        domain_statistics: &domain_statistics,
        table_header_domain: &table_header_domain,
        table_header_users: &table_header_users,
        table_header_aliases: &table_header_aliases,
        table_header_relays: &table_header_relays,
        table_header_relocated: &table_header_relocated,
        empty_title: &empty_title,
        empty_description: &empty_description,
        domain_stats: &domain_stats,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering domain statistics report template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    let template = match BaseTemplate::with_i18n(
        title.clone(),
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

// MX Servers Report
pub async fn mx_servers_report(
    Query(params): Query<crate::models::MxServersReportParams>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get pagination and filter parameters
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let sort_by = params.sort_by.as_deref().unwrap_or("domain");
    let sort_order = params.sort_order.as_deref().unwrap_or("asc");
    let exclude_disabled = params.exclude_disabled.unwrap_or(false);
    let exclude_subdomains = params.exclude_subdomains.unwrap_or(false);

    // Parse filter_status parameter
    let filter_status = params
        .filter_status
        .as_ref()
        .and_then(|s| crate::models::MxStatus::from_str(s));
    let filter_status_str = params.filter_status.clone().unwrap_or_default();

    // Get MX servers report data
    let paginated_result = match db::get_mx_servers_report(
        &pool,
        &state.config.mail_servers,
        page,
        per_page,
        sort_by,
        sort_order,
        exclude_disabled,
        exclude_subdomains,
        filter_status.clone(),
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Error generating MX servers report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // If the requested page is beyond total pages and there are results, redirect to page 1
    if page > paginated_result.total_pages && paginated_result.total_count > 0 {
        let redirect_url = format!(
            "/reports/mx-servers?page=1&per_page={}&exclude_disabled={}&exclude_subdomains={}&filter_status={}",
            per_page, exclude_disabled, exclude_subdomains, filter_status_str
        );
        return Ok(Html(format!(
            r#"<script>window.location.href="{}";</script>"#,
            redirect_url
        )));
    }

    // Get translations
    let title = get_translation(&state, &locale, "reports-mx-servers-title").await;
    let description = get_translation(&state, &locale, "reports-mx-servers-description").await;
    let domain_header = get_translation(&state, &locale, "reports-domain-header").await;
    let mx_status_header = get_translation(&state, &locale, "reports-mx-status-header").await;
    let mx_records_header = get_translation(&state, &locale, "reports-mx-records-header").await;
    let missing_servers_header =
        get_translation(&state, &locale, "reports-missing-servers-header").await;
    let unexpected_servers_header =
        get_translation(&state, &locale, "reports-unexpected-servers-header").await;
    let legend_title = get_translation(&state, &locale, "reports-legend-title").await;
    let status_compliant = get_translation(&state, &locale, "reports-mx-status-compliant").await;
    let status_non_compliant =
        get_translation(&state, &locale, "reports-mx-status-non-compliant").await;
    let status_empty = get_translation(&state, &locale, "reports-mx-status-empty").await;
    let status_error = get_translation(&state, &locale, "reports-mx-status-error").await;
    let no_domains = get_translation(&state, &locale, "reports-no-domains").await;
    let no_domains_description =
        get_translation(&state, &locale, "reports-no-domains-description").await;
    let back_to_reports = get_translation(&state, &locale, "reports-back-to-reports").await;
    let configured_mail_servers =
        get_translation(&state, &locale, "reports-configured-mail-servers").await;
    let summary_title = get_translation(&state, &locale, "reports-summary-title").await;
    let total_domains = get_translation(&state, &locale, "reports-total-domains").await;
    let compliant_domains = get_translation(&state, &locale, "reports-compliant-domains").await;
    let non_compliant_domains =
        get_translation(&state, &locale, "reports-non-compliant-domains").await;
    let empty_mx_domains = get_translation(&state, &locale, "reports-empty-mx-domains").await;
    let error_domains = get_translation(&state, &locale, "reports-error-domains").await;

    // Get pagination translations
    let pagination_previous = get_translation(&state, &locale, "pagination-previous").await;
    let pagination_next = get_translation(&state, &locale, "pagination-next").await;
    let pagination_showing = get_translation(&state, &locale, "pagination-showing").await;
    let pagination_to = get_translation(&state, &locale, "pagination-to").await;
    let pagination_of = get_translation(&state, &locale, "pagination-of").await;
    let pagination_results = get_translation(&state, &locale, "pagination-results").await;

    // Get filter translations
    let exclude_disabled_label = get_translation(&state, &locale, "reports-exclude-disabled").await;
    let exclude_subdomains_label =
        get_translation(&state, &locale, "reports-exclude-subdomains").await;
    let filters_label = get_translation(&state, &locale, "reports-filters").await;
    let filter_status_label = get_translation(&state, &locale, "reports-filter-status").await;
    let filter_status_all = get_translation(&state, &locale, "reports-filter-status-all").await;

    // Generate page range for pagination UI
    let page_range: Vec<i64> = (1..=paginated_result.total_pages).collect();

    let content_template = MxServersReportTemplate {
        title: &title,
        description: &description,
        domain_header: &domain_header,
        mx_status_header: &mx_status_header,
        mx_records_header: &mx_records_header,
        missing_servers_header: &missing_servers_header,
        unexpected_servers_header: &unexpected_servers_header,
        legend_title: &legend_title,
        status_compliant: &status_compliant,
        status_non_compliant: &status_non_compliant,
        status_empty: &status_empty,
        status_error: &status_error,
        no_domains: &no_domains,
        no_domains_description: &no_domains_description,
        back_to_reports: &back_to_reports,
        configured_mail_servers: &configured_mail_servers,
        summary_title: &summary_title,
        total_domains: &total_domains,
        compliant_domains: &compliant_domains,
        non_compliant_domains: &non_compliant_domains,
        empty_mx_domains: &empty_mx_domains,
        error_domains: &error_domains,
        paginated_result: &paginated_result,
        mail_servers: &state.config.mail_servers,
        page_range: &page_range,
        pagination_previous: &pagination_previous,
        pagination_next: &pagination_next,
        pagination_showing: &pagination_showing,
        pagination_to: &pagination_to,
        pagination_of: &pagination_of,
        pagination_results: &pagination_results,
        exclude_disabled,
        exclude_subdomains,
        exclude_disabled_label: &exclude_disabled_label,
        exclude_subdomains_label: &exclude_subdomains_label,
        filters_label: &filters_label,
        filter_status: &filter_status_str,
        filter_status_label: &filter_status_label,
        filter_status_all: &filter_status_all,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering MX servers report template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    let template = match BaseTemplate::with_i18n(
        title.clone(),
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

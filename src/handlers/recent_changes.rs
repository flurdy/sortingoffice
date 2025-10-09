use crate::templates::layout::BaseTemplate;
use crate::templates::reports::RecentChangesReportTemplate;
use crate::{db, AppState};
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};

pub async fn recent_changes_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get recent changes report data
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let report = match db::get_recent_changes_report(&pool, Some(50)).await {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Error generating recent changes report: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::translations::get_translations_batch(
        &state,
        &locale,
        &[
            "reports-recent-changes-title",
            "reports-recent-changes-description",
            "reports-table-header-resource-type",
            "reports-table-header-resource-name",
            "reports-table-header-action",
            "reports-table-header-timestamp",
            "reports-table-header-status",
            "reports-table-header-actions",
            "reports-action-created",
            "reports-action-updated",
            "reports-action-deleted",
            "reports-resource-type-domain",
            "reports-resource-type-user",
            "reports-resource-type-alias",
            "reports-resource-type-backup",
            "reports-resource-type-relay",
            "reports-resource-type-relocated",
            "reports-resource-type-client",
            "status-enabled",
            "status-disabled",
            "reports-view-resource",
            "reports-no-changes",
            "reports-no-changes-description",
            "reports-back-to-reports",
        ],
    )
    .await;

    // Create the recent changes report template
    let content_template = RecentChangesReportTemplate {
        title: &form_translations["reports-recent-changes-title"],
        description: &form_translations["reports-recent-changes-description"],
        table_header_resource_type: &form_translations["reports-table-header-resource-type"],
        table_header_resource_name: &form_translations["reports-table-header-resource-name"],
        table_header_action: &form_translations["reports-table-header-action"],
        table_header_timestamp: &form_translations["reports-table-header-timestamp"],
        table_header_status: &form_translations["reports-table-header-status"],
        table_header_actions: &form_translations["reports-table-header-actions"],
        action_created: &form_translations["reports-action-created"],
        action_updated: &form_translations["reports-action-updated"],
        action_deleted: &form_translations["reports-action-deleted"],
        resource_type_domain: &form_translations["reports-resource-type-domain"],
        resource_type_user: &form_translations["reports-resource-type-user"],
        resource_type_alias: &form_translations["reports-resource-type-alias"],
        resource_type_backup: &form_translations["reports-resource-type-backup"],
        resource_type_relay: &form_translations["reports-resource-type-relay"],
        resource_type_relocated: &form_translations["reports-resource-type-relocated"],
        resource_type_client: &form_translations["reports-resource-type-client"],
        status_enabled: &form_translations["status-enabled"],
        status_disabled: &form_translations["status-disabled"],
        view_resource: &form_translations["reports-view-resource"],
        no_changes: &form_translations["reports-no-changes"],
        no_changes_description: &form_translations["reports-no-changes-description"],
        back_to_reports: &form_translations["reports-back-to-reports"],
        report: &report,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering recent changes report template: {:?}", e);
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
        form_translations["reports-recent-changes-title"].clone(),
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

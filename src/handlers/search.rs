use crate::db;
use crate::templates::layout::BaseTemplate;
use crate::templates::search::SearchPageTemplate;
use crate::AppState;
use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub enabled_only: Option<bool>,
    // Individual type checkboxes
    pub domain: Option<bool>,
    pub user: Option<bool>,
    pub alias: Option<bool>,
    pub backup: Option<bool>,
    pub relay: Option<bool>,
    pub relocated: Option<bool>,
    pub client: Option<bool>,
}

pub async fn search_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(_error_html) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get translations
    let form_translations = crate::handlers::translations::get_translations_batch(
        &state,
        &locale,
        &[
            "search-title",
            "search-description",
            "search-placeholder",
            "search-button",
            "search-results-title",
            "search-no-results",
            "search-no-results-description",
            "search-results-count",
            "search-time",
            "search-filter-types",
            "search-filter-enabled-only",
            "search-resource-type-domain",
            "search-resource-type-user",
            "search-resource-type-alias",
            "search-resource-type-backup",
            "search-resource-type-relay",
            "search-resource-type-relocated",
            "search-resource-type-client",
            "search-match-fields",
            "search-view-resource",
            "search-created",
            "search-modified",
            "status-enabled",
            "status-disabled",
        ],
    )
    .await;

    // Parse resource types filter from individual checkboxes
    let mut resource_types = Vec::new();
    if params.domain.unwrap_or(false) {
        resource_types.push("domain".to_string());
    }
    if params.user.unwrap_or(false) {
        resource_types.push("user".to_string());
    }
    if params.alias.unwrap_or(false) {
        resource_types.push("alias".to_string());
    }
    if params.backup.unwrap_or(false) {
        resource_types.push("backup".to_string());
    }
    if params.relay.unwrap_or(false) {
        resource_types.push("relay".to_string());
    }
    if params.relocated.unwrap_or(false) {
        resource_types.push("relocated".to_string());
    }
    if params.client.unwrap_or(false) {
        resource_types.push("client".to_string());
    }

    let resource_types = if resource_types.is_empty() {
        None
    } else {
        Some(resource_types)
    };

    // Perform search if query is provided
    let search_results = if let Some(query) = &params.q {
        if query.trim().is_empty() {
            None
        } else {
            match db::search_all_tables(
                &pool,
                query.trim(),
                resource_types.as_deref(),
                params.enabled_only,
                Some(100),
            )
            .await
            {
                Ok(results) => Some(results),
                Err(e) => {
                    tracing::error!("Error performing search: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    } else {
        None
    };

    let (has_results, total_count, search_time_ms, results) = if let Some(ref sr) = search_results {
        (true, sr.total_count, sr.search_time_ms, sr.results.clone())
    } else {
        (false, 0, 0, Vec::new())
    };

    let content_template = SearchPageTemplate {
        title: &form_translations["search-title"],
        description: &form_translations["search-description"],
        search_placeholder: &form_translations["search-placeholder"],
        search_button: &form_translations["search-button"],
        search_results_title: &form_translations["search-results-title"],
        search_no_results: &form_translations["search-no-results"],
        search_no_results_description: &form_translations["search-no-results-description"],
        search_results_count: &form_translations["search-results-count"],
        search_time: &form_translations["search-time"],
        search_filter_types: &form_translations["search-filter-types"],
        search_filter_enabled_only: &form_translations["search-filter-enabled-only"],
        search_resource_type_domain: &form_translations["search-resource-type-domain"],
        search_resource_type_user: &form_translations["search-resource-type-user"],
        search_resource_type_alias: &form_translations["search-resource-type-alias"],
        search_resource_type_backup: &form_translations["search-resource-type-backup"],
        search_resource_type_relay: &form_translations["search-resource-type-relay"],
        search_resource_type_relocated: &form_translations["search-resource-type-relocated"],
        search_resource_type_client: &form_translations["search-resource-type-client"],
        search_match_fields: &form_translations["search-match-fields"],
        search_view_resource: &form_translations["search-view-resource"],
        search_created: &form_translations["search-created"],
        search_modified: &form_translations["search-modified"],
        status_enabled: &form_translations["status-enabled"],
        status_disabled: &form_translations["status-disabled"],
        query: params.q.unwrap_or_default(),
        selected_domain: params.domain.unwrap_or(false),
        selected_user: params.user.unwrap_or(false),
        selected_alias: params.alias.unwrap_or(false),
        selected_backup: params.backup.unwrap_or(false),
        selected_relay: params.relay.unwrap_or(false),
        selected_relocated: params.relocated.unwrap_or(false),
        selected_client: params.client.unwrap_or(false),
        enabled_only: params.enabled_only.unwrap_or(false),
        has_results,
        total_count,
        search_time_ms,
        results,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering search page template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
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

    let template = match BaseTemplate::with_i18n(
        form_translations["search-title"].clone(),
        content,
        &state,
        &locale,
        current_db_id,
        current_db_label,
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

use crate::i18n::get_translation;
use crate::AppState;
use std::collections::HashMap;
use tracing::debug;

/// Batch translation fetcher
pub async fn get_translations_batch(
    state: &AppState,
    locale: &str,
    keys: &[&str],
) -> HashMap<String, String> {
    let mut translations = HashMap::new();
    for key in keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }
    translations
}

/// Helper function to fetch multiple form-related translations at once
pub async fn get_entity_form_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Load common form translations
    let common_keys = vec![
        "form-error",
        "form-cancel",
        "action-save",
        "action-cancel",
        "form-enabled",
        "form-disabled",
        "form-create-domain",
        "form-update-domain",
        "form-placeholder-domain",
        "form-placeholder-transport",
        "form-tooltip-domain",
        "form-tooltip-transport",
        "form-tooltip-enable",
    ];

    for key in common_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    // Generate entity-specific keys
    let singular = if entity == "aliases" {
        "alias"
    } else {
        entity.trim_end_matches('s')
    };

    let entity_keys = vec![
        format!("{entity}-add-title"),
        format!("{entity}-edit-title"),
        format!("{entity}-new-{singular}"),
        format!("{entity}-edit-{singular}"),
    ];

    for key in entity_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Final translations map: {translations:#?}");
    translations
}

/// Helper function to fetch field-related translations
pub async fn get_field_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
    fields: &[&str],
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    for field in fields {
        let key = format!("{entity}-field-{}", field);
        let value = get_translation(state, locale, &key).await;
        translations.insert(field.to_string(), value);
    }

    debug!("Field translations: {translations:#?}");
    translations
}

/// Helper function to fetch status-related translations
pub async fn get_status_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let status_keys = vec![
        format!("{entity}-status-enabled"),
        format!("{entity}-status-disabled"),
        format!("{entity}-status-active"),
        format!("{entity}-status-inactive"),
        format!("{entity}-status-pending"),
        format!("{entity}-status-completed"),
        format!("{entity}-status-failed"),
    ];

    for key in status_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Status translations: {translations:#?}");
    translations
}

/// Helper function to fetch action-related translations
pub async fn get_action_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let action_keys = vec![
        format!("{entity}-action-view"),
        format!("{entity}-action-edit"),
        format!("{entity}-action-delete"),
        format!("{entity}-action-create"),
        format!("{entity}-action-update"),
        format!("{entity}-action-toggle"),
        format!("{entity}-action-search"),
        format!("{entity}-action-filter"),
        format!("{entity}-action-sort"),
        format!("{entity}-action-export"),
        format!("{entity}-action-import"),
    ];

    for key in action_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Action translations: {translations:#?}");
    translations
}

/// Helper function to fetch table-related translations
pub async fn get_table_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let table_keys = vec![
        "table-header-id",
        "table-header-name",
        "table-header-email",
        "table-header-domain",
        "table-header-status",
        "table-header-created",
        "table-header-updated",
        "table-header-actions",
        "table-empty-message",
        "table-loading-message",
        "table-error-message",
        "table-pagination-info",
        "table-pagination-previous",
        "table-pagination-next",
        "table-pagination-first",
        "table-pagination-last",
    ];

    for key in table_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    debug!("Table translations: {translations:#?}");
    translations
}

/// Helper function to fetch entity-specific translations for list pages
pub async fn get_entity_list_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Common entity list translations
    let entity_keys = vec![
        format!("{entity}-title"),
        format!("{entity}-add"),
        format!("{entity}-list-description"),
        format!("{entity}-empty-title"),
        format!("{entity}-empty-description"),
        format!("{entity}-not-found"),
        format!("{entity}-not-available"),
    ];

    // Table header translations
    let table_header_keys = vec![
        format!("{entity}-table-header-id"),
        format!("{entity}-table-header-name"),
        format!("{entity}-table-header-email"),
        format!("{entity}-table-header-domain"),
        format!("{entity}-table-header-status"),
        format!("{entity}-table-header-created"),
        format!("{entity}-table-header-updated"),
        format!("{entity}-table-header-actions"),
    ];

    // Combine all keys
    let all_keys = [entity_keys, table_header_keys].concat();

    for key in all_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Entity list translations for {}: {translations:#?}", entity);
    translations
}

/// Helper function to fetch entity-specific translations for show pages
pub async fn get_entity_show_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let show_keys = vec![
        format!("{entity}-show-title"),
        format!("{entity}-show-description"),
        format!("{entity}-show-not-found"),
        format!("{entity}-show-error"),
        format!("{entity}-show-loading"),
        format!("{entity}-show-actions"),
        format!("{entity}-show-edit"),
        format!("{entity}-show-delete"),
        format!("{entity}-show-back"),
        format!("{entity}-show-details"),
        format!("{entity}-show-metadata"),
        format!("{entity}-show-created"),
        format!("{entity}-show-updated"),
        format!("{entity}-show-status"),
        format!("{entity}-show-id"),
    ];

    for key in show_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Entity show translations for {}: {translations:#?}", entity);
    translations
}

/// Helper function to fetch entity-specific error translations
pub async fn get_entity_error_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let error_keys = vec![
        format!("{entity}-error-not-found"),
        format!("{entity}-error-invalid"),
        format!("{entity}-error-duplicate"),
        format!("{entity}-error-required"),
        format!("{entity}-error-format"),
        format!("{entity}-error-length"),
        format!("{entity}-error-unique"),
        format!("{entity}-error-database"),
        format!("{entity}-error-permission"),
        format!("{entity}-error-validation"),
    ];

    for key in error_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!(
        "Entity error translations for {}: {translations:#?}",
        entity
    );
    translations
}

/// Helper function to fetch all entity-specific translations
pub async fn get_entity_all_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Get all types of translations for this entity
    let form_translations = get_entity_form_translations(state, locale, entity).await;
    let list_translations = get_entity_list_translations(state, locale, entity).await;
    let show_translations = get_entity_show_translations(state, locale, entity).await;
    let error_translations = get_entity_error_translations(state, locale, entity).await;
    let field_translations = get_field_translations(
        state,
        locale,
        entity,
        &[
            "id", "name", "email", "domain", "status", "created", "updated",
        ],
    )
    .await;
    let status_translations = get_status_translations(state, locale, entity).await;
    let action_translations = get_action_translations(state, locale, entity).await;

    // Merge all translations
    translations.extend(form_translations);
    translations.extend(list_translations);
    translations.extend(show_translations);
    translations.extend(error_translations);
    translations.extend(field_translations);
    translations.extend(status_translations);
    translations.extend(action_translations);

    debug!("All entity translations for {}: {translations:#?}", entity);
    translations
}

/// Helper function to fetch login-related translations
pub async fn get_login_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let login_keys = vec![
        "login-title",
        "login-username",
        "login-password",
        "login-submit",
        "login-error",
        "login-success",
        "login-logout",
        "login-session",
        "login-remember",
        "login-forgot-password",
        "login-reset-password",
        "login-change-password",
        "login-current-password",
        "login-new-password",
        "login-confirm-password",
        "login-password-strength",
        "login-password-requirements",
    ];

    for key in login_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    debug!("Login translations: {translations:#?}");
    translations
}

/// Helper function to fetch reports-related translations
pub async fn get_reports_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let reports_keys = vec![
        "reports-title",
        "reports-description",
        "reports-generate",
        "reports-download",
        "reports-export",
        "reports-format",
        "reports-date-range",
        "reports-filters",
        "reports-summary",
        "reports-details",
        "reports-charts",
        "reports-statistics",
        "reports-analytics",
        "reports-trends",
        "reports-comparison",
        "reports-breakdown",
        "reports-distribution",
        "reports-top-items",
        "reports-bottom-items",
        "reports-average",
        "reports-total",
        "reports-count",
        "reports-percentage",
        "reports-growth",
        "reports-decline",
        "reports-stable",
        "reports-volatile",
        "reports-predictable",
        "reports-unpredictable",
        "reports-seasonal",
        "reports-cyclical",
        "reports-trending",
        "reports-emerging",
        "reports-declining",
        "reports-stable",
        "reports-volatile",
    ];

    for key in reports_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    debug!("Reports translations: {translations:#?}");
    translations
}

/// Helper function to fetch not found page translations
pub async fn get_not_found_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let not_found_keys = vec![
        "not-found-title",
        "not-found-description",
        "not-found-message",
        "not-found-suggestions",
        "not-found-back-home",
        "not-found-search",
        "not-found-contact",
        "not-found-help",
    ];

    for key in not_found_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    debug!("Not found translations: {translations:#?}");
    translations
}

/// Helper function to fetch pagination-related translations
pub async fn get_pagination_translations(
    state: &AppState,
    locale: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let pagination_keys = vec![
        "pagination-info",
        "pagination-previous",
        "pagination-next",
        "pagination-first",
        "pagination-last",
        "pagination-page",
        "pagination-of",
        "pagination-showing",
        "pagination-to",
        "pagination-of-total",
        "pagination-items",
        "pagination-per-page",
        "pagination-go-to",
        "pagination-jump-to",
        "pagination-loading",
        "pagination-error",
        "pagination-empty",
    ];

    for key in pagination_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    debug!("Pagination translations: {translations:#?}");
    translations
}

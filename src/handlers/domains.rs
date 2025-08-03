use crate::{
    analytics::find_database_common_aliases,
    db,
    handlers::auth::get_selected_database,
    i18n::get_translation,
    models::{Domain, DomainForm, NewDomain, PaginatedResult, PaginationParams},
    templates::domains::DomainFormTemplate,
    AppState,
};

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Html,
    Form,
};

use tracing::{error, info};

// Helper function to validate domain form
fn validate_domain_form(form: &DomainForm) -> Result<(), String> {
    if form.domain.trim().is_empty() {
        return Err("validation-domain-required".to_string());
    }

    // Add comprehensive domain validation
    match crate::validation::validate_domain(form.domain.trim()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("validation-domain-invalid: {e}")),
    }
}

// Helper function to handle domain form errors
async fn handle_domain_form_error(
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    form: DomainForm,
    error_key: &str,
    is_edit: bool,
) -> Html<String> {
    let error_msg = get_translation(state, locale, error_key).await;

    let form_translations =
        crate::handlers::utils::get_entity_form_translations(state, locale, "domains").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        state,
        locale,
        "domains",
        &["domain", "transport", "active"],
    )
    .await;

    // Create owned strings to avoid lifetime issues
    let title = if is_edit {
        form_translations
            .get("domains-edit-domain")
            .unwrap_or(&"Edit Domain".to_string())
            .clone()
    } else {
        form_translations
            .get("domains-new-domain")
            .unwrap_or(&"New Domain".to_string())
            .clone()
    };

    let form_error = form_translations
        .get("form-error")
        .unwrap_or(&"Form Error".to_string())
        .clone();
    let form_domain = field_translations
        .get("domains-field-domain")
        .unwrap_or(&"Domain".to_string())
        .clone();
    let form_transport = field_translations
        .get("domains-field-transport")
        .unwrap_or(&"Transport".to_string())
        .clone();
    let form_active = field_translations
        .get("domains-field-active")
        .unwrap_or(&"Active".to_string())
        .clone();
    let form_cancel = form_translations
        .get("form-cancel")
        .unwrap_or(&"Cancel".to_string())
        .clone();
    let form_create_domain = form_translations
        .get("action-save")
        .unwrap_or(&"Save".to_string())
        .clone();
    let form_update_domain = form_translations
        .get("action-save")
        .unwrap_or(&"Save".to_string())
        .clone();
    let form_placeholder_domain = field_translations
        .get("domains-placeholder-domain")
        .unwrap_or(&"Enter domain".to_string())
        .clone();
    let form_placeholder_transport = field_translations
        .get("domains-placeholder-transport")
        .unwrap_or(&"Enter transport".to_string())
        .clone();
    let form_tooltip_domain = field_translations
        .get("domains-field-domain-help")
        .unwrap_or(&"Domain tooltip".to_string())
        .clone();
    let form_tooltip_transport = field_translations
        .get("domains-field-transport-help")
        .unwrap_or(&"Transport tooltip".to_string())
        .clone();
    let form_tooltip_enable = field_translations
        .get("domains-field-active-help")
        .unwrap_or(&"Active tooltip".to_string())
        .clone();

    let content_template = DomainFormTemplate {
        title: &title,
        domain: None,
        form,
        error: Some(error_msg),
        form_error: &form_error,
        form_domain: &form_domain,
        form_transport: &form_transport,
        form_active: &form_active,
        form_cancel: &form_cancel,
        form_create_domain: &form_create_domain,
        form_update_domain: &form_update_domain,
        form_placeholder_domain: &form_placeholder_domain,
        form_placeholder_transport: &form_placeholder_transport,
        form_tooltip_domain: &form_tooltip_domain,
        form_tooltip_transport: &form_tooltip_transport,
        form_tooltip_enable: &form_tooltip_enable,
        form_enabled: &form_translations["form-enabled"],
        form_disabled: &form_translations["form-disabled"],
    };

    crate::handlers::utils::render_form_template(
        content_template,
        state,
        locale,
        headers,
        title.clone(),
    )
    .await
}

/// Shared function to render domain show page
pub async fn render_domain_show_page(
    state: &AppState,
    headers: &HeaderMap,
    domain: Domain,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(state, headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(headers);

    // Get alias report and existing aliases
    let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
    let existing_aliases = db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

    // Get analytics-driven common aliases
    let analytics_common_aliases = find_database_common_aliases(state, headers, 10, 3).await;

    // Filter out analytics aliases that are already in the domain or in config
    let config_required = state.config.required_aliases.clone();
    let config_common = state.config.common_aliases.clone();
    let existing_alias_names: Vec<String> = existing_aliases
        .iter()
        .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
        .collect();

    let filtered_analytics_aliases: Vec<String> = analytics_common_aliases
        .iter()
        .filter(|alias| {
            !config_required.contains(alias)
                && !config_common.contains(alias)
                && !existing_alias_names.contains(alias)
        })
        .cloned()
        .collect();

    // Use the utils.rs helper function
    crate::handlers::utils::render_domain_show_page(
        domain,
        alias_report,
        existing_aliases,
        filtered_analytics_aliases,
        state,
        &locale,
        headers,
    )
    .await
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // Get domains with error handling
    let paginated_domains = match db::get_domains_paginated(&pool, page, per_page) {
        Ok(domains) => domains,
        Err(e) => {
            error!("Failed to retrieve domains: {:?}", e);
            PaginatedResult::new(vec![], 0, 1, per_page)
        }
    };

    // Get backups with error handling
    let backups = match db::get_backups(&pool) {
        Ok(backups) => backups,
        Err(e) => {
            error!("Failed to retrieve backups: {:?}", e);
            vec![]
        }
    };

    // Use the new resource-specific helper function
    crate::handlers::utils::render_domain_list_page(
        paginated_domains.items.clone(),
        backups,
        &paginated_domains,
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let form = DomainForm {
        domain: "".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
    };

    // Use the new resource-specific helper function
    crate::handlers::utils::render_domain_form_page(
        form,
        None, // No existing domain for new form
        "domains-add-title",
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "domains",
                "domains-not-found",
            )
            .await;
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get alias report and existing aliases
    let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
    let existing_aliases = db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

    // Get analytics-driven common aliases
    let analytics_common_aliases = find_database_common_aliases(&state, &headers, 10, 3).await;

    // Filter out analytics aliases that are already in the domain or in config
    let config_required = state.config.required_aliases.clone();
    let config_common = state.config.common_aliases.clone();
    let existing_alias_names: Vec<String> = existing_aliases
        .iter()
        .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
        .collect();

    let filtered_analytics_aliases: Vec<String> = analytics_common_aliases
        .iter()
        .filter(|alias| {
            !config_required.contains(alias)
                && !config_common.contains(alias)
                && !existing_alias_names.contains(alias)
        })
        .cloned()
        .collect();

    // Use the new resource-specific helper function
    crate::handlers::utils::render_domain_show_page(
        domain,
        alias_report,
        existing_aliases,
        filtered_analytics_aliases,
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "domains",
                "domains-not-found",
            )
            .await;
        }
    };

    let form = DomainForm {
        domain: domain.domain.clone(),
        transport: domain.transport.clone().unwrap_or_default(),
        enabled: domain.enabled,
    };

    // Use the new resource-specific helper function
    crate::handlers::utils::render_domain_form_page(
        form,
        Some(domain), // Pass the existing domain for edit form
        "domains-edit-domain",
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get current database ID for restriction checks
    let current_db_id = get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "create_domain")
    {
        return handle_domain_form_error(
            &state,
            &locale,
            &headers,
            form,
            "error-operation-not-allowed",
            false,
        )
        .await;
    }

    // Validate form data
    if let Err(error_key) = validate_domain_form(&form) {
        return handle_domain_form_error(&state, &locale, &headers, form, &error_key, false).await;
    }

    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let new_domain = NewDomain {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
        enabled: form.enabled,
    };

    match db::create_domain(&pool, new_domain) {
        Ok(_) => {
            info!("Successfully created domain: {}", form.domain);

            // Redirect to domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after creation: {:?}", e);
                    vec![]
                }
            };

            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            // Use the utils.rs helper function
            crate::handlers::utils::render_domain_list_page(
                domains,
                backups,
                &paginated_domains,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(e) => {
            error!("Failed to create domain: {:?}", e);
            let error_message = crate::handlers::utils::handle_database_error(
                &state,
                &locale,
                e,
                "domain",
                &form.domain,
            )
            .await;

            return handle_domain_form_error(
                &state,
                &locale,
                &headers,
                form,
                &error_message,
                false,
            )
            .await;
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get current database ID for restriction checks
    let current_db_id = get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "update_domain")
    {
        return handle_domain_form_error(
            &state,
            &locale,
            &headers,
            form,
            "error-operation-not-allowed",
            true,
        )
        .await;
    }

    // Validate form data
    if let Err(error_key) = validate_domain_form(&form) {
        return handle_domain_form_error(&state, &locale, &headers, form, &error_key, true).await;
    }

    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let domain_name = form.domain.clone();

    match db::update_domain(&pool, id, form) {
        Ok(_) => {
            info!("Successfully updated domain: {}", domain_name);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            render_domain_show_page(&state, &headers, domain).await
        }
        Err(e) => {
            error!("Failed to update domain: {:?}", e);
            let error_message = crate::handlers::utils::handle_database_error(
                &state,
                &locale,
                e,
                "domain",
                &domain_name,
            )
            .await;

            // Recreate the form for error display
            let error_form = DomainForm {
                domain: domain_name,
                transport: "virtual".to_string(),
                enabled: true,
            };

            handle_domain_form_error(&state, &locale, &headers, error_form, &error_message, true)
                .await
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    match db::delete_domain(&pool, id) {
        Ok(_) => {
            info!("Successfully deleted domain with ID: {}", id);

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after deletion: {:?}", e);
                    vec![]
                }
            };

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            // Use the helper function for rendering
            crate::handlers::utils::render_domain_list_page(
                domains,
                backups,
                &paginated_domains,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(e) => {
            error!("Failed to delete domain: {:?}", e);
            return crate::handlers::utils::render_500_page(&state, &headers).await;
        }
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!("Successfully toggled domain enabled status for ID: {}", id);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            render_domain_show_page(&state, &headers, domain).await
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            return crate::handlers::utils::render_500_page(&state, &headers).await;
        }
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!(
                "Successfully toggled domain enabled status for ID: {} (list view)",
                id
            );

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after toggle: {:?}", e);
                    vec![]
                }
            };

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            // Use the helper function for rendering
            crate::handlers::utils::render_domain_list_page(
                domains,
                backups,
                &paginated_domains,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            return crate::handlers::utils::render_500_page(&state, &headers).await;
        }
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!(
                "Successfully toggled domain enabled status for ID: {} (show view)",
                id
            );

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            render_domain_show_page(&state, &headers, domain).await
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            return crate::handlers::utils::render_500_page(&state, &headers).await;
        }
    }
}

// Add missing required aliases for a domain
pub async fn add_missing_required_aliases(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Load configuration to get required aliases
    let _config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Get current alias report to see what's missing
    let alias_report = match db::get_domain_alias_report(&pool, &domain.domain) {
        Ok(report) => report,
        Err(e) => {
            error!(
                "Failed to get alias report for domain {}: {:?}",
                domain.domain, e
            );
            let error_msg = get_translation(&state, &locale, "domains-error-loading-report").await;
            return Html(error_msg);
        }
    };

    // Create aliases for missing required aliases
    let aliases_to_create: Vec<(String, String)> = alias_report
        .missing_required_aliases
        .iter()
        .map(|alias| (alias.clone(), format!("admin@{}", domain.domain)))
        .collect();

    if !aliases_to_create.is_empty() {
        match db::create_domain_aliases(&pool, &domain.domain, aliases_to_create) {
            Ok(created_aliases) => {
                info!(
                    "Created {} missing required aliases for domain {}",
                    created_aliases.len(),
                    domain.domain
                );
            }
            Err(e) => {
                error!(
                    "Failed to create missing required aliases for domain {}: {:?}",
                    domain.domain, e
                );
                let error_msg =
                    get_translation(&state, &locale, "domains-error-creating-aliases").await;
                return Html(error_msg);
            }
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!(
        "<script>window.location.href = '{redirect_url}';</script>"
    ))
}

// Add a single missing required alias for a domain
pub async fn add_missing_required_alias(
    State(state): State<AppState>,
    Path((id, alias)): Path<(i32, String)>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Create the alias (destination defaults to admin@domain)
    let destination = format!("admin@{}", domain.domain);
    let aliases_to_create = vec![(alias.clone(), destination)];

    match db::create_domain_aliases(&pool, &domain.domain, aliases_to_create) {
        Ok(_created_aliases) => {
            info!(
                "Created missing required alias {} for domain {}",
                alias, domain.domain
            );
        }
        Err(e) => {
            error!(
                "Failed to create missing required alias {} for domain {}: {:?}",
                alias, domain.domain, e
            );
            let error_msg =
                get_translation(&state, &locale, "domains-error-creating-aliases").await;
            return Html(error_msg);
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!(
        "<script>window.location.href = '{redirect_url}';</script>"
    ))
}

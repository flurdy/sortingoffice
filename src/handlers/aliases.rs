use axum::{
    extract::{Form, Path, Query, State},
    http::HeaderMap,
    response::Html,
};
use serde::Deserialize;

use crate::{
    analytics::find_database_common_aliases,
    db,
    handlers::{
        database_ops::{get_entity_or_handle_error, handle_entity_operation},
        language::get_user_locale,
        translations::get_translations_batch,
        utils::handle_database_error,
    },
    i18n::get_translation,
    models::{PaginatedResult, PaginationParams},
    templates::aliases::{AliasSearchResultsTemplate, DomainSearchResultsTemplate},
    AppState,
};

#[derive(Deserialize)]
pub struct AliasPrefill {
    pub domain: Option<String>,
    pub alias: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AliasSearchQuery {
    pub destination: Option<String>,
    pub alias: Option<String>,
    pub limit: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct DomainSearchQuery {
    pub domain: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let sort_by = params.sort_by.as_deref();
    let sort_order = params.sort_order.as_deref();

    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    let paginated_aliases =
        match db::get_aliases_paginated(&pool, page, per_page, sort_by, sort_order) {
            Ok(aliases) => aliases,
            Err(_) => PaginatedResult::new(vec![], 0, 1, per_page),
        };

    let locale = get_user_locale(&headers);

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_alias_list_page(
        paginated_aliases.items.clone(),
        &paginated_aliases,
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn new(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(prefill): Query<AliasPrefill>,
) -> Html<String> {
    let return_url = headers
        .get("referer")
        .and_then(|r| r.to_str().ok())
        .filter(|r| r.contains("/domains/"))
        .map(|r| r.to_string());
    log::info!(
        "Add Alias form requested. prefill: alias={:?}, domain={:?}, referer={:?}",
        prefill.alias,
        prefill.domain,
        return_url
    );
    let mail = match (&prefill.alias, &prefill.domain) {
        (Some(alias), Some(domain)) => {
            // Special case: if alias is "@", don't add another "@" for catch-all
            if alias == "@" {
                format!("@{domain}")
            } else {
                format!("{alias}@{domain}")
            }
        }
        (Some(alias), None) => alias.clone(),
        (None, Some(domain)) => domain.to_string(),
        (None, None) => "".to_string(),
    };
    let form = crate::models::AliasForm {
        mail,
        destination: "".to_string(),
        enabled: true,
        return_url: None,
        redirect_to: None,
    };
    let locale = get_user_locale(&headers);

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_alias_form_page(
        form,
        None, // No existing alias for new form
        "aliases-add-title",
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    let alias = match db::get_alias(&pool, id) {
        Ok(alias) => alias,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "aliases",
                "aliases-not-found",
            )
            .await;
        }
    };

    let locale = get_user_locale(&headers);

    // Extract domain from alias mail and look it up
    let domain_name = alias.mail.split('@').next_back().unwrap_or("");
    let domain_info = db::get_domain_by_name(&pool, domain_name).ok();

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_alias_show_page(
        alias,
        domain_info,
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    let alias = match db::get_alias(&pool, id) {
        Ok(alias) => alias,
        Err(_) => {
            return crate::handlers::errors::render_alias_not_found_page(&state, &headers).await
        }
    };

    let form = crate::models::AliasForm {
        mail: alias.mail.clone(),
        destination: alias.destination.clone(),
        enabled: alias.enabled,
        return_url: None,
        redirect_to: None,
    };

    let locale = get_user_locale(&headers);

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_alias_form_page(
        form,
        Some(alias), // Pass the existing alias for edit form
        "aliases-edit-title",
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<crate::models::AliasForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    // Validate alias mail using helper function
    if let Err(error_html) = validate_alias_form_field(
        &state,
        &headers,
        &form,
        |f| crate::validation::validate_alias_mail(&f.mail),
        "validation-alias-mail-invalid",
    )
    .await
    {
        return error_html;
    }

    // Validate alias destination using helper function
    if let Err(error_html) = validate_alias_form_field(
        &state,
        &headers,
        &form,
        |f| crate::validation::validate_alias_destination(&f.destination),
        "validation-alias-destination-invalid",
    )
    .await
    {
        return error_html;
    }

    match db::create_alias(&pool, form.clone()) {
        Ok(created_alias) => {
            // Extract domain from the created alias
            let domain_name = created_alias.domain();

            // Try to find the domain by name
            match db::get_domain_by_name(&pool, &domain_name) {
                Ok(domain) => {
                    // Determine redirect destination based on form parameter
                    match form.redirect_to.as_deref() {
                        Some("aliases") => {
                            // Redirect to aliases list page using resource-specific helper
                            let aliases = db::get_aliases(&pool).unwrap_or_default();
                            let locale = get_user_locale(&headers);
                            let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);

                            crate::handlers::rendering::render_alias_list_page(
                                aliases, &paginated, &state, &locale, &headers,
                            )
                            .await
                        }
                        _ => {
                            // Default: redirect to domain show page with proper alias data
                            let locale = get_user_locale(&headers);
                            let alias_report =
                                db::get_domain_alias_report(&pool, &domain.domain).ok();
                            let existing_aliases =
                                db::get_aliases_for_domain(&pool, &domain.domain)
                                    .unwrap_or_default();
                            let analytics_common_aliases =
                                find_database_common_aliases(&state, &headers, 10, 3).await;

                            // Get relays for this domain
                            let domain_relays = db::get_relays_for_domain(&pool, &domain.domain)
                                .unwrap_or_default();

                            crate::handlers::rendering::render_domain_show_page(
                                domain,
                                alias_report,
                                existing_aliases,
                                analytics_common_aliases,
                                domain_relays,
                                &state,
                                &locale,
                                &headers,
                            )
                            .await
                        }
                    }
                }
                Err(_) => {
                    // Domain not found, redirect to aliases list
                    let aliases = db::get_aliases(&pool).unwrap_or_default();
                    let locale = get_user_locale(&headers);
                    let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);

                    crate::handlers::rendering::render_alias_list_page(
                        aliases, &paginated, &state, &locale, &headers,
                    )
                    .await
                }
            }
        }
        Err(e) => {
            let locale = get_user_locale(&headers);
            let _error_msg = handle_database_error(&state, &locale, e, "aliases", &form.mail).await;

            // Use resource-specific helper for form with error
            crate::handlers::rendering::render_alias_form_page(
                form,
                None,
                "aliases-add-title",
                &state,
                &locale,
                &headers,
            )
            .await
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<crate::models::AliasForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    // Validate alias mail
    match crate::validation::validate_alias_mail(&form.mail) {
        Ok(_) => {}
        Err(_e) => {
            let locale = get_user_locale(&headers);
            let _error_msg =
                get_translation(&state, &locale, "validation-alias-mail-invalid").await;

            // Use resource-specific helper for form with error
            return crate::handlers::rendering::render_alias_form_page(
                form,
                None,
                "aliases-edit-title",
                &state,
                &locale,
                &headers,
            )
            .await;
        }
    }

    // Validate alias destination
    match crate::validation::validate_alias_destination(&form.destination) {
        Ok(_) => {}
        Err(_e) => {
            let locale = get_user_locale(&headers);
            let _error_msg =
                get_translation(&state, &locale, "validation-alias-destination-invalid").await;

            // Use resource-specific helper for form with error
            return crate::handlers::rendering::render_alias_form_page(
                form,
                None,
                "aliases-edit-title",
                &state,
                &locale,
                &headers,
            )
            .await;
        }
    }

    match db::update_alias(&pool, id, form.clone()) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => {
                    return crate::handlers::utils::handle_entity_not_found(
                        &state,
                        &headers,
                        "aliases",
                        "aliases-not-found",
                    )
                    .await;
                }
            };

            // Extract domain from alias mail and look it up
            let domain_name = alias.mail.split('@').next_back().unwrap_or("");
            let domain_info = db::get_domain_by_name(&pool, domain_name).ok();

            // Use resource-specific helper for alias show page
            let locale = get_user_locale(&headers);
            crate::handlers::rendering::render_alias_show_page(
                alias,
                domain_info,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(e) => {
            let locale = get_user_locale(&headers);
            let _error_msg = handle_database_error(&state, &locale, e, "aliases", &form.mail).await;

            // Use resource-specific helper for form with error
            crate::handlers::rendering::render_alias_form_page(
                form,
                None,
                "aliases-edit-title",
                &state,
                &locale,
                &headers,
            )
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

    let locale = get_user_locale(&headers);

    // Use helper function for entity operation
    match handle_entity_operation(
        || async { db::delete_alias(&pool, id) },
        &state,
        &locale,
        "delete alias",
        &id.to_string(),
        "Successfully deleted alias",
    )
    .await
    {
        Ok(_) => {
            // Get updated aliases list and use resource-specific helper
            let aliases = db::get_aliases(&pool).unwrap_or_default();
            let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);

            crate::handlers::rendering::render_alias_list_page(
                aliases, &paginated, &state, &locale, &headers,
            )
            .await
        }
        Err(error) => error,
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let locale = get_user_locale(&headers);

    // Use helper function for entity operation
    match handle_entity_operation(
        || async { db::toggle_alias_enabled(&pool, id) },
        &state,
        &locale,
        "toggle alias",
        &id.to_string(),
        "Successfully toggled alias",
    )
    .await
    {
        Ok(_) => {
            // Get updated alias using helper function
            match get_entity_or_handle_error(
                || async { db::get_alias(&pool, id) },
                &state,
                &locale,
                "aliases-not-found",
            )
            .await
            {
                Ok(alias) => {
                    // Extract domain from alias mail and look it up
                    let domain_name = alias.mail.split('@').next_back().unwrap_or("");
                    let domain_info = db::get_domain_by_name(&pool, domain_name).ok();

                    // Use resource-specific helper for alias show page
                    crate::handlers::rendering::render_alias_show_page(
                        alias,
                        domain_info,
                        &state,
                        &locale,
                        &headers,
                    )
                    .await
                }
                Err(error) => error,
            }
        }
        Err(error) => error,
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let aliases = db::get_aliases(&pool).unwrap_or_default();
            let locale = get_user_locale(&headers);
            let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);

            // Use the new resource-specific helper function
            crate::handlers::rendering::render_alias_list_page(
                aliases, &paginated, &state, &locale, &headers,
            )
            .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => {
                    return crate::handlers::errors::render_alias_not_found_page(&state, &headers)
                        .await
                }
            };

            let locale = get_user_locale(&headers);

            // Extract domain from alias mail and look it up
            let domain_name = alias.mail.split('@').next_back().unwrap_or("");
            let domain_info = db::get_domain_by_name(&pool, domain_name).ok();

            // Use the new resource-specific helper function
            crate::handlers::rendering::render_alias_show_page(
                alias,
                domain_info,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

pub async fn toggle_enabled_domain_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => {
                    return crate::handlers::errors::render_alias_not_found_page(&state, &headers)
                        .await
                }
            };

            // Extract domain from alias mail and look it up
            let domain_name = alias.mail.split('@').next_back().unwrap_or("");
            let domain = match db::get_domain_by_name(&pool, domain_name) {
                Ok(domain) => domain,
                Err(_) => {
                    return crate::handlers::errors::render_domain_not_found_page(&state, &headers)
                        .await
                }
            };

            let locale = get_user_locale(&headers);

            // Use resource-specific helper for domain show page with proper alias data
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();
            let analytics_common_aliases =
                find_database_common_aliases(&state, &headers, 10, 3).await;

            // Get relays for this domain
            let domain_relays =
                db::get_relays_for_domain(&pool, &domain.domain).unwrap_or_default();

            crate::handlers::rendering::render_domain_show_page(
                domain,
                alias_report,
                existing_aliases,
                analytics_common_aliases,
                domain_relays,
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

pub async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AliasSearchQuery>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    // Extract query string from search parameters
    let query_string = extract_search_query_string(&query);

    // Handle empty or missing query
    if query_string.len() < 2 {
        return render_empty_search_results(&state, &headers).await;
    }

    let limit = query.limit.unwrap_or(10);

    // Search across all data sources
    let search_results = search_all_data_sources(&pool, &query_string, limit).await;

    // Render search results
    render_search_results(&state, &headers, search_results).await
}

pub async fn domain_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DomainSearchQuery>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    let query_string = query.domain.unwrap_or_default();

    // Handle empty or missing query
    if query_string.len() < 2 {
        let locale = get_user_locale(&headers);
        let translations = get_translations_batch(
            &state,
            &locale,
            &[
                "domains-search-no-results",
                "domains-search-select",
                "status-active",
                "status-inactive",
            ],
        )
        .await;
        let content_template = DomainSearchResultsTemplate {
            domains: &[],
            no_results: &translations["domains-search-no-results"],
            select_text: &translations["domains-search-select"],
            status_active: &translations["status-active"],
            status_inactive: &translations["status-inactive"],
        };
        return match crate::handlers::templates::render_template_safely(content_template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(&state, &headers).await,
        };
    }

    let limit = query.limit.unwrap_or(10);
    let search_results = db::search_domains(&pool, &query_string, limit);

    let domains = search_results.unwrap_or_default();

    let locale = get_user_locale(&headers);
    let translations = get_translations_batch(
        &state,
        &locale,
        &[
            "domains-search-no-results",
            "domains-search-select",
            "status-active",
            "status-inactive",
        ],
    )
    .await;
    let content_template = DomainSearchResultsTemplate {
        domains: &domains,
        no_results: &translations["domains-search-no-results"],
        select_text: &translations["domains-search-select"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
    };
    match crate::handlers::templates::render_template_safely(content_template) {
        Ok(content) => Html(content),
        Err(_) => crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

/// Validate alias form field with error handling
pub async fn validate_alias_form_field<F>(
    state: &AppState,
    headers: &HeaderMap,
    form: &crate::models::AliasForm,
    validator: F,
    error_key: &str,
) -> Result<(), Html<String>>
where
    F: FnOnce(&crate::models::AliasForm) -> Result<(), crate::validation::ValidationError>,
{
    match validator(form) {
        Ok(_) => Ok(()),
        Err(_) => {
            let locale = crate::handlers::http_helpers::get_user_locale(headers);
            let error_msg = crate::i18n::get_translation(state, &locale, error_key).await;

            // Get form translations
            let form_translations = crate::handlers::translations::get_entity_form_translations(
                state, &locale, "aliases",
            )
            .await;

            // Get individual translations like the rendering module does
            let mail_address =
                crate::i18n::get_translation(state, &locale, "aliases-field-mail").await;
            let destination =
                crate::i18n::get_translation(state, &locale, "aliases-field-destination").await;
            let placeholder_mail =
                crate::i18n::get_translation(state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination =
                crate::i18n::get_translation(state, &locale, "aliases-placeholder-destination")
                    .await;
            let tooltip_mail =
                crate::i18n::get_translation(state, &locale, "aliases-field-mail-help").await;
            let tooltip_destination =
                crate::i18n::get_translation(state, &locale, "aliases-field-destination-help")
                    .await;
            let active = crate::i18n::get_translation(state, &locale, "form-enabled").await;
            let tooltip_active =
                crate::i18n::get_translation(state, &locale, "aliases-tooltip-enabled").await;
            let cancel = crate::i18n::get_translation(state, &locale, "form-cancel").await;
            let update_alias =
                crate::i18n::get_translation(state, &locale, "aliases-update-alias").await;
            let create_alias =
                crate::i18n::get_translation(state, &locale, "aliases-create-alias").await;

            let content_template = crate::templates::aliases::AliasFormTemplate {
                title: &form_translations["aliases-add-title"],
                alias: None,
                form: form.clone(),
                error: Some(error_msg),
                return_url: form.return_url.clone(),
                edit_alias: &form_translations["aliases-edit-alias"],
                new_alias: &form_translations["aliases-new-alias"],
                form_error: &form_translations["form-error"],
                mail_address: &mail_address,
                destination: &destination,
                placeholder_mail: &placeholder_mail,
                placeholder_destination: &placeholder_destination,
                tooltip_mail: &tooltip_mail,
                tooltip_destination: &tooltip_destination,
                active: &active,
                tooltip_active: &tooltip_active,
                cancel: &cancel,
                update_alias: &update_alias,
                create_alias: &create_alias,
                not_available: &form_translations["not-available"],
            };

            let error_html = crate::handlers::utils::render_form_template(
                content_template,
                state,
                &locale,
                headers,
                form_translations["aliases-add-title"].clone(),
            )
            .await;
            Err(error_html)
        }
    }
}

/// Extract search query string from search parameters
fn extract_search_query_string(query: &AliasSearchQuery) -> String {
    if let Some(alias_query) = &query.alias {
        alias_query.clone()
    } else if let Some(dest_query) = &query.destination {
        dest_query.clone()
    } else {
        String::new()
    }
}

/// Render empty search results when query is too short
async fn render_empty_search_results(state: &AppState, headers: &HeaderMap) -> Html<String> {
    let locale = get_user_locale(headers);
    let translations = get_translations_batch(
        state,
        &locale,
        &["aliases-search-no-results", "aliases-search-select"],
    )
    .await;
    let content_template = AliasSearchResultsTemplate {
        aliases: &[],
        no_results: &translations["aliases-search-no-results"],
        select_text: &translations["aliases-search-select"],
    };
    match crate::handlers::templates::render_template_safely(content_template) {
        Ok(content) => Html(content),
        Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
    }
}

/// Search across all data sources (aliases, users)
async fn search_all_data_sources(
    pool: &crate::DbPool,
    query_string: &str,
    limit: i64,
) -> Vec<String> {
    let mut values = std::collections::HashSet::new();

    // Search aliases
    search_aliases_data(pool, query_string, limit, &mut values).await;

    // Search users
    search_users_data(pool, query_string, limit, &mut values).await;

    // Sort and limit results
    let mut results: Vec<String> = values.into_iter().collect();
    results.sort_by_key(|a| a.to_lowercase());
    results.truncate(limit as usize);
    results
}

/// Search aliases data source
async fn search_aliases_data(
    pool: &crate::DbPool,
    query_string: &str,
    limit: i64,
    values: &mut std::collections::HashSet<String>,
) {
    if let Ok(aliases) = db::search_aliases(pool, query_string, limit * 2) {
        for alias in aliases {
            if alias.mail.contains(query_string) {
                values.insert(alias.mail);
            }
            if alias.destination.contains(query_string) {
                values.insert(alias.destination);
            }
        }
    }
}

/// Search users data source
async fn search_users_data(
    pool: &crate::DbPool,
    query_string: &str,
    limit: i64,
    values: &mut std::collections::HashSet<String>,
) {
    use diesel::prelude::*;
    if let Ok(mut conn) = pool.get() {
        let search_pattern = format!("%{query_string}%");
        let user_ids: Vec<String> = crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::id.like(&search_pattern))
            .select(crate::schema::users::dsl::id)
            .limit(limit * 2)
            .load::<String>(&mut conn)
            .unwrap_or_default();
        for user_id in user_ids {
            values.insert(user_id);
        }
    }
}

/// Render search results as HTML
async fn render_search_results(
    state: &AppState,
    headers: &HeaderMap,
    search_results: Vec<String>,
) -> Html<String> {
    let html = if search_results.is_empty() {
        let locale = get_user_locale(headers);
        let translations = get_translations_batch(
            state,
            &locale,
            &["aliases-search-no-results", "aliases-search-select"],
        )
        .await;
        format!(
            "<ul><li class=\"text-gray-400\">{}</li></ul>",
            translations["aliases-search-no-results"]
        )
    } else {
        let items: String = search_results
            .into_iter()
            .map(|v| format!("<li class=\"cursor-pointer\">{v}</li>"))
            .collect();
        format!("<ul>{items}</ul>")
    };

    Html(html)
}

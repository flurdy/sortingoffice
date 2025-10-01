use crate::{db, get_entity_or_not_found, i18n::get_translation, models::*, AppState};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use log::error;

use crate::handlers::database_ops::{get_entity_or_handle_error, handle_entity_operation};
use crate::handlers::rendering::{render_backup_form_page, render_backup_show_page};
use std::fmt::Write as FmtWrite;

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = BackupForm {
        domain: String::new(),
        transport: "smtp:[]".to_string(),
        enabled: true,
    };

    render_backup_form_page(form, None, "backups-new-backup", &state, &locale, &headers).await
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
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = get_entity_or_not_found!(
        db::get_backup(&pool, id),
        &state,
        &headers,
        "backups-not-found"
    );

    render_backup_show_page(backup, &state, &locale, &headers).await
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
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = get_entity_or_not_found!(
        db::get_backup(&pool, id),
        &state,
        &headers,
        "backups-not-found"
    );

    let form = BackupForm {
        domain: backup.domain.clone(),
        transport: backup
            .transport
            .clone()
            .unwrap_or_else(|| "smtp:[]".to_string()),
        enabled: backup.enabled,
    };

    render_backup_form_page(
        form,
        Some(backup),
        "backups-edit-backup-title",
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate form data
    if form.domain.trim().is_empty() {
        return crate::handlers::rendering::render_backup_form_page_with_error(
            form,
            None,
            "backups-new-backup",
            "validation-domain-required",
            &state,
            &locale,
            &headers,
        )
        .await;
    }

    // Validate domain format
    match crate::validation::validate_domain(form.domain.trim()) {
        Ok(_) => {}
        Err(_e) => {
            return crate::handlers::rendering::render_backup_form_page_with_error(
                form,
                None,
                "backups-new-backup",
                "validation-domain-invalid",
                &state,
                &locale,
                &headers,
            )
            .await;
        }
    }

    let new_backup = NewBackup {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
        enabled: form.enabled,
    };

    match db::create_backup(&pool, new_backup) {
        Ok(_) => {
            // Redirect to domains page after creating backup
            Html("<script>window.location.href='/domains';</script>".to_string())
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-backup")
                    .await
                    .replace("{domain}", &form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => get_translation(&state, &locale, "error-constraint-violation").await,
                _ => get_translation(&state, &locale, "error-unexpected").await,
            };

            return crate::handlers::rendering::render_backup_form_page_with_error(
                form,
                None,
                "backups-new-backup",
                &error_message,
                &state,
                &locale,
                &headers,
            )
            .await;
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate form data
    if form.domain.trim().is_empty() {
        return crate::handlers::rendering::render_backup_form_page_with_error(
            form,
            None,
            "backups-edit-backup-title",
            "validation-domain-required",
            &state,
            &locale,
            &headers,
        )
        .await;
    }

    match db::update_backup(&pool, id, form.clone()) {
        Ok(_) => {
            let backup = match db::get_backup(&pool, id) {
                Ok(backup) => backup,
                Err(_) => {
                    return crate::handlers::errors::render_backup_not_found_page(&state, &headers)
                        .await
                }
            };
            // Use the helper function for rendering
            crate::handlers::rendering::render_backup_show_page(backup, &state, &locale, &headers)
                .await
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-backup")
                    .await
                    .replace("{domain}", &form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => get_translation(&state, &locale, "error-constraint-violation").await,
                _ => get_translation(&state, &locale, "error-unexpected").await,
            };

            return crate::handlers::rendering::render_backup_form_page_with_error(
                form,
                None,
                "backups-edit-backup-title",
                &error_message,
                &state,
                &locale,
                &headers,
            )
            .await;
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

    let locale = crate::handlers::language::get_user_locale(&headers);

    // Use helper function for entity operation
    match handle_entity_operation(
        || async { db::delete_backup(&pool, id) },
        &state,
        &locale,
        "delete backup",
        &id.to_string(),
        "Successfully deleted backup",
    )
    .await
    {
        Ok(_) => {
            // Redirect to domains page after deleting backup
            Html("<script>window.location.href='/domains';</script>".to_string())
        }
        Err(error) => error,
    }
}

#[derive(serde::Deserialize, Default)]
pub struct DnsLookupForm {
    selector: Option<String>,
}

pub async fn dns_lookup(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<DnsLookupForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(_) => return Html("Database connection error".to_string()),
    };
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = match crate::db::get_backup(&pool, id) {
        Ok(b) => b,
        Err(_) => {
            let not_found_msg =
                crate::i18n::get_translation(&state, &locale, "backups-not-found").await;
            return Html(not_found_msg);
        }
    };

    let resolver = match crate::services::dns_lookup::DnsLookupService::new_system().await {
        Ok(r) => r,
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    };

    let result = match resolver.lookup_all(&backup.domain).await {
        Ok(r) => r,
        Err(_) => crate::services::dns_lookup::DnsLookupResult::default(),
    };

    // i18n for fragment
    let dns_records_title =
        crate::i18n::get_translation(&state, &locale, "dns-records-title").await;
    let dns_ns_header = crate::i18n::get_translation(&state, &locale, "dns-ns-header").await;
    let dns_mx_header = crate::i18n::get_translation(&state, &locale, "dns-mx-header").await;
    let dns_txt_header = crate::i18n::get_translation(&state, &locale, "dns-txt-header").await;
    let dns_dkim_header = crate::i18n::get_translation(&state, &locale, "dns-dkim-header").await;

    let mut html = String::new();
    html.push_str("<div class=\"mt-6\"><h3 class=\"text-lg font-medium\">");
    let _ = FmtWrite::write_fmt(
        &mut html,
        format_args!(
            "{}",
            askama_escape::escape(&dns_records_title, askama_escape::Html)
        ),
    );
    html.push_str("</h3>");
    if !result.ns_records.is_empty() {
        html.push_str("<div class=\"mt-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900\">");
        html.push_str("<h4 class=\"font-semibold text-gray-900 dark:text-gray-100\">");
        let _ = FmtWrite::write_fmt(
            &mut html,
            format_args!(
                "{}",
                askama_escape::escape(&dns_ns_header, askama_escape::Html)
            ),
        );
        html.push_str(
            "</h4><ul class=\"list-disc ml-5 mt-2 text-sm text-gray-800 dark:text-gray-200\">",
        );
        for ns in result.ns_records {
            html.push_str(&format!("<li>{}</li>", ns));
        }
        html.push_str("</ul></div>");
    }
    if !result.mx_records.is_empty() {
        html.push_str("<div class=\"mt-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900\">");
        html.push_str("<h4 class=\"font-semibold text-gray-900 dark:text-gray-100\">");
        let _ = FmtWrite::write_fmt(
            &mut html,
            format_args!(
                "{}",
                askama_escape::escape(&dns_mx_header, askama_escape::Html)
            ),
        );
        html.push_str(
            "</h4><ul class=\"list-disc ml-5 mt-2 text-sm text-gray-800 dark:text-gray-200\">",
        );
        for mx in result.mx_records {
            html.push_str(&format!("<li>{}: {}</li>", mx.priority, mx.exchange));
        }
        html.push_str("</ul></div>");
    }
    if !result.txt_records.is_empty() {
        html.push_str("<div class=\"mt-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900\">");
        html.push_str("<h4 class=\"font-semibold text-gray-900 dark:text-gray-100\">");
        let _ = FmtWrite::write_fmt(
            &mut html,
            format_args!(
                "{}",
                askama_escape::escape(&dns_txt_header, askama_escape::Html)
            ),
        );
        html.push_str(
            "</h4><ul class=\"list-disc ml-5 mt-2 text-sm text-gray-800 dark:text-gray-200\">",
        );
        for t in result.txt_records {
            let _ = FmtWrite::write_fmt(
                &mut html,
                format_args!(
                    "<li><code>{}</code></li>",
                    askama_escape::escape(&t, askama_escape::Html)
                ),
            );
        }
        html.push_str("</ul></div>");
    }

    // DKIM: use provided selector or try common ones
    if let Some(sel) = form
        .selector
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Ok(records) = resolver.lookup_dkim(sel, &backup.domain).await {
            if !records.is_empty() {
                html.push_str("<div class=\"mt-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900\">");
                html.push_str("<h4 class=\"font-semibold text-gray-900 dark:text-gray-100\">");
                let _ = FmtWrite::write_fmt(
                    &mut html,
                    format_args!(
                        "{}",
                        askama_escape::escape(&dns_dkim_header, askama_escape::Html)
                    ),
                );
                html.push_str("</h4><ul class=\"list-disc ml-5 mt-2 text-sm text-gray-800 dark:text-gray-200\">");
                for rec in records {
                    let _ = FmtWrite::write_fmt(
                        &mut html,
                        format_args!(
                            "<li><code>{}</code></li>",
                            askama_escape::escape(&rec, askama_escape::Html)
                        ),
                    );
                }
                html.push_str("</ul></div>");
            }
        }
    } else {
        let common_selectors = [
            "s1",
            "s2",
            "default",
            "selector1",
            "selector",
            "k1",
            "google",
        ];
        let mut any_found = false;
        let mut per_selector: Vec<(String, Vec<String>)> = Vec::new();
        for s in common_selectors.iter() {
            if let Ok(records) = resolver.lookup_dkim(s, &backup.domain).await {
                if !records.is_empty() {
                    any_found = true;
                    per_selector.push((s.to_string(), records));
                }
            }
        }
        if any_found {
            let dkim_fallback_desc =
                crate::i18n::get_translation(&state, &locale, "dns-dkim-fallback-description")
                    .await;
            html.push_str("<div class=\"mt-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900\">");
            html.push_str("<h4 class=\"font-semibold text-gray-900 dark:text-gray-100\">");
            let _ = FmtWrite::write_fmt(
                &mut html,
                format_args!(
                    "{}",
                    askama_escape::escape(&dns_dkim_header, askama_escape::Html)
                ),
            );
            html.push_str("</h4>");
            let _ = FmtWrite::write_fmt(
                &mut html,
                format_args!(
                    "<p class=\"mt-1 text-xs text-gray-600 dark:text-gray-400\">{}</p>",
                    askama_escape::escape(&dkim_fallback_desc, askama_escape::Html)
                ),
            );
            for (selector, recs) in per_selector {
                let _ = FmtWrite::write_fmt(
                    &mut html,
                    format_args!("<div class=\"mt-2\"><div class=\"text-sm text-gray-600 dark:text-gray-300\">selector: <code>{}</code></div>", askama_escape::escape(&selector, askama_escape::Html)),
                );
                html.push_str(
                    "<ul class=\"list-disc ml-5 mt-1 text-sm text-gray-800 dark:text-gray-200\">",
                );
                for rec in recs {
                    let _ = FmtWrite::write_fmt(
                        &mut html,
                        format_args!(
                            "<li><code>{}</code></li>",
                            askama_escape::escape(&rec, askama_escape::Html)
                        ),
                    );
                }
                html.push_str("</ul></div>");
            }
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {e:?}");
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    match handle_entity_operation(
        || async { db::toggle_backup_enabled(&pool, id) },
        &state,
        &locale,
        "backup",
        &id.to_string(),
        "backups-toggle-success",
    )
    .await
    {
        Ok(_) => {
            // Get updated backup using helper function
            match get_entity_or_handle_error(
                || async { db::get_backup(&pool, id) },
                &state,
                &locale,
                "backups-not-found",
            )
            .await
            {
                Ok(backup) => {
                    // Use the helper function for rendering
                    crate::handlers::rendering::render_backup_show_page(
                        backup, &state, &locale, &headers,
                    )
                    .await
                }
                Err(error) => error,
            }
        }
        Err(error) => error,
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
    let locale = crate::handlers::language::get_user_locale(&headers);
    match db::toggle_backup_enabled(&pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(&pool, id) {
                Ok(backup) => backup,
                Err(_) => {
                    return crate::handlers::errors::render_backup_not_found_page(&state, &headers)
                        .await
                }
            };
            // Use the helper function for rendering
            crate::handlers::rendering::render_backup_show_page(backup, &state, &locale, &headers)
                .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

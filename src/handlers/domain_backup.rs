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
// use std::fmt::Write as FmtWrite;

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

    // Fetch related data for this backup domain and pass to renderer
    let (domain_relays, domain_users, existing_aliases) = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => {
            let relays = crate::db::get_relays_for_domain(&pool, &backup.domain).unwrap_or_default();
            let users = crate::db::get_users_for_domain(&pool, &backup.domain).unwrap_or_default();
            let aliases = crate::db::get_aliases_for_domain(&pool, &backup.domain).unwrap_or_default();
            (relays, users, aliases)
        },
        Err(_) => (Vec::new(), Vec::new(), Vec::new()),
    };

    render_backup_show_page(backup, domain_relays, domain_users, existing_aliases, &state, &locale, &headers).await
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
            // Reload relays as well
            {
                let (domain_relays, domain_users, existing_aliases) = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
                    Ok(pool) => {
                        let relays = crate::db::get_relays_for_domain(&pool, &backup.domain).unwrap_or_default();
                        let users = crate::db::get_users_for_domain(&pool, &backup.domain).unwrap_or_default();
                        let aliases = crate::db::get_aliases_for_domain(&pool, &backup.domain).unwrap_or_default();
                        (relays, users, aliases)
                    },
                    Err(_) => (Vec::new(), Vec::new(), Vec::new()),
                };
                crate::handlers::rendering::render_backup_show_page(backup, domain_relays, domain_users, existing_aliases, &state, &locale, &headers)
            }
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
    Form(form): Form<DnsLookupForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(_) => return Html("Database connection error".to_string()),
    };

    let backup = match crate::db::get_backup(&pool, id) {
        Ok(b) => b,
        Err(_) => {
            let locale = crate::handlers::language::get_user_locale(&headers);
            let not_found =
                crate::i18n::get_translation(&state, &locale, "backups-not-found").await;
            return Html(not_found);
        }
    };

    crate::handlers::dns::render_dns_fragment(&state, &headers, &backup.domain, form.selector).await
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
                    // Use the helper function for rendering with relays
                    {
                        let (domain_relays, domain_users, existing_aliases) = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
                            Ok(pool) => {
                                let relays = crate::db::get_relays_for_domain(&pool, &backup.domain).unwrap_or_default();
                                let users = crate::db::get_users_for_domain(&pool, &backup.domain).unwrap_or_default();
                                let aliases = crate::db::get_aliases_for_domain(&pool, &backup.domain).unwrap_or_default();
                                (relays, users, aliases)
                            },
                            Err(_) => (Vec::new(), Vec::new(), Vec::new()),
                        };
                        crate::handlers::rendering::render_backup_show_page(
                            backup,
                            domain_relays,
                            domain_users,
                            existing_aliases,
                            &state,
                            &locale,
                            &headers,
                        )
                    }
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
            {
                let (domain_relays, domain_users, existing_aliases) = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
                    Ok(pool) => {
                        let relays = crate::db::get_relays_for_domain(&pool, &backup.domain).unwrap_or_default();
                        let users = crate::db::get_users_for_domain(&pool, &backup.domain).unwrap_or_default();
                        let aliases = crate::db::get_aliases_for_domain(&pool, &backup.domain).unwrap_or_default();
                        (relays, users, aliases)
                    },
                    Err(_) => (Vec::new(), Vec::new(), Vec::new()),
                };
                crate::handlers::rendering::render_backup_show_page(backup, domain_relays, domain_users, existing_aliases, &state, &locale, &headers)
            }
                .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}

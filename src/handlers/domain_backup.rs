use crate::{db, get_entity_or_not_found, i18n::get_translation, models::*, AppState};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use log::error;

use crate::handlers::utils::{render_backup_form_page, render_backup_show_page};

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = BackupForm {
        domain: String::new(),
        transport: "smtp:[]".to_string(),
        enabled: true,
    };

    render_backup_form_page(form, None, "backups-add-title", &state, &locale, &headers).await
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
        return crate::handlers::utils::render_backup_form_page_with_error(
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
            return crate::handlers::utils::render_backup_form_page_with_error(
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

            return crate::handlers::utils::render_backup_form_page_with_error(
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
        return crate::handlers::utils::render_backup_form_page_with_error(
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
                    return crate::handlers::utils::render_backup_not_found_page(&state, &headers)
                        .await
                }
            };
            // Use the helper function for rendering
            crate::handlers::utils::render_backup_show_page(backup, &state, &locale, &headers).await
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

            return crate::handlers::utils::render_backup_form_page_with_error(
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
    match crate::handlers::utils::handle_entity_operation(
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

    let locale = crate::handlers::utils::get_user_locale(&headers);

    match crate::handlers::utils::handle_entity_operation(
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
            match crate::handlers::utils::get_entity_or_handle_error(
                || async { db::get_backup(&pool, id) },
                &state,
                &locale,
                "backups-not-found",
            )
            .await
            {
                Ok(backup) => {
                    // Use the helper function for rendering
                    crate::handlers::utils::render_backup_show_page(
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
                    return crate::handlers::utils::render_backup_not_found_page(&state, &headers)
                        .await
                }
            };
            // Use the helper function for rendering
            crate::handlers::utils::render_backup_show_page(backup, &state, &locale, &headers).await
        }
        Err(_) => return crate::handlers::utils::render_500_page(&state, &headers).await,
    }
}

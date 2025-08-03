use crate::templates::domain_backup::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, get_entity_or_not_found, i18n::get_translation, models::*, AppState};
use askama::Template;
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
        let content_template = BackupFormTemplate {
            title: get_translation(&state, &locale, "backups-new-backup").await,
            form_error: get_translation(&state, &locale, "backups-form-error").await,
            form_domain: get_translation(&state, &locale, "backups-form-domain").await,
            form_transport: get_translation(&state, &locale, "backups-form-transport").await,
            form_active: get_translation(&state, &locale, "backups-form-active").await,
            placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain")
                .await,
            placeholder_transport: get_translation(
                &state,
                &locale,
                "backups-placeholder-transport",
            )
            .await,
            tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
            tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport").await,
            tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
            cancel: get_translation(&state, &locale, "backups-cancel").await,
            create_backup: get_translation(&state, &locale, "backups-create-backup").await,
            update_backup: get_translation(&state, &locale, "backups-update-backup").await,
            new_backup: get_translation(&state, &locale, "backups-new-backup").await,
            edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title").await,
            backup: None,
            form,
            error: Some(get_translation(&state, &locale, "validation-domain-required").await),
        };
        return match crate::handlers::utils::render_template_safely(content_template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
        };
    }

    // Validate domain format
    match crate::validation::validate_domain(form.domain.trim()) {
        Ok(_) => {}
        Err(_e) => {
            let content_template = BackupFormTemplate {
                title: get_translation(&state, &locale, "backups-new-backup").await,
                form_error: get_translation(&state, &locale, "backups-form-error").await,
                form_domain: get_translation(&state, &locale, "backups-form-domain").await,
                form_transport: get_translation(&state, &locale, "backups-form-transport").await,
                form_active: get_translation(&state, &locale, "backups-form-active").await,
                placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain")
                    .await,
                placeholder_transport: get_translation(
                    &state,
                    &locale,
                    "backups-placeholder-transport",
                )
                .await,
                tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
                tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport")
                    .await,
                tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
                cancel: get_translation(&state, &locale, "backups-cancel").await,
                create_backup: get_translation(&state, &locale, "backups-create-backup").await,
                update_backup: get_translation(&state, &locale, "backups-update-backup").await,
                new_backup: get_translation(&state, &locale, "backups-new-backup").await,
                edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title")
                    .await,
                backup: None,
                form,
                error: Some(get_translation(&state, &locale, "validation-domain-invalid").await),
            };
            return match crate::handlers::utils::render_template_safely(content_template) {
                Ok(content) => Html(content),
                Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
            };
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

            let content_template = BackupFormTemplate {
                title: get_translation(&state, &locale, "backups-new-backup").await,
                form_error: get_translation(&state, &locale, "backups-form-error").await,
                form_domain: get_translation(&state, &locale, "backups-form-domain").await,
                form_transport: get_translation(&state, &locale, "backups-form-transport").await,
                form_active: get_translation(&state, &locale, "backups-form-active").await,
                placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain")
                    .await,
                placeholder_transport: get_translation(
                    &state,
                    &locale,
                    "backups-placeholder-transport",
                )
                .await,
                tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
                tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport")
                    .await,
                tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
                cancel: get_translation(&state, &locale, "backups-cancel").await,
                create_backup: get_translation(&state, &locale, "backups-create-backup").await,
                update_backup: get_translation(&state, &locale, "backups-update-backup").await,
                new_backup: get_translation(&state, &locale, "backups-new-backup").await,
                edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title")
                    .await,
                backup: None,
                form,
                error: Some(error_message),
            };
            let content = match crate::handlers::utils::render_template_safely(content_template) {
                Ok(content) => content,
                Err(_) => return crate::handlers::utils::render_500_page(&state, &headers).await,
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

            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "backups-title").await,
                content,
                &state,
                &locale,
                current_db_label,
                current_db_id,
            )
            .await
            .unwrap();

            match crate::handlers::utils::render_template_safely(template) {
                Ok(content) => Html(content),
                Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
            }
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
        let content_template = BackupFormTemplate {
            title: get_translation(&state, &locale, "backups-edit-backup-title").await,
            form_error: get_translation(&state, &locale, "backups-form-error").await,
            form_domain: get_translation(&state, &locale, "backups-form-domain").await,
            form_transport: get_translation(&state, &locale, "backups-form-transport").await,
            form_active: get_translation(&state, &locale, "backups-form-active").await,
            placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain")
                .await,
            placeholder_transport: get_translation(
                &state,
                &locale,
                "backups-placeholder-transport",
            )
            .await,
            tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
            tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport").await,
            tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
            cancel: get_translation(&state, &locale, "backups-cancel").await,
            create_backup: get_translation(&state, &locale, "backups-create-backup").await,
            update_backup: get_translation(&state, &locale, "backups-update-backup").await,
            new_backup: get_translation(&state, &locale, "backups-new-backup").await,
            edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title").await,
            backup: None,
            form,
            error: Some(get_translation(&state, &locale, "validation-domain-required").await),
        };
        return match crate::handlers::utils::render_template_safely(content_template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
        };
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
            let content_template = BackupShowTemplate {
                title: get_translation(&state, &locale, "backups-show-title").await,
                view_edit_settings: get_translation(&state, &locale, "backups-view-edit-settings")
                    .await,
                back_to_domains: get_translation(&state, &locale, "domains-back-to-domains").await,
                backup_information: get_translation(&state, &locale, "backups-backup-information")
                    .await,
                backup_details: get_translation(&state, &locale, "backups-backup-details").await,
                domain: get_translation(&state, &locale, "backups-domain").await,
                transport: get_translation(&state, &locale, "backups-transport").await,
                status: get_translation(&state, &locale, "backups-status").await,
                created: get_translation(&state, &locale, "backups-created").await,
                modified: get_translation(&state, &locale, "backups-modified").await,
                status_active: get_translation(&state, &locale, "status-active").await,
                status_inactive: get_translation(&state, &locale, "status-inactive").await,
                edit_backup: get_translation(&state, &locale, "backups-edit-backup").await,
                enable_backup: get_translation(&state, &locale, "backups-enable-backup").await,
                disable_backup: get_translation(&state, &locale, "backups-disable-backup").await,
                delete_backup: get_translation(&state, &locale, "backups-delete-backup").await,
                delete_confirm: get_translation(&state, &locale, "backups-delete-confirm").await,
                backup,
            };
            match crate::handlers::utils::render_template_safely(content_template) {
                Ok(content) => Html(content),
                Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
            }
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

            let content_template = BackupFormTemplate {
                title: get_translation(&state, &locale, "backups-edit-backup-title").await,
                form_error: get_translation(&state, &locale, "backups-form-error").await,
                form_domain: get_translation(&state, &locale, "backups-form-domain").await,
                form_transport: get_translation(&state, &locale, "backups-form-transport").await,
                form_active: get_translation(&state, &locale, "backups-form-active").await,
                placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain")
                    .await,
                placeholder_transport: get_translation(
                    &state,
                    &locale,
                    "backups-placeholder-transport",
                )
                .await,
                tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
                tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport")
                    .await,
                tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
                cancel: get_translation(&state, &locale, "backups-cancel").await,
                create_backup: get_translation(&state, &locale, "backups-create-backup").await,
                update_backup: get_translation(&state, &locale, "backups-update-backup").await,
                new_backup: get_translation(&state, &locale, "backups-new-backup").await,
                edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title")
                    .await,
                backup: None,
                form,
                error: Some(error_message),
            };
            Html(content_template.render().unwrap())
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
                    let content_template = BackupShowTemplate {
                        title: get_translation(&state, &locale, "backups-show-title").await,
                        view_edit_settings: get_translation(
                            &state,
                            &locale,
                            "backups-view-edit-settings",
                        )
                        .await,
                        back_to_domains: get_translation(
                            &state,
                            &locale,
                            "domains-back-to-domains",
                        )
                        .await,
                        backup_information: get_translation(
                            &state,
                            &locale,
                            "backups-backup-information",
                        )
                        .await,
                        backup_details: get_translation(&state, &locale, "backups-backup-details")
                            .await,
                        domain: get_translation(&state, &locale, "backups-domain").await,
                        transport: get_translation(&state, &locale, "backups-transport").await,
                        status: get_translation(&state, &locale, "backups-status").await,
                        created: get_translation(&state, &locale, "backups-created").await,
                        modified: get_translation(&state, &locale, "backups-modified").await,
                        status_active: get_translation(&state, &locale, "status-active").await,
                        status_inactive: get_translation(&state, &locale, "status-inactive").await,
                        edit_backup: get_translation(&state, &locale, "backups-edit-backup").await,
                        enable_backup: get_translation(&state, &locale, "backups-enable-backup")
                            .await,
                        disable_backup: get_translation(&state, &locale, "backups-disable-backup")
                            .await,
                        delete_backup: get_translation(&state, &locale, "backups-delete-backup")
                            .await,
                        delete_confirm: get_translation(&state, &locale, "backups-delete-confirm")
                            .await,
                        backup,
                    };

                    // Use helper function for template rendering
                    crate::handlers::utils::render_show_template(
                        content_template,
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
            let content_template = BackupShowTemplate {
                title: get_translation(&state, &locale, "backups-show-title").await,
                view_edit_settings: get_translation(&state, &locale, "backups-view-edit-settings")
                    .await,
                back_to_domains: get_translation(&state, &locale, "domains-back-to-domains").await,
                backup_information: get_translation(&state, &locale, "backups-backup-information")
                    .await,
                backup_details: get_translation(&state, &locale, "backups-backup-details").await,
                domain: get_translation(&state, &locale, "backups-domain").await,
                transport: get_translation(&state, &locale, "backups-transport").await,
                status: get_translation(&state, &locale, "backups-status").await,
                created: get_translation(&state, &locale, "backups-created").await,
                modified: get_translation(&state, &locale, "backups-modified").await,
                status_active: get_translation(&state, &locale, "status-active").await,
                status_inactive: get_translation(&state, &locale, "status-inactive").await,
                edit_backup: get_translation(&state, &locale, "backups-edit-backup").await,
                enable_backup: get_translation(&state, &locale, "backups-enable-backup").await,
                disable_backup: get_translation(&state, &locale, "backups-disable-backup").await,
                delete_backup: get_translation(&state, &locale, "backups-delete-backup").await,
                delete_confirm: get_translation(&state, &locale, "backups-delete-confirm").await,
                backup,
            };
            match crate::handlers::utils::render_template_safely(content_template) {
                Ok(content) => Html(content),
                Err(_) => crate::handlers::utils::render_500_page(&state, &headers).await,
            }
        }
        Err(_) => return crate::handlers::utils::render_500_page(&state, &headers).await,
    }
}

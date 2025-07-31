use crate::templates::backups::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, i18n::get_translation, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = BackupForm {
        domain: "".to_string(),
        transport: "smtp:[]".to_string(),
        enabled: true,
    };

    let content_template = BackupFormTemplate {
        title: get_translation(&state, &locale, "backups-new-backup").await,
        form_error: get_translation(&state, &locale, "backups-form-error").await,
        form_domain: get_translation(&state, &locale, "backups-form-domain").await,
        form_transport: get_translation(&state, &locale, "backups-form-transport").await,
        form_active: get_translation(&state, &locale, "backups-form-active").await,
        placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
        placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport")
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
        error: None,
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        get_translation(&state, &locale, "backups-add-title").await,
    )
    .await
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = match db::get_backup(&pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup not found".to_string()),
    };

    let content_template = BackupShowTemplate {
        title: get_translation(&state, &locale, "backups-show-title").await,
        view_edit_settings: get_translation(&state, &locale, "backups-view-edit-settings").await,
        back_to_domains: get_translation(&state, &locale, "domains-back-to-domains").await,
        backup_information: get_translation(&state, &locale, "backups-backup-information").await,
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
    let content = content_template.render().unwrap();

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

    Html(template.render().unwrap())
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = match db::get_backup(&pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup not found".to_string()),
    };

    let form = BackupForm {
        domain: backup.domain.clone(),
        transport: backup.transport.clone().unwrap_or_default(),
        enabled: backup.enabled,
    };

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "backups-edit-backup-title",
            "backups-form-error",
            "backups-cancel",
            "backups-create-backup",
            "backups-update-backup",
            "backups-new-backup",
            "backups-edit-title",
        ],
    )
    .await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "backups",
        &["domain", "transport", "active"],
    )
    .await;

    let content_template = BackupFormTemplate {
        title: form_translations
            .get("backups-edit-backup-title")
            .unwrap_or(&"Edit Backup".to_string())
            .clone(),
        form_error: form_translations
            .get("backups-form-error")
            .unwrap_or(&"Form Error".to_string())
            .clone(),
        form_domain: field_translations
            .get("backups-form-domain")
            .unwrap_or(&"Domain".to_string())
            .clone(),
        form_transport: field_translations
            .get("backups-form-transport")
            .unwrap_or(&"Transport".to_string())
            .clone(),
        form_active: field_translations
            .get("backups-form-active")
            .unwrap_or(&"Active".to_string())
            .clone(),
        placeholder_domain: field_translations
            .get("backups-placeholder-domain")
            .unwrap_or(&"Enter domain".to_string())
            .clone(),
        placeholder_transport: field_translations
            .get("backups-placeholder-transport")
            .unwrap_or(&"Enter transport".to_string())
            .clone(),
        tooltip_domain: field_translations
            .get("backups-tooltip-domain")
            .unwrap_or(&"Domain tooltip".to_string())
            .clone(),
        tooltip_transport: field_translations
            .get("backups-tooltip-transport")
            .unwrap_or(&"Transport tooltip".to_string())
            .clone(),
        tooltip_active: field_translations
            .get("backups-tooltip-active")
            .unwrap_or(&"Active tooltip".to_string())
            .clone(),
        cancel: form_translations
            .get("backups-cancel")
            .unwrap_or(&"Cancel".to_string())
            .clone(),
        create_backup: form_translations
            .get("backups-create-backup")
            .unwrap_or(&"Create Backup".to_string())
            .clone(),
        update_backup: form_translations
            .get("backups-update-backup")
            .unwrap_or(&"Update Backup".to_string())
            .clone(),
        new_backup: form_translations
            .get("backups-new-backup")
            .unwrap_or(&"New Backup".to_string())
            .clone(),
        edit_backup_title: form_translations
            .get("backups-edit-backup-title")
            .unwrap_or(&"Edit Backup".to_string())
            .clone(),
        backup: Some(backup),
        form,
        error: None,
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations
            .get("backups-edit-title")
            .unwrap_or(&"Edit Backup".to_string())
            .clone(),
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
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
        return Html(content_template.render().unwrap());
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
            return Html(content_template.render().unwrap());
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
            let content = content_template.render().unwrap();

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

            Html(template.render().unwrap())
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
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
        return Html(content_template.render().unwrap());
    }

    match db::update_backup(&pool, id, form.clone()) {
        Ok(_) => {
            let backup = match db::get_backup(&pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup not found".to_string()),
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
            Html(content_template.render().unwrap())
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::delete_backup(&pool, id) {
        Ok(_) => {
            // Redirect to domains page after deleting backup
            Html("<script>window.location.href='/domains';</script>".to_string())
        }
        Err(_) => Html("Error deleting backup".to_string()),
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_backup_enabled(&pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(&pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup not found".to_string()),
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
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    match db::toggle_backup_enabled(&pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(&pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup not found".to_string()),
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
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
}

use crate::templates::backups::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
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
        placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
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
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = match db::get_backup(pool, id) {
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

    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "backups-title").await,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let backup = match db::get_backup(pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup not found".to_string()),
    };

    let form = BackupForm {
        domain: backup.domain.clone(),
        transport: backup.transport.clone().unwrap_or_default(),
        enabled: backup.enabled,
    };

    let content_template = BackupFormTemplate {
        title: get_translation(&state, &locale, "backups-edit-backup-title").await,
        form_error: get_translation(&state, &locale, "backups-form-error").await,
        form_domain: get_translation(&state, &locale, "backups-form-domain").await,
        form_transport: get_translation(&state, &locale, "backups-form-transport").await,
        form_active: get_translation(&state, &locale, "backups-form-active").await,
        placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
        placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
        tooltip_domain: get_translation(&state, &locale, "backups-tooltip-domain").await,
        tooltip_transport: get_translation(&state, &locale, "backups-tooltip-transport").await,
        tooltip_active: get_translation(&state, &locale, "backups-tooltip-active").await,
        cancel: get_translation(&state, &locale, "backups-cancel").await,
        create_backup: get_translation(&state, &locale, "backups-create-backup").await,
        update_backup: get_translation(&state, &locale, "backups-update-backup").await,
        new_backup: get_translation(&state, &locale, "backups-new-backup").await,
        edit_backup_title: get_translation(&state, &locale, "backups-edit-backup-title").await,
        backup: Some(backup),
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, headers: HeaderMap, Form(form): Form<BackupForm>) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = BackupFormTemplate {
            title: get_translation(&state, &locale, "backups-new-backup").await,
            form_error: get_translation(&state, &locale, "backups-form-error").await,
            form_domain: get_translation(&state, &locale, "backups-form-domain").await,
            form_transport: get_translation(&state, &locale, "backups-form-transport").await,
            form_active: get_translation(&state, &locale, "backups-form-active").await,
            placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
            placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
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

    let new_backup = NewBackup {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
        enabled: form.enabled,
    };

    match db::create_backup(pool, new_backup) {
        Ok(_) => {
            // Redirect to domains page after creating backup
            return Html("<script>window.location.href='/domains';</script>".to_string());
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-backup").await.replace("{domain}", &form.domain),
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
                placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
                placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
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
                error: Some(error_message),
            };
            let content = content_template.render().unwrap();

            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "backups-title").await,
                content,
                &state,
                &locale,
            ).await.unwrap();
            
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
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = BackupFormTemplate {
            title: get_translation(&state, &locale, "backups-edit-backup-title").await,
            form_error: get_translation(&state, &locale, "backups-form-error").await,
            form_domain: get_translation(&state, &locale, "backups-form-domain").await,
            form_transport: get_translation(&state, &locale, "backups-form-transport").await,
            form_active: get_translation(&state, &locale, "backups-form-active").await,
            placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
            placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
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

    match db::update_backup(pool, id, form.clone()) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
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
            Html(content_template.render().unwrap())
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-backup").await.replace("{domain}", &form.domain),
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
                placeholder_domain: get_translation(&state, &locale, "backups-placeholder-domain").await,
                placeholder_transport: get_translation(&state, &locale, "backups-placeholder-transport").await,
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
                error: Some(error_message),
            };
            return Html(content_template.render().unwrap());
        }
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::delete_backup(pool, id) {
        Ok(_) => {
            // Redirect to domains page after deleting backup
            return Html("<script>window.location.href='/domains';</script>".to_string());
        }
        Err(_) => Html("Error deleting backup".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
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
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);
    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
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
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
} 

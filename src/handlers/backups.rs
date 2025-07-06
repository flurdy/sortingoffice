use crate::templates::backups::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;

    tracing::debug!("Handling backups list request");
    let backups = match db::get_backups(pool) {
        Ok(backups) => {
            tracing::info!("Successfully retrieved {} backups", backups.len());
            backups
        },
        Err(e) => {
            tracing::error!("Failed to retrieve backups: {:?}", e);
            vec![]
        },
    };

    tracing::debug!("Rendering template with {} backups", backups.len());
    let content_template = BackupListTemplate {
        title: "Backups",
        backups,
    };
    let content = match content_template.render() {
        Ok(content) => {
            tracing::debug!("Template rendered successfully, content length: {}", content.len());
            content
        },
        Err(e) => {
            tracing::error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    let template = BaseTemplate {
        title: "Backups".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn new() -> Html<String> {
    let form = BackupForm {
        domain: "".to_string(),
        transport: "smtp:[]".to_string(),
        enabled: true,
    };

    let content_template = BackupFormTemplate {
        title: "New Backup",
        backup: None,
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let backup = match db::get_backup(pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup not found".to_string()),
    };

    let content_template = BackupShowTemplate {
        title: "Show Backup",
        backup,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "Show Backup".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

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
        title: "Edit Backup",
        backup: Some(backup),
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, Form(form): Form<BackupForm>) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = BackupFormTemplate {
            title: "New Backup",
            backup: None,
            form,
            error: Some("Domain name is required. Please enter a valid domain name.".to_string()),
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
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(_) => vec![],
            };
            let template = BackupListTemplate {
                title: "Backups",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("A backup server for domain '{}' already exists.", form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The backup data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while creating the backup. Please try again.".to_string(),
            };

            let content_template = BackupFormTemplate {
                title: "New Backup",
                backup: None,
                form,
                error: Some(error_message),
            };
            Html(content_template.render().unwrap())
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = BackupFormTemplate {
            title: "Edit Backup",
            backup: None,
            form,
            error: Some("Domain name is required. Please enter a valid domain name.".to_string()),
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
                title: "Show Backup",
                backup,
            };
            Html(content_template.render().unwrap())
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("A backup server for domain '{}' already exists.", form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The backup data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the backup. Please try again.".to_string(),
            };

            let content_template = BackupFormTemplate {
                title: "Edit Backup",
                backup: None,
                form,
                error: Some(error_message),
            };
            Html(content_template.render().unwrap())
        }
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::delete_backup(pool, id) {
        Ok(_) => {
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(_) => vec![],
            };
            let template = BackupListTemplate {
                title: "Backups",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting backup".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup not found".to_string()),
            };

            let content_template = BackupShowTemplate {
                title: "Show Backup",
                backup,
            };
            let content = content_template.render().unwrap();

            let template = BaseTemplate {
                title: "Show Backup".to_string(),
                content,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(_) => vec![],
            };
            let template = BackupListTemplate {
                title: "Backups",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup not found".to_string()),
            };
            let content_template = BackupShowTemplate {
                title: "Show Backup",
                backup,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup status".to_string()),
    }
} 

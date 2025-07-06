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
        title: "Backup MX",
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
        title: "Backup MX".to_string(),
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
        title: "New Backup MX",
        backup: None,
        form,
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let backup = match db::get_backup(pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup MX not found".to_string()),
    };

    let content_template = BackupShowTemplate {
        title: "Show Backup MX",
        backup,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "Show Backup MX".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let backup = match db::get_backup(pool, id) {
        Ok(backup) => backup,
        Err(_) => return Html("Backup MX not found".to_string()),
    };

    let form = BackupForm {
        domain: backup.domain.clone(),
        transport: backup.transport.clone().unwrap_or_default(),
        enabled: backup.enabled,
    };

    let content_template = BackupFormTemplate {
        title: "Edit Backup MX",
        backup: Some(backup),
        form,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, Form(form): Form<BackupForm>) -> Html<String> {
    let pool = &state.pool;

    let new_backup = NewBackup {
        domain: form.domain,
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
                title: "Backup MX",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error creating backup MX".to_string()),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<BackupForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::update_backup(pool, id, form) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup MX not found".to_string()),
            };
            let content_template = BackupShowTemplate {
                title: "Show Backup MX",
                backup,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error updating backup MX".to_string()),
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
                title: "Backup MX",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting backup MX".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_backup_enabled(pool, id) {
        Ok(_) => {
            let backup = match db::get_backup(pool, id) {
                Ok(backup) => backup,
                Err(_) => return Html("Backup MX not found".to_string()),
            };

            let content_template = BackupShowTemplate {
                title: "Show Backup MX",
                backup,
            };
            let content = content_template.render().unwrap();

            let template = BaseTemplate {
                title: "Show Backup MX".to_string(),
                content,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup MX status".to_string()),
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
                title: "Backup MX",
                backups,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup MX status".to_string()),
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
                Err(_) => return Html("Backup MX not found".to_string()),
            };
            let content_template = BackupShowTemplate {
                title: "Show Backup MX",
                backup,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling backup MX status".to_string()),
    }
} 

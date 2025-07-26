use crate::AppState;
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
    Json,
};
use serde::Deserialize;
use std::fs;
use std::path::Path as FsPath;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs as tokio_fs;
use url::Url;

#[derive(Deserialize)]
pub struct BackupForm {
    database_id: String,
}

/// Show the backup page
pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "database-backup-title",
            "database-backup-description",
            "database-backup-database-select",
            "database-backup-create-button",
            "database-backup-download-button",
            "database-backup-status-success",
            "database-backup-status-error",
            "database-backup-no-databases",
            "database-backup-create-new-section",
            "database-backup-existing-section",
            "database-backup-select-placeholder",
            "database-backup-loading",
            "database-backup-please-select",
            "database-backup-creating",
            "database-backup-session-expired",
            "database-backup-unexpected-response",
            "database-backup-failed-create",
            "database-backup-session-expired-load",
            "database-backup-failed-load",
            "database-backup-no-backups",
            "database-backup-failed-load-error",
            "database-backup-confirm-delete",
            "database-backup-session-expired-delete",
            "database-backup-failed-delete",
            "database-backup-failed-delete-error",
        ],
    )
    .await;

    // Get database configs as owned data
    let databases = state.db_manager.get_configs().to_vec();

    let content_template = crate::templates::backup::BackupTemplate {
        title: &translations["database-backup-title"],
        description: &translations["database-backup-description"],
        database_select: &translations["database-backup-database-select"],
        create_button: &translations["database-backup-create-button"],
        download_button: &translations["database-backup-download-button"],
        status_success: &translations["database-backup-status-success"],
        status_error: &translations["database-backup-status-error"],
        no_databases: &translations["database-backup-no-databases"],
        create_new_section: &translations["database-backup-create-new-section"],
        existing_section: &translations["database-backup-existing-section"],
        select_placeholder: &translations["database-backup-select-placeholder"],
        loading: &translations["database-backup-loading"],
        please_select: &translations["database-backup-please-select"],
        creating: &translations["database-backup-creating"],
        session_expired: &translations["database-backup-session-expired"],
        unexpected_response: &translations["database-backup-unexpected-response"],
        failed_create: &translations["database-backup-failed-create"],
        session_expired_load: &translations["database-backup-session-expired-load"],
        failed_load: &translations["database-backup-failed-load"],
        no_backups: &translations["database-backup-no-backups"],
        failed_load_error: &translations["database-backup-failed-load-error"],
        confirm_delete: &translations["database-backup-confirm-delete"],
        session_expired_delete: &translations["database-backup-session-expired-delete"],
        failed_delete: &translations["database-backup-failed-delete"],
        failed_delete_error: &translations["database-backup-failed-delete-error"],
        databases,
    };

    // Use the render_template_with_title macro
    crate::render_template_with_title!(
        content_template,
        &translations["database-backup-title"],
        &state,
        &locale,
        &headers
    )
}

/// Create a backup of a specific database
pub async fn create_backup(
    State(state): State<AppState>,
    Form(form): axum::Form<BackupForm>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let database_id = form.database_id;

    // Validate database exists
    if !state.db_manager.has_database(&database_id).await {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get database config
    let db_config = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == database_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Parse database URL to extract connection details
    let url = Url::parse(&db_config.url).map_err(|_| StatusCode::BAD_REQUEST)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(3306);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    // Create temporary file for backup
    let temp_file = NamedTempFile::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let backup_path = temp_file.path().to_path_buf();

    // Run mysqldump command
    let output = Command::new("mysqldump")
        .arg("--single-transaction")
        .arg("--routines")
        .arg("--triggers")
        .arg("--add-drop-database")
        .arg("--create-options")
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg(format!("--password={}", password))
        .arg(database)
        .arg("--result-file")
        .arg(&backup_path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Generate filename with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("{}_{}_{}.sql", database_id, database, timestamp);

                // Create backup directory if it doesn't exist
                let backup_dir = FsPath::new("backups");
                if !backup_dir.exists() {
                    fs::create_dir(backup_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                }

                // Move temp file to backup directory
                let final_path = backup_dir.join(&filename);
                fs::rename(&backup_path, &final_path)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                tracing::info!("Backup created successfully: {}", final_path.display());

                // Return success response with file info
                let response = serde_json::json!({
                    "success": true,
                    "message": "Backup created successfully",
                    "filename": filename,
                    "path": final_path.to_string_lossy()
                });

                Ok(Json(response))
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                tracing::error!("Backup failed: {}", error_msg);

                let response = serde_json::json!({
                    "success": false,
                    "message": format!("Backup failed: {}", error_msg)
                });

                Ok(Json(response))
            }
        }
        Err(e) => {
            tracing::error!("Failed to execute mysqldump: {}", e);

            let response = serde_json::json!({
                "success": false,
                "message": format!("Failed to execute backup: {}", e)
            });

            Ok(Json(response))
        }
    }
}

/// Download a backup file
pub async fn download_backup(
    Path(filename): Path<String>,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let backup_path = FsPath::new("backups").join(&filename);

    // Security check: ensure the file is within the backups directory
    if !backup_path.starts_with("backups") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !backup_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file content
    let content = tokio_fs::read(&backup_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return file as download
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/sql")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .header("Content-Length", content.len().to_string())
        .body(axum::body::Body::from(content))
        .unwrap())
}

/// List available backup files
pub async fn list_backups() -> Result<axum::Json<Vec<String>>, StatusCode> {
    let backup_dir = FsPath::new("backups");

    if !backup_dir.exists() {
        return Ok(axum::Json(Vec::new()));
    }

    let mut backups = Vec::new();

    match fs::read_dir(backup_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".sql") {
                            backups.push(filename.to_string());
                        }
                    }
                }
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Sort backups by name (which includes timestamp)
    backups.sort();
    backups.reverse(); // Most recent first

    Ok(axum::Json(backups))
}

/// Delete a backup file
pub async fn delete_backup(
    Path(filename): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let backup_path = FsPath::new("backups").join(&filename);

    // Security check: ensure the file is within the backups directory
    if !backup_path.starts_with("backups") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !backup_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete the file
    match fs::remove_file(&backup_path) {
        Ok(_) => {
            tracing::info!("Backup deleted: {}", filename);

            let response = serde_json::json!({
                "success": true,
                "message": "Backup deleted successfully"
            });

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to delete backup {}: {}", filename, e);

            let response = serde_json::json!({
                "success": false,
                "message": format!("Failed to delete backup: {}", e)
            });

            Ok(Json(response))
        }
    }
}

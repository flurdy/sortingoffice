use crate::AppState;
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path as FsPath;
use std::process::Command;

use chrono::{DateTime, TimeZone, Utc};
use tempfile::NamedTempFile;
use tokio::fs as tokio_fs;
use url::Url;

#[derive(Deserialize)]
pub struct BackupForm {
    database_id: String,
}

#[derive(Serialize)]
pub struct BackupInfo {
    filename: String,
    database_id: String,
    database_name: String,
    created_at: DateTime<Utc>,
    size_bytes: u64,
    size_formatted: String,
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
            "database-backup-table-header-database",
            "database-backup-table-header-created",
            "database-backup-table-header-size",
            "database-backup-table-header-filename",
            "database-backup-table-header-actions",
            "database-backup-action-download",
            "database-backup-action-delete",
        ],
    )
    .await;

    // Get database configs as owned data
    let databases = state.db_manager.get_configs().to_vec();

    let content_template = crate::templates::database_backup::BackupTemplate {
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
        table_header_database: &translations["database-backup-table-header-database"],
        table_header_created: &translations["database-backup-table-header-created"],
        table_header_size: &translations["database-backup-table-header-size"],
        table_header_filename: &translations["database-backup-table-header-filename"],
        table_header_actions: &translations["database-backup-table-header-actions"],
        action_download: &translations["database-backup-action-download"],
        action_delete: &translations["database-backup-action-delete"],
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

    // Run mysqldump with minimal options that work with basic SELECT privileges
    let output = Command::new("mysqldump")
        .arg("--no-tablespaces")
        .arg("--skip-lock-tables")
        .arg("--skip-add-drop-table")
        .arg("--skip-add-locks")
        .arg("--skip-comments")
        .arg("--skip-set-charset")
        .arg("--skip-routines")
        .arg("--skip-triggers")
        .arg("--no-create-db")
        .arg("--no-create-info")
        .arg("--complete-insert")
        .arg(format!("--host={host}"))
        .arg(format!("--port={port}"))
        .arg(format!("--user={username}"))
        .arg(format!("--password={password}"))
        .arg(database)
        .arg("--result-file")
        .arg(&backup_path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Generate filename with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("{database_id}_{database}_{timestamp}.sql");

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

/// Create a backup of a specific database (HTMX version - returns HTML)
pub async fn create_backup_htmx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): axum::Form<BackupForm>,
) -> Result<Html<String>, StatusCode> {
    let database_id = form.database_id;

    tracing::info!("Creating backup for database: {}", database_id);

    // Validate database exists
    if !state.db_manager.has_database(&database_id).await {
        tracing::error!("Database not found: {}", database_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get database config
    let db_config = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == database_id)
        .ok_or_else(|| {
            tracing::error!("Database config not found for: {}", database_id);
            StatusCode::BAD_REQUEST
        })?;

    // Parse database URL to extract connection details
    let url = Url::parse(&db_config.url).map_err(|e| {
        tracing::error!("Failed to parse database URL: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(3306);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    tracing::info!(
        "Backup parameters: host={}, port={}, user={}, database={}",
        host,
        port,
        username,
        database
    );

    // Generate filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{database_id}_{database}_{timestamp}.sql");

    // Create backup directory if it doesn't exist
    let backup_dir = FsPath::new("backups");
    if !backup_dir.exists() {
        std::fs::create_dir_all(backup_dir).map_err(|e| {
            tracing::error!("Failed to create backup directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // Create backup file directly in the backup directory
    let final_path = backup_dir.join(&filename);

    // Run mysqldump with minimal options that work with basic SELECT privileges
    let output = Command::new("/home/linuxbrew/.linuxbrew/opt/mysql-client/bin/mysqldump")
        .arg("--no-tablespaces")
        .arg("--skip-lock-tables")
        .arg("--skip-add-drop-table")
        .arg("--skip-add-locks")
        .arg("--skip-comments")
        .arg("--skip-set-charset")
        .arg("--skip-routines")
        .arg("--skip-triggers")
        .arg("--no-create-db")
        .arg("--no-create-info")
        .arg("--complete-insert")
        .arg(format!("--host={host}"))
        .arg(format!("--port={port}"))
        .arg(format!("--user={username}"))
        .arg(format!("--password={password}"))
        .arg(database)
        .arg("--result-file")
        .arg(&final_path)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(e) => {
            tracing::error!("Failed to execute mysqldump: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if output.status.success() {
        tracing::info!("Backup created successfully: {}", final_path.display());

        // Get locale for translations
        let locale = crate::handlers::language::get_user_locale(&headers);
        let translations = crate::handlers::utils::get_translations_batch(
            &state,
            &locale,
            &[
                "database-backup-status-success",
                "database-backup-download-button",
            ],
        )
        .await;

        // Return HTML fragment for HTMX
        let html = format!(
            r#"
            <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md">
                <p class="text-green-800 dark:text-green-200">
                    {}
                </p>
                <div class="mt-2">
                    <a href="/database_backup/download/{}" 
                       class="inline-flex items-center px-3 py-1 bg-green-600 hover:bg-green-700 text-white text-sm rounded-md transition duration-200">
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                        {}
                    </a>
                </div>
            </div>
            <script>
                // Refresh the backups list after successful creation
                loadBackups();
            </script>
            "#,
            translations["database-backup-status-success"],
            filename,
            translations["database-backup-download-button"]
        );

        Ok(Html(html))
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        tracing::error!("mysqldump failed: {}", error_message);

        // Get locale for translations
        let locale = crate::handlers::language::get_user_locale(&headers);
        let translations = crate::handlers::utils::get_translations_batch(
            &state,
            &locale,
            &["database-backup-status-error"],
        )
        .await;

        let html = format!(
            r#"
            <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
                <p class="text-red-800 dark:text-red-200">
                    {}: {}
                </p>
            </div>
            "#,
            translations["database-backup-status-error"], error_message
        );

        Ok(Html(html))
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
            format!("attachment; filename=\"{filename}\""),
        )
        .header("Content-Length", content.len().to_string())
        .body(axum::body::Body::from(content))
        .unwrap())
}

/// List available backup files with detailed information
pub async fn list_backups() -> Result<axum::Json<Vec<BackupInfo>>, StatusCode> {
    let backup_dir = FsPath::new("backups");

    if !backup_dir.exists() {
        return Ok(axum::Json(Vec::new()));
    }

    let mut backups = Vec::new();

    match fs::read_dir(backup_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".sql") {
                        let file_path = entry.path();

                        // Get file metadata
                        let metadata = match fs::metadata(&file_path) {
                            Ok(meta) => meta,
                            Err(_) => continue,
                        };

                        // Parse filename to extract database info and timestamp
                        let (database_id, database_name, created_at) =
                            parse_backup_filename(filename);

                        // Format file size
                        let size_bytes = metadata.len();
                        let size_formatted = format_file_size(size_bytes);

                        let backup_info = BackupInfo {
                            filename: filename.to_string(),
                            database_id,
                            database_name,
                            created_at,
                            size_bytes,
                            size_formatted,
                        };

                        backups.push(backup_info);
                    }
                }
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Sort backups by creation date (most recent first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(axum::Json(backups))
}

/// Parse backup filename to extract database info and timestamp
fn parse_backup_filename(filename: &str) -> (String, String, DateTime<Utc>) {
    // Expected format: database_id_database_name_YYYYMMDD_HHMMSS.sql
    // Example: primary_sortingoffice_20250726_165857.sql
    // Example: backup1_sortingoffice_backup_20250726_165857.sql

    // Remove .sql extension
    let name_without_ext = filename.replace(".sql", "");
    let parts: Vec<&str> = name_without_ext.split('_').collect();

    tracing::debug!("Parsing filename: '{}', parts: {:?}", filename, parts);

    // We need at least 4 parts: database_id, database_name (may contain underscores), date, time
    if parts.len() >= 4 {
        let database_id = parts[0].to_string();

        // The database name may contain underscores, so we need to reconstruct it
        // Everything between database_id and the last two parts (date and time) is the database name
        let database_name_parts = &parts[1..parts.len() - 2];
        let database_name = database_name_parts.join("_");

        // Extract timestamp from last two parts (YYYYMMDD_HHMMSS)
        let date_part = parts[parts.len() - 2];
        let time_part = parts[parts.len() - 1];

        tracing::debug!(
            "Extracted: db_id='{}', db_name='{}', date='{}', time='{}'",
            database_id,
            database_name,
            date_part,
            time_part
        );

        // Parse date and time separately
        if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute), Ok(second)) = (
            date_part[0..4].parse::<u32>(),
            date_part[4..6].parse::<u32>(),
            date_part[6..8].parse::<u32>(),
            time_part[0..2].parse::<u32>(),
            time_part[2..4].parse::<u32>(),
            time_part[4..6].parse::<u32>(),
        ) {
            if let chrono::LocalResult::Single(created_at) =
                Utc.with_ymd_and_hms(year as i32, month, day, hour, minute, second)
            {
                tracing::debug!("Successfully parsed timestamp: {}", created_at);
                return (database_id, database_name, created_at);
            }
        }
        tracing::debug!(
            "Failed to parse timestamp from date='{}' time='{}'",
            date_part,
            time_part
        );
    } else {
        tracing::debug!("Expected at least 4 parts, got {}", parts.len());
    }

    // Fallback: use current time if parsing fails
    tracing::debug!("Using fallback values for filename: {}", filename);
    (filename.to_string(), "Unknown".to_string(), Utc::now())
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        0..KB => format!("{bytes} B"),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        _ => format!("{:.1} GB", bytes as f64 / GB as f64),
    }
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

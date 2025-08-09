use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use serde_json::json;

/// Health check endpoint that returns basic application status
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // Check if we have any databases configured
    let has_databases = !state.db_manager.get_configs().is_empty();

    let health_status = json!({
        "status": if has_databases { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "databases_configured": has_databases,
    });

    let status_code = if has_databases {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health_status))
}

/// Detailed health check with database connection pool statistics
pub async fn detailed_health_check(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Check if we have any databases configured
    let has_databases = !state.db_manager.get_configs().is_empty();

    if !has_databases {
        let error_status = json!({
            "status": "unhealthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
            "error": "No databases configured",
            "databases": {}
        });
        return (StatusCode::SERVICE_UNAVAILABLE, Json(error_status));
    }

    // Get connection pool statistics
    let pool_stats = state.db_manager.get_all_pool_stats().await;
    let health_status = state.db_manager.health_check_all().await;

    let mut detailed_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "databases": {}
    });

    // Add database-specific information
    if let Some(databases) = detailed_status.get_mut("databases") {
        for config in state.db_manager.get_configs() {
            let pool_stat = pool_stats.get(&config.id);
            let is_healthy = health_status.get(&config.id).unwrap_or(&false);

            databases[&config.id] = json!({
                "label": config.label,
                "healthy": is_healthy,
                "pool_stats": {
                    "max_size": pool_stat.map(|s| s.max_size).unwrap_or(0),
                    "current_size": pool_stat.map(|s| s.size).unwrap_or(0),
                    "available": pool_stat.map(|s| s.available).unwrap_or(0),
                    "in_use": pool_stat.map(|s| s.in_use).unwrap_or(0),
                    "utilization_percentage": pool_stat.map(|s| s.utilization_percentage()).unwrap_or(0.0),
                    "under_high_load": pool_stat.map(|s| s.is_under_high_load()).unwrap_or(false),
                },
                "features": {
                    "read_only": config.features.read_only,
                    "disabled": config.features.disabled,
                }
            });
        }
    }

    // Determine overall health status
    let all_healthy = health_status.values().all(|&healthy| healthy);
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    if !all_healthy {
        detailed_status["status"] = json!("unhealthy");
    }

    (status_code, Json(detailed_status))
}

/// Connection pool statistics endpoint
pub async fn pool_stats(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let pool_stats = state.db_manager.get_all_pool_stats().await;

    let mut stats = json!({});

    for (db_id, stat) in pool_stats {
        stats[&db_id] = json!({
            "max_size": stat.max_size,
            "current_size": stat.size,
            "available": stat.available,
            "in_use": stat.in_use,
            "utilization_percentage": stat.utilization_percentage(),
            "under_high_load": stat.is_under_high_load(),
            "has_available_connections": stat.has_available_connections(),
        });
    }

    (StatusCode::OK, Json(stats))
}

/// HTML health check page for browser viewing
pub async fn health_check_html(State(state): State<AppState>) -> Html<String> {
    // Check if we have any databases configured
    let has_databases = !state.db_manager.get_configs().is_empty();

    let pool_stats = if has_databases {
        state.db_manager.get_all_pool_stats().await
    } else {
        std::collections::HashMap::new()
    };

    let health_status = if has_databases {
        state.db_manager.health_check_all().await
    } else {
        std::collections::HashMap::new()
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>SortingOffice Health Check</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; margin-top: 20px; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #f2f2f2; }\n");
    html.push_str(".healthy { color: green; }\n");
    html.push_str(".unhealthy { color: red; }\n");
    html.push_str(".warning { color: orange; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    html.push_str("<h1>SortingOffice Health Check</h1>\n");
    html.push_str(&format!(
        "<p><strong>Timestamp:</strong> {}</p>\n",
        chrono::Utc::now().to_rfc3339()
    ));
    html.push_str(&format!(
        "<p><strong>Version:</strong> {}</p>\n",
        env!("CARGO_PKG_VERSION")
    ));

    if !has_databases {
        html.push_str(
            "<p><strong>Overall Status:</strong> <span class=\"unhealthy\">Unhealthy</span></p>\n",
        );
        html.push_str("<p><strong>Error:</strong> No databases configured</p>\n");
        html.push_str("</body>\n</html>\n");
        return Html(html);
    }

    let all_healthy = health_status.values().all(|&healthy| healthy);
    let status_class = if all_healthy { "healthy" } else { "unhealthy" };
    html.push_str(&format!(
        "<p><strong>Overall Status:</strong> <span class=\"{}\">{}</span></p>\n",
        status_class,
        if all_healthy { "Healthy" } else { "Unhealthy" }
    ));

    html.push_str("<h2>Database Connection Pools</h2>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>Database</th><th>Status</th><th>Max Size</th><th>Current Size</th><th>Available</th><th>In Use</th><th>Utilization</th><th>Load</th></tr>\n");

    for config in state.db_manager.get_configs() {
        let pool_stat = pool_stats.get(&config.id);
        let is_healthy = health_status.get(&config.id).unwrap_or(&false);

        let status_class = if *is_healthy { "healthy" } else { "unhealthy" };
        let status_text = if *is_healthy { "Healthy" } else { "Unhealthy" };

        let utilization = pool_stat.map(|s| s.utilization_percentage()).unwrap_or(0.0);
        let load_class = if utilization > 80.0 {
            "warning"
        } else {
            "healthy"
        };
        let load_text = if utilization > 80.0 { "High" } else { "Normal" };

        html.push_str(&format!("<tr>\n"));
        html.push_str(&format!("<td>{}</td>\n", config.label));
        html.push_str(&format!(
            "<td class=\"{}\">{}</td>\n",
            status_class, status_text
        ));
        html.push_str(&format!(
            "<td>{}</td>\n",
            pool_stat.map(|s| s.max_size).unwrap_or(0)
        ));
        html.push_str(&format!(
            "<td>{}</td>\n",
            pool_stat.map(|s| s.size).unwrap_or(0)
        ));
        html.push_str(&format!(
            "<td>{}</td>\n",
            pool_stat.map(|s| s.available).unwrap_or(0)
        ));
        html.push_str(&format!(
            "<td>{}</td>\n",
            pool_stat.map(|s| s.in_use).unwrap_or(0)
        ));
        html.push_str(&format!("<td>{:.1}%</td>\n", utilization));
        html.push_str(&format!(
            "<td class=\"{}\">{}</td>\n",
            load_class, load_text
        ));
        html.push_str("</tr>\n");
    }

    html.push_str("</table>\n");
    html.push_str("</body>\n</html>\n");

    Html(html)
}

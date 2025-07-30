pub mod about;
pub mod aliases;
pub mod auth;
pub mod backup;
pub mod backups;
pub mod clients;
pub mod config;
pub mod contact;
pub mod dashboard;
pub mod database;
pub mod domains;
pub mod health; // Add health module
pub mod language;
pub mod not_found;
pub mod relays;
pub mod relocated;
pub mod reports;
pub mod stats;
pub mod theme;
pub mod users;
pub mod utils;
pub mod wizard;

// Re-export specific functions and types
pub use about::index as about_index;
pub use contact::index as contact_index;
pub use aliases::{
    create, delete, domain_search, edit, list, new, search, show, toggle_enabled,
    toggle_enabled_domain_show, toggle_enabled_list, toggle_enabled_show, update,
};
pub use auth::{login, login_form, logout, require_auth, require_edit_permissions};
pub use backup::{
    create_backup, create_backup_htmx, delete_backup, download_backup, index as backup_index,
    list_backups,
};
pub use clients::{
    create_client, create_client_form, delete_client, edit_client_form, list_clients, show_client,
    toggle_client, update_client,
};
pub use config::view_config;
pub use dashboard::index as dashboard_index;
pub use database::{dropdown, index as database_index, list_databases, select};
pub use domains::{
    add_missing_required_alias, create as create_domain, delete as delete_domain,
    edit as edit_domain, list as list_domains, new as new_domain, show as show_domain,
    toggle_enabled as toggle_domain_enabled, toggle_enabled_list as toggle_domain_enabled_list,
    toggle_enabled_show as toggle_domain_enabled_show, update as update_domain,
};
pub use health::*;
pub use language::{get_user_locale, set_language};
pub use not_found::not_found;
pub use relays::{
    create_form as create_relay_form, create_relay, delete_relay, edit_form as edit_relay_form,
    list_relays, show_relay, toggle_enabled as toggle_relay_enabled, update_relay,
};
pub use relocated::{
    create_form as create_relocated_form, create_relocated, delete_relocated,
    edit_form as edit_relocated_form, list_relocated, show_relocated,
    toggle_enabled as toggle_relocated_enabled, update_relocated,
};
pub use reports::{
    alias_cross_domain_report, cross_database_domain_matrix_report,
    cross_database_feature_toggle_report, cross_database_migration_report,
    cross_database_user_distribution_report, external_forwarders_report, matrix_report,
    orphaned_report, reports_list,
};
pub use stats::index as stats_index;
pub use theme::toggle_theme;
pub use users::{
    change_password_form, change_password_post, create as create_user, delete as delete_user,
    edit as edit_user, list as list_users, new as new_user, show as show_user,
    toggle_change_password, toggle_enabled as toggle_user_enabled,
    toggle_enabled_list as toggle_user_enabled_list,
    toggle_enabled_show as toggle_user_enabled_show, update as update_user,
};
pub use utils::get_user_locale as get_user_locale_util; // Export health handlers
pub use wizard::{
    alias_config, alias_config_post, complete, destination_search, domain_config,
    domain_config_post, execute, index as wizard_index, review,
};

use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

use crate::AppState;

pub fn create_app(app_state: AppState) -> Router<AppState> {
    // Create read-only routes (require authentication but not edit permissions)
    let read_only_routes = Router::new()
        .route("/", axum::routing::get(dashboard_index))
        .route("/about", axum::routing::get(about_index))
        .route("/contact", axum::routing::get(contact_index))
        // Read-only domain operations
        .route("/domains", axum::routing::get(list_domains))
        .route("/domains/{id}", axum::routing::get(show_domain))
        // Read-only user operations
        .route("/users", axum::routing::get(list_users))
        .route("/users/{id}", axum::routing::get(show_user))
        // Read-only alias operations
        .route("/aliases", axum::routing::get(list))
        .route("/aliases/{id}", axum::routing::get(show))
        .route("/aliases/search", axum::routing::get(search))
        .route("/aliases/domain-search", axum::routing::get(domain_search))
        // Read-only backup operations
        .route("/backups/{id}", axum::routing::get(backups::show))
        // Read-only relay operations
        .route("/relays", axum::routing::get(list_relays))
        .route("/relays/{id}", axum::routing::get(show_relay))
        // Read-only relocated operations
        .route("/relocated", axum::routing::get(list_relocated))
        .route("/relocated/{id}", axum::routing::get(show_relocated))
        // Read-only client operations
        .route("/clients", axum::routing::get(list_clients))
        .route("/clients/{id}", axum::routing::get(show_client))
        // Stats and reports
        .route("/stats", axum::routing::get(stats_index))
        .route("/reports", axum::routing::get(reports_list))
        .route("/reports/matrix", axum::routing::get(matrix_report))
        .route(
            "/reports/cross-database-matrix",
            axum::routing::get(cross_database_domain_matrix_report),
        )
        .route("/reports/orphaned", axum::routing::get(orphaned_report))
        .route(
            "/reports/external-forwarders",
            axum::routing::get(external_forwarders_report),
        )
        .route(
            "/reports/alias-cross-domain",
            axum::routing::get(alias_cross_domain_report),
        )
        .route(
            "/reports/cross-database-user-distribution",
            axum::routing::get(cross_database_user_distribution_report),
        )
        .route(
            "/reports/cross-database-feature-toggle",
            axum::routing::get(cross_database_feature_toggle_report),
        )
        .route(
            "/reports/cross-database-migration",
            axum::routing::get(cross_database_migration_report),
        )
        // Configuration
        .route("/config", axum::routing::get(view_config))
        // Database selection
        .route("/database", axum::routing::get(database_index))
        .route("/database/select", axum::routing::post(select))
        .route("/database/dropdown", axum::routing::get(dropdown))
        .route("/api/databases", axum::routing::get(list_databases))
        // Backup management
        .route("/backup", axum::routing::get(backup_index))
        .route("/backup/create", axum::routing::post(create_backup))
        .route(
            "/backup/create-htmx",
            axum::routing::post(create_backup_htmx),
        )
        .route(
            "/backup/download/{filename}",
            axum::routing::get(download_backup),
        )
        .route(
            "/backup/delete/{filename}",
            axum::routing::delete(delete_backup),
        )
        .route("/backup/list", axum::routing::get(list_backups))
        // Wizard routes (read-only access)
        .route("/wizard", axum::routing::get(wizard_index))
        .route("/wizard/domain-config", axum::routing::get(domain_config))
        .route("/wizard/alias-config", axum::routing::get(alias_config))
        .route("/wizard/review", axum::routing::get(review))
        .route("/wizard/complete", axum::routing::get(complete))
        .route(
            "/wizard/destination-search",
            axum::routing::get(destination_search),
        )
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_auth,
        ));

    // Create edit routes (require edit permissions)
    let edit_routes = Router::new()
        // Domain edit operations
        .route("/domains", axum::routing::post(create_domain))
        .route("/domains/new", axum::routing::get(new_domain))
        .route(
            "/domains/{id}",
            axum::routing::put(update_domain).delete(delete_domain),
        )
        .route("/domains/{id}/edit", axum::routing::get(edit_domain))
        .route(
            "/domains/{id}/toggle-list",
            axum::routing::post(toggle_domain_enabled_list),
        )
        .route(
            "/domains/{id}/toggle-show",
            axum::routing::post(toggle_domain_enabled_show),
        )
        .route(
            "/domains/{id}/toggle",
            axum::routing::post(toggle_domain_enabled),
        )
        .route(
            "/domains/{id}/add-missing-alias/{alias}",
            axum::routing::post(add_missing_required_alias),
        )
        // User edit operations
        .route("/users", axum::routing::post(create_user))
        .route("/users/new", axum::routing::get(new_user))
        .route(
            "/users/{id}",
            axum::routing::put(update_user).delete(delete_user),
        )
        .route("/users/{id}/edit", axum::routing::get(edit_user))
        .route(
            "/users/{id}/toggle-list",
            axum::routing::post(toggle_user_enabled_list),
        )
        .route(
            "/users/{id}/toggle-show",
            axum::routing::post(toggle_user_enabled_show),
        )
        .route(
            "/users/{id}/toggle",
            axum::routing::post(toggle_user_enabled),
        )
        .route(
            "/users/{id}/change-password",
            axum::routing::get(change_password_form),
        )
        .route(
            "/users/{id}/change-password",
            axum::routing::post(change_password_post),
        )
        .route(
            "/users/{id}/toggle-change-password",
            axum::routing::post(toggle_change_password),
        )
        // Alias edit operations
        .route("/aliases", axum::routing::post(create))
        .route("/aliases/new", axum::routing::get(new))
        .route("/aliases/{id}", axum::routing::put(update).delete(delete))
        .route("/aliases/{id}/edit", axum::routing::get(edit))
        .route(
            "/aliases/{id}/toggle-list",
            axum::routing::post(toggle_enabled_list),
        )
        .route(
            "/aliases/{id}/toggle-show",
            axum::routing::post(toggle_enabled_show),
        )
        .route(
            "/aliases/{id}/toggle-domain-show",
            axum::routing::post(toggle_enabled_domain_show),
        )
        .route("/aliases/{id}/toggle", axum::routing::post(toggle_enabled))
        // Backup edit operations
        .route("/backups", axum::routing::post(backups::create))
        .route("/backups/new", axum::routing::get(backups::new))
        .route(
            "/backups/{id}",
            axum::routing::put(backups::update).delete(backups::delete),
        )
        .route("/backups/{id}/edit", axum::routing::get(backups::edit))
        .route(
            "/backups/{id}/toggle-show",
            axum::routing::post(backups::toggle_enabled_show),
        )
        .route(
            "/backups/{id}/toggle",
            axum::routing::post(backups::toggle_enabled),
        )
        // Relay edit operations
        .route("/relays", axum::routing::post(create_relay))
        .route("/relays/new", axum::routing::get(create_relay_form))
        .route(
            "/relays/{id}",
            axum::routing::put(update_relay).delete(delete_relay),
        )
        .route("/relays/{id}/edit", axum::routing::get(edit_relay_form))
        .route(
            "/relays/{id}/toggle-enabled",
            axum::routing::post(toggle_relay_enabled),
        )
        // Relocated edit operations
        .route("/relocated", axum::routing::post(create_relocated))
        .route("/relocated/new", axum::routing::get(create_relocated_form))
        .route(
            "/relocated/{id}",
            axum::routing::put(update_relocated).delete(delete_relocated),
        )
        .route(
            "/relocated/{id}/edit",
            axum::routing::get(edit_relocated_form),
        )
        .route(
            "/relocated/{id}/toggle-enabled",
            axum::routing::post(toggle_relocated_enabled),
        )
        // Client edit operations
        .route("/clients", axum::routing::post(create_client))
        .route("/clients/new", axum::routing::get(create_client_form))
        .route(
            "/clients/{id}",
            axum::routing::put(update_client).delete(delete_client),
        )
        .route("/clients/{id}/edit", axum::routing::get(edit_client_form))
        .route("/clients/{id}/toggle", axum::routing::put(toggle_client))
        // Wizard edit operations (require edit permissions)
        .route("/wizard/execute", axum::routing::post(execute))
        .route(
            "/wizard/domain-config",
            axum::routing::post(domain_config_post),
        )
        .route(
            "/wizard/alias-config",
            axum::routing::post(alias_config_post),
        )
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_auth,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_edit_permissions,
        ));

    // Create the main app with public and protected routes
    Router::new()
        // Public routes (no authentication required)
        .route("/login", axum::routing::get(login_form).post(login))
        .route("/logout", axum::routing::post(logout))
        .route("/health", axum::routing::get(health::health_check)) // Add health endpoint
        // Theme and language
        .route("/theme/toggle", axum::routing::post(toggle_theme))
        .route("/language/set", axum::routing::post(set_language))
        // Serve static files at /static from the ./static directory
        .nest_service("/static", tower_http::services::ServeDir::new("./static"))
        // Merge read-only and edit routes
        .merge(read_only_routes)
        .merge(edit_routes)
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http())
        .fallback(|headers, state| async move { not_found(headers, state).await })
}

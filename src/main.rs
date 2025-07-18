use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod db;
pub mod handlers;
pub mod i18n;
pub mod models;
pub mod schema;
pub mod templates;

#[cfg(test)]
pub mod tests;

// Production uses MySQL
pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct AppState {
    db_manager: db::DatabaseManager,
    i18n: i18n::I18n,
    config: config::Config,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Load configuration first
    let config = config::Config::load().expect("Failed to load configuration");

    // Create database manager with multiple databases
    let db_manager = if config.databases.is_empty() {
        // Fallback to single database from environment variable
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let fallback_config = vec![config::DatabaseConfig {
            id: "primary".to_string(),
            label: "Primary Database".to_string(),
            url: db_url,
            features: config::DatabaseFeatures::default(),
        }];
        db::DatabaseManager::new(fallback_config)
            .await
            .expect("Failed to create database manager")
    } else {
        db::DatabaseManager::new(config.databases.clone())
            .await
            .expect("Failed to create database manager")
    };

    // Run database migrations on all configured databases
    if let Err(e) = db_manager.run_migrations_on_all_databases().await {
        tracing::error!("Failed to run migrations on all databases: {}", e);
        std::process::exit(1);
    }

    // Initialize i18n
    let i18n = i18n::I18n::new("en-US").expect("Failed to initialize i18n");
    // Preload Spanish locale
    i18n.load_locale("es-ES")
        .await
        .expect("Failed to load Spanish locale");
    // Preload Norwegian locale
    i18n.load_locale("nb-NO")
        .await
        .expect("Failed to load Norwegian locale");
    // Preload French locale
    i18n.load_locale("fr-FR")
        .await
        .expect("Failed to load French locale");

    let app_state = AppState {
        db_manager,
        i18n,
        config,
    };

    // Create read-only routes (require authentication but not edit permissions)
    let read_only_routes = Router::new()
        .route("/", get(handlers::dashboard::index))
        .route("/about", get(handlers::about::index))
        // Read-only domain operations
        .route("/domains", get(handlers::domains::list))
        .route("/domains/{id}", get(handlers::domains::show))
        // Read-only user operations
        .route("/users", get(handlers::users::list))
        .route("/users/{id}", get(handlers::users::show))
        // Read-only alias operations
        .route("/aliases", get(handlers::aliases::list))
        .route("/aliases/{id}", get(handlers::aliases::show))
        .route("/aliases/search", get(handlers::aliases::search))
        .route(
            "/aliases/domain-search",
            get(handlers::aliases::domain_search),
        )
        // Read-only backup operations
        .route("/backups/{id}", get(handlers::backups::show))
        // Read-only relay operations
        .route("/relays", get(handlers::relays::list_relays))
        .route("/relays/{id}", get(handlers::relays::show_relay))
        // Read-only relocated operations
        .route("/relocated", get(handlers::relocated::list_relocated))
        .route("/relocated/{id}", get(handlers::relocated::show_relocated))
        // Read-only client operations
        .route("/clients", get(handlers::clients::list_clients))
        .route("/clients/{id}", get(handlers::clients::show_client))
        // Stats and reports
        .route("/stats", get(handlers::stats::index))
        .route("/reports", get(handlers::reports::reports_list))
        .route("/reports/matrix", get(handlers::reports::matrix_report))
        .route(
            "/reports/cross-database-matrix",
            get(handlers::reports::cross_database_domain_matrix_report),
        )
        .route("/reports/orphaned", get(handlers::reports::orphaned_report))
        .route(
            "/reports/external-forwarders",
            get(handlers::reports::external_forwarders_report),
        )
        .route(
            "/reports/alias-cross-domain",
            get(handlers::reports::alias_cross_domain_report),
        )
        .route(
            "/reports/cross-database-user-distribution",
            get(handlers::reports::cross_database_user_distribution_report),
        )
        .route(
            "/reports/cross-database-feature-toggle",
            get(handlers::reports::cross_database_feature_toggle_report),
        )
        .route(
            "/reports/cross-database-migration",
            get(handlers::reports::cross_database_migration_report),
        )
        // Configuration
        .route("/config", get(handlers::config::view_config))
        // Database selection
        .route("/database", get(handlers::database::index))
        .route("/database/select", post(handlers::database::select))
        .route("/database/dropdown", get(handlers::database::dropdown))
        .route(
            "/database/migrate",
            post(handlers::database::run_migrations),
        )
        .route("/api/databases", get(handlers::database::list_databases))
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            handlers::auth::require_auth,
        ));

    // Create edit routes (require edit permissions)
    let edit_routes = Router::new()
        // Domain edit operations
        .route("/domains", post(handlers::domains::create))
        .route("/domains/new", get(handlers::domains::new))
        .route(
            "/domains/{id}",
            put(handlers::domains::update).delete(handlers::domains::delete),
        )
        .route("/domains/{id}/edit", get(handlers::domains::edit))
        .route(
            "/domains/{id}/toggle-list",
            post(handlers::domains::toggle_enabled_list),
        )
        .route(
            "/domains/{id}/toggle-show",
            post(handlers::domains::toggle_enabled_show),
        )
        .route(
            "/domains/{id}/toggle",
            post(handlers::domains::toggle_enabled),
        )
        .route(
            "/domains/{id}/add-missing-alias/{alias}",
            post(handlers::domains::add_missing_required_alias),
        )
        // User edit operations
        .route("/users", post(handlers::users::create))
        .route("/users/new", get(handlers::users::new))
        .route(
            "/users/{id}",
            put(handlers::users::update).delete(handlers::users::delete),
        )
        .route("/users/{id}/edit", get(handlers::users::edit))
        .route(
            "/users/{id}/toggle-list",
            post(handlers::users::toggle_enabled_list),
        )
        .route(
            "/users/{id}/toggle-show",
            post(handlers::users::toggle_enabled_show),
        )
        .route("/users/{id}/toggle", post(handlers::users::toggle_enabled))
        .route(
            "/users/{id}/change-password",
            get(handlers::users::change_password_form),
        )
        .route(
            "/users/{id}/change-password",
            post(handlers::users::change_password_post),
        )
        .route(
            "/users/{id}/toggle-change-password",
            post(handlers::users::toggle_change_password),
        )
        // Alias edit operations
        .route("/aliases", post(handlers::aliases::create))
        .route("/aliases/new", get(handlers::aliases::new))
        .route(
            "/aliases/{id}",
            put(handlers::aliases::update).delete(handlers::aliases::delete),
        )
        .route("/aliases/{id}/edit", get(handlers::aliases::edit))
        .route(
            "/aliases/{id}/toggle-list",
            post(handlers::aliases::toggle_enabled_list),
        )
        .route(
            "/aliases/{id}/toggle-show",
            post(handlers::aliases::toggle_enabled_show),
        )
        .route(
            "/aliases/{id}/toggle-domain-show",
            post(handlers::aliases::toggle_enabled_domain_show),
        )
        .route(
            "/aliases/{id}/toggle",
            post(handlers::aliases::toggle_enabled),
        )
        // Backup edit operations
        .route("/backups", post(handlers::backups::create))
        .route("/backups/new", get(handlers::backups::new))
        .route(
            "/backups/{id}",
            put(handlers::backups::update).delete(handlers::backups::delete),
        )
        .route("/backups/{id}/edit", get(handlers::backups::edit))
        .route(
            "/backups/{id}/toggle-show",
            post(handlers::backups::toggle_enabled_show),
        )
        .route(
            "/backups/{id}/toggle",
            post(handlers::backups::toggle_enabled),
        )
        // Relay edit operations
        .route("/relays", post(handlers::relays::create_relay))
        .route("/relays/new", get(handlers::relays::create_form))
        .route(
            "/relays/{id}",
            put(handlers::relays::update_relay).delete(handlers::relays::delete_relay),
        )
        .route("/relays/{id}/edit", get(handlers::relays::edit_form))
        .route(
            "/relays/{id}/toggle-enabled",
            post(handlers::relays::toggle_enabled),
        )
        // Relocated edit operations
        .route("/relocated", post(handlers::relocated::create_relocated))
        .route("/relocated/new", get(handlers::relocated::create_form))
        .route(
            "/relocated/{id}",
            put(handlers::relocated::update_relocated)
                .delete(handlers::relocated::delete_relocated),
        )
        .route("/relocated/{id}/edit", get(handlers::relocated::edit_form))
        .route(
            "/relocated/{id}/toggle-enabled",
            post(handlers::relocated::toggle_enabled),
        )
        // Client edit operations
        .route("/clients", post(handlers::clients::create_client))
        .route("/clients/new", get(handlers::clients::create_client_form))
        .route(
            "/clients/{id}",
            put(handlers::clients::update_client).delete(handlers::clients::delete_client),
        )
        .route(
            "/clients/{id}/edit",
            get(handlers::clients::edit_client_form),
        )
        .route(
            "/clients/{id}/toggle",
            put(handlers::clients::toggle_client),
        )
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            handlers::auth::require_auth,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            handlers::auth::require_edit_permissions,
        ));

    // Create the main app with public and protected routes
    let app = Router::new()
        // Public routes (no authentication required)
        .route(
            "/login",
            get(handlers::auth::login_form).post(handlers::auth::login),
        )
        .route("/logout", post(handlers::auth::logout))
        // Theme and language
        .route("/theme/toggle", post(handlers::theme::toggle_theme))
        .route("/language/set", post(handlers::language::set_language))
        // Serve static files at /static from the ./static directory
        .nest_service("/static", tower_http::services::ServeDir::new("./static"))
        // Merge read-only and edit routes
        .merge(read_only_routes)
        .merge(edit_routes)
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .fallback(handlers::not_found);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

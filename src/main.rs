use axum::{
    routing::{get, post},
    Router,
};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod db;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod templates;

#[cfg(test)]
pub mod tests;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct AppState {
    pool: DbPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Run database migrations
    let mut conn = pool.get().expect("Failed to get db connection from pool");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run db migrations");

    let app_state = AppState { pool };

    let app = Router::new()
        // Auth
        .route(
            "/login",
            get(handlers::auth::login_form).post(handlers::auth::login),
        )
        .route("/logout", post(handlers::auth::logout))
        // Dashboard
        .route("/", get(handlers::dashboard::index))
        // Domains
        .route(
            "/domains",
            get(handlers::domains::list).post(handlers::domains::create),
        )
        .route("/domains/new", get(handlers::domains::new))
        .route(
            "/domains/:id",
            get(handlers::domains::show)
                .put(handlers::domains::update)
                .delete(handlers::domains::delete),
        )
        .route("/domains/:id/edit", get(handlers::domains::edit))
        .route(
            "/domains/:id/toggle-list",
            post(handlers::domains::toggle_enabled_list),
        )
        .route(
            "/domains/:id/toggle-show",
            post(handlers::domains::toggle_enabled_show),
        )
        .route(
            "/domains/:id/toggle",
            post(handlers::domains::toggle_enabled),
        )
        // Users
        .route(
            "/users",
            get(handlers::users::list).post(handlers::users::create),
        )
        .route("/users/new", get(handlers::users::new))
        .route(
            "/users/:id",
            get(handlers::users::show)
                .put(handlers::users::update)
                .delete(handlers::users::delete),
        )
        .route("/users/:id/edit", get(handlers::users::edit))
        .route(
            "/users/:id/toggle-list",
            post(handlers::users::toggle_enabled_list),
        )
        .route(
            "/users/:id/toggle-show",
            post(handlers::users::toggle_enabled_show),
        )
        .route("/users/:id/toggle", post(handlers::users::toggle_enabled))
        // Aliases
        .route(
            "/aliases",
            get(handlers::aliases::list).post(handlers::aliases::create),
        )
        .route("/aliases/new", get(handlers::aliases::new))
        .route(
            "/aliases/:id",
            get(handlers::aliases::show)
                .put(handlers::aliases::update)
                .delete(handlers::aliases::delete),
        )
        .route("/aliases/:id/edit", get(handlers::aliases::edit))
        .route(
            "/aliases/:id/toggle-list",
            post(handlers::aliases::toggle_enabled_list),
        )
        .route(
            "/aliases/:id/toggle-show",
            post(handlers::aliases::toggle_enabled_show),
        )
        .route(
            "/aliases/:id/toggle",
            post(handlers::aliases::toggle_enabled),
        )
        // Stats
        .route("/stats", get(handlers::stats::index))
        // Theme
        .route("/theme/toggle", post(handlers::theme::toggle_theme))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

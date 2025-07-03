use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod db;
mod handlers;
mod models;
mod schema;
mod templates;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type AppState = Arc<SharedState>;

#[derive(Clone)]
pub struct SharedState {
    pub pool: DbPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    // Run migrations
    diesel_migrations::embed_migrations!();
    embedded_migrations::run(&mut pool.get().unwrap())
        .expect("Failed to run migrations");

    let state = Arc::new(SharedState { pool });

    // Build application
    let app = Router::new()
        .route("/", get(handlers::dashboard::index))
        .route("/login", get(handlers::auth::login_page))
        .route("/login", post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/domains", get(handlers::domains::list))
        .route("/domains", post(handlers::domains::create))
        .route("/domains/:id", get(handlers::domains::show))
        .route("/domains/:id", post(handlers::domains::update))
        .route("/domains/:id/delete", post(handlers::domains::delete))
        .route("/users", get(handlers::users::list))
        .route("/users", post(handlers::users::create))
        .route("/users/:id", get(handlers::users::show))
        .route("/users/:id", post(handlers::users::update))
        .route("/users/:id/delete", post(handlers::users::delete))
        .route("/aliases", get(handlers::aliases::list))
        .route("/aliases", post(handlers::aliases::create))
        .route("/aliases/:id", get(handlers::aliases::show))
        .route("/aliases/:id", post(handlers::aliases::update))
        .route("/aliases/:id/delete", post(handlers::aliases::delete))
        .route("/mailboxes", get(handlers::mailboxes::list))
        .route("/mailboxes", post(handlers::mailboxes::create))
        .route("/mailboxes/:id", get(handlers::mailboxes::show))
        .route("/mailboxes/:id", post(handlers::mailboxes::update))
        .route("/mailboxes/:id/delete", post(handlers::mailboxes::delete))
        .route("/stats", get(handlers::stats::index))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
} 

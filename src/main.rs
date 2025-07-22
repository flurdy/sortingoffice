use sortingoffice::{AppState, Config, DatabaseManager, I18n};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize i18n
    let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

    // Initialize database manager
    let db_manager = DatabaseManager::new(config.databases.clone())
        .await
        .expect("Failed to initialize database manager");

    // Create app state
    let state = AppState {
        db_manager,
        i18n,
        config,
    };

    // Create app
    let app = sortingoffice::handlers::create_app(state);

    // Bind to port from env or default 3000
    let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);

    // Start server
    axum::serve(listener, app).await.unwrap();
}

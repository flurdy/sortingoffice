use sortingoffice::{AppState, Config, DatabaseManager, I18n};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize database manager
    let db_manager = DatabaseManager::new(config.databases.clone())
        .await
        .expect("Failed to initialize database manager");

    // Initialize i18n
    let i18n = I18n::new("en-US").expect("Failed to initialize i18n");

    // Create app state
    let state = AppState {
        db_manager,
        i18n,
        config,
    };

    // Create and run the app
    let app = sortingoffice::handlers::create_app(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    log::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

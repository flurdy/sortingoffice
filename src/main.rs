use sortingoffice::{AppState, Config, DatabaseManager, I18n};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let config_path = "config/config.toml";
    let config = Config::load_config_with_env(config_path)
        .unwrap_or_else(|e| panic!("Failed to load configuration: {:?}", e));

    // Debug: log all loaded database configs
    fn mask_db_url(url: &str) -> String {
        // Only mask mysql://user:pass@host/db or similar
        if let Some(idx) = url.find("//") {
            let (scheme, rest) = url.split_at(idx + 2);
            if let Some(at_idx) = rest.find('@') {
                let (userpass, hostrest) = rest.split_at(at_idx);
                if let Some(colon_idx) = userpass.find(':') {
                    let (user, _pass) = userpass.split_at(colon_idx);
                    return format!("{}{}:***{}", scheme, user, hostrest);
                }
            }
        }
        url.to_string()
    }
    for db in &config.databases {
        println!(
            "[DEBUG] Loaded database config: id={} url={}",
            db.id,
            mask_db_url(&db.url)
        );
    }
    // Debug: log all loaded admin users
    for admin in &config.admins {
        println!(
            "[DEBUG] Loaded admin: username={} role={:?}",
            admin.username, admin.role
        );
    }

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
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);

    // Start server
    axum::serve(listener, app).await.unwrap();
}

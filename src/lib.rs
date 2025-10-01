pub mod analytics;
pub mod config;
pub mod config_utils;
pub mod db;
pub mod handlers;
pub mod i18n;
pub mod models;
pub mod schema;
pub mod services;
pub mod templates;
pub mod test_helpers;
pub mod validation;

pub use crate::config::Config;
pub use crate::db::DatabaseManager;
pub use crate::i18n::I18n;

use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Clone)]
pub struct AppState {
    pub db_manager: DatabaseManager,
    pub i18n: I18n,
    pub config: Config,
}

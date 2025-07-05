pub mod models;
pub mod db;
pub mod handlers;
pub mod utils;
pub mod integration;

#[cfg(test)]
mod common {
    use crate::DbPool;
    use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::RunQueryDsl;
use std::sync::Once;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    static INIT: Once = Once::new();

    pub fn setup_test_db() -> DbPool {
        INIT.call_once(|| {
            std::env::set_var("RUST_LOG", "debug");
            tracing_subscriber::fmt::init();
        });

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/sortingoffice_test".to_string());
        
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        // Run migrations
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        pool
    }

    pub fn cleanup_test_db(pool: &DbPool) {
        let mut conn = pool.get().expect("Failed to get connection");
        
        // Clean up test data
        diesel::delete(crate::schema::aliases::table)
            .execute(&mut conn)
            .ok();
        diesel::delete(crate::schema::users::table)
            .execute(&mut conn)
            .ok();
        diesel::delete(crate::schema::domains::table)
            .execute(&mut conn)
            .ok();
    }
} 

//! Database connection management module
//!
//! This module provides a connection pool management system for PostgreSQL databases,
//! supporting separate read and write connections for better scalability.
//!
//! # Environment Variables
//! - DATABASE_URL: Default connection string
//! - DATABASE_WRITE_URL: Writer connection string (takes precedence over DATABASE_URL)
//! - DATABASE_READ_URL: Reader connection string (optional, defaults to writer connection)
//!
//! # Example Usage
//! ```
//! async fn example() {
//!     // Initialize the database connections
//!     database::init().await;
//!
//!     // Get connection pools
//!     let reader = database::reader();
//!     let writer = database::writer();
//!
//!     // Execute queries
//!     let result = sqlx::query("SELECT * FROM users")
//!         .fetch_all(reader)
//!         .await?;
//! }
//! ```

use async_once_cell::OnceCell;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};

// Global database instance wrapped in a thread-safe, lazy-initialized container
static DATABASE: OnceCell<Arc<Database>> = OnceCell::new();

/// Main database connection manager that holds both reader and writer pools
#[derive(Clone, Debug)]
pub struct Database {
    /// Connection string used to establish the connection
    pub url: String,
    /// Connection pool for read operations (maybe same as writer in single-db setups)
    pub reader: Pool<Postgres>,
    /// Connection pool for write operations
    pub writer: Pool<Postgres>,
}

/// Initialize the global database instance
///
/// This function must be called before any database operations can be performed.
/// It will initialize the connection pools based on environment variables.
///
/// # Panics
/// If required environment variables are missing or connections fail.
pub async fn init() {
    DATABASE.get_or_init(async {
        Arc::new(Database::init().await)
    }).await;
}

/// Get a reference to the reader connection pool
///
/// # Returns
/// Reference to the PostgreSQL connection pool configured for read operations
///
/// # Panics
/// If database has not been initialized via `init()`
pub fn reader<'a>() -> &'a Pool<Postgres> {
    if let Some(database) = DATABASE.get() {
        return database.reader();
    }

    panic!("Database not initialized")
}

/// Get a reference to the writer connection pool
///
/// # Returns
/// Reference to the PostgreSQL connection pool configured for write operations
///
/// # Panics
/// If database has not been initialized via `init()`
pub fn writer<'a>() -> &'a Pool<Postgres> {
    if let Some(database) = DATABASE.get() {
        return database.writer();
    }

    panic!("Database not initialized")
}

/// Get the connection URL string
///
/// # Example
/// ```
/// fn log_connection_info() {
///     println!("Connected to database: {}", database::url());
/// }
/// ```
///
/// # Returns
/// String containing the connection URL
///
/// # Panics
/// If database has not been initialized via `init(
pub fn url() -> String {
    if let Some(database) = DATABASE.get() {
        return database.url.clone();
    }

    panic!("Database not initialized")
}

impl Database {
    /// Create a new Database instance with configured connection pools
    ///
    /// # Example
    /// ```
    /// use database::Database;
    ///
    /// async fn main() {
    ///     // Set up environment variables
    ///     std::env::set_var("DATABASE_WRITE_URL", "postgres://user:pass@primary-db/app");
    ///     std::env::set_var("DATABASE_READ_URL", "postgres://user:pass@replica-db/app");
    ///
    ///     // Initialize database
    ///     let db = Database::init().await;
    ///
    ///     // Use the database instance
    ///     let writer_pool = db.writer();
    ///     let reader_pool = db.reader();
    /// }
    /// ```
    ///
    /// # Connection Priority
    /// 1. DATABASE_WRITE_URL for writer connection
    /// 2. DATABASE_URL as fallback for writer connection
    /// 3. DATABASE_READ_URL for reader connection (optional)
    ///
    /// If DATABASE_READ_URL is not provided, reader will use the same pool as writer
    ///
    /// # Panics
    /// - If no valid connection URL is provided via environment variables
    /// - If connection pool creation fails
    pub async fn init() -> Self {
        let mut is_valid_connection = false;
        let mut writer = String::default();
        let mut url = String::default();

        if let Ok(string) = env::var("DATABASE_URL") {
            is_valid_connection = true;
            writer = string.clone();
            url = string;
        }

        if let Ok(string) = env::var("DATABASE_WRITE_URL") {
            is_valid_connection = true;
            writer = string.clone();
            url = string;
        }

        if !is_valid_connection {
            panic!("Unable to connect to the database");
        }

        if let Ok(writer) = PgPoolOptions::new()
            .connect(&writer)
            .await
        {
            let mut reader = writer.clone();
            if let Ok(string) = env::var("DATABASE_READ_URL") {
                url = string;

                if let Ok(pool) = PgPoolOptions::new()
                    .connect(&url)
                    .await
                {
                    reader = pool;
                }
            }

            return Self { url, reader, writer };
        }

        panic!("Invalid database connection string");
    }

    /// Get a reference to the reader connection pool
    pub fn reader(&self) -> &Pool<Postgres> {
        &self.reader
    }

    /// Get a reference to the writer connection pool
    pub fn writer(&self) -> &Pool<Postgres> {
        &self.writer
    }
}
# PostgreSQL Connection Manager

A Rust library for managing PostgreSQL database connections with read/write connection pool separation.

## Features

- **Connection Pool Management**: Efficiently manages PostgreSQL connection pools
- **Read/Write Separation**: Supports separate connection pools for read and write operations
- **Environment-based Configuration**: Simple setup through environment variables
- **Lazy Initialization**: Connections are established only when needed
- **Thread Safety**: Safe to use in concurrent environments

## Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
database = { git = "https://github.com/yourusername/database.git" }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
async-once-cell = "0.5"
```

## Environment Variables

Configure your database connections using these environment variables:

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | Default connection string | Yes (if `DATABASE_WRITE_URL` not provided) |
| `DATABASE_WRITE_URL` | Writer connection string (takes precedence over `DATABASE_URL`) | No |
| `DATABASE_READ_URL` | Reader connection string | No (defaults to writer connection) |

## Usage

### Basic Example

```rust
use database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database connection pools
    database::init().await;
    
    // Execute a read query
    let users = sqlx::query!("SELECT id, name FROM users")
        .fetch_all(database::reader())
        .await?;
        
    // Execute a write query
    let result = sqlx::query!("INSERT INTO logs (message) VALUES ($1)", "Test log")
        .execute(database::writer())
        .await?;
        
    println!("Connection URL: {}", database::url());
    
    Ok(())
}
```

### Custom Instance Example

```rust
use database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set environment variables programmatically
    std::env::set_var("DATABASE_WRITE_URL", "postgres://user:pass@primary-db/app");
    std::env::set_var("DATABASE_READ_URL", "postgres://user:pass@replica-db/app");
    
    // Create a custom database instance
    let db = Database::init().await;
    
    // Use the custom instance directly
    let users = sqlx::query!("SELECT id, name FROM users")
        .fetch_all(db.reader())
        .await?;
    
    Ok(())
}
```

## Connection Priority

The library determines which connection strings to use with the following priority:

1. `DATABASE_WRITE_URL` for writer connection (highest priority)
2. `DATABASE_URL` as fallback for writer connection
3. `DATABASE_READ_URL` for reader connection (optional)

If `DATABASE_READ_URL` is not provided, the reader will use the same pool as the writer.

## API Reference

### Functions

- `init()` - Initialize the global database instance (must be called before using other functions)
- `reader()` - Get a reference to the reader connection pool
- `writer()` - Get a reference to the writer connection pool
- `url()` - Get the connection URL string

### Structs

- `Database` - Main connection manager that holds both reader and writer pools

## Error Handling

The library will panic in the following scenarios:

- If no valid connection URL is provided via environment variables
- If connection pool creation fails
- If you attempt to use `reader()`, `writer()`, or `url()` before calling `init()`

## License

[MIT](LICENSE)
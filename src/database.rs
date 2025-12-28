use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs::File;
use std::path::Path;

pub async fn init_db() -> Result<Pool<Sqlite>, sqlx::Error> {
    if !Path::new("database.db").exists() {
        File::create("database.db").expect("Impossible de créer le fichier database.db");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://database.db")
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tickets (
            user_id INTEGER PRIMARY KEY,
            channel_id INTEGER NOT NULL,
            category TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            initial_message TEXT NOT NULL,
            last_activity INTEGER NOT NULL,
            has_been_reminded BOOLEAN NOT NULL DEFAULT 0
        )"
    ).execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS blacklist (
            user_id INTEGER PRIMARY KEY,
            reason TEXT NOT NULL,
            by_staff INTEGER NOT NULL,
            date INTEGER NOT NULL
        )"
    ).execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ticket_counts (
            category TEXT PRIMARY KEY,
            count INTEGER NOT NULL DEFAULT 0
        )"
    ).execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS voice_channels (
            channel_id INTEGER PRIMARY KEY,
            owner_id INTEGER NOT NULL
        )"
    ).execute(&pool).await?;

    Ok(pool)
}

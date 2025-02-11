use std::{env, path::PathBuf};
use sqlx::{migrate::MigrateDatabase, query, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use logfather::{info, error};

pub mod users;
pub mod prefixes;
pub mod wolvesville;

pub async fn get_pool() -> anyhow::Result<SqlitePool> {
    let (db_url, _) = get_db_url()?;

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    Ok(pool)
}

fn get_db_url() -> anyhow::Result<(String, PathBuf)> {
    let cwd = env::current_dir()?;
    let filepath = cwd.join("res/database/main.db");

    let url = format!("sqlite://{}", filepath.display());

    Ok((url, filepath))
}

pub async fn create_db() {
    let (db_url, filepath) = match get_db_url() {
        Ok(url_info) => url_info,
        Err(err) => { error!("Failed to get db url: {}", err); return}
    };

    let filename = filepath.file_name().unwrap().to_str().unwrap();

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        info!("Creating database: {}", filename);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => info!("Database created: {}", filename),
            Err(err) => { error!("Failed to create database: {}", err); return}
        };
        let pool = match get_pool().await {
            Ok(pool) => pool,
            Err(err) => { error!("Failed to get pool: {}", err); return}
        };
        match initialize_schema(&pool).await {
            Ok(_) => info!("Schema initialized: {}", filename),
            Err(err) => { error!("Failed to initialize schema: {}", err); return}
        };
        pool.close().await;
    } else {
        info!("Detected database: {}", filename);
    }
}

async fn initialize_schema(pool: &SqlitePool) -> anyhow::Result<()> {
    // NOT FINISHED
    let q = r#"
        CREATE TABLE IF NOT EXISTS prefixes (
            guild_id TEXT PRIMARY KEY,
            prefix TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            discord_id TEXT PRIMARY KEY,
            language_code TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS wolvesville_clans (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            json JSON NOT NULL,
            members_json JSON,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS wolvesville_players (
            id TEXT PRIMARY KEY,
            personal_message TEXT,
            json JSON NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS wolvesville_player_usernames (
            player_id TEXT NOT NULL,
            username TEXT NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(player_id, username),
            FOREIGN KEY(player_id) REFERENCES wolvesville_players(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS wolvesville_player_ranked_skill (
            player_id TEXT NOT NULL,
            skill INTEGER NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(player_id, timestamp),
            FOREIGN KEY(player_id) REFERENCES wolvesville_players(id) ON DELETE CASCADE
        );
    "#;

    query(q).execute(pool).await?;

    Ok(())
}
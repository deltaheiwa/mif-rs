use std::{env, path::PathBuf};
use sqlx::{migrate::MigrateDatabase, query, Sqlite, SqlitePool, Row};
use tracing::{info, error};


async fn get_pool() -> anyhow::Result<SqlitePool> {
    let (db_url, _) = get_db_url()?;

    let pool = SqlitePool::connect(&db_url).await?;

    Ok(pool)
}

fn get_db_url() -> anyhow::Result<(String, PathBuf)> {
    let cwd = env::current_dir()?;
    let filepath = cwd.join("res/database/prefixes.db");

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
        match initialize_schema().await {
            Ok(_) => info!("Schema initialized: {}", filename),
            Err(err) => { error!("Failed to initialize schema: {}", err); return}
        };
    } else {
        info!("Detected database: {}", filename);
    }
}

async fn initialize_schema() -> anyhow::Result<()> {
    let pool = get_pool().await?;

    let q = r#"
        CREATE TABLE IF NOT EXISTS prefixes (
            guild_id TEXT PRIMARY KEY,
            prefix TEXT NOT NULL
        );
    "#;

    query(q).execute(&pool).await?;

    Ok(())
}

pub async fn get_prefix(guild_id: &String) -> anyhow::Result<String> {
    let pool = get_pool().await?;

    let q = r#"
        SELECT prefix FROM prefixes WHERE guild_id = $1;
    "#;

    let row = query(q).bind(guild_id).fetch_one(&pool).await?;

    Ok(row.get("prefix"))
}

pub async fn set_prefix(guild_id: &String, prefix: &String) -> anyhow::Result<()> {
    let pool = get_pool().await?;

    let q = r#"
        INSERT INTO prefixes (guild_id, prefix) VALUES ($1, $2)
        ON CONFLICT(guild_id) DO UPDATE SET prefix = $2;
    "#;

    query(q).bind(guild_id).bind(prefix).execute(&pool).await?;

    Ok(())
}

pub async fn delete_prefix(guild_id: &String) -> anyhow::Result<()> {
    let pool = get_pool().await?;

    let q = r#"
        DELETE FROM prefixes WHERE guild_id = $1;
    "#;

    query(q).bind(guild_id).execute(&pool).await?;

    Ok(())
}

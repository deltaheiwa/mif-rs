use std::{env, path::PathBuf};
use sqlx::{migrate::MigrateDatabase, Sqlite};
use tracing::{info, error};


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
    } else {
        info!("Detected database: {}", filename);
    }
}
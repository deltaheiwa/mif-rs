use sqlx::{query, Row, SqlitePool};


/// Check if a user exists in the database.
pub async fn hit_user(pool: &SqlitePool, user_id: &String) -> anyhow::Result<bool> {
    let q = r#"
        SELECT 1 AS exists FROM users WHERE user_id = $1;
    "#;

    let row = query(q).bind(user_id).fetch_one(pool).await?;

    Ok(row.get("exists"))
} 

pub async fn get_language_code(pool: &SqlitePool, user_id: &String) -> anyhow::Result<String> {
    let q = r#"
        SELECT language_code FROM users WHERE user_id = $1;
    "#;

    let row = query(q).bind(user_id).fetch_one(pool).await;

    match row {
        Ok(row) => Ok(row.get("language_code")),
        Err(_) => Ok(String::from("en"))
    }
}

pub async fn add_user(pool: &SqlitePool, user_id: &String) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO users (user_id, language_code) VALUES ($1, "en")
        ON CONFLICT(user_id) DO NOTHING;
    "#;

    query(q).bind(user_id).execute(pool).await?;

    Ok(())
}

pub async fn set_language_code(pool: &SqlitePool, user_id: &String, language_code: &String) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO users (user_id, language_code) VALUES ($1, $2)
        ON CONFLICT(user_id) DO UPDATE SET language_code = $2;
    "#;

    query(q).bind(user_id).bind(language_code).execute(pool).await?;

    Ok(())
}
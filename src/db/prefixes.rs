use sqlx::{query, Row, SqlitePool};
use logfather::info;


pub async fn get_prefix(pool: &SqlitePool, discord_id: &String) -> anyhow::Result<Option<String>> {
    let q = r#"
        SELECT prefix FROM prefixes WHERE discord_id = $1;
    "#;

    let row = query(q).bind(discord_id).fetch_optional(pool).await?;

    Ok(row.map(|r| r.get("prefix")))
}

pub async fn set_prefix(pool: &SqlitePool, discord_id: &String, prefix: &String) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO prefixes (discord_id, prefix) VALUES ($1, $2)
        ON CONFLICT(discord_id) DO UPDATE SET prefix = $2;
    "#;

    query(q).bind(discord_id).bind(prefix).execute(pool).await?;

    info!("Set prefix for discord entity {}: `{}`", discord_id, prefix);

    Ok(())
}

pub async fn delete_prefix(pool: &SqlitePool, discord_id: &String) -> anyhow::Result<()> {
    let q = r#"
        DELETE FROM prefixes WHERE discord_id = $1;
    "#;

    query(q).bind(discord_id).execute(pool).await?;

    info!("Deleted prefix for discord entity {}", discord_id);
    
    Ok(())
}

use sqlx::{query, Row, SqlitePool};
use logfather::info;


pub async fn get_prefix(pool: &SqlitePool, guild_id: &String) -> anyhow::Result<String> {
    let q = r#"
        SELECT prefix FROM prefixes WHERE guild_id = $1;
    "#;

    let row = query(q).bind(guild_id).fetch_one(pool).await?;

    Ok(row.get("prefix"))
}

pub async fn set_prefix(pool: &SqlitePool, guild_id: &String, prefix: &String) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO prefixes (guild_id, prefix) VALUES ($1, $2)
        ON CONFLICT(guild_id) DO UPDATE SET prefix = $2;
    "#;

    query(q).bind(guild_id).bind(prefix).execute(pool).await?;

    info!("Set prefix for guild {}: `{}`", guild_id, prefix);

    Ok(())
}

pub async fn delete_prefix(pool: &SqlitePool, guild_id: &String) -> anyhow::Result<()> {
    let q = r#"
        DELETE FROM prefixes WHERE guild_id = $1;
    "#;

    query(q).bind(guild_id).execute(pool).await?;

    info!("Deleted prefix for guild {}", guild_id);
    
    Ok(())
}

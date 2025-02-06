use logfather::debug;
use sqlx::{query, Row, SqlitePool};
use crate::utils::apicallers::wolvesville::models::WolvesvilleClan;

pub async fn get_wolvesville_clan_info_by_id(pool: &SqlitePool, clan_id: &str) -> anyhow::Result<Option<WolvesvilleClan>> {
    let q = r#"
        SELECT wc.* FROM wolvesville_clans wc
        WHERE wc.id = $1;
    "#;

    let row = query(q).bind(clan_id).fetch_optional(pool).await?;

    if row.is_none() {
        return Ok(None);
    }

    let row = row.unwrap();

    let deserialized_json = serde_json::from_value::<WolvesvilleClan>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize clan: {}", err))?;

    debug!("Got clan by id: {}", clan_id);

    Ok(Some(deserialized_json))
}

pub async fn get_wolvesville_clan_info_by_name(pool: &SqlitePool, clan_name: &str) -> anyhow::Result<Vec<WolvesvilleClan>> {
    let q = r#"
        SELECT wc.* FROM wolvesville_clans wc
        WHERE wc.name = LIKE $1;
    "#;

    let clan_name_sanitized = format!("%{}%", clan_name);

    let row = query(q).bind(clan_name_sanitized).fetch_optional(pool).await?;

    if row.is_none() {
        return Ok(vec![]);
    }

    let row = row.unwrap();

    let deserialized_json = serde_json::from_value::<WolvesvilleClan>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize clan: {}", err))?;

    debug!("Got clans by name: {}", clan_name);

    Ok(vec![deserialized_json])
}

pub async fn upsert_wolvesville_clan(pool: &SqlitePool, clan: &WolvesvilleClan) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO wolvesville_clans (id, name, json)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET
            name = $2,
            json = $3;
    "#;

    query(q)
        .bind(clan.id.as_str())
        .bind(clan.name.as_str())
        .bind(serde_json::to_value(clan)?)
        .execute(pool)
        .await?;

    debug!("Upserted clan: {}", clan.id);

    Ok(())
}
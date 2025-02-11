use logfather::debug;
use sqlx::{query, Row, SqlitePool};
use crate::utils::apicallers::wolvesville::models::{WolvesvilleClan, WolvesvilleClanMember};

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

    let mut deserialized_json = serde_json::from_value::<WolvesvilleClan>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize clan: {}", err))?;

    deserialized_json.members = serde_json::from_value(row.get("members_json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize clan members: {}", err))
        .ok();

    deserialized_json.timestamp = row.get("timestamp");

    debug!("Got clan by id: {}", clan_id);

    Ok(Some(deserialized_json))
}

pub async fn get_wolvesville_clan_info_by_name(pool: &SqlitePool, clan_name: &str) -> anyhow::Result<Vec<WolvesvilleClan>> {
    let q = r#"
        SELECT wc.* FROM wolvesville_clans wc
        WHERE wc.name LIKE '%' || $1 || '%';
    "#;

    let rows = query(q).bind(clan_name).fetch_all(pool).await?;
    let mut found_clans: Vec<WolvesvilleClan> = Vec::new();

    for row in rows {
        let mut deserialized_json = serde_json::from_value::<WolvesvilleClan>(row.get("json"))
            .map_err(|err| anyhow::anyhow!("Failed to deserialize clan: {}", err))?;

        deserialized_json.members = serde_json::from_value(row.get("members_json"))
            .map_err(|err| anyhow::anyhow!("Failed to deserialize clan members: {}", err))
            .ok();

        deserialized_json.timestamp = row.get("timestamp");

        found_clans.push(deserialized_json);
    } 

    debug!("Got {} clans by name: {}", found_clans.len(), clan_name);

    if found_clans.len() != 0 { Ok(found_clans) } else { Err(anyhow::anyhow!("No clans found by name: {}", clan_name)) }
}

pub async fn upsert_wolvesville_clan(pool: &SqlitePool, mut clan: WolvesvilleClan) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO wolvesville_clans (id, name, json, members_json)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            name = $2,
            json = $3,
            members_json = $4;
    "#;

    let members = serde_json::to_value(clan.members)?;
    clan.members = None;

    query(q)
        .bind(clan.id.as_str())
        .bind(clan.name.as_str())
        .bind(serde_json::to_value(&clan)?)
        .bind(members)
        .execute(pool)
        .await?;

    debug!("Upserted clan: {}", clan.id);

    Ok(())
}

pub async fn upsert_multiple_wolvesville_clans(pool: &SqlitePool, clans: &Vec<WolvesvilleClan>) -> anyhow::Result<()> {
    let mut q = r#"
        INSERT INTO wolvesville_clans (id, name, json, members_json)
        VALUES
    "#.to_string();

    for i in 0..clans.len() {
        q.push_str(&format!("(${}, ${}, ${}, ${})", i * 4 + 1, i * 4 + 2, i * 4 + 3, i * 4 + 4));

        if i != clans.len() - 1 {
            q.push_str(", ");
        }
    }

    q.push_str(" ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, json = EXCLUDED.json, members_json = EXCLUDED.members_json;");

    let mut query = query(q.as_str());

    for clan in clans {
        query = query
        .bind(clan.id.clone())
        .bind(clan.name.clone())
        .bind(serde_json::to_value(&clan)?)
        .bind(serde_json::to_value(&clan.members)?);
    }

    query.execute(pool).await?;

    Ok(())
}

/// Updates the members of a clan explicitly. The reason for that is that there shouldn't be a clan record without other columns, so upserting makes no sense.
/// 
/// # Arguments
///    * `pool` - The database pool obtained from `Data` struct.
///    * `clan_id` - The id of the clan to update.
///    * `members` - The members to update the clan with.
pub async fn update_wolvesville_clan_members_explicitly(pool: &SqlitePool, clan_id: &str, members: &Vec<WolvesvilleClanMember>) -> anyhow::Result<()> {
    let q = r#"
        UPDATE wolvesville_clans
        SET members_json = $2
        WHERE id = $1;
    "#;

    query(q)
        .bind(clan_id)
        .bind(serde_json::to_value(members)?)
        .execute(pool)
        .await?;

    debug!("Updated clan members for clan: {}", clan_id);

    Ok(())
}
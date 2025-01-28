use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{query, Row, Sqlite, SqlitePool, Transaction};
use logfather::{debug, info};
use crate::utils::apicallers::wolvesville::models::WolvesvillePlayer;

#[derive(Debug)]
pub struct SPRecord {
    pub skill: u32,
    pub timestamp: DateTime<Utc>,
}

fn pack_player(timestamp: NaiveDateTime, p: &mut WolvesvillePlayer, previous_username: Option<String>) {
    p.previous_username = previous_username;
    p.timestamp = Some(DateTime::from_naive_utc_and_offset(timestamp, Utc));
}

pub async fn _get_player_by_id(pool: &SqlitePool, player_id: &str) -> anyhow::Result<Option<WolvesvillePlayer>> {
    let q = r#"
        SELECT * FROM wolvesville_players
        WHERE player_id = $1;
    "#;

    let pq = r#"
        SELECT wpu.username AS previous_username FROM wolvesville_player_usernames wpu
        WHERE wpu.player_id = $1
        ORDER BY wpu.timestamp DESC
        LIMIT 1 OFFSET 1;
    "#;

    let row = query(q).bind(player_id).fetch_optional(pool).await?;
    let pu_row = query(pq).bind(player_id).fetch_optional(pool).await?;

    if row.is_none() {
        return Ok(None);
    }

    let row = row.unwrap();
    let pu_row = pu_row.unwrap();

    let mut deserialized_json = serde_json::from_value::<WolvesvillePlayer>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize player: {}", err))?;

    pack_player(row.get::<NaiveDateTime, _>("timestamp"), &mut deserialized_json, pu_row.get("previous_username"));

    Ok(Some(deserialized_json))
}

pub async fn get_player_by_username(pool: &SqlitePool, username: &str) -> anyhow::Result<Option<WolvesvillePlayer>> {
    let q = r#"
        SELECT wp.* FROM wolvesville_player_usernames wpu
        JOIN wolvesville_players wp ON wp.id = wpu.player_id
        WHERE wpu.username = $1 AND wpu.timestamp = (
            SELECT MAX(wpu.timestamp) FROM wolvesville_player_usernames wpu
            WHERE wpu.player_id = wp.id
        )
        ORDER BY wpu.timestamp DESC
        LIMIT 1;
    "#;

    let row = query(q).bind(username).fetch_optional(pool).await?;

    if row.is_none() {
        return Ok(None);
    }

    let row = row.unwrap();

    let mut deserialized_json = serde_json::from_value::<WolvesvillePlayer>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize player: {}", err))?;

    debug!("Got player by username: {}", username);

    let previous_username_q = r#"
        SELECT wpu.username AS previous_username FROM wolvesville_player_usernames wpu
        WHERE wpu.player_id = $1 AND wpu.timestamp < (
            SELECT MAX(wpu.timestamp) FROM wolvesville_player_usernames wpu
            WHERE wpu.player_id = $1
        )
        ORDER BY wpu.timestamp DESC
        LIMIT 1 OFFSET 1;
    "#;

    let pu_row = query(previous_username_q).bind(&deserialized_json.id).fetch_optional(pool).await?;

    pack_player(row.get::<NaiveDateTime, _>("timestamp"), &mut deserialized_json, pu_row.map(|r| r.get("previous_username")));

    Ok(Some(deserialized_json))
}

pub async fn get_player_by_previous_username(pool: &SqlitePool, previous_username: &str) -> anyhow::Result<Option<WolvesvillePlayer>> {
    let q = r#"
        SELECT wp.* FROM wolvesville_player_usernames wpu
        JOIN wolvesville_players wp ON wp.id = wpu.player_id
        WHERE wpu.username = $1
        ORDER BY wpu.timestamp DESC
        LIMIT 1;
    "#;

    let row = query(q).bind(previous_username).fetch_optional(pool).await?;

    if row.is_none() {
        return Ok(None);
    }

    let row = row.unwrap();

    let mut deserialized_json = serde_json::from_value::<WolvesvillePlayer>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize player: {}", err))?;
    debug!("Got player by previous username: {}", previous_username);

    pack_player(row.get::<NaiveDateTime, _>("timestamp"), &mut deserialized_json, Some(previous_username.to_string()));

    Ok(Some(deserialized_json))
}

pub async fn insert_or_update_full_player(pool: &SqlitePool, player: &WolvesvillePlayer) -> anyhow::Result<()> {
    let mut transaction: Transaction<'_, Sqlite> = pool.begin().await?;

    let sql_wp = r#"
        INSERT INTO wolvesville_players (id, personal_message, json)
        VALUES ($1, $2, $3)
        ON CONFLICT(id) DO UPDATE SET
            personal_message = $2,
            json = $3,
            timestamp = CURRENT_TIMESTAMP;
    "#;

    query(sql_wp)
        .bind(&player.id)
        .bind(&player.personal_message)
        .bind(serde_json::to_value(player).unwrap())
        .execute(&mut *transaction)
        .await?;

    let sql_wpu = r#"
        INSERT INTO wolvesville_player_usernames (player_id, username)
        VALUES ($1, $2)
        ON CONFLICT(player_id, username) DO UPDATE SET
            timestamp = CURRENT_TIMESTAMP;
    "#;

    query(sql_wpu)
        .bind(&player.id)
        .bind(&player.username)
        .execute(&mut *transaction)
        .await?;

    if let Some(sp) = &player.ranked_season_skill {
        let sql_wpr = r#"
            INSERT INTO wolvesville_player_ranked_skill (player_id, skill)
            VALUES ($1, $2);
        "#;

        query(sql_wpr)
            .bind(&player.id)
            .bind(sp)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    info!("Player {} inserted or updated", player.username);
    Ok(())
}

pub async fn get_all_sp_records_of_player_for_last_n_days(pool: &SqlitePool, player_id: &String, days: i64) -> anyhow::Result<Vec<SPRecord>> {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::days(days);

    let q = r#"
        SELECT skill, timestamp FROM wolvesville_player_ranked_skill
        WHERE player_id = $1 AND timestamp >= datetime($2)
        ORDER BY timestamp DESC;
    "#;

    let rows = query(q).bind(player_id).bind(cutoff).fetch_all(pool).await?;
    let mut records = Vec::new();

    let mut iterator = rows.iter();
    while let Some(row) = iterator.next() {
        records.push(SPRecord {
            skill: row.get("skill"),
            timestamp: row.get("timestamp"),
        });
    }

    debug!("Got {} records for player {}", records.len(), player_id);

    Ok(records)
}
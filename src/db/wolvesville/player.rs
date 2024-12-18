use chrono::{DateTime, Utc};
use sqlx::{query, Row, SqlitePool};
use logfather::info;
use crate::utils::apicallers::wolvesville::models::WolvesvillePlayer;


pub async fn get_player_by_id(pool: &SqlitePool, player_id: &str) -> anyhow::Result<WolvesvillePlayer> {
    let q = r#"
        SELECT wp.*, wppu.username FROM wolvesville_players wp
        JOIN wolvesville_players_previous_usernames wppu ON wp.player_id = wppu.player_id
        WHERE wp.player_id = $1;
    "#;

    let row = query(q).bind(player_id).fetch_one(pool).await?;

    let mut deserialized_json = serde_json::from_value::<WolvesvillePlayer>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize player: {}", err))?;

    deserialized_json.id = row.get("player_id");
    deserialized_json.username = row.get("username");
    deserialized_json.personal_message = row.try_get("personal_message").ok();
    deserialized_json.clan_id = row.try_get("clan_id").ok();
    deserialized_json.timestamp = DateTime::<Utc>::from_timestamp(row.get("timestamp"), 0);

    Ok(deserialized_json)
}

pub async fn insert_or_update_full_player(pool: &SqlitePool, player: &WolvesvillePlayer) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO wolvesville_players (id, username, personal_message, clan_id, json, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT(id) DO UPDATE SET
            username = $2,
            personal_message = $3,
            clan_id = $4,
            json = $5,
            timestamp = $6;
    "#;

    query(q)
        .bind(&player.id)
        .bind(&player.username)
        .bind(&player.personal_message)
        .bind(&player.clan_id)
        .bind(serde_json::to_value(player).unwrap())
        .bind(player.timestamp.unwrap_or(Utc::now()).timestamp())
        .execute(pool)
        .await?;
    info!("Player {} inserted or updated", player.username);
    Ok(())
}
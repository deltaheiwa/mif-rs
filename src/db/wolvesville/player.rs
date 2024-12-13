use chrono::{DateTime, Utc};
use sqlx::{query, Row, SqlitePool};
use logfather::info;
use crate::utils::apicallers::wolvesville::models::WolvesvillePlayer;


pub async fn get_player_by_id(pool: &SqlitePool, player_id: &str) -> anyhow::Result<WolvesvillePlayer> {
    let q = r#"
        SELECT wolvesville_players.*, wolvesville FROM wolvesville_players WHERE player_id = $1;
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


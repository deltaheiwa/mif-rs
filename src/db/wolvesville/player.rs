use chrono::{DateTime, Utc};
use sqlx::{query, Row, Sqlite, SqlitePool, Transaction};
use logfather::info;
use crate::utils::apicallers::wolvesville::models::WolvesvillePlayer;


pub async fn get_player_by_id(pool: &SqlitePool, player_id: &str) -> anyhow::Result<WolvesvillePlayer> {
    let q = r#"
        SELECT wp.*, wppu.username AS previous_username FROM wolvesville_players wp
        JOIN wolvesville_players_previous_usernames wppu ON wp.player_id = wppu.player_id
        WHERE wp.player_id = $1
        ORDER BY wppu.timestamp DESC;
    "#;

    let row = query(q).bind(player_id).fetch_one(pool).await?;

    let mut deserialized_json = serde_json::from_value::<WolvesvillePlayer>(row.get("json"))
        .map_err(|err| anyhow::anyhow!("Failed to deserialize player: {}", err))?;

    deserialized_json.previous_username = row.get("previous_username");

    Ok(deserialized_json)
}

pub async fn insert_or_update_full_player(pool: &SqlitePool, player: &WolvesvillePlayer) -> anyhow::Result<()> {
    let mut transaction: Transaction<'_, Sqlite> = pool.begin().await?;

    let sql_wp = r#"
        INSERT INTO wolvesville_players (id, username, personal_message, json)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            username = $2,
            personal_message = $3,
            json = $4;
    "#;

    query(sql_wp)
        .bind(&player.id)
        .bind(&player.username)
        .bind(&player.personal_message)
        .bind(serde_json::to_value(player).unwrap())
        .execute(&mut *transaction)
        .await?;

    if let Some(previous_username) = &player.previous_username {
        let sql_wppu = r#"
            INSERT INTO wolvesville_players_previous_usernames (player_id, username, timestamp)
            VALUES ($1, $2, $3);
        "#;

        query(sql_wppu)
            .bind(&player.id)
            .bind(previous_username)
            .execute(&mut *transaction)
            .await?;
    }

    if let Some(sp) = &player.ranked_season_skill {
        let sql_wpr = r#"
            INSERT INTO wolvesville_players_ranked_skill (player_id, skill)
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
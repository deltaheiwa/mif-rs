use std::sync::Arc;
use reqwest::Client;

use tokio::test;
use dotenvy::dotenv;
use super::*;


fn setup() -> Arc<Client> {
    dotenv().ok();
    wolvesville::initialize_client()
}

#[test]
async fn test_get_wolvesville_player_by_id() {
    let client = setup();
    let player = wolvesville::get_wolvesville_player_by_id(&client, "1939b906-1d10-435c-806c-370b657fc2e7").await;
    assert!(player.is_ok());
    let player: serde_json::Value = match player {
        Ok(player) => player,
        Err(_) => panic!("Failed to get player")
        
    };

    assert_eq!(player["id"], "1939b906-1d10-435c-806c-370b657fc2e7");
}

#[test]
async fn test_get_wolvesville_player_by_id_invalid() {
    let client = setup();
    let player = wolvesville::get_wolvesville_player_by_id(&client, "non-existant-id").await;
    
    let player: serde_json::Value = match player {
        Ok(player) => player,
        Err(_) => panic!("Failed to get player")
    };

    assert_eq!(player["code"], 404);
}
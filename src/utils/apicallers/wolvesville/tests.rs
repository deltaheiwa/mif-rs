use std::sync::Arc;
use reqwest::Client;

use tokio::test;
use dotenvy::dotenv;
use crate::utils::apicallers::*;


fn setup() -> Arc<Client> {
    dotenv().ok();
    wolvesville::initialize_client()
}

#[test]
async fn test_get_wolvesville_player_by_id() {
    let client = setup();
    let player = wolvesville::get_wolvesville_player_by_id(&client, "1939b906-1d10-435c-806c-370b657fc2e7").await;
    assert!(player.is_ok());
    let player = player.unwrap();
    assert!(player.is_some());
    let player_value = player.unwrap();

    assert_eq!(player_value.id, "1939b906-1d10-435c-806c-370b657fc2e7");
}

#[test]
async fn test_get_wolvesville_player_by_id_invalid() {
    let client = setup();
    let player = wolvesville::get_wolvesville_player_by_id(&client, "non-existent-id").await;
    assert!(player.is_ok());
    assert!(player.unwrap().is_none());
}

#[test]
async fn test_get_wolvesville_player_by_username() {
    let client = setup();
    let username = "Username";
    let player = wolvesville::get_wolvesville_player_by_username(&client, username).await;
    assert!(player.is_ok());
    let player = player.unwrap();
    assert!(player.is_some());
    let player_value = player.unwrap();

    assert_eq!(player_value.username, username);
}

#[test]
async fn test_get_wolvesville_clan_by_name_no_clans() {
    let client = setup();
    let clan_name = "non-existent-clan-FFFFFF";
    let clans = wolvesville::get_wolvesville_clan_info_by_name(&client, clan_name).await;
    assert!(clans.is_ok());
    let clans = clans.unwrap();
    assert!(clans.is_none());
}

#[test]
async fn test_get_wolvesville_clan_by_name_with_whitespace() {
    let client = setup();
    let clan_name = "Test Clan";
    let clans = wolvesville::get_wolvesville_clan_info_by_name(&client, clan_name).await;
    assert!(clans.is_ok());
    let clans = clans.unwrap();
    assert!(clans.is_some());
}
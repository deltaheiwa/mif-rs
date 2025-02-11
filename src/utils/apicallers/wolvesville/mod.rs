use std::sync::Arc;
use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION}, Client};
use crate::bot::core::structs::ApiResult;
use crate::bot::core::constants::WOLVESVILLE_API_URL;
use models::{WolvesvilleClan, WolvesvillePlayer, WolvesvilleClanMember};

#[cfg(test)]
mod tests;
pub mod models;


pub fn initialize_client() -> Arc<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bot {}", std::env::var("WOV_API_TOKEN").unwrap()).as_str()).unwrap());

    Arc::new(Client::builder().default_headers(headers).build().unwrap())
}


pub async fn get_wolvesville_player_by_id(client: &Arc<Client>, player_id: &str) -> ApiResult<WolvesvillePlayer> {
    let url = format!("{}/players/{}", WOLVESVILLE_API_URL, player_id);
    let response = client
        .get(&url)
        .send()
        .await?;
    if response.status().as_u16() == 404 { return Ok(None); }
    let json = response.json::<WolvesvillePlayer>().await?;
    Ok(Some(json))
}

pub async fn get_wolvesville_player_by_username(client: &Arc<Client>, username: &str) -> ApiResult<WolvesvillePlayer> {
    let url = format!("{}/players/search?username={}", WOLVESVILLE_API_URL, username);
    let response = client
        .get(&url)
        .send()
        .await?;
    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    let json = response.json::<WolvesvillePlayer>().await?;
    Ok(Some(json))
}

pub async fn get_wolvesville_clan_info_by_id(client: &Arc<Client>, clan_id: &str) -> ApiResult<WolvesvilleClan> {
    let url = format!("{}/clans/{}/info", WOLVESVILLE_API_URL, clan_id);
    let response = client
        .get(&url)
        .send()
        .await?;
    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    let json = response.json::<WolvesvilleClan>().await?;
    Ok(Some(json))
}

pub async fn get_wolvesville_clan_info_by_name(client: &Arc<Client>, clan_name: &str) -> ApiResult<Vec<WolvesvilleClan>> {
    let url = format!("{}/clans/search?name={}", WOLVESVILLE_API_URL, clan_name);
    let response = client
        .get(&url)
        .send()
        .await?;
    if response.status().as_u16() == 404 { return Ok(None); }
    let json = response.json::<Vec<WolvesvilleClan>>().await?;
    if json.is_empty() { return Ok(None); }
    Ok(Some(json))
}

pub async fn get_wolvesville_clan_members_by_id(client: &Arc<Client>, clan_id: &str) -> ApiResult<Vec<WolvesvilleClanMember>> {
    let url = format!("{}/clans/{}/members", WOLVESVILLE_API_URL, clan_id);
    let response = client
        .get(&url)
        .send()
        .await?;
    if response.status().as_u16() == 404 { return Ok(None); }
    let json = response.json::<Vec<WolvesvilleClanMember>>().await?;
    Ok(Some(json))
}
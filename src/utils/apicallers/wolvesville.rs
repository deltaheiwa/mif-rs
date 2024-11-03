use std::sync::Arc;
use serde_json::Value;
use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION}, Client, Error};

const WOLVESVILLE_API_URL: &str = "https://api.wolvesville.com";


pub fn initialize_client() -> Arc<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bot {}", std::env::var("WOV_API_TOKEN").unwrap()).as_str()).unwrap());
    
    Arc::new(Client::builder().default_headers(headers).build().unwrap())
}


pub async fn get_wolvesville_player_by_id(client: &Arc<Client>, player_id: &str) -> Result<Value, Error> {
    let url = format!("{}/players/{}", WOLVESVILLE_API_URL, player_id);
    let response = client
        .get(&url)
        .send()
        .await?;
    let json = response.json::<Value>().await?;
    Ok(json)
}
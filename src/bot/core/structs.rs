extern crate lru;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use lru::LruCache;
use poise::serenity_prelude as serenity;
use serenity::prelude::TypeMapKey;

#[derive(Clone)]
pub struct Data {
    pub db_pool: sqlx::SqlitePool,
    pub prefix_cache: Arc<Mutex<LruCache<String, String>>>,
    pub language_cache: Arc<Mutex<LruCache<String, String>>>,
    pub wolvesville_client: Arc<reqwest::Client>,
    pub custom_emojis: HashMap<String, serenity::Emoji>,
}

impl TypeMapKey for Data {
    type Value = Arc<Self>;
}

pub struct CustomColor;

impl CustomColor {
    pub const CYAN: serenity::Color = serenity::Color::from_rgb(0, 255, 255);
}

pub struct CustomEmoji;

impl CustomEmoji {
    pub const SINGLE_ROSE: &'static str = "single_rose";
    pub const LETS_PLAY: &'static str = "lets_play";
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub type ApiResult<T> = Result<Option<T>, reqwest::Error>;
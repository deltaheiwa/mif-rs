extern crate lru;

use std::sync::Arc;
use tokio::sync::Mutex;
use lru::LruCache;
use serenity::prelude::TypeMapKey;

#[derive(Clone)]
pub struct Data {
    pub db_pool: sqlx::SqlitePool,
    pub prefix_cache: Arc<Mutex<LruCache::<String, String>>>,
} 

impl TypeMapKey for Data {
    type Value = Arc<Self>;
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
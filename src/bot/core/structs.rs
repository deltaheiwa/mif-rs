use std::{collections::HashMap, sync::{Arc, Mutex}};

use serenity::prelude::TypeMapKey;

#[derive(Clone)]
pub struct Data {
    pub prefixes_db_pool: sqlx::SqlitePool,
    pub user_language_cache: Arc<Mutex<HashMap<String, String>>>,
} 

impl TypeMapKey for Data {
    type Value = Arc<Self>;
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
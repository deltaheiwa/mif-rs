mod handlers;
mod commands;
pub mod core;
pub mod background;

use poise::serenity_prelude as serenity;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;
use ::serenity::all::ActivityData;
use lru::LruCache;
use logfather::error;
use sqlx::SqlitePool;
use crate::{db::{get_pool, prefixes::get_prefix}, utils::apicallers::wolvesville};
use core::{structs::{Data, Error, PartialContext}, constants::DEFAULT_PREFIX};
use commands::*;
use crate::utils::scheduler::{JobRegistry, Scheduler};

/// This function is used to determine the prefix on a command call for each separate server/user.
/// It first checks the cache, if the prefix is not found in the cache, it queries the database, or the default '.' prefix if it's not found in the database either.
/// Prioritizes user-specific prefixes over guild prefixes, and returns the default prefix if no specific prefix is set.
async fn determine_prefix(ctx: PartialContext<'_>) -> Result<Option<String>, Error> {
    let guild_id = ctx.guild_id.map(|id| id.to_string());
    let user_id = ctx.author.id.to_string();
    
    let mut prefix_cache = ctx.data.prefix_cache.lock().await;
    
    if let Some(user_prefix) = prefix_cache.get(&user_id) {
        return Ok(Some(user_prefix.clone()));
    }

    if let Ok(Some(user_prefix)) = get_prefix(&ctx.data.db_pool, &user_id).await {
        prefix_cache.put(user_id, user_prefix.clone());
        return Ok(Some(user_prefix));
    }
    
    if let Some(guild_id) = guild_id {
        if let Some(guild_prefix) = prefix_cache.get(&guild_id) {
            return Ok(Some(guild_prefix.clone()));
        }

        if let Ok(Some(guild_prefix)) = get_prefix(&ctx.data.db_pool, &guild_id).await {
            prefix_cache.put(guild_id.clone(), guild_prefix.clone());
            return Ok(Some(guild_prefix));
        }
    }

    Ok(Some(DEFAULT_PREFIX.to_string()))
}

pub struct Bot {
    client: serenity::Client,
    job_registry: Arc<JobRegistry>,
    scheduler: Scheduler,
}

impl Bot {
    pub async fn new(token: String) -> Self {
        let (client, pool) = build_client(token).await;
        let job_registry = Arc::new(JobRegistry::new());
        Bot { 
            client: client.expect("Failed to create Serenity client"),
            job_registry: job_registry.clone(),
            scheduler: Scheduler::new(pool, job_registry),
        }
    }

    pub async fn start(&mut self) {
        if let Err(why) = self.client.start().await {
            error!("An error occurred while running the client: {:?}", why);
        }
    }
}

async fn build_client(token: String) -> (Result<serenity::Client, serenity::Error>, Arc<SqlitePool>) {
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT 
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;
    
    let pool = Arc::new(get_pool().await.map_err(|e| {
        error!("Failed to get database pool: {}", e);
        serenity::Error::Other("Failed to get database pool")
    }).expect("Failed to get database pool"));
    
    let pool_return = pool.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                informative::help::help(),
                informative::ping::ping(),
                informative::userinfo::user_info(),
                administrative::prefix(),
                directive::preferences(),
                wov::wolvesville(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(DEFAULT_PREFIX.to_string()),
                dynamic_prefix: Some(|ctx| Box::pin(determine_prefix(ctx))),
                ..poise::PrefixFrameworkOptions::default()
            },
            on_error: |error| {
                Box::pin(handlers::on_error(error))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let data = Data {
                    db_pool: pool.clone(),
                    prefix_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
                    language_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
                    wolvesville_player_refresh_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(20).unwrap()))),
                    wolvesville_client: wolvesville::initialize_client(),
                    custom_emojis: ctx.get_application_emojis().await.unwrap().iter().map(|emoji| (emoji.name.clone(), emoji.clone())).collect(),
                };

                // I also need to insert the data into the context of serenity
                let mut data_lock = ctx.data.write().await;
                data_lock.insert::<Data>(Arc::new(data.clone()));

                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .status(serenity::OnlineStatus::Online)
        .activity(ActivityData::listening("voices in my RAM"))
        .event_handler(handlers::Handler)
        .await;

    (client, pool_return)
}

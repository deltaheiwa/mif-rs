mod handlers;
mod commands;
pub mod core;

use poise::serenity_prelude as serenity;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;
use ::serenity::all::ActivityData;
use lru::LruCache;
use logfather::error;

use crate::{db::{get_pool, prefixes::get_prefix}, utils::apicallers::wolvesville};
use core::structs::{Data, Error, PartialContext};
use commands::*;


const DEFAULT_PREFIX: &str = "m.";

/// This function is used to determine the prefix on a command call for each separate server.
/// It first checks the cache, if the prefix is not found in the cache, it queries the database, or the default '.' prefix if it's not found in the database either.
async fn determine_prefix(ctx: PartialContext<'_>) -> Result<Option<String>, Error> {
    let guild_id = match ctx.guild_id {
        Some(guild_id) => &guild_id.to_string(),
        None => {
            return Ok(Some(String::from(".")));
        },
    };
    let mut prefix_cache = ctx.data.prefix_cache.lock().await;
    let prefix = prefix_cache
        .get(guild_id)
        .unwrap_or(
            &get_prefix(&ctx.data.db_pool, guild_id)
            .await
            .unwrap_or(String::from(".")))
        .clone();

    Ok(Some(prefix))
}

pub struct Bot {
    client: serenity::Client
}

impl Bot {
    pub async fn new(token: std::string::String) -> Self {
        let client = build_client(token).await.expect("Failed to build the client");
        Bot { client }
    }

    pub async fn start(&mut self) {
        if let Err(why) = self.client.start().await {
            error!("An error occurred while running the client: {:?}", why);
        }
    }
}

async fn build_client(token: std::string::String) -> Result<serenity::Client, serenity::Error> {
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT 
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                informative::ping(),
                informative::user_info(),
                administrative::prefix(),
                poise::Command {
                    subcommands: vec![
                        directive::show(),
                        directive::language(),
                    ],
                    ..directive::preferences()
                },
                poise::Command {
                    subcommands: vec![
                        poise::Command {
                            subcommands: vec![
                                wov::player::search(),
                            ],
                            ..wov::player::player()
                        },
                    ],
                    ..wov::wolvesville()
                },
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
                    db_pool: get_pool().await?,
                    prefix_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
                    language_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
                    wolvesville_client: wolvesville::initialize_client(),
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

    client
}

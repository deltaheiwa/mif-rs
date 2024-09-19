mod handlers;
mod commands;
mod core;

use poise::serenity_prelude as serenity;
use ::serenity::all::ActivityData;
use tracing::error;
use crate::db::prefixes::get_prefix;

use core::structs::{Data, Error, PartialContext};
use commands::*;


const DEFAULT_PREFIX: &str = "m.";

async fn determine_prefix(ctx: PartialContext<'_>) -> Result<Option<String>, Error> {
    let prefix = get_prefix(&ctx.guild_id.unwrap().to_string()).await.unwrap_or(String::from("."));

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
        | serenity::GatewayIntents::GUILD_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                informative::ping(),
                administrative::prefix(),
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
                Ok(Data {})
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

pub mod handlers;

use poise::serenity_prelude as serenity;
use tracing::{error, info};
use crate::db::prefixes::get_prefix;


struct Data {} 
pub type Error = Box<dyn std::error::Error + Send + Sync>;
type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DEFAULT_PREFIX: &str = "m.";

async fn determine_prefix(ctx: PartialContext<'_>) -> Result<Option<String>, Error> {
    let prefix = get_prefix(&ctx.guild_id.unwrap().to_string()).await.unwrap_or(String::from("."));

    Ok(Some(prefix))
}

#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let runners = &ctx.framework().shard_manager.runners.lock().await;
    let runner_info = runners.get(&serenity::ShardId(0)).unwrap();

    // Attempt to retrieve latency
    if let Some(latency) = runner_info.latency {
        ctx.reply(format!("Pong! `{}ms`", latency.as_millis())).await?;
    } else {
        // If latency is unavailable (shard just connected, and there was no heartbeat from discord yet)
        ctx.reply("Pong!").await?;
    }
    Ok(())
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
            commands: vec![ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(DEFAULT_PREFIX.to_string()),
                dynamic_prefix: Some(|ctx| Box::pin(determine_prefix(ctx))),
                ..poise::PrefixFrameworkOptions::default()
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
        .event_handler(handlers::Handler)
        .await;

    client
}

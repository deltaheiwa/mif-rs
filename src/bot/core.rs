use poise::serenity_prelude as serenity;

struct Data {} 
pub type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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


pub async fn build_client(token: std::string::String) -> Result<serenity::Client, serenity::Error> {
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT 
        | serenity::GatewayIntents::GUILD_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
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
        .await;

    client
}

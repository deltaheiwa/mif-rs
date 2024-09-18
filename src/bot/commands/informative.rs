use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error};


#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
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
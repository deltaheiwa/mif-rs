use poise::serenity_prelude as serenity;

use crate::bot::core::structs::{Context, Error};
use crate::utils::language::get_language;


/// Pong. Check if the bot is alive
#[poise::command(
    slash_command, prefix_command,
    name_localized("uk", "пінг"),
    description_localized("uk", "Понг. Перевір чи бот живий"),
    category = "informative",
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;
    let runners = &ctx.framework().shard_manager.runners.lock().await;
    let runner_info = runners.get(&serenity::ShardId(0)).unwrap();

    // Attempt to retrieve latency
    if let Some(latency) = runner_info.latency {
        ctx.reply(format!("{}", t!("commands.info.ping.latency", latency = latency.as_millis(), locale = language))).await?;
    } else {
        // If latency is unavailable (shard just connected, and there was no heartbeat from discord yet)
        ctx.reply(format!("{}", t!("commands.info.ping.no_latency", locale = language))).await?;
    }
    Ok(())
}

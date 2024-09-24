use crate::bot::core::structs::{Context, Error};
use crate::db::prefixes;
use logfather::error;

#[poise::command(slash_command, prefix_command)]
pub async fn prefix(ctx: Context<'_>, new_prefix: String) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild_id = guild_id.to_string();

    let mut prefix_cache = ctx.data().prefix_cache.lock().await;

    let result = prefixes::set_prefix(&ctx.data().db_pool, &guild_id, &new_prefix).await;

    match result {
        Ok(_) => {
            ctx.reply(format!("Prefix set to `{}`", new_prefix)).await?;
            prefix_cache.put(guild_id, new_prefix);
        }
        Err(err) => {
            ctx.reply("Failed to set prefix. Please try again later").await?;
            error!("Failed to set prefix `{}` for guild {}: {}", new_prefix, guild_id, err);
        }
    }
    Ok(())
}
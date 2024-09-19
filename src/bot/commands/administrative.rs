use crate::bot::core::structs::{Context, Error};
use crate::db::prefixes;
use tracing::error;

#[poise::command(slash_command, prefix_command)]
pub async fn prefix(ctx: Context<'_>, new_prefix: String) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild_id = guild_id.to_string();
    let result = prefixes::set_prefix(&guild_id, &new_prefix).await;
    match result {
        Ok(_) => {
            ctx.reply(format!("Prefix set to `{}`", new_prefix)).await?;
        }
        Err(err) => {
            ctx.reply("Failed to set prefix. Please try again later").await?;
            error!("Failed to set prefix `{}` for guild {}: {}", new_prefix, guild_id, err);
        }
    }
    Ok(())
}
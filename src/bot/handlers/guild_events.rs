use poise::serenity_prelude as serenity;
use tracing::error;

use crate::db::prefixes;

pub async fn on_guild_join(_ctx: serenity::Context, guild: serenity::Guild) {
    match prefixes::set_prefix(&guild.id.get().to_string(), &".".to_string()).await {
        Ok(_) => {}
        Err(err) => error!("Failed to set default prefix for guild {}: {}", guild.id, err),
    }
}

pub async fn on_guild_remove(_ctx: serenity::Context, guild_id: String) {
    match prefixes::delete_prefix(&guild_id).await {
        Ok(_) => {}
        Err(err) => error!("Failed to delete prefix for guild {}: {}", guild_id, err),
    }
}
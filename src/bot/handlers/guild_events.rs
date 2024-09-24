use poise::serenity_prelude as serenity;
use logfather::error;
use crate::bot::core::structs::Data;

use crate::db::prefixes;

pub async fn on_guild_join(ctx: serenity::Context, guild: serenity::Guild) {
    let data = ctx.data.blocking_read();
    let pool = &data.get::<Data>().unwrap().db_pool;
    let prefix_cache = &data.get::<Data>().unwrap().prefix_cache;

    match prefixes::set_prefix(pool, &guild.id.get().to_string(), &".".to_string()).await {
        Ok(_) => {}
        Err(err) => error!("Failed to set default prefix for guild {}: {}", guild.id, err),
    }

    prefix_cache.lock().await.put(guild.id.get().to_string(), ".".to_string());
}

pub async fn on_guild_remove(ctx: serenity::Context, guild_id: String) {
    let data = ctx.data.blocking_read();
    let pool = &data.get::<Data>().unwrap().db_pool;
    let prefix_cache = &data.get::<Data>().unwrap().prefix_cache;

    match prefixes::delete_prefix(pool, &guild_id).await {
        Ok(_) => {}
        Err(err) => error!("Failed to delete prefix for guild {}: {}", guild_id, err),
    }

    prefix_cache.lock().await.pop(&guild_id);
}
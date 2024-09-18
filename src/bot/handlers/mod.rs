use poise::serenity_prelude as serenity;
use ::serenity::async_trait;


mod guild_events;
mod ready;

pub struct Handler;

#[async_trait]
impl serenity::EventHandler for Handler {
    async fn guild_create(&self, ctx: serenity::Context, guild: serenity::Guild, is_new: Option<bool>) {
        if is_new.unwrap_or(true) {
            guild_events::on_guild_join(ctx, guild).await;
        }
    }

    async fn guild_delete(&self, ctx: serenity::Context, incomplete: serenity::UnavailableGuild, _full: Option<serenity::Guild>) {
        let guild_id = incomplete.id.get().to_string();
        guild_events::on_guild_remove(ctx, guild_id).await;
    }

    async fn ready(&self, ctx: serenity::Context, ready: serenity::Ready) {
        ready::on_ready(ctx, ready).await;
    }
}
use poise::{serenity_prelude as serenity, CreateReply};
use ::serenity::async_trait;
use tracing::error;

use crate::bot::core::structs::{Context, Data, Error};

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


pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!("An error occurred during setup: {:?}", error);
        },
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("An error occurred while running a command: {:?}", error);

            let embed = serenity::CreateEmbed::default()
                .title("Error")
                .description("An error occurred while running the command. It has been automatically reported to the developer.\n\nPlease try again later.")
                .color(serenity::Colour::RED);

            if let Err(err) = ctx.send(CreateReply::default().embed(embed)).await {
                error!("Failed to respond to the command error: {:?}", err);
            };
        },
        error => {
            error!("An error occurred: {:?}", error.to_string());
        }
    }
}
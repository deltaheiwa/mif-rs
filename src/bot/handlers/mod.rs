use poise::{serenity_prelude as serenity, CreateReply};
use logfather::{warn, info, error};
use crate::bot::core::structs::{Data, Error};

mod guild_events;
mod ready;

pub struct Handler;

#[serenity::async_trait]
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

    async fn resume(&self, _ctx: serenity::Context, _event: serenity::ResumedEvent) {
        info!("Resumed");
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
        poise::FrameworkError::MissingUserPermissions { missing_permissions, ctx, .. } => {
            let missing_permissions = missing_permissions
                .unwrap_or(serenity::Permissions::empty())
                .iter_names()
                .map(|(name, _)| name)
                .collect::<Vec<_>>();


            warn!("{} is missing permissions: {:?}", ctx.author().name, missing_permissions.join(", "));

            let embed = serenity::CreateEmbed::default()
                .title("Error")
                .description(format!("You are missing `{:?}` permissions to run this command.", missing_permissions.join(", ")))
                .color(serenity::Colour::RED);

            if let Err(err) = ctx.send(CreateReply::default().embed(embed)).await {
                error!("Failed to respond to the missing permissions error: {:?}", err);
            };
        },
        poise::FrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {
            let missing_permissions = missing_permissions
                .iter_names()
                .map(|(name, _)| name)
                .collect::<Vec<_>>();

            warn!("Bot is missing permissions in guild {}: {:?}", ctx.guild().unwrap().name, missing_permissions.join(", "));

            let embed = serenity::CreateEmbed::default()
                .title("Error")
                .description(format!("I am missing `{:?}` permissions to execute this command.", missing_permissions.join(", ")))
                .color(serenity::Colour::RED);

            if let Err(err) = ctx.send(CreateReply::default().embed(embed)).await {
                error!("Failed to respond to the missing permissions error: {:?}", err);
            };
        },
        error => {
            error!("An error occurred: {:?}", error.to_string());
        }
    }
}
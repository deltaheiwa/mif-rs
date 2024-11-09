use std::fs::File;
use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, Data};
use crate::utils::{language::get_language, apicallers::wolvesville};
use logfather::{debug, error};

async fn on_missing_username_input(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::ArgumentParse { input, ctx, .. } => {
            let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

            if input == None {
                let embed = serenity::CreateEmbed::default()
                    .title(t!("common.error", locale = language))
                    .description(t!("commands.wov.player.search.no_input", locale = language))
                    .color(serenity::Color::RED);
                ctx.send(poise::CreateReply::default().reply(true).embed(embed)).await.unwrap();
            }
        }
        _ => {
            error!("An error occurred while running the player command: {}", error);
        }
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn player(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command, prefix_command,
    on_error = on_missing_username_input
)]
pub async fn search(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let data = ctx.data();
    let language = get_language(data, &ctx.author().id.to_string()).await;

    if username.len() < 3 {
        let embed_too_short = serenity::CreateEmbed::default()
            .title(t!("common.error", locale = language))
            .description(t!("commands.wov.player.search.too_short", username = username, locale = language))
            .color(serenity::Color::RED);
        ctx.send(poise::CreateReply::default().reply(true).embed(embed_too_short)).await.unwrap();
        return Ok(());
    }

    let player = wolvesville::get_wolvesville_player_by_username(&data.wolvesville_client, &username).await;
    let player = match player {
        Ok(player) => player,
        Err(e) => {
            error!("An error occurred while running the `wolvesville player search` command: {:?}", e);
            let embed_error = serenity::CreateEmbed::default()
                .title(t!("common.error", locale = language))
                .description(t!("common.api_error", locale = language))
                .color(serenity::Color::RED);
            ctx.send(poise::CreateReply::default().reply(true).embed(embed_error)).await.unwrap();
            return Ok(());
        }
    };
    let player = match player {
        Some(player) => player,
        None => {
            let embed_not_found = serenity::CreateEmbed::default()
                .title(t!("common.error", locale = language))
                .description(t!("commands.wov.player.search.not_found", username = username, locale = language))
                .color(serenity::Color::RED);
            ctx.send(poise::CreateReply::default().reply(true).embed(embed_not_found)).await.unwrap();
            return Ok(());
        }
    };

    // Converting a string with hex code of the color to u32. If it fails, it will be black (0)
    let color = u32::from_str_radix(player.profile_icon_color.trim_start_matches("#"), 16).unwrap_or(0);

    debug!("{}", color);

    let mut embed = serenity::CreateEmbed::default()
        .title(format!("{}", player.username))
        .description(t!("commands.wov.player.search.description", locale = language))
        .color(serenity::Color::new(color));
        //.thumbnail(player.profile_icon_url);

    ctx.send(poise::CreateReply::default().embed(embed)).await.unwrap();

    Ok(())
}
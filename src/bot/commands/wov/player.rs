use std::fs::File;
use std::path::Path;
use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, Data, CustomEmoji};
use crate::utils::{language::get_language, apicallers::wolvesville};
use logfather::{debug, error};
use chrono::{DateTime, TimeDelta, Utc};
use crate::utils::apicallers::save_to_file;
use crate::utils::time::{get_long_date, get_relative_timestamp};

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

    // debug!("{:?}", player);
    // save_to_file(&player, player.username.as_str());

    // 434 Bytes all private
    // 1.2 kB all private with redundant fields

    // Converting a string with hex code of the color to u32. If it fails, it will be black (0)
    let color = u32::from_str_radix(player.profile_icon_color.trim_start_matches("#"), 16).unwrap_or(0);

    let image = serenity::CreateAttachment::path(Path::new("res/images/wov_logo.png")).await.unwrap();

    let mut embed = serenity::CreateEmbed::default()
        .title(format!("{}", player.username))
        .description(t!("commands.wov.player.search.description", locale = language))
        .color(serenity::Color::new(color))
        .thumbnail("attachment://wov_logo.png"); // Temporary solution until I manage to render player's equipped avatar

    embed = match player.personal_message {
        Some(pm) => embed.field(t!("commands.wov.player.search.personal_message", locale = language), pm, false),
        None => embed
    };

    let level = match player.level {
        Some(level) => level.to_string(),
        None => "?".to_string()
    };
    embed = embed.field(t!("commands.wov.player.search.level", locale = language), level, true);

    embed = embed.field(t!("commands.wov.player.search.online_status", locale = language),
                        t!(format!("commands.wov.player.search.online_status_value.{}", player.status), locale = language), true);

    let last_online = DateTime::parse_from_rfc3339(&player.last_online.unwrap()).unwrap();
    let last_online = match Utc::now() - last_online.with_timezone(&Utc) < TimeDelta::minutes(7) {
        true => "Just now".to_string(),
        false => get_relative_timestamp(&last_online.timestamp())
    };
    
    embed = embed.field(t!("commands.wov.player.search.last_online", locale = language), last_online, true);

    let total_playtime = player.game_stats.total_play_time_in_minutes.unwrap_or_else(|| -1);

    let created_at = if let Some(created_at) = player.creation_time  {
        let created_at = DateTime::parse_from_rfc3339(&created_at).unwrap();
        get_long_date(&created_at.timestamp())
    } else if total_playtime < 0 {
        format!("{}", t!("commands.wov.player.search.created_on.unknown", locale = language))
    } else {
        format!("{}", t!("commands.wov.player.search.created_on.august_3rd_2018", locale = language))
    };

    embed = embed.field(t!("commands.wov.player.search.created_on", locale = language), created_at, true);

    let roses_sent = match player.sent_roses_count {
        Some(-1) => "Private".to_string(),
        Some(roses_sent) => roses_sent.to_string(),
        None => "?".to_string()
    };

    let roses_received = match player.received_roses_count {
        Some(-1) => "Private".to_string(),
        Some(roses_received) => roses_received.to_string(),
        None => "?".to_string()
    };

    let rose_difference = match player.received_roses_count {
        Some(received) => received - player.sent_roses_count.unwrap_or(0),
        None => 0
    };

    embed = embed.field(
        t!("commands.wov.player.search.roses", locale = language),
        t!("commands.wov.player.search.roses.value",
            roses_sent = roses_sent,
            roses_received = roses_received,
            rose_emoji = ctx.data().custom_emojis.get(CustomEmoji::SINGLE_ROSE).unwrap().to_string(),
            rose_difference = rose_difference,
            locale = language),
        true
    );

    // Empty field to make the embed look better
    embed = embed.field("\u{200B}", "\u{200B}", false);

    // If ranked season count is -1, then the ranked data is private overall
    // But if ranked season count is 0, then the data is public but the player hasn't played any ranked games
    // If the ranked season count is 1 or more, but ranked season skill is -1, then the player hasn't played any ranked games in this season



    ctx.send(poise::CreateReply::default().embed(embed).attachment(image)).await.unwrap();
    Ok(())
}
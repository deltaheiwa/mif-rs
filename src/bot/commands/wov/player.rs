use std::io::Cursor;
use std::path::Path;
use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, Data, CustomEmoji};
use crate::utils::{language::get_language, apicallers::wolvesville, math::calculate_percentage, image::wolvesville as wov_image};
use logfather::{debug, error};
use chrono::{DateTime, TimeDelta, Utc};
use tokio::fs::File;
use crate::db;
use crate::utils::apicallers::wolvesville::models::WolvesvillePlayer;
use crate::utils::time::{get_long_date, get_relative_timestamp, pretty_time_delta};

#[allow(unused_imports)]
use crate::utils::apicallers::save_to_file;


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

#[poise::command(slash_command, prefix_command,on_error = on_missing_username_input)]
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
    debug!("Searching for player: {}", username);
    // Check local database for player by searching for username and previous usernames
    let player = match db::wolvesville::player::get_player_by_username(&data.db_pool, &username).await.or_else(|e| { error!("{}", e); Ok::<Option<WolvesvillePlayer>, anyhow::Error>(None) }).and_then(|player| Ok(player)) {
        Ok(Some(db_player)) => { db_player } // If player is found in the database, return it
        _ => {
            // Otherwise, query the API
            debug!("Player not found in the database, querying the API");
            match wolvesville::get_wolvesville_player_by_username(&data.wolvesville_client, &username).await {
                // If the API call is successful, unpack the player
                Ok(api_player) => match api_player {
                    // If the player is found, save to db and return it
                    Some(unpacked) => {
                        debug!("Player found in the API, saving to the database");
                        db::wolvesville::player::insert_or_update_full_player(&data.db_pool, &unpacked).await.map_err(|e| {
                        error!("An error occurred while inserting or updating the player: {:?}", e); e})?;
                        unpacked
                    },
                    // If the player is not found, look for the player by previous username
                    None => {
                        debug!("Player not found in the API, looking for the player by previous username");
                        match db::wolvesville::player::get_player_by_previous_username(&data.db_pool, &username).await.or_else(|e| { error!("{}", e); Ok::<Option<WolvesvillePlayer>, anyhow::Error>(None) }).and_then(|player| Ok(player)) {
                            Ok(Some(db_player)) => { db_player },
                            _ => {
                                debug!("Player not found by previous username in the database, returning an error message");
                                let embed_not_found = serenity::CreateEmbed::default()
                                    .title(t!("common.error", locale = language))
                                    .description(t!("commands.wov.player.search.not_found", username = username, locale = language))
                                    .color(serenity::Color::RED);
                                ctx.send(poise::CreateReply::default().reply(true).embed(embed_not_found)).await.unwrap();
                                return Ok(());
                            }
                        }
                    }
                },
                // If API is down or any other network error occurs, return an error message
                Err(e) => {
                    error!("An error occurred while running the `wolvesville player search` command: {:?}", e);
                    let embed_error = serenity::CreateEmbed::default()
                        .title(t!("common.error", locale = language))
                        .description(t!("common.api_error", locale = language))
                        .color(serenity::Color::RED);
                    ctx.send(poise::CreateReply::default().reply(true).embed(embed_error)).await.unwrap();
                    return Ok(());
                }
            }
        }
    };

    let player = match player.is_outdated() {
        true => {
            debug!("Player outdated, queried the API for updated information");
            match wolvesville::get_wolvesville_player_by_id(&data.wolvesville_client, &player.id).await {
                Ok(api_player) => match api_player {
                    Some(unpacked) => {
                        db::wolvesville::player::insert_or_update_full_player(&data.db_pool, &unpacked).await.map_err(|e| {
                            error!("An error occurred while inserting or updating the player: {:?}", e);
                            e
                        })?;
                        unpacked
                    },
                    None => player
                },
                Err(e) => {
                    error!("An error occurred while updating outdated information in the `wolvesville player search` command at request for the player by username: {:?}", e);
                    player
                }
            }
        },
        false => player
    };


    // debug!("{:?}", player);
    // save_to_file(&player, player.username.as_str());

    // ~434 Bytes all private
    // ~1.2 kB all private with redundant fields

    // Converting a string with hex code of the color to u32. If it fails, it will be black (0)
    let color = u32::from_str_radix(player.profile_icon_color.trim_start_matches("#"), 16).unwrap_or(0);
    let avatar_thumbnail = match player.equipped_avatar {
        Some(avatar) => {
            let rendered_avatar = wov_image::render_wolvesville_avatar(avatar).await?;
            let mut buf = Vec::new();
            rendered_avatar.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png).expect("Failed to convert image to bytes");
            serenity::CreateAttachment::bytes(buf, "avatar.png")
        },
        None => serenity::CreateAttachment::file(&File::open(Path::new("res/images/wov_logo.png")).await.expect("Couldn't find wov_logo.png in res/images"), "avatar.png").await?
    };

    let mut embed = serenity::CreateEmbed::default()
        .title(format!("{}", player.username))
        .description(if player.previous_username.is_none() {t!("commands.wov.player.search.description.no_previous_username", username=player.username, locale = language)}
            else {t!("commands.wov.player.search.description.has_previous_username", username=player.username, previous_username=player.previous_username.unwrap(), locale = language)})
        .color(serenity::Color::new(color))
        .thumbnail("attachment://avatar.png");

    embed = match player.personal_message {
        Some(pm) => if !pm.is_empty() { embed } else { embed.field(t!("commands.wov.player.search.personal_message", locale = language), pm, false) },
        None => embed
    };

    let level = match player.level {
        Some(level) => if level > 0 { level.to_string() } else { "?".to_string() },
        None => "?".to_string()
    };
    embed = embed.field(t!("commands.wov.player.search.level", locale = language), format!("**{}**", level), true);

    let status_emoji = match player.status.as_str() {
        "PLAY" => data.custom_emojis.get(CustomEmoji::LETS_PLAY).unwrap().to_string(),
        "DEFAULT" => "🟢".to_string(),
        "DND" => "🔴".to_string(),
        _ => "⚫".to_string(),
    };

    embed = embed.field(t!("commands.wov.player.search.online_status", locale = language),
                        format!("{} **{}**", status_emoji, t!(format!("commands.wov.player.search.online_status.{}", player.status), locale = language)), true);

    let last_online = DateTime::parse_from_rfc3339(&player.last_online.unwrap()).unwrap();
    let last_online = match Utc::now() - last_online.with_timezone(&Utc) < TimeDelta::minutes(7) {
        true => format!("{}", t!("commands.wov.player.search.last_online.just_now", locale = language)),
        false => get_relative_timestamp(&last_online.timestamp())
    };
    
    embed = embed.field(t!("commands.wov.player.search.last_online", locale = language), last_online, true);

    let created_at = if let Some(created_at) = player.creation_time  {
        let created_at = DateTime::parse_from_rfc3339(&created_at).unwrap();
        get_long_date(&created_at.timestamp())
    } else if player.game_stats.total_play_time_in_minutes < 0 {
        format!("{}", t!("commands.wov.player.search.created_on.private", locale = language))
    } else {
        format!("{}", t!("commands.wov.player.search.created_on.august_3rd_2018", locale = language))
    };

    embed = embed.field(t!("commands.wov.player.search.created_on", locale = language), created_at, true);

    let roses_sent = match player.sent_roses_count {
        Some(-1) | None => "?".to_string(),
        Some(roses_sent) => roses_sent.to_string(),

    };

    let roses_received = match player.received_roses_count {
        Some(-1) | None => "?".to_string(),
        Some(roses_received) => roses_received.to_string(),
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
            rose_emoji = data.custom_emojis.get(CustomEmoji::SINGLE_ROSE).unwrap().to_string(),
            rose_difference = rose_difference,
            locale = language),
        true
    );

    // Empty field to make the embed look better
    embed = embed.field("\u{200B}", "\u{200B}", false);

    // If ranked season count is -1, then the ranked data is private overall
    // But if ranked season count is 0, then the data is public but the player hasn't played any ranked games
    // If the ranked season count is 1 or more, but ranked season skill is -1, then the player hasn't played any ranked games in this season

    let ranked_season_played_count = player.ranked_season_played_count.unwrap_or(-1);

    if ranked_season_played_count == -1 {
        embed = embed.field(t!("commands.wov.player.search.ranked_stats", locale = language), t!("commands.wov.player.search.ranked_stats.private", locale = language), true);
    } else if ranked_season_played_count == 0 {
        embed = embed.field(t!("commands.wov.player.search.ranked_stats", locale = language), t!("commands.wov.player.search.ranked_stats.no_games", locale = language), true);
    } else {
        // No games played so skill resets to 1500 (data is not private)
        let ranked_season_skill = match player.ranked_season_skill.unwrap_or(-1) {
            -1 => 1500,
            skill => skill
        };
        let ranked_season_max_skill = player.ranked_season_max_skill.unwrap_or(-1);
        let ranked_season_best_rank = player.ranked_season_best_rank.unwrap_or(-1);

        embed = embed.field(
            t!("commands.wov.player.search.ranked_stats", locale = language),
            t!("commands.wov.player.search.ranked_stats.value",
                skill = ranked_season_skill,
                max_skill = ranked_season_max_skill,
                best_rank = ranked_season_best_rank,
                seasons_played = ranked_season_played_count,
                locale = language),
            true
        );
    }

    if player.game_stats.total_win_count < 0 {
        embed = embed.field(
            t!("commands.wov.player.search.general_stats", locale = language),
            t!("commands.wov.player.search.private", locale = language),
            true
        );
    } else {
        let total_amount_of_games =
            player.game_stats.total_win_count +
            player.game_stats.total_lose_count +
            player.game_stats.total_tie_count +
            player.game_stats.exit_game_by_suicide_count;

        let total_playtime = match player.game_stats.total_play_time_in_minutes {
            -1 => format!("{}", t!("commands.wov.player.search.private", locale = language)),
            minutes => pretty_time_delta(&TimeDelta::minutes(minutes as i64))
        };

        embed = embed.field(
            t!("commands.wov.player.search.general_stats", locale = language),
            t!(
                "commands.wov.player.search.general_stats.value",
                total_games = total_amount_of_games,
                total_wins = player.game_stats.total_win_count,
                win_percentage = format!("{:.2}", calculate_percentage(player.game_stats.total_win_count, total_amount_of_games)),
                total_losses = player.game_stats.total_lose_count,
                lose_percentage = format!("{:.2}", calculate_percentage(player.game_stats.total_lose_count, total_amount_of_games)),
                total_ties = player.game_stats.total_tie_count,
                tie_percentage = format!("{:.2}", calculate_percentage(player.game_stats.total_tie_count, total_amount_of_games)),
                total_flees = player.game_stats.exit_game_by_suicide_count,
                flee_percentage = format!("{:.2}", calculate_percentage(player.game_stats.exit_game_by_suicide_count, total_amount_of_games)),
                total_playtime = total_playtime,
                locale = language
            ),
            true
        );
    }

    if player.game_stats.village_win_count < 0 {
        embed = embed.field(
            t!("commands.wov.player.search.team_stats", locale = language),
            t!("commands.wov.player.search.private", locale = language),
            false
        );
    } else {
        embed = embed.field(
            t!("commands.wov.player.search.team_stats", locale = language),
            t!(
                "commands.wov.player.search.team_stats.value",
                village_wins = player.game_stats.village_win_count,
                village_losses = player.game_stats.village_lose_count,
                village_wr = format!("{:.2}", calculate_percentage(player.game_stats.village_win_count, player.game_stats.village_win_count + player.game_stats.village_lose_count)),
                werewolf_wins = player.game_stats.werewolf_win_count,
                werewolf_losses = player.game_stats.werewolf_lose_count,
                werewolf_wr = format!("{:.2}", calculate_percentage(player.game_stats.werewolf_win_count, player.game_stats.werewolf_win_count + player.game_stats.werewolf_lose_count)),
                voting_wins = player.game_stats.voting_win_count,
                voting_losses = player.game_stats.voting_lose_count,
                voting_wr = format!("{:.2}", calculate_percentage(player.game_stats.voting_win_count, player.game_stats.voting_win_count + player.game_stats.voting_lose_count)),
                solo_wins = player.game_stats.solo_win_count,
                solo_losses = player.game_stats.solo_lose_count,
                solo_wr = format!("{:.2}", calculate_percentage(player.game_stats.solo_win_count, player.game_stats.solo_win_count + player.game_stats.solo_lose_count)),
                locale = language
            ),
            false
        );
    }

    match player.clan_id {
        Some(clan_id) => {
            let clan_info = wolvesville::get_wolvesville_clan_info_by_id(&data.wolvesville_client, &clan_id).await;
            let clan_info = clan_info.unwrap_or_else(|e| {
                error!("An error occurred while running the `wolvesville player search` command at request for the clan: {:?}", e);
                None
            });
            match clan_info {
                Some(clan_info) => {
                    let clan_description: String = match clan_info.description {
                        Some(description) => {
                            match description.split_once('\n') {
                                Some((first_line, _)) => format!("{}***...***", first_line),
                                None => description
                            }
                        },
                        None => format!("{}", t!("commands.wov.player.search.no_description", locale = language))
                    };
                    embed = embed.field(
                        t!("commands.wov.player.search.clan", locale = language),
                        t!(
                            "commands.wov.player.search.clan.value",
                            clan_tag = clan_info.tag.unwrap_or(" ".to_string()), locale = language,
                            clan_name = clan_info.name,
                            clan_language = clan_info.language.to_lowercase(),
                            clan_description = clan_description
                            ),
                        false
                    );
                },
                None => {
                    // This may not trigger, because clans themselves can't be private, but this *might* change in the future, so consider it a safety measure
                    embed = embed.field(t!("commands.wov.player.search.clan", locale = language), t!("commands.wov.player.search.clan.hidden", locale = language), false);
                }
            }
        },
        None => {
            embed = embed.field(t!("commands.wov.player.search.clan", locale = language), t!("commands.wov.player.search.no_clan", locale = language), false);
        }
    }

    ctx.send(poise::CreateReply::default().attachment(avatar_thumbnail).embed(embed)).await.unwrap();
    Ok(())
}
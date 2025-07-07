use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use poise::{serenity_prelude as serenity, CreateReply};
use crate::bot::core::structs::{Context, Error, Data, CustomEmoji, CustomColor};
use crate::utils::{language::get_language, apicallers::wolvesville, math::calculate_percentage, image::wolvesville as wov_image};
use logfather::{debug, info, error};
use chrono::{DateTime, TimeDelta, Utc};
use image::{DynamicImage, ImageFormat};
use tokio::fs::File;
use crate::db;
use crate::utils::apicallers::wolvesville::models::{Avatar, Refreshable, WolvesvillePlayer};
use crate::utils::time::{get_long_date, get_relative_timestamp, pretty_time_delta};

#[allow(unused_imports)]
use crate::utils::apicallers::save_to_file;


async fn on_missing_username_input(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::ArgumentParse { input, ctx, .. } => {
            let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

            if input.is_none() {
                let embed = serenity::CreateEmbed::default()
                    .title(t!("common.error", locale = language))
                    .description(t!("commands.wov.player.search.no_input", locale = language))
                    .color(serenity::Color::RED);
                ctx.send(CreateReply::default().reply(true).embed(embed)).await.unwrap();
            }
        }
        _ => {
            error!("An error occurred while running the player command: {}", error);
        }
    }
}

#[poise::command(
    slash_command, prefix_command,
    name_localized("uk", "гравець"),
    subcommands("search"),
    subcommand_required = true,
)]
pub async fn player(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Search for a Wolvesville player by their username.
#[poise::command(
    slash_command, prefix_command,
    on_error = on_missing_username_input,
    name_localized("uk", "пошук"),
    description_localized("uk", "Знайдіть гравця Wolvesville за їхнім ім'ям.")
)]
pub async fn search(
    ctx: Context<'_>, 
    #[name_localized("uk", "імя_користувача")] username: String
) -> Result<(), Error> {
    let data = ctx.data();
    let ctx_id = ctx.id();
    let language = Arc::new(get_language(data, &ctx.author().id.to_string()).await);

    if username.len() < 3 {
        let embed_too_short = serenity::CreateEmbed::default()
            .title(t!("common.error", locale = language))
            .description(t!("commands.wov.player.search.too_short", username = username, locale = language))
            .color(serenity::Color::RED);
        ctx.send(CreateReply::default().reply(true).embed(embed_too_short)).await.unwrap();
        return Ok(());
    }
    info!("Searching for player: {}", username);
    // Check local database for player by searching for username and previous usernames
    let mut player = match db::wolvesville::player::get_player_by_username(&data.db_pool, &username).await.or_else(|e| { error!("{}", e); Ok::<Option<WolvesvillePlayer>, anyhow::Error>(None) }).and_then(|player| Ok(player)) {
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
                        db::wolvesville::player::upsert_full_player(&data.db_pool, &unpacked).await.map_err(|e| {
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
                                ctx.send(CreateReply::default().reply(true).embed(embed_not_found)).await.unwrap();
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
                    ctx.send(CreateReply::default().reply(true).embed(embed_error)).await.unwrap();
                    return Ok(());
                }
            }
        }
    };

    player = match player.is_outdated() {
        true => {
            debug!("Player outdated, queried the API for updated information");
            match wolvesville::get_wolvesville_player_by_id(&data.wolvesville_client, &player.id).await {
                Ok(api_player) => match api_player {
                    Some(unpacked) => {
                        db::wolvesville::player::upsert_full_player(&data.db_pool, &unpacked).await.map_err(|e| {
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
    // ~1.2 kB all private with redundant

    let avatar_thumbnail = get_thumbnail_attachment(player.avatars.clone().unwrap().into_iter().next(), player.level).await;

    let embed = construct_player_embed(data, &language, &mut player, avatar_thumbnail.filename.as_str()).await;

    let button_components = get_player_search_buttons(ctx_id, false, false, false, &language);

    let loading_emoji = data.custom_emojis.get(CustomEmoji::LOADING).unwrap().to_string();

    let create_reply = CreateReply::default()
        .attachment(avatar_thumbnail.clone())
        .embed(embed).components(vec![button_components]);

    let message = ctx.send(create_reply.clone()).await.unwrap();

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(&ctx)
        .filter(move |press| press.data.custom_id.starts_with(ctx_id.to_string().as_str()))
        .timeout(std::time::Duration::from_secs(600)) // Timeout after 10 minutes
        .await
    {
        match press.data.custom_id.as_str() {
            id if id.ends_with(".avatars") => {
                let embed = serenity::CreateEmbed::default()
                    .title(t!("commands.wov.player.search.avatars.all_avatars", locale = &language))
                    .description(t!("commands.wov.player.search.avatars.rendering",
                            loading_emoji = loading_emoji,
                            locale = &language
                        ))
                    .color(CustomColor::CYAN);

                press.create_response(
                    ctx.http(),
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::default()
                            .embed(embed.clone())
                    )
                ).await?;

                let ordered_avatars: Vec<Avatar> = player.avatars.clone().unwrap().into_iter().collect();

                if ordered_avatars.is_empty() {
                    let embed_error_no_avatars = serenity::CreateEmbed::default()
                        .title(t!("common.error", locale = &language))
                        .description(t!("commands.wov.player.search.avatars.no_avatars", locale = &language))
                        .color(serenity::Color::RED);
                    press.edit_followup(
                        &ctx.http(),
                        press.get_response(&ctx.http()).await.unwrap().id,
                        serenity::CreateInteractionResponseFollowup::new()
                            .embed(embed_error_no_avatars)
                    ).await.unwrap();
                    continue;
                }

                let unique_avatars: Vec<Avatar> = player.avatars.clone().unwrap()
                    .into_iter()
                    .collect::<HashSet<_>>() // Deduplicate the avatars
                    .into_iter()
                    .collect();

                let gathered_futures: Vec<_> = unique_avatars
                    .into_iter()
                    .map(|avatar| wov_image::render_wolvesville_avatar(avatar, None))
                    .collect();

                let avatar_images_mapped: HashMap<String, DynamicImage> = futures::future::join_all(gathered_futures).await
                    .into_iter().filter_map(|image| image.ok()).map(|image| (image.0, image.1)).collect();
                let all_avatars_image: DynamicImage = wov_image::render_all_wolvesville_avatars(
                    &ordered_avatars.iter().map(|avatar| avatar.url.clone()).collect(), &avatar_images_mapped).await?;

                // Fix the ordering according to ordered_avatars
                let mut avatar_images: VecDeque<DynamicImage> = VecDeque::new();
                for avatar in ordered_avatars.iter() {
                    avatar_images.push_back(avatar_images_mapped.get(avatar.url.as_str()).unwrap().clone());
                }

                avatar_images.push_front(all_avatars_image);


                let mut attachments: Vec<serenity::CreateAttachment> = Vec::new();
                for (index, avatar_image) in avatar_images.iter().enumerate() {
                    let mut buf = Vec::new();
                    avatar_image.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).expect("Failed to convert image to bytes");
                    attachments.push(serenity::CreateAttachment::bytes(buf, format!("avatar_{}.png", index)));
                }

                let select_menu = serenity::CreateActionRow::SelectMenu(
                    serenity::CreateSelectMenu::new(
                        format!("{}.avatars.select", press.id.to_string()),
                        serenity::CreateSelectMenuKind::String {
                            options: avatar_images.iter().enumerate().map(|(index, _)| {
                                serenity::CreateSelectMenuOption::new(
                                    if index != 0 { t!("commands.wov.player.search.avatars.select_option", index = index, locale = &language)}
                                    else { t!("commands.wov.player.search.avatars.all_avatars", locale = &language) },
                                    index.to_string()
                                )
                            }).collect()
                        })
                        .placeholder(t!("commands.wov.player.search.avatars.select_placeholder", locale = &language))
                );

                press.edit_followup(
                    &ctx.http(),
                    press.get_response(&ctx.http()).await.unwrap().id,
                    serenity::CreateInteractionResponseFollowup::new()
                        .embed(embed.clone().description("").image(format!("attachment://avatar_{}.png", 0)))
                        .add_file(attachments.get(0).unwrap().clone())
                        .components(vec![select_menu])
                ).await.unwrap();

                let shard = ctx.serenity_context().shard.clone();
                let http_cache = ctx.serenity_context().http.clone();
                let language_inner = language.clone();
                // This needs to be in a separate thread because the select menu is blocking
                tokio::spawn(async move {
                    let mut current_page;


                    while let Some(select_press) = serenity::collector::ComponentInteractionCollector::new(&shard)
                        .filter(move |select_press| select_press.data.custom_id.starts_with(&press.id.to_string()))
                        .timeout(std::time::Duration::from_secs(600)) // Timeout after 10 minutes
                        .await
                    {
                        let selected_index = match &select_press.data.kind {
                            serenity::ComponentInteractionDataKind::StringSelect { values, .. } => values[0].parse::<usize>().unwrap_or(0),
                            _ => 0
                        };
                        current_page = selected_index;

                        select_press.create_response(
                            &http_cache,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::default()
                                    .embed(serenity::CreateEmbed::default()
                                        .title(if selected_index != 0 { t!("commands.wov.player.search.avatars.select_option", index = selected_index, locale = &language_inner) }
                                            else { t!("commands.wov.player.search.avatars.all_avatars", locale = &language_inner) })
                                        .description("")
                                        .color(CustomColor::CYAN)
                                        .image(format!("attachment://avatar_{}.png", selected_index))
                                    )
                                    .add_file(attachments.get(current_page).unwrap().clone())
                            )
                        ).await.unwrap();
                    }
                });
            },
            id if id.ends_with(".sp_plot") => {
                let data = db::wolvesville::player::get_all_sp_records_of_player_for_last_n_days(&data.db_pool, &player.id, 30).await.map_err(|e| {
                    error!("An error occurred while running the `wolvesville player search` command at request for the SP plot: {:?}", e);
                    e
                })?;
                if data.len() < 3 {
                    let embed_error_not_enough_data = serenity::CreateEmbed::default()
                        .title(t!("common.error", locale = language))
                        .description(t!("commands.wov.player.search.buttons.sp_plot.not_enough_data", locale = language))
                        .color(serenity::Color::RED);
                    press.create_response(
                        ctx.http(),
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::default()
                                .embed(embed_error_not_enough_data)
                        )
                    ).await.unwrap();
                } else {
                    match wov_image::draw_sp_plot(&data, &player.username, &language) {
                        Ok(plot) => {
                            let mut buf = Vec::new();
                            plot.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).expect("Failed to convert image to bytes");
                            let attachment = serenity::CreateAttachment::bytes(buf, "sp_plot.png");

                            press.create_response(
                                ctx.http(),
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::default()
                                        .add_file(attachment)
                                )
                            ).await.unwrap();
                        },
                        Err(e) => {
                            error!("An error occurred while drawing the SP plot: {:?}", e);
                            let embed_error = serenity::CreateEmbed::default()
                                .title(t!("common.error", locale = language))
                                .description(t!("common.api_error", locale = language))
                                .color(serenity::Color::RED);
                            press.create_response(
                                ctx.http(),
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::default()
                                        .embed(embed_error)
                                )
                            ).await.unwrap();
                            return Ok(());
                        }
                    }
                }
            },
            id if id.ends_with(".refresh") => {
                let player_log_timestamp = player.timestamp.unwrap_or(Utc::now());
                let time_difference = Utc::now() - player_log_timestamp;
                if time_difference < TimeDelta::minutes(30) {
                    // Disable the button
                    let button_components = get_player_search_buttons(ctx_id, false, false, true, &language);

                    press.create_response(
                        ctx.http(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::default()
                                .components(vec![button_components])
                                .add_file(avatar_thumbnail.clone())
                        )
                    ).await.unwrap();

                    press.create_followup(
                        ctx.http(),
                        serenity::CreateInteractionResponseFollowup::new()
                            .content(
                                t!(
                                    "commands.wov.common.buttons.refresh.too_frequent",
                                    interval = 30,
                                    time_left = get_relative_timestamp(&(Utc::now()+(TimeDelta::minutes(30)-time_difference)).timestamp()),
                                    locale = language
                                ))
                            .ephemeral(true)
                    ).await.unwrap();
                } else {
                    match wolvesville::get_wolvesville_player_by_id(&data.wolvesville_client, &player.id).await {
                        Ok(Some(mut unpacked)) => {
                            db::wolvesville::player::upsert_full_player(&data.db_pool, &unpacked).await.map_err(|e| {
                                error!("An error occurred while inserting or updating the player on refresh: {:?}", e);
                                e
                            })?;
                            let avatar_thumbnail = get_thumbnail_attachment(unpacked.avatars.clone().unwrap().into_iter().next(), unpacked.level).await;
                            let embed = construct_player_embed(data, &language, &mut unpacked, avatar_thumbnail.filename.as_str()).await;
                            let button_components = get_player_search_buttons(ctx_id, false, false, false, &language);
                            press.create_response(
                                ctx.http(),
                                serenity::CreateInteractionResponse::UpdateMessage(
                                    serenity::CreateInteractionResponseMessage::default()
                                        .add_file(avatar_thumbnail)
                                        .embed(embed)
                                        .components(vec![button_components])
                                )
                            ).await.unwrap();
                        },
                        Ok(None) => {
                            let embed_error_not_found = serenity::CreateEmbed::default()
                                .title(t!("common.error", locale = language))
                                .description(t!("commands.wov.player.search.not_found", username = player.username, locale = language))
                                .color(serenity::Color::RED);
                            press.create_response(
                                ctx.http(),
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::default()
                                        .embed(embed_error_not_found)
                                )
                            ).await.unwrap();
                        },
                        Err(e) => {
                            error!("An error occurred while updating outdated information in the `wolvesville player search` command at request for the player by username: {:?}", e);
                            let embed_api_error = serenity::CreateEmbed::default()
                                .title(t!("common.error", locale = language))
                                .description(t!("common.api_error", locale = language))
                                .color(serenity::Color::RED);
                            press.create_response(
                                ctx.http(),
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::default()
                                        .embed(embed_api_error)
                                )
                            ).await.unwrap();
                        }
                    };
                }
            },
            _ => {}
        }
    }

    message.edit(
        ctx,
        create_reply
            .components(vec![get_player_search_buttons(ctx_id, true, true, true, &language)])
    ).await?;
    Ok(())
}

async fn get_thumbnail_attachment(avatar: Option<Avatar>, level: Option<i32>) -> serenity::CreateAttachment {
    match avatar {
        Some(avatar) => {
            match wov_image::render_wolvesville_avatar(avatar, level).await {
                Ok(avatar) => {
                    let mut buf = Vec::new();
                    avatar.1.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).expect("Failed to convert image to bytes");
                    serenity::CreateAttachment::bytes(buf, "avatar.png")
                },
                Err(e) => {
                    error!("An error occurred while rendering the avatar: {:?}", e);
                    serenity::CreateAttachment::file(&File::open(Path::new("res/images/wov_logo.png")).await.expect("Couldn't find wov_logo.png in res/images"), "avatar.png").await.unwrap()
                }
            }
        },
        None => serenity::CreateAttachment::file(&File::open(Path::new("res/images/wov_logo.png")).await.expect("Couldn't find wov_logo.png in res/images"), "avatar.png").await.unwrap()
    }
}

fn get_player_search_buttons(ctx_id: u64, disable_avatars: bool, disable_sp_plot: bool, disable_refresh: bool, language: &String) -> serenity::CreateActionRow {
    serenity::CreateActionRow::Buttons(
        vec![
            serenity::CreateButton::new(format!("{}.avatars", ctx_id))
                .label(t!("commands.wov.player.search.buttons.avatars", locale = language))
                .style(serenity::ButtonStyle::Primary)
                .disabled(disable_avatars),
            serenity::CreateButton::new(format!("{}.sp_plot", ctx_id))
                .label(t!("commands.wov.player.search.buttons.sp_plot", locale = language))
                .style(serenity::ButtonStyle::Primary)
                .disabled(disable_sp_plot),
            serenity::CreateButton::new(format!("{}.refresh", ctx_id))
                .emoji(serenity::ReactionType::Unicode("🔄".to_string()))
                .style(serenity::ButtonStyle::Secondary)
                .disabled(disable_refresh)
        ]
    )

}

async fn construct_player_embed(ctx_data: &Data, language: &String, player: &mut WolvesvillePlayer, thumbnail_filename: &str) -> serenity::CreateEmbed {
    // Converting a string with hex code of the color to u32. If it fails, it will be black (0)
    let color = u32::from_str_radix(player.profile_icon_color.trim_start_matches("#"), 16).unwrap_or(0);


    let mut embed = serenity::CreateEmbed::default()
        .title(format!("{}", player.username))
        .description(if player.previous_username.is_none() {t!("commands.wov.player.search.description.no_previous_username", username=player.username, locale = language)}
        else {t!("commands.wov.player.search.description.has_previous_username", username=player.username, previous_username= player.previous_username.as_mut().unwrap(), locale = language)})
        .color(serenity::Color::new(color))
        .thumbnail(format!("attachment://{}", thumbnail_filename))
        .timestamp(player.timestamp.unwrap_or(Utc::now()));

    embed = match player.personal_message {
        Some(ref mut pm) => if !pm.is_empty() { embed } else { embed.field(t!("commands.wov.player.search.personal_message", locale = language), pm.clone(), false) },
        None => embed
    };

    let level = match player.level {
        Some(level) => if level > 0 { level.to_string() } else { "?".to_string() },
        None => "?".to_string()
    };
    embed = embed.field(t!("commands.wov.player.search.level", locale = language), format!("**{}**", level), true);

    let status_emoji = match player.status.as_str() {
        "PLAY" => ctx_data.custom_emojis.get(CustomEmoji::LETS_PLAY).unwrap().to_string(),
        "DEFAULT" => "🟢".to_string(),
        "DND" => "🔴".to_string(),
        _ => "⚫".to_string(),
    };

    embed = embed.field(t!("commands.wov.player.search.online_status", locale = language),
                        format!("{} **{}**", status_emoji, t!(format!("commands.wov.player.search.online_status.{}", player.status), locale = language)), true);

    let last_online = DateTime::parse_from_rfc3339(player.last_online.as_mut().unwrap().as_str()).unwrap();
    let last_online = match Utc::now() - last_online.with_timezone(&Utc) < TimeDelta::minutes(7) {
        true => format!("{}", t!("commands.wov.player.search.last_online.just_now", locale = language)),
        false => get_relative_timestamp(&last_online.timestamp())
    };

    embed = embed.field(t!("commands.wov.player.search.last_online", locale = language), last_online, true);

    let created_at = if let Some(ref mut created_at) = player.creation_time  {
        let created_at = DateTime::parse_from_rfc3339(created_at).unwrap();
        get_long_date(&created_at.timestamp())
    } else if player.game_stats.total_play_time_in_minutes < 0 {
        format!("{}", t!("commands.wov.common.created_on.private", locale = language))
    } else {
        format!("{}", t!("commands.wov.common.created_on.august_3rd_2018", locale = language))
    };

    embed = embed.field(t!("commands.wov.common.created_on", locale = language), created_at, true);

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
            rose_emoji = ctx_data.custom_emojis.get(CustomEmoji::SINGLE_ROSE).unwrap().to_string(),
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
        Some(ref mut clan_id) => {
            let clan_info = match db::wolvesville::clan::get_wolvesville_clan_info_by_id(&ctx_data.db_pool, &clan_id).await {
                Ok(Some(clan_info)) => Some(clan_info),
                Err(_) | Ok(None) => {
                    match wolvesville::get_wolvesville_clan_info_by_id(&ctx_data.wolvesville_client, &clan_id).await {
                        Ok(Some(clan_info)) => {
                            db::wolvesville::clan::upsert_wolvesville_clan(&ctx_data.db_pool, clan_info.clone()).await.map_err(|e| {
                                error!("An error occurred while inserting or updating the clan: {:?}", e);
                                e
                            }).unwrap();
                            Some(clan_info)
                        },
                        Ok(None) => None,
                        Err(e) => {
                            error!("An error occurred while running the `wolvesville player search` command at request for the clan: {:?}", e);
                            None
                        }
                    }
                },
            };
            
            match clan_info {
                Some(clan_info) => {
                    let clan_description: String = match clan_info.description {
                        Some(description) => {
                            match description.split_once('\n') {
                                Some((first_line, _)) => format!("{}***...***", first_line),
                                None => description
                            }
                        },
                        None => format!("{}", t!("commands.wov.player.search.clan.no_description", locale = language))
                    };
                    embed = embed.field(
                        t!("commands.wov.player.search.clan", locale = language),
                        format!(
                            "`{clan_tag}` | **{clan_name}** :flag_{clan_language}: \n{clan_description}",
                            clan_tag = clan_info.tag.unwrap_or(" ".to_string()),
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
    embed
}

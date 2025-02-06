use std::path::Path;

use chrono::{DateTime, Utc};
use logfather::{debug, error, info};
use poise::{serenity_prelude as serenity, CreateReply, ReplyHandle};
use tokio::fs::File;
use crate::bot::core::structs::{Context, CustomColor, Data, Error};
use crate::utils::comma_readable_number;
use crate::utils::time::get_long_date;
use crate::{db, utils};
use crate::utils::apicallers::wolvesville;
use crate::utils::apicallers::wolvesville::models::WolvesvilleClan;
use crate::utils::language::get_language;

async fn on_missing_clan_name(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::ArgumentParse { input, ctx, ..} => {
            let language = get_language(&ctx.data(), &ctx.author().id.to_string()).await;

            if input == None {
                let embed = serenity::CreateEmbed::default()
                    .title(t!("common.error", locale = language))
                    .description(t!("commands.wov.clan.search.no_input", locale = language))
                    .color(serenity::Color::RED);
                ctx.send(CreateReply::default().reply(true).embed(embed)).await.unwrap();
            }
        }
        _ => {
            error!("Unexpected error when running wolvesville clan search command: {}", error);
        }
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn clan(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, on_error = on_missing_clan_name)]
pub async fn search(ctx: Context<'_>, #[rest] clan_name: String) -> Result<(), Error> {
    let data = ctx.data();
    let language = get_language(data, &ctx.author().id.to_string()).await;

    info!("Searching for clan: {}", clan_name);
    let clans: Vec<WolvesvilleClan> = match db::wolvesville::clan::get_wolvesville_clan_info_by_name(&data.db_pool, clan_name.as_str()).await {
        Ok(db_clan) => db_clan,
        _ => {
            debug!("Clan not found in the database, searching in the API");
            match wolvesville::get_wolvesville_clan_info_by_name(&data.wolvesville_client, clan_name.as_str()).await {
                Ok(Some(api_clans)) => {
                    api_clans
                },
                Ok(None) => {
                    let embed = serenity::CreateEmbed::default()
                        .title(t!("common.error", locale = language))
                        .description(t!("commands.wov.clan.search.not_found", locale = language))
                        .color(serenity::Color::RED);
                    ctx.send(CreateReply::default().reply(true).embed(embed)).await.unwrap();
                    return Ok(());
                },
                Err(err) => {
                    error!("Failed to get clan from the API: {}", err);
                    let embed = serenity::CreateEmbed::default()
                        .title(t!("common.error", locale = language))
                        .description(t!("common.api_error", locale = language))
                        .color(serenity::Color::RED);
                    ctx.send(CreateReply::default().reply(true).embed(embed)).await.unwrap();
                    return Ok(());
                }
            }
        }
    };

    let mut clan: Option<&WolvesvilleClan> = None;
    let mut main_message: Option<ReplyHandle> = None;
    let ctx_id = ctx.id().clone();


    if clans.len() > 1 {
        debug!("Multiple clans found, asking user to select one");
        let mut embed = serenity::CreateEmbed::default()
            .title(t!("commands.wov.clan.search.multiple_results.title", locale = language))
            .description(t!("commands.wov.clan.search.multiple_results.description", locale = language))
            .color(CustomColor::CYAN);

        let mut pos_counter = 0;
        let mut select_menu_options: Vec<serenity::CreateSelectMenuOption> = vec![];

        for clan in clans.iter() {
            pos_counter += 1;
            let clan_description = utils::get_first_part_of_string(&clan.description.clone().unwrap_or(t!("commands.wov.clan.search.no_description", locale = language).to_string()), '\n');
            let clan_tag = clan.tag.clone().unwrap_or_default();
            embed = embed.field(
                format!(
                    "**{}** `{}` | **{}** :flag_{}:", 
                    &pos_counter,
                    clan_tag,
                    clan.name,
                    clan.language.to_lowercase()
                ), 
                clan_description,
                true
            );
            select_menu_options.push(serenity::CreateSelectMenuOption::new(format!("{} | {}", clan_tag, clan.name), pos_counter.to_string()));
        }

        let select_menu = serenity::CreateSelectMenu::new(format!("{}.multiple", ctx.id()), serenity::CreateSelectMenuKind::String { options: select_menu_options })
            .placeholder(t!("commands.wov.clan.search.multiple_results.select_menu_placeholder", locale = language))
            .min_values(1)
            .max_values(1);

        main_message = Some(ctx.send(CreateReply::default().reply(true).embed(embed).components(vec![serenity::CreateActionRow::SelectMenu(select_menu)])).await.unwrap());

        let shard = ctx.serenity_context().shard.clone();
        let ctx_author = ctx.author().id.clone();

        while let Some(press) = serenity::collector::ComponentInteractionCollector::new(&shard)
            .filter(move |press| press.data.custom_id == format!("{}.multiple", &ctx_id) && &press.user.id == &ctx_author)
            .timeout(std::time::Duration::from_secs(600))  // 10 minutes
            .await 
        {
            let selected_option = match &press.data.kind {
                serenity::ComponentInteractionDataKind::StringSelect { values, .. } => values.first().unwrap(),
                _ => continue,
            };
            clan = Some(clans.get(selected_option.parse::<usize>().unwrap() - 1).unwrap());
            press.create_response(&ctx.serenity_context(), serenity::CreateInteractionResponse::Acknowledge).await.unwrap();
            break;
        }

        if clan.is_none() {
            let embed = serenity::CreateEmbed::default()
                .title(t!("common.timeout_error", locale = language))
                .description(t!("commands.wov.clan.search.multiple_results.no_selection", locale = language))
                .color(serenity::Color::RED);
            
            main_message.unwrap().edit(ctx, CreateReply::default().reply(true).embed(embed)).await.unwrap();
            return Ok(());
        }
    } 
    else if clans.len() == 1 { clan = clans.first();}
    else {
        let embed = serenity::CreateEmbed::default()
            .title(t!("common.error", locale = language))
            .description(t!("commands.wov.clan.search.not_found", locale = language))
            .color(serenity::Color::RED);
        ctx.send(CreateReply::default().reply(true).embed(embed)).await.unwrap();
        return Ok(());
    }

    let clan = clan.unwrap();

    debug!("Clan found: {:?}", clan);

    let embed_thumbnail = serenity::CreateAttachment::file(&File::open(Path::new("res/images/wov_logo.png")).await.unwrap(), "wov_logo.png").await.unwrap();
    let embed = construct_clan_embed(clan, &language).await;

    let button_components = get_clan_search_buttons(ctx.id(), false, false, &language);

    if let Some(main_message) = main_message {
        main_message.edit(ctx, CreateReply::default().reply(true).embed(embed).attachment(embed_thumbnail).components(vec![button_components])).await.unwrap();
    } else {
        main_message = Some(ctx.send(CreateReply::default().reply(true).embed(embed).attachment(embed_thumbnail).components(vec![button_components])).await.unwrap());
    }

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(&ctx.serenity_context().shard.clone())
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(600))  // 10 minutes
        .await 
    {
        match press.data.custom_id.as_str() {
            id if id.ends_with(".fetch_members") => {
                todo!()
            },
            id if id.ends_with(".refresh") => {
                todo!()
            },
            _ => {}
        }
        
    }

    Ok(())
}

async fn construct_clan_embed(clan: &WolvesvilleClan, language: &String) -> serenity::CreateEmbed {
    serenity::CreateEmbed::default()
        .title(format!("`{}` | {}", clan.tag.clone().unwrap_or("\u{200B}".to_string()), clan.name))
        .description(clan.description.clone().unwrap_or(t!("commands.wov.clan.search.no_description", locale = language).to_string()))
        .color(serenity::Color::new(u32::from_str_radix(&clan.icon_color.trim_start_matches("#"), 16).unwrap_or(0)))
        .timestamp(clan.timestamp.unwrap_or(Utc::now()))
        .thumbnail("attachment://wov_logo.png")
        .field("XP", format!("**{}**", comma_readable_number(clan.xp as i64)), true)
        .field(t!("commands.wov.clan.search.language", locale = language), format!(":flag_{}:", clan.language.to_lowercase()), true)
        .field(t!("commands.wov.clan.search.member_count", locale = language), format!("**{}/50**", clan.member_count), true)
        .field(t!("commands.wov.common.created_on", locale = language), get_long_date(&DateTime::parse_from_rfc3339(&clan.creation_time).unwrap().timestamp()), true)
        .field(t!("commands.wov.clan.search.status", locale = language), format!("**{}**", t!(format!("commands.wov.clan.search.status.{}", clan.join_type), locale = language)), true)
        .field(t!("commands.wov.clan.search.minimum_level", locale = language), format!("**{}**", clan.min_level), true)
        .field(t!("commands.wov.clan.search.quests_done", locale = language), format!("**{}**", clan.quest_history_count), true)
}

fn get_clan_search_buttons(ctx_id: u64, disable_fetch_members: bool, disable_refresh: bool, language: &String) -> serenity::CreateActionRow {
    serenity::CreateActionRow::Buttons(
        vec![
            serenity::CreateButton::new(format!("{}.fetch_members", ctx_id))
                .label(t!("commands.wov.clan.search.fetch_members", locale = language))
                .style(serenity::ButtonStyle::Primary)
                .disabled(disable_fetch_members),
            serenity::CreateButton::new(format!("{}.refresh", ctx_id))
                .label(t!("commands.wov.clan.search.refresh", locale = language))
                .style(serenity::ButtonStyle::Secondary)
                .disabled(disable_refresh)
        ]
    )
}

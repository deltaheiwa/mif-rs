use std::collections::HashMap;

use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, CustomColor};
use crate::utils::language::get_language;
use crate::db::users::{add_user, hit_user, set_language_code};
use logfather::error;


async fn show_common(ctx: Context<'_>) -> Result<(), Error> {
    if !(hit_user(&ctx.data().db_pool, &ctx.author().id.to_string()).await?) { 
        match add_user(&ctx.data().db_pool, &ctx.author().id.to_string()).await {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to add user in preferences with ID {}: {:?}", &ctx.author().id, e);
                ctx.reply("Database error. Please try again later").await?;
                return Ok(());
            }
        }
    }

    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

    let mut embed = serenity::CreateEmbed::default()
        .title(t!("commands.directive.preferences.title", locale = language))
        .description(t!("commands.directive.preferences.description", locale = language, username = ctx.author().name))
        .color(CustomColor::CYAN);

    embed = embed.field(
        t!("commands.directive.preferences.fields.language.name", locale = language), 
        t!("commands.directive.preferences.fields.language.value", locale = language), 
        false
    );
    
    let ctx_id: u64 = ctx.id();
    let preferences_button_id: String = format!("{}pref", ctx_id);

    let buttons = serenity::CreateActionRow::Buttons(
        vec![
            serenity::CreateButton::new(&preferences_button_id).label("Change preferences")
        ]
    );

    ctx.send(poise::CreateReply::default().embed(embed).components(vec![buttons])).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60 * 10)) // Timeout after 10 minutes
        .await
    {
        let modal_id: String = format!("{}modal", ctx_id);

        let modal = serenity::CreateModal::new(&modal_id, "Preferences")
            .components(vec![]);

        press.create_response(
            &ctx.serenity_context(), 
            serenity::CreateInteractionResponse::Modal(
                modal
            )
        ).await?;
    }

    Ok(())
}


#[poise::command(
    prefix_command, slash_command
)]
pub async fn preferences(ctx: Context<'_>) -> Result<(), Error> {
    show_common(ctx).await
}

#[poise::command(
    slash_command, prefix_command
)]
pub async fn show(ctx: Context<'_>) -> Result<(), Error> {
    show_common(ctx).await
}

#[poise::command(
    slash_command, prefix_command,
    )]
pub async fn language(ctx: Context<'_>, new_language: String) -> Result<(), Error> {
    let new_language = new_language.to_lowercase();
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

    let languages = vec!["en", "uk", "ua", "ukrainian", "english"];

    let language_code_map: HashMap<&str, &str> = vec![
        ("en", "English"),
        ("uk", "Ukrainian"),
    ].into_iter().collect();

    if languages.contains(&new_language.as_str()) {
        let new_language = match new_language.as_str() {
            "ua" => "uk",
            "ukrainian" => "uk",
            "english" => "en",
            _ => new_language.as_str()
        };
        let language_name = *language_code_map.get(new_language).unwrap_or(&"Unknown");

        match set_language_code(&ctx.data().db_pool, &ctx.author().id.to_string(), new_language).await {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set language for user with ID {}: {:?}", &ctx.author().id, e);
                ctx.reply("Database error. Please try again later").await?;
                return Ok(());
            }
        }

        ctx.reply(format!("{}", t!("commands.directive.preferences.change_language.success", locale = language, language_success = language_name))).await?;
    } else {
        ctx.reply(format!("{}", t!("commands.directive.preferences.change_language.fail", locale = language, language_fail = new_language))).await?;
    }

    Ok(())
}

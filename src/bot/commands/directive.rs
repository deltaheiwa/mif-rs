use std::collections::HashMap;

use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, CustomColor};
use crate::utils::language::{get_language, set_language};
use crate::db::users::{add_user, hit_user, set_language_code};
use logfather::error;
use crate::bot::core::constants::DEFAULT_PREFIX;
use crate::bot::determine_prefix;
use crate::db::prefixes;

async fn show_common(ctx: Context<'_>) -> Result<(), Error> {
    if !hit_user(&ctx.data().db_pool, &ctx.author().id.to_string()).await? { 
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
    let prefix = determine_prefix(ctx.into()).await?;

    let embed = serenity::CreateEmbed::default()
        .title(t!("commands.directive.preferences.title", locale = language))
        .description(t!("commands.directive.preferences.description", locale = language, username = ctx.author().name))
        .color(CustomColor::CYAN)
        .field(
            t!("commands.directive.preferences.fields.language.name", locale = language), 
            t!("commands.directive.preferences.fields.language.value", locale = language), 
            false
        )
        .field(
            t!("commands.directive.preferences.fields.prefix.name", locale = language),
            prefix.unwrap_or_else(|| DEFAULT_PREFIX.to_string()),
            false
        );
    
    
    let ctx_id: u64 = ctx.id();
    let preferences_button_id: String = format!("{}.pref", ctx_id);

    let _buttons = serenity::CreateActionRow::Buttons(
        vec![
            serenity::CreateButton::new(&preferences_button_id).label("Change preferences")
        ]
    );

    // Removed view as not needed. Button code is currently redundant, but will come in handy when adding more preferences
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(600)) // Timeout after 10 minutes
        .await
    {
        let modal_id: String = format!("{}modal", ctx_id);

        // Left for future reference
        let modal = serenity::CreateModal::new(&modal_id, "Preferences")
            .components(vec![
                serenity::CreateActionRow::InputText(
                    serenity::CreateInputText::new(serenity::InputTextStyle::Short, "Language", format!("{}c1", modal_id))
                        .placeholder("Enter a language code")
                        .required(true)
                )
            ]);

        press.create_response(
            &ctx.serenity_context(), 
            serenity::CreateInteractionResponse::Modal(
                modal
            )
        ).await?;
    }

    Ok(())
}

/// Configure the bot to your liking.
#[poise::command(
    prefix_command, slash_command,
    name_localized("uk", "налаштування"),
    description_localized("uk", "Налаштуйте бота на свій смак."),
    category = "config",
    subcommands("show", "language", "prefix"),
    subcommand_required = false,
)]
pub async fn preferences(ctx: Context<'_>) -> Result<(), Error> {
    show_common(ctx).await
}


/// Check your preferences.
#[poise::command(
    slash_command, prefix_command,
    name_localized("uk", "показати"),
    description_localized("uk", "Перевірте ваші налаштування")
)]
pub async fn show(ctx: Context<'_>) -> Result<(), Error> {
    show_common(ctx).await
}

/// Set your locale. Currently supported languages are English and Ukrainian.
#[poise::command(
    slash_command, prefix_command,
    rename = "locale",
    name_localized("uk", "локалізація"),
    description_localized("uk", "Змініть мову бота. Підтримуються англійська та українська."),
    )]
pub async fn language(
    ctx: Context<'_>, 
    #[rename = "language"] #[name_localized("uk", "мова")]
    new_language: String
) -> Result<(), Error> {
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
            Ok(_) => {set_language(ctx.data(), &ctx.author().id.to_string(), new_language).await;},
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


/// Set your own custom prefix for the bot.
#[poise::command(
    slash_command, prefix_command,
    rename = "prefix",
    name_localized("uk", "префікс"),
    description_localized("uk", "Поставте власний префікс для бота."),
)]
pub async fn prefix(
    ctx: Context<'_>,
    #[rename = "new_prefix"] #[name_localized("uk", "новий_префікс")]
    new_prefix: Option<String>
) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let language = get_language(ctx.data(), &user_id).await;
    
    let mut prefix_cache = ctx.data().prefix_cache.lock().await;
    
    match new_prefix {
        Some(prefix) => {
            if prefix.len() > 5 {
                ctx.reply(t!("commands.admin.prefix.too_long", locale = language)).await?;
                return Ok(());
            }

            prefixes::set_prefix(&ctx.data().db_pool, &user_id, &prefix).await?;
            prefix_cache.put(user_id, prefix.clone());

            ctx.reply(t!("commands.admin.prefix.success", prefix = prefix, locale = language)).await?;
        }
        None => {
            // Reset prefix
            match prefixes::delete_prefix(&ctx.data().db_pool, &user_id).await {
                Ok(_) => {
                    prefix_cache.pop(&user_id);
                    ctx.reply(t!("commands.admin.prefix.reset.success", locale = language)).await?;
                },
                Err(e) => {
                    error!("Failed to delete prefix for user {}: {:?}", user_id, e);
                    ctx.reply(t!("commands.admin.prefix.reset.fail", locale = language)).await?;
                }
            }
            
        }
    }

    Ok(())
}
use std::collections::HashMap;

use poise::serenity_prelude as serenity;
use ::serenity::all::CreateSelectMenuOption;
use crate::bot::core::structs::{Context, Error, CustomColor};
use crate::utils::language::get_language;
use crate::db::users::{add_user, hit_user};
use logfather::error;


#[poise::command(
    slash_command, prefix_command
)]
pub async fn preferences(ctx: Context<'_>) -> Result<(), Error> {
    if !(hit_user(&ctx.data().db_pool, &ctx.author().id.to_string()).await?) { 
        match add_user(&ctx.data().db_pool, &ctx.author().id.to_string()).await {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to add user in preferences with ID {}: {:?}", &ctx.author().id, e);
                ctx.reply("Database error. Please try again error").await?;
                return Ok(());
            }
        }
    }

    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

    let mut embed = serenity::CreateEmbed::default()
        .title(t!("commands.directive.preferences.title", locale = language))
        .description(t!("commands.directive.preferences.description", locale = language))
        .color(CustomColor::CYAN);

    let language_code_map: HashMap<&str, &str> = vec![
        ("en", "English"),
        ("uk", "Ukrainian"),
    ].into_iter().collect();

    let language_name = *language_code_map.get(language.as_str()).unwrap_or(&"Unknown");

    embed = embed.field(t!("commands.directive.preferences.language", locale = language), language_name, false);
    
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
        let language_options = Vec::from_iter(language_code_map.iter().map(|(k, v)| {
            CreateSelectMenuOption::new(v as &str, k as &str).description(t!("commands.directive.preferences.select_menu.description", locale = k))
        }));
        let select_menu = serenity::CreateSelectMenu::new(
                format!("{}language", modal_id), 
                serenity::CreateSelectMenuKind::String {
                    options: language_options
        });

        let modal = serenity::CreateModal::new(&modal_id, "Preferences")
            .components(vec![
                serenity::CreateActionRow::SelectMenu(select_menu)
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



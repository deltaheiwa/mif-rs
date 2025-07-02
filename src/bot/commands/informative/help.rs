use poise::{serenity_prelude as serenity, CreateReply};
use crate::bot::core::structs::{Context, CustomColor, Data, Error};
use crate::utils::language::get_language;


/// Help command to display available commands and their descriptions. I love recursion.
#[poise::command(
    slash_command, prefix_command,
    name_localized("uk", "довідка"),
    description_localized("uk", "Показати доступні команди та їхні описи. Я люблю рекурсію."),
    category = "info",
)]
pub async fn help(
    ctx: Context<'_>, 
    #[name_localized("uk", "команда")] command: Option<String>
) -> Result<(), Error> {
    // for command in &ctx.framework().options().commands {
    //    debug!("{}: {}", command.name, command.help_text.clone().unwrap_or("No help text".to_string()));
    // }
    
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;
    
    if let Some(_) = command {
        // TODO: Add command-specific help text
        let embed = serenity::CreateEmbed::default()
            .title(t!("common.under_construction.title", locale = language))
            .description(t!("common.under_construction", locale = language))
            .color(serenity::Color::from_rgb(250, 0, 0));
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(())
    }
    
    let mut categories = std::collections::HashMap::new();
    let mut select_menu_options = Vec::new();
    for command in &ctx.framework().options().commands {
        let category = command.category.clone().unwrap_or_else(|| "uncategorized".to_string());
        categories.entry(category).or_insert_with(Vec::new).push(command);
    }
    
    for (category, _) in &categories {
        let category_translated = t!(format!("help.{}", category), locale = language);
        select_menu_options.push(serenity::CreateSelectMenuOption::new(
            category_translated,
            category.clone()
        ));
    }

    let ctx_author = ctx.author().id.clone();
    let mut embed = serenity::CreateEmbed::default()
        .title(t!("commands.info.help.title", locale = language))
        .description(t!("commands.info.help.description", locale = language))
        .color(CustomColor::CYAN);

    let select_menu = serenity::CreateSelectMenu::new(format!("help_select_{}", ctx_author.get()), serenity::CreateSelectMenuKind::String { options: select_menu_options })
        .placeholder(t!("commands.info.help.select_menu_placeholder", locale = language))
        .min_values(1)
        .max_values(1);
    
    let message = ctx.send(CreateReply::default().embed(embed.clone()).components(vec![serenity::CreateActionRow::SelectMenu(select_menu)])).await?;
    
    let shard = ctx.serenity_context().shard.clone();

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(&shard)
        .author_id(ctx_author)
        .filter(move |i| i.data.custom_id == format!("help_select_{}", ctx_author.get()))
        .timeout(std::time::Duration::from_secs(180))
        .await
    {
        let selected_category = match &press.data.kind { 
            serenity::ComponentInteractionDataKind::StringSelect { values, .. } => values.first().unwrap(),
            _ => continue
        };
        
        if let Some(commands) = categories.get(selected_category) {
            embed = serenity::CreateEmbed::default()
                .title(t!(format!("help.{}", selected_category), locale = language))
                .description(t!("commands.info.help.category.description", locale = language))
                .color(CustomColor::CYAN);
            
            embed = construct_embed_for_category(embed, &selected_category, commands, &language);
            
            press.create_response(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default().embed(embed.clone())
                )
            ).await?;
        }
    }
    
    message.edit(
        *&ctx,
        CreateReply::default().embed(embed).components(vec![])
    ).await?;
    
    Ok(())
}

fn construct_embed_for_category(
    mut embed: serenity::CreateEmbed,
    _category: &str,
    commands: &[&poise::Command<Data, Error>],
    locale: &str,
) -> serenity::CreateEmbed {
    let no_description = t!("help.no_description", locale = locale).to_string(); 
    for command in commands {
        let name = &command.name_localizations.get(locale)
            .unwrap_or(&command.name_localizations.get("en").unwrap_or(&command.name))
            .to_string();
        let descriptions = &command.description_localizations;
        let mut value_str = match descriptions.get(locale) { 
            Some(desc) => desc.to_string(),
            None => command.description.as_ref().unwrap_or(&no_description).to_string()
        };
        
        let mut sub_command_lines: Vec<String> = Vec::new();


        for sub_command in &command.subcommands {
            if sub_command.subcommands.len() == 1
                && sub_command.subcommands[0].subcommands.is_empty()
            {
                let final_sub_command = &sub_command.subcommands[0];
                let combined_name =
                    format!("{} {}", sub_command.name_localizations.get(locale)
                        .unwrap_or(&sub_command.name),
                        final_sub_command.name_localizations.get(locale)
                        .unwrap_or(&final_sub_command.name));
                
                let descriptions = &final_sub_command.description_localizations;
                
                let description = match descriptions.get(locale) {
                    Some(desc) => desc.to_string(),
                    None => final_sub_command.description.as_ref()
                        .unwrap_or(&no_description).to_string()
                };
                    
                
                sub_command_lines
                    .push(format!("- **{}**: {}", combined_name, description));
            } else {
                sub_command_lines.push(format_command_recursively(
                    sub_command,
                    0,
                    locale,
                ));
            }
        }
        
        if !sub_command_lines.is_empty() {
            value_str.push('\n');
            value_str.push_str(&sub_command_lines.join("\n"));
        }
        
        embed = embed.field(name, value_str, false);
    }

    embed
}

/// Helper to format a command and its children recursively for nested lists.
fn format_command_recursively(
    command: &poise::Command<Data, Error>,
    layer: usize,
    locale: &str,
) -> String {
    let indent = "  ".repeat(layer);
    let no_description = t!("help.no_description", locale = locale).to_string();
    let description = match command.description_localizations.get(locale) {
        Some(desc) => desc.to_string(),
        None => command.description.as_ref().unwrap_or(&no_description).to_string()
    };
    let mut result =
        format!(
            "{}- **{}**: {}", 
            indent, 
            command.name_localizations
                .get(locale).unwrap_or(&command.name),
            description);
    
    if !command.subcommands.is_empty() {
        let mut child_lines = Vec::new();
        for child in &command.subcommands {
            child_lines.push(format_command_recursively(child, layer + 1, locale));
        }
        result.push('\n');
        result.push_str(&child_lines.join("\n"));
    }

    result
}
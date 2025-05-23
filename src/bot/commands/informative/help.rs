use poise::serenity_prelude as serenity;
use logfather::debug;
use crate::bot::core::structs::{Context, CustomColor, Error};

#[poise::command(
    slash_command, prefix_command,
    category = "informative",
)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    for command in &ctx.framework().options().commands {
        debug!("{}: {}", command.name, command.help_text.clone().unwrap_or("No help text".to_string()));
    }
    
    if let Some(command) = command {
        if let Some(cmd) = ctx.framework().options().commands.iter().find(|c| c.name == command) {
            ctx.send(poise::CreateReply::default().content(command)).await?;
        } else {
            ctx.send(poise::CreateReply::default().content(format!("Command `{}` not found.", command))).await?;
        }
        return Ok(())
    }
    
    // TODO: Categorized multi list with select menu
    
    let mut categories = std::collections::HashMap::new();
    for command in &ctx.framework().options().commands {
        let category = command.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
        categories.entry(category).or_insert_with(Vec::new).push(command);
    }

    let mut embed = serenity::CreateEmbed::default()
        .title(t!("commands.info.help.title"))
        .description(t!("commands.info.help.description"))
        .color(CustomColor::CYAN);
    
    for (category, commands) in categories {
        let mut commands_list = commands.iter().map(|c| c.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        for command in &commands {
            if !command.subcommands.is_empty() {
                let subcommands_list = command.subcommands.iter().map(|c| c.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                commands_list.push_str(&format!("\n{}: {}", command.name, subcommands_list));
            }
            
        }
        
        embed = embed.field(
            category,
            format!("`{}`", commands_list),
            false,
        );
    }
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    
    Ok(())
}

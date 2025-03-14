use logfather::debug;
// use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error};
use poise::builtins;

#[poise::command(
    slash_command, prefix_command,
    category = "informative",
)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    // for command in &ctx.framework().options().commands {
    //     debug!("{}: {}", command.name, command.help_text.clone().unwrap_or("No help text".to_string()));
    // }
    let config = builtins::HelpConfiguration {
        show_subcommands: true,
        ..Default::default()
    };

    builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

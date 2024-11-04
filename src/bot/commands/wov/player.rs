use crate::bot::core::structs::{Context, Error};
use poise::serenity_prelude as serenity;
use crate::utils::{apicallers::wolvesville, language::get_language};

#[poise::command(
    slash_command, prefix_command
)]
pub async fn player(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command, prefix_command
)]
pub async fn search(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;
    let player = "a";
    Ok(())
}
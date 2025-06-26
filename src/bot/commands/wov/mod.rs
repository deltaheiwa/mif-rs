pub mod player;
pub mod clan;

use crate::bot::wov::{player::player, clan::clan };
use crate::bot::core::structs::{Context, Error};

#[poise::command(
    slash_command, prefix_command,
    category = "wolvesville",
    subcommands("player", "clan"),
    subcommand_required = true,
)]
pub async fn wolvesville(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
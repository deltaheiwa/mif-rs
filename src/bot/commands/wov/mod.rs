pub mod player;
pub mod clan;

use crate::bot::wov::{player::player, clan::clan };
use crate::bot::core::structs::{Context, Error};


/// Wolvesville related commands.
#[poise::command(
    slash_command, prefix_command,
    category = "wolvesville",
    description_localized("uk", "Команди Wolvesville."),
    subcommands("player", "clan"),
    subcommand_required = true,
)]
pub async fn wolvesville(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
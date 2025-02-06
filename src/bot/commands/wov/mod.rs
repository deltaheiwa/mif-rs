pub mod player;
pub mod clan;

use crate::bot::core::structs::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn wolvesville(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
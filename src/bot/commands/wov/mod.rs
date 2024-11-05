pub mod player;

use crate::bot::core::structs::{Context, Error, Data};

#[poise::command(slash_command, prefix_command)]
pub async fn wolvesville(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
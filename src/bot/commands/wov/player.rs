use std::fs::File;
use poise::serenity_prelude as serenity;
use crate::bot::core::structs::{Context, Error, Data};
use crate::utils::{language::get_language, apicallers::wolvesville};
use logfather::{debug, error};

async fn on_missing_username_input(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::ArgumentParse { input, ctx, .. } => {
            let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

            if input == None {
                let embed = serenity::CreateEmbed::default()
                    .title(t!("common.error", locale = language))
                    .description(t!("commands.wov.player.no_input", locale = language))
                    .color(serenity::Color::RED);
                ctx.send(poise::CreateReply::default().reply(true).embed(embed)).await.unwrap();
            }
        }
        _ => {
            error!("An error occurred while running the player command: {}", error);
        }
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn player(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command, prefix_command,
    on_error = on_missing_username_input
)]
pub async fn search(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let data = ctx.data();
    let language = get_language(data, &ctx.author().id.to_string()).await;

    if username.len() < 3 {
        let embed_too_short = serenity::CreateEmbed::default()
            .title(t!("common.error", locale = language))
            .description(t!("commands.wov.player.too_short", username = username, locale = language))
            .color(serenity::Color::RED);
        ctx.send(poise::CreateReply::default().reply(true).embed(embed_too_short)).await.unwrap();
        return Ok(());
    }

    let player = wolvesville::get_wolvesville_player_by_username(&data.wolvesville_client, &username).await.unwrap();
    // To check the output of the API call, uncomment the following lines:
    // let file = File::create("wolvesville_player.json").unwrap();
    // serde_json::to_writer_pretty(file, &player).unwrap();


    Ok(())
}
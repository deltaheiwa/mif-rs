use poise::serenity_prelude as serenity;
use ::serenity::all::Presence;
use crate::bot::core::structs::{Context, CustomColor, Error};
use crate::utils::{time, language::get_language};


#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;
    let runners = &ctx.framework().shard_manager.runners.lock().await;
    let runner_info = runners.get(&serenity::ShardId(0)).unwrap();

    // Attempt to retrieve latency
    if let Some(latency) = runner_info.latency {
        ctx.reply(format!("{}", t!("commands.info.ping.latency", latency = latency.as_millis(), locale = language))).await?;
    } else {
        // If latency is unavailable (shard just connected, and there was no heartbeat from discord yet)
        ctx.reply(format!("{}", t!("commands.info.ping.no_latency", locale = language))).await?;
    }
    Ok(())
}


/// Get information about a user
#[poise::command(
    slash_command, prefix_command,
    rename = "user-info",
    name_localized("uk", "інфо-користувача"),
    description_localized("uk", "Отримай інформацію про користувача")
)]
pub async fn user_info(
    ctx: Context<'_>, 
    #[name_localized("uk", "користувач")] user: Option<serenity::User>
) -> Result<(), Error> {
    let user_info = user.unwrap_or(ctx.author().clone());
    let language = get_language(ctx.data(), &ctx.author().id.to_string()).await;

    let embed = match ctx.guild() {
        Some(guild) =>  {
            match guild.members.get(&user_info.id) {
                Some(member) => {
                    let color = match guild.member_highest_role(member) {
                        Some(role) => role.colour,
                        None => CustomColor::CYAN
                    };
                    let presence = guild.presences.get(&user_info.id);
                    build_member_embed(member, &language, color, presence)
                },
                None => build_user_embed(&user_info, &language, None, &user_info.global_name)
            }
        },
        None => build_user_embed(&user_info, &language, None, &user_info.global_name)
    };
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;


    Ok(())
}

fn build_user_embed(user: &serenity::User, language: &String, embed_color: Option<serenity::Color>, nickname: &Option<String>) -> serenity::CreateEmbed {
    let color = embed_color.unwrap_or(CustomColor::CYAN);

    let mut embed = serenity::CreateEmbed::default()
        .title(format!("{}", t!("commands.info.user_info.title", locale = language)))
        .description(format!("{}", t!("commands.info.user_info.description", user = user.tag(), locale = language)))
        .color(color)
        .thumbnail(user.face());

    embed = embed.field(
        format!("{}", t!("commands.info.user_info.fields.username", locale = language)),
        format!("{}", user.name),
        true
    );
    
    if let Some(nickname) = nickname {
        embed = embed.field(
            format!("{}", t!("commands.info.user_info.fields.nickname", locale = language)),
            format!("{}", nickname),
            true
        );
    }

    embed = embed.field(
        "ID",
        format!("{}", user.id),
        true
    );

    embed = embed.field(
        format!("{}", t!("commands.info.user_info.fields.created_at", locale = language)),
        format!("{}", time::get_relative_timestamp(user.created_at())),
        true
    );

    

    embed
}

fn build_member_embed(member: &serenity::Member, language: &String, embed_color: serenity::Color, presence: Option<&Presence>) -> serenity::CreateEmbed {
    let user = &member.user;
    let mut embed = build_user_embed(user, language, Some(embed_color), &Some(String::from(member.display_name())));

    if let Some(joined_at) = member.joined_at {
        embed = embed.field(
            format!("{}", t!("commands.info.user_info.fields.joined_at", locale = language)),
            format!("{}", time::get_relative_timestamp(joined_at)),
            true
        );
    }

    if let Some(presence) = presence {
        embed = embed.field(
            format!("{}", t!("commands.info.user_info.fields.status", locale = language)),
            format!("{}", presence.status.name()),
            true
        );
    }

    embed
}

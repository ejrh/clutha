use std::collections::HashSet;

use serenity::all::{CreateEmbed, CreateMessage, PartialGuild};
use serenity::client::Context;
use serenity::framework::standard::help_commands;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, HelpOptions};
use serenity::framework::standard::macros::{command, group, help};
use serenity::model::prelude::{Message, UserId};
use serenity::utils::MessageBuilder;
use tracing::error;

use crate::discord::{BotContainer, ShardManagerContainer, system_message};

#[command]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    system_message(ctx, msg, &format!("Clutha version {VERSION}")).await?;

    Ok(())
}

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    system_message(ctx, msg, "Shutting down").await?;

    let data = ctx.data.read().await;
    if let Some(shard_manager) = data.get::<ShardManagerContainer>() {
        shard_manager.shutdown_all().await;
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let channel = msg.channel_id.to_channel(&ctx).await?;

    let response = MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&msg.author.name)
        .push(" used the 'ping' command in the ")
        .mention(&channel)
        .push(" channel")
        .build();

    system_message(ctx, msg, &response).await?;

    Ok(())
}

#[command]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let Some(bot) = data.get_mut::<BotContainer>()
    else {
        error!("Couldn't get bot object!");
        return Ok(())
    };

    bot.dialogue.reset();

    system_message(ctx, msg, "Dialogue reset").await?;

    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let Some(bot) = data.get_mut::<BotContainer>()
        else {
            error!("Couldn't get bot object!");
            return Ok(())
        };

    let mut context = MessageBuilder::new();
    if msg.is_private() {
        context.push("Private chat with ").mention(&msg.author);
    } else {
        let channel = msg.channel_id.to_channel(&ctx).await?;
        let guild_name = if let Some(gid) = msg.guild_id {
            let guild = PartialGuild::get(&ctx.http, gid).await?;
            guild.name
        } else { "???".to_string() };
        context.push("Channel ").mention(&channel)
            .push(" on server ").push_bold_safe(&guild_name);
    };

    let mode_str = "active";
    let prompt_str = "default";

    let embed = CreateEmbed::new().description(context.build())
        .field("Mode", mode_str, true)
        .field("Prompt", prompt_str, true)
        .field("Dialogue size", format!("{} / {}", bot.dialogue.total_len, bot.dialogue.max_len), true);

    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;

    Ok(())
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[group]
#[commands(ping, version, reset, info)]
struct General;

#[group]
#[owners_only]
#[commands(shutdown)]
struct Admin;

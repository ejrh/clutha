use std::collections::HashSet;

use serenity::client::Context;
use serenity::framework::standard::help_commands;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, HelpOptions};
use serenity::framework::standard::macros::{command, group, help};
use serenity::model::prelude::{Message, UserId};
use serenity::utils::MessageBuilder;
use tracing::error;

use crate::discord::{BotContainer, ShardManagerContainer};

#[command]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    msg.channel_id.say(&ctx, &format!("Clutha version {VERSION}")).await?;

    Ok(())
}

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "Shutting down").await?;

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

    msg.channel_id.say(&ctx, &response).await?;

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

    msg.channel_id.say(&ctx, "Dialogue reset").await?;

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
#[commands(ping, version, shutdown, reset)]
struct General;

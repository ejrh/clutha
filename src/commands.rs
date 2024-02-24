use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::Message;
use serenity::utils::MessageBuilder;

use crate::discord::ShardManagerContainer;

#[command]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    msg.channel_id.say(&ctx.http, &format!("Clutha version {VERSION}")).await?;

    Ok(())
}

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Shutting down").await?;

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

    msg.channel_id.say(&ctx.http, &response).await?;

    Ok(())
}

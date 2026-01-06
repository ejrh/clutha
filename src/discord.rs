use std::sync::Arc;

use serenity::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, Error};
use serenity::all::{ChannelId, CreateEmbed, CreateMessage};
use tracing::{error, info};

use crate::bot::Bot;
use crate::commands::run_command;

struct Handler;

pub(crate) struct ShardManagerContainer(Arc<ShardManager>);

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub(crate) struct BotContainer(Arc<Mutex<Bot>>);

impl TypeMapKey for BotContainer {
    type Value = Arc<Mutex<Bot>>;
}

async fn get_bot(ctx: &Context) -> Result<Arc<Mutex<Bot>>, &'static str> {
    let data = ctx.data.read().await;
    let bot = data.get::<BotContainer>().ok_or("BotContainer not found!")?;
    Ok(bot.clone())
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let bot = match get_bot(&ctx).await {
            Err(err) => {
                error!("Could not get bot: {:?}", err);
                return;
            },
            Ok(bot) => bot,
        };

        let trimmed = msg.content.trim_start();
        if trimmed.starts_with('~') {
            let mut bot = bot.lock().await;
            match run_command(&mut bot, &ctx, &msg).await {
                Ok(_) => (),
                Err(why) => {
                    system_error(&ctx, msg.channel_id, &format!("Could not run command: {:?}", why)).await;
                    error!("Could not run command: {:?}", why);
                }
            }
            return;
        }

        let mut bot = bot.lock().await;

        match bot.handle_dialogue(&ctx, &msg).await {
            Ok(_) => (),
            Err(why) => {
                system_error(&ctx, msg.channel_id, &format!("Could not handle dialogue: {:?}", why)).await;
                error!("Could not handle dialogue: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub(crate) async fn run_bot(bot: Bot, token: &str) -> Result<(), Error> {
    let bot = Arc::new(Mutex::new(bot));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<BotContainer>(bot);
    }

    client.start().await?;

    Ok(())
}

pub(crate) async fn system_message(ctx: &Context, channel: ChannelId, text: &str) -> Result<(), Error> {
    let embed = CreateEmbed::new().description(text).color((64, 64, 128));
    let message = CreateMessage::new().embed(embed);
    channel.send_message(&ctx, message).await?;
    Ok(())
}

pub(crate) async fn system_info(ctx: &Context, channel: ChannelId, text: &str) -> Result<(), Error> {
    let embed = CreateEmbed::new().description(text).color((64, 128, 64));
    let message = CreateMessage::new().embed(embed);
    channel.send_message(&ctx, message).await?;
    Ok(())
}

pub(crate) async fn system_error(ctx: &Context, channel: ChannelId, text: &str) {
    let embed = CreateEmbed::new().description(text).color((128, 64, 64));
    let message = CreateMessage::new().embed(embed);
    if let Err(e) = channel.send_message(&ctx, message).await {
        error!("Could not send error message: {:?}", e);
    }
}

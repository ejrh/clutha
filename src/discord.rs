use std::sync::Arc;

use serenity::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, Error};
use tracing::{error, info};

use crate::bot::Bot;
use crate::commands::create_framework;

struct Handler;

pub(crate) struct ShardManagerContainer(Arc<ShardManager>);

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub(crate) struct BotContainer(Arc<Mutex<Bot>>);

impl TypeMapKey for BotContainer {
    type Value = Arc<Mutex<Bot>>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let trimmed = msg.content.trim_start();
        if trimmed.starts_with('~') {
            return;
        }

        let mut data = ctx.data.write().await;
        let Some(bot) = data.get_mut::<BotContainer>() else {
            error!("Couldn't get bot object!");
            return;
        };

        let mut bot = bot.lock().await;

        match bot.handle_dialogue(&ctx, &msg).await {
            Ok(_) => (),
            Err(why) => {
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

    let framework = create_framework(bot.clone())?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .framework(framework)
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


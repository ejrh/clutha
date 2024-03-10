use std::collections::HashSet;
use std::sync::Arc;

use serenity::{async_trait, Error};
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::all::standard::CommandResult;
use serenity::framework::standard::Configuration;
use serenity::framework::{Framework, StandardFramework};
use serenity::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

use crate::bot::Bot;
use crate::commands::*;

struct Handler;

pub(crate) struct ShardManagerContainer(Arc<ShardManager>);

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub(crate) struct BotContainer(Arc<Bot>);

impl TypeMapKey for BotContainer {
    type Value = Bot;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let trimmed = msg.content.trim_start();
        if trimmed.starts_with('~') {
            return
        }

        let mut data = ctx.data.write().await;
        let Some(bot) = data.get_mut::<BotContainer>()
        else {
            error!("Couldn't get bot object!");
            return
        };

        match bot.handle_dialogue(&ctx, &msg).await {
            Ok(_) => (),
            Err(why) => {
                error!("Could not handle dialogue: {:?}", why);
            },
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub(crate) async fn create_framework(token: &str) -> Result<StandardFramework, Error> {
    let http = Http::new(token);
    let info = http.get_current_application_info().await?;
    let owners: HashSet<_> = info.owner.iter().map(|u| u.id).collect();

    let framework = StandardFramework::new()
        .group(&ADMIN_GROUP)
        .group(&GENERAL_GROUP)
        .group(&PROMPT_GROUP)
        .help(&MY_HELP);
    framework.configure(Configuration::new().owners(owners).prefix("~"));

    Ok(framework)
}

pub(crate) async fn run_bot(bot: Bot, token: &str) -> Result<(), Error> {
    let framework = create_framework(token).await?;

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

pub(crate) async fn system_message(ctx: &Context, msg: &Message, text: &str) -> CommandResult {
    let mut embed = CreateEmbed::new().description(text);
    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}

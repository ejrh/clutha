use std::collections::HashSet;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Configuration;
use serenity::framework::StandardFramework;
use serenity::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::commands::*;
use crate::dialogue::{Dialogue, handle_dialogue};

struct Handler;

pub(crate) struct ShardManagerContainer(Arc<ShardManager>);

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub(crate) struct DialogueContainer(Arc<Dialogue>);

impl TypeMapKey for DialogueContainer {
    type Value = Dialogue;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let trimmed = msg.content.trim_start();
        if trimmed.starts_with('~') {
            return
        }

        match handle_dialogue(&ctx, &msg).await {
            Ok(_) => (),
            Err(why) => {
                println!("Could not handle dialogue: {:?}", why);
            },
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping, version, shutdown)]
struct General;

pub(crate) async fn create_framework(token: &str) -> StandardFramework {
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().owners(owners).prefix("~"));

    framework
}

pub(crate) async fn run_bot(gemini_api_key: &str, token: &str) -> Result<(), serenity::Error> {
    let framework = create_framework(token).await;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<DialogueContainer>(Dialogue::new(gemini_api_key));
    }

    client.start().await?;

    Ok(())
}

use std::collections::{HashMap, HashSet};

use serenity::all::{CreateEmbed, Message, MessageBuilder, PartialGuild};
use serenity::builder::CreateMessage;
use serenity::client::Context;
use itertools::Itertools;

use crate::bot::Bot;
use crate::discord::system_message;
use crate::discord::ShardManagerContainer;

type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[async_trait::async_trait]
trait CommandFn {
    async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult;
}

pub(crate) struct Command {
    category: String,
    description: String,
    parameters: Vec<String>,
    preset_parameters: Vec<String>,
    function: Box<dyn CommandFn + Send + Sync>,
}

pub(crate) struct Invocation<'inv> {
    command: &'inv str,
    parameters: Vec<&'inv str>,
    bot: &'inv Bot,
    ctx: &'inv Context,
    message: &'inv Message,
}

mod fns {
    use crate::discord::system_info;
    use super::*;

    pub(crate) struct Shutdown;

    #[async_trait::async_trait]
    impl CommandFn for Shutdown {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            system_message(inv.ctx, inv.message.channel_id, "Shutting down").await?;

            let data = inv.ctx.data.read().await;
            let Some(shard_manager) = data.get::<ShardManagerContainer>() else {
                return Err("Couldn't get shard manager object!".into());
            };

            shard_manager.shutdown_all().await;

            Ok(())
        }
    }

    pub(crate) struct Version;

    #[async_trait::async_trait]
    impl CommandFn for Version {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            const VERSION: &str = env!("CARGO_PKG_VERSION");

            system_info(inv.ctx, inv.message.channel_id, &format!("Clutha version {VERSION}")).await?;

            Ok(())
        }
    }

    pub(crate) struct Ping;

    #[async_trait::async_trait]
    impl CommandFn for Ping {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            let channel_id = inv.message.channel_id;
            let author = &inv.message.author;
            let channel = channel_id.to_channel(&inv.ctx).await?;

            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            system_message(inv.ctx, channel_id, &response).await?;

            Ok(())
        }
    }

    pub(crate) struct Reset;

    #[async_trait::async_trait]
    impl CommandFn for Reset {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            let state = inv.bot.channel_state(inv.ctx, inv.message.channel_id).await?;
            state.lock().await.reset_dialogue();

            system_message(inv.ctx, inv.message.channel_id, "Dialogue reset").await?;

            Ok(())
        }
    }

    pub(crate) struct Info;

    #[async_trait::async_trait]
    impl CommandFn for Info {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            let state = inv.bot.channel_state(inv.ctx, inv.message.channel_id).await?;
            let state = state.lock().await;

            let mut context = MessageBuilder::new();
            if inv.message.guild_id.is_none() {
                context.push("Private chat with ").mention(&inv.message.author);
            } else {
                let channel = inv.message.channel_id.to_channel(inv.ctx).await?;
                let guild_name = if let Some(gid) = inv.message.guild_id {
                    let guild = PartialGuild::get(inv.ctx, gid).await?;
                    guild.name
                } else {
                    "???".to_string()
                };
                context
                    .push("Channel ")
                    .mention(&channel)
                    .push(" on server ")
                    .push_bold_safe(&guild_name);
            };

            let mode_str = format!("{:?}", state.mode);
            let prompt_str = &state.prompt.filename;

            let embed = CreateEmbed::new()
                .description(context.build())
                .color((0, 255, 0))
                .field("Mode", mode_str, true)
                .field("Prompt", prompt_str, true)
                .field("Prompt size", format!("{}", state.prompt.prompt.total_len), true)
                .field(
                    "Dialogue size",
                    format!("{} / {}", state.dialogue.total_len, state.dialogue.max_len),
                    true,
                );

            let builder = CreateMessage::default().embed(embed);
            inv.message.channel_id.send_message(inv.ctx, builder).await?;

            Ok(())
        }
    }

    pub(crate) struct Mode;

    #[async_trait::async_trait]
    impl CommandFn for Mode {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
            let mode = *inv.parameters.get(0).ok_or("No mode specified")?;
            let state = inv.bot.channel_state(inv.ctx, inv.message.channel_id).await?;
            let mut state = state.lock().await;

            let new_mode: crate::channel::Mode = mode.try_into()
                .map_err(|_| "Invalid mode")?;

            state.mode = new_mode;

            system_message(inv.ctx, inv.message.channel_id, format!("Mode set to *{new_mode:?}*").as_str()).await?;

            Ok(())
        }
    }

    pub(crate) struct Prompt;

    #[async_trait::async_trait]
    impl CommandFn for Prompt {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {
             let prompt_name = *inv.parameters.get(0).ok_or("No prompt specified")?;
            let needs_response = inv.bot.set_prompt(inv.ctx, inv.message.channel_id, prompt_name).await?;

            system_message(inv.ctx, inv.message.channel_id, format!("Prompt set to *{prompt_name:?}*").as_str()).await?;

            if needs_response {
                inv.bot.do_ai_response(inv.ctx, inv.message.channel_id, None).await?;
            }

            Ok(())
        }
    }

    pub(crate) struct Help;

    #[async_trait::async_trait]
    impl CommandFn for Help {
        async fn call<'e>(&self, inv: Invocation<'e>) -> CommandResult {

            let categories = inv.bot.commands.values()
                .map(|c| &c.category)
                .collect::<HashSet<_>>();

            let mut embed = CreateEmbed::new().title("Available commands").color((0, 255, 0));

            for category in categories.iter().sorted() {
                let commands = inv.bot.commands.iter().filter(|(_, c)| &c.category == *category).collect::<Vec<_>>();
                if commands.is_empty() { continue }

                let mut list = String::new();
                for (n, c) in commands.iter().sorted_by_key(|(n, _)| n) {
                    list.push_str(&format!("  ~{} {}: {}\n", n, c.parameters.join(" "), c.description));
                }

                embed = embed.field(*category, list, false);
            }

            let builder = CreateMessage::default().embed(embed);
            inv.message.channel_id.send_message(inv.ctx, builder).await?;

            Ok(())
        }
    }
}

macro_rules! commands {
    (
        $( $name:ident $( = $class:tt )? ( $( $bits:tt )* ) ),* $(,)?
    ) => {
        HashMap::from([
            $( commands!(@line $name $( = $class )? ( $( $bits )* )) ),*
        ])
    };

    (
        @line $class:tt ( $( $bits:tt )* )
    ) => {
        commands!(@item @nm=(stringify!($class).to_lowercase().to_string()) $class ( $( $bits )* ))
    };

    (
        @line $name:ident = $class:tt ( $( $bits:tt )* )
    ) => {
        commands!(@item @nm=(stringify!($name).to_string()) $class ( $( $bits )* ))
    };

    (
        @item @nm=($name:expr) $class:tt ( $cat:expr, $desc:expr )
    ) => {
        commands!(@nm=($name) @ps=() $class ( $cat, $desc ))
    };

    (
        @item @nm=($name:expr) $class:tt ( $( $p:expr ),+ ; $cat:expr, $desc:expr )
    ) => {
        commands!(@nm=($name) @ps=($( $p ),*) $class ( $cat, $desc ))
    };

    (
        @nm=($name:expr) @ps=($( $params:expr ),*) $class:tt ( $cat:expr, $desc:expr )
    ) => {
        ($name, Command {
                category: stringify!($cat).to_string(),
                description: $desc.to_string(),
                parameters: vec![],
                preset_parameters: vec![$( ($params).to_string(), )*],
                function: Box::new(fns::$class),
            })
    };
}

pub(crate) fn commands() -> HashMap<String, Command> {
    commands! {
        Shutdown(Admin, "Shut the Bot down"),
        Version(General, "Information about the Bot version"),
        Ping(General, "Ping the Bot"),
        Reset(General, "Reset the dialogue for this channel"),
        Info(General, "Information about this channel's dialogue"),
        Mode(General, "Set the channel mode"),
        Prompt(General, "Set the channel prompt"),
        Help(General, "Information about commands"),

        default = Prompt("default"; Prompt, "Use the 'default' prompt"),
        about = Prompt("about"; Prompt, "Use the 'about' prompt"),
    }
}

pub(crate) async fn run_command(bot: &Bot, ctx: &Context, message: &Message) -> CommandResult {
    let words: Vec<_> = message.content.split(' ').collect();
    let command_name = words[0][1..].to_lowercase();
    let cmd = bot.commands.get(&command_name).ok_or(format!("No command found: {command_name}"))?;
    let func = &cmd.function;

    let presets = cmd.preset_parameters.iter().map(|p| p.as_ref());
    let params = words[1..].iter().map(|p| *p);
    let params: Vec<_> = presets.chain(params).collect();

    let invocation = Invocation {
        command: &command_name,
        parameters: params,
        bot,
        ctx,
        message,
    };

    let result = func.call(invocation).await;

    result
}

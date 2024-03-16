use std::sync::Arc;

use poise::builtins::HelpConfiguration;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::all::{CreateEmbed, PartialGuild};
use serenity::framework::Framework;
use serenity::utils::MessageBuilder;
use tokio::sync::Mutex;

use crate::bot::Bot;

pub(crate) struct Data {
    bot: Arc<Mutex<Bot>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;
type CommandResult = Result<(), Error>;

#[poise::command(
    prefix_command,
    category = "Admin"
)]
async fn shutdown(ctx: Context<'_>) -> CommandResult {
    system_message(ctx, "Shutting down").await?;

    ctx.framework().shard_manager.shutdown_all().await;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "General"
)]
async fn version(ctx: Context<'_>) -> CommandResult {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    system_message(ctx, &format!("Clutha version {VERSION}")).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "General"
)]
async fn ping(ctx: Context<'_>) -> CommandResult {
    let channel_id = ctx.channel_id();
    let author = ctx.author();
    let channel = channel_id.to_channel(&ctx).await?;

    let response = MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&author.name)
        .push(" used the 'ping' command in the ")
        .mention(&channel)
        .push(" channel")
        .build();

    system_message(ctx, &response).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "General"
)]
async fn reset(ctx: Context<'_>) -> CommandResult {
    let mut bot = ctx.data().bot.lock().await;
    bot.dialogue.reset();

    system_message(ctx, "Dialogue reset").await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "General"
)]
async fn info(ctx: Context<'_>) -> CommandResult {
    let bot = ctx.data().bot.lock().await;

    let mut context = MessageBuilder::new();
    if ctx.guild_id().is_none() {
        context.push("Private chat with ").mention(ctx.author());
    } else {
        let channel = ctx.channel_id().to_channel(&ctx).await?;
        let guild_name = if let Some(gid) = ctx.guild_id() {
            let guild = PartialGuild::get(&ctx.http(), gid).await?;
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

    let mode_str = "active";
    let prompt_str = "default";

    let embed = CreateEmbed::new()
        .description(context.build())
        .field("Mode", mode_str, true)
        .field("Prompt", prompt_str, true)
        .field(
            "Dialogue size",
            format!("{} / {}", bot.dialogue.total_len, bot.dialogue.max_len),
            true,
        );

    let builder = CreateReply::default().embed(embed);
    ctx.send(builder).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Prompt"
)]
async fn default(ctx: Context<'_>) -> CommandResult {
    let mut bot = ctx.data().bot.lock().await;
    bot.set_prompt(ctx, "default").await?;

    system_message(ctx, "Prompt set to *default*").await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Prompt"
)]
async fn about(ctx: Context<'_>) -> CommandResult {
    let mut bot = ctx.data().bot.lock().await;
    bot.set_prompt(ctx, "about").await?;

    system_message(ctx, "Prompt set to *about*").await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "General"
)]
async fn help(
    ctx: Context<'_>,
    #[rest]
    command: Option<String>,
) -> CommandResult {
    let extra_text_at_bottom = "\
Type `?help command` for more info on a command.
You can edit your `?help` message to the bot and the bot will edit its response.";

    let config = HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

pub(crate) async fn system_message(ctx: Context<'_>, text: &str) -> CommandResult {
    let embed = CreateEmbed::new().description(text);
    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;
    Ok(())
}

pub fn create_framework(bot: Arc<Mutex<Bot>>) -> Result<impl Framework, serenity::Error> {
    let data = Data { bot };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                shutdown(),
                version(),
                ping(),
                reset(),
                info(),
                help(),
                default(),
                about(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    Ok(framework)
}

use std::process::ExitCode;
use tracing::error;

use crate::bot::Bot;
use crate::gemini::Gemini;

mod bot;
mod channel;
mod commands;
mod dialogue;
mod discord;
mod gemini;
mod prompt;

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let Ok(api_key) = std::env::var("GEMINI_API_KEY") else {
        error!("GEMINI_API_KEY not set in environment");
        return ExitCode::FAILURE;
    };

    let Ok(token) = std::env::var("DISCORD_TOKEN") else {
        error!("DISCORD_TOKEN not set in environment");
        return ExitCode::FAILURE;
    };

    let gemini = Gemini::new(&api_key);
    let bot = Bot { gemini, channels: Default::default() };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        let result = discord::run_bot(bot, &token).await;
        if let Err(err) = result {
            error!("Clutha bot finished with error: {err}");
        }
    });

    ExitCode::SUCCESS
}

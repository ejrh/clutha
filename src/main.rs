use crate::bot::Bot;
use crate::dialogue::Dialogue;
use crate::gemini::Gemini;

mod bot;
mod commands;
mod dialogue;
mod discord;
mod gemini;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build().unwrap();

    let api_key = std::env::var("GEMINI_API_KEY").unwrap();

    let token = std::env::var("DISCORD_TOKEN").unwrap();

    let gemini = Gemini::new(&api_key);
    let dialogue = Dialogue::new();
    let bot = Bot { gemini, dialogue };

    rt.block_on(async {
        println!("{:?}", discord::run_bot(bot, &token).await);
    });
}

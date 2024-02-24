mod commands;
mod dialogue;
mod discord;
mod gemini;

fn main() {
    let api_key = std::env::var("GEMINI_KEY").unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build().unwrap();

    let token = std::env::var("DISCORD_TOKEN").unwrap();

    rt.block_on(async {
        println!("{:?}", discord::run_bot(&api_key, &token).await);
    });
}

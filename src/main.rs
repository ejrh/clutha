mod discord;
mod gemini;

fn main() {
    println!("Clutha");

    let api_key = std::env::var("GEMINI_KEY").unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("{}", gemini::generate_content(&api_key, "Hello Clutha").await.unwrap());
    });

    let token = std::env::var("DISCORD_TOKEN").unwrap();

    rt.block_on(async {
        println!("{:?}", discord::say_hello(&token).await);
    });
}

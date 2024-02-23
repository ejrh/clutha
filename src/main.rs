mod gemini;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("hello");
        println!("{}", gemini::generate_content("Hello Clutha").await.unwrap());
    });
}

use std::collections::VecDeque;
use serenity::all::{Context, Message};
use serenity::framework::standard::CommandResult;
use crate::discord::DialogueContainer;
use crate::gemini::generate_content;

pub(crate) struct Dialogue {
    api_key: String,
    parts: VecDeque<String>,
}

impl Dialogue {
    pub(crate) fn new(api_key: &str) -> Dialogue {
        Dialogue {
            api_key: api_key.to_string(),
            parts: VecDeque::new(),
        }
    }

    pub(crate) fn push(&mut self, str: &str) {
        self.parts.push_back(str.to_string());
    }

    pub(crate) fn assemble(&self) -> String {
        let mut result = String::new();
        for part in &self.parts {
            result.push_str(part);
            result.push('\n');
        }
        result
    }
}

pub(crate) async fn handle_dialogue(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let Some(dialogue) = data.get_mut::<DialogueContainer>()
        else { return Ok(()) };

    let channel_name = msg.channel_id.name(ctx).await?;
    if channel_name != "chatter-bot" { return Ok(()) }

    let user_name = &msg.author.name;
    if user_name == "Clutha" { return Ok(()) }

    let text = &msg.content;
    dialogue.push(text);

    println!("### {}", text);

    let prompt = dialogue.assemble();
    let result = generate_content(&dialogue.api_key, &prompt).await?;

    println!(">>> {}\n", result);

    msg.channel_id.say(&ctx, result).await?;

    Ok(())
}

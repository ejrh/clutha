use std::collections::VecDeque;

use serenity::all::{Context, Message};
use serenity::framework::standard::CommandResult;

use crate::discord::DialogueContainer;
use crate::gemini::generate_content;

pub(crate) struct Part {
    role: String,
    text: String,
}

pub(crate) struct Dialogue {
    api_key: String,
    parts: VecDeque<Part>,
    total_len: u64,
    max_len: u64
}

const MAXIMUM_DIALOGUE_LEN: u64 = 1_000;

impl Dialogue {
    pub(crate) fn new(api_key: &str) -> Dialogue {
        Dialogue {
            api_key: api_key.to_string(),
            parts: VecDeque::new(),
            total_len: 0,
            max_len: MAXIMUM_DIALOGUE_LEN,
        }
    }

    pub(crate) fn push(&mut self, role: &str, text: &str) {
        let part = Part { role: role.to_string(), text: text.to_string() };
        self.total_len += part.len();
        self.parts.push_back(part);
        self.truncate_to_size();
    }

    fn truncate_to_size(&mut self) {
        while self.total_len > self.max_len {
            let Some(part) = self.parts.pop_front()
                else { break };
            self.total_len -= part.len();
        }
    }

    pub(crate) fn assemble_prompt(&self) -> Vec<(String, String)> {
        let mut prompt = Vec::new();
        for part in &self.parts {
            prompt.push((part.role.clone(), part.text.clone()));
        }
        prompt
    }
}

impl Part {
    fn len(&self) -> u64 {
        let text = self.text.trim();
        let words = text.split(' ').collect::<Vec<_>>();
        words.len() as u64
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
    dialogue.push("user", text);

    println!("### {}", text);

    // Release the lock
    drop(data);

    do_ai_response(ctx, msg).await?;

    Ok(())
}

async fn do_ai_response(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let Some(dialogue) = data.get_mut::<DialogueContainer>()
        else { return Ok(()) };

    let typing = msg.channel_id.start_typing(&ctx.http);

    let prompt = dialogue.assemble_prompt();
    let result = generate_content(&dialogue.api_key, prompt).await?;

    dialogue.push("model", &result);

    println!(">>> {}\n", result);

    msg.channel_id.say(&ctx, result).await?;

    typing.stop();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dialogue_len_and_truncation() {
        let mut d = Dialogue::new("mock key");
        let big_str = "test ".repeat(400);
        let part = Part { role: "t".to_string(), text: big_str.clone() };
        assert_eq!(400, part.len());
        d.push("t", &big_str.clone());
        assert_eq!(400, d.total_len);
        d.push("t", &big_str.clone());
        assert_eq!(800, d.total_len);
        d.push("t", &big_str.clone());
        assert_eq!(800, d.total_len);
    }
}

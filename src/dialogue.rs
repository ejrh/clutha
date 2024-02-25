use std::collections::VecDeque;
use serenity::all::{Context, Message};
use serenity::framework::standard::CommandResult;
use crate::discord::DialogueContainer;
use crate::gemini::generate_content;

pub(crate) struct Dialogue {
    api_key: String,
    parts: VecDeque<String>,
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

    pub(crate) fn push(&mut self, part: &str) {
        self.total_len += part_len(part);
        self.parts.push_back(part.to_string());
        self.truncate_to_size();
        println!("pushed to {}", self.total_len);
    }

    fn truncate_to_size(&mut self) {
        while self.total_len > self.max_len {
            let Some(popped_part) = self.parts.pop_front()
                else { break };
            self.total_len -= part_len(&popped_part);
            println!("popped to {}", self.total_len);
        }
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

fn part_len(part: &str) -> u64 {
    let part = part.trim();
    let words = part.split(' ').collect::<Vec<_>>();
    words.len() as u64
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dialogue_len_and_truncation() {
        let mut d = Dialogue::new("mock key");
        let big_str = "test ".repeat(400);
        assert_eq!(400, part_len(&big_str));
        d.push(&big_str.clone());
        assert_eq!(400, d.total_len);
        d.push(&big_str.clone());
        assert_eq!(800, d.total_len);
        d.push(&big_str.clone());
        assert_eq!(800, d.total_len);
    }
}

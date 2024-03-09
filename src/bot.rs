use serenity::all::{Context, Message};
use serenity::all::standard::CommandResult;
use tracing_subscriber::registry::Data;

use crate::dialogue::Dialogue;
use crate::gemini::Gemini;

pub(crate) struct Bot {
    pub(crate) gemini: Gemini,
    pub(crate) dialogue: Dialogue,
}

impl Bot {
    pub(crate) async fn handle_dialogue(&mut self, ctx: &Context, msg: &Message) -> CommandResult {
        let channel_name = msg.channel_id.name(ctx).await?;
        if channel_name != "chatter-bot" { return Ok(()) }

        let user_name = &msg.author.name;
        if user_name == "Clutha" { return Ok(()) }

        let text = &msg.content;
        self.dialogue.push("user", text);

        println!("### {}", text);

        self.do_ai_response(ctx, msg).await?;

        Ok(())
    }

    async fn do_ai_response(&mut self, ctx: &Context, msg: &Message) -> CommandResult {
        let typing = msg.channel_id.start_typing(&ctx.http);

        let prompt = assemble_prompt(&self.dialogue);
        let result = self.gemini.generate_content(prompt).await?;

        self.dialogue.push("model", &result);

        println!(">>> {}\n", result);

        let result_segments = prepare_response(result);
        for segment in result_segments {
            msg.channel_id.say(&ctx, segment).await?;
        }

        typing.stop();

        Ok(())
    }
}

fn assemble_prompt(dialogue: &Dialogue) -> Vec<(String, String)> {
    let mut prompt = Vec::new();
    for part in &dialogue.parts {
        prompt.push((part.role.clone(), part.text.clone()));
    }
    prompt
}

// This seems to be Discord's limit; make our limit slightly smaller to allow to overhead
const DISCORD_MAX_SEGMENT_SIZE: usize = 2000;
const MAX_SEGMENT_SIZE: usize = DISCORD_MAX_SEGMENT_SIZE - 100;

fn prepare_response(result: String) -> Vec<String> {
    if result.len() < MAX_SEGMENT_SIZE {
        return vec![result];
    }

    let groups = crate::dialogue::split_result(result, MAX_SEGMENT_SIZE);
    crate::dialogue::merge_groups(groups, MAX_SEGMENT_SIZE)
}

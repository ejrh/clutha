use serenity::all::{Context, Message};
use serenity::all::standard::CommandResult;

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

        let prompt = self.dialogue.assemble_prompt();
        let result = self.gemini.generate_content(prompt).await?;

        self.dialogue.push("model", &result);

        println!(">>> {}\n", result);

        let result_segments = crate::dialogue::prepare_result(result);
        for segment in result_segments {
            msg.channel_id.say(&ctx, segment).await?;
        }

        typing.stop();

        Ok(())
    }
}
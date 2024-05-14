use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serenity::all::standard::CommandResult;
use serenity::all::{ChannelId, Context, Message};
use tokio::sync::Mutex;

use crate::channel::State;
use crate::dialogue::{Dialogue, Part};
use crate::gemini::Gemini;
use crate::prompt::load_prompt;

pub(crate) struct Bot {
    pub(crate) gemini: Gemini,
    pub(crate) channels: RefCell<HashMap<ChannelId, Arc<Mutex<State>>>>,
}

impl Bot {
    pub(crate) async fn handle_dialogue(&mut self, ctx: &Context, msg: &Message) -> CommandResult {
        let channel_name = msg.channel_id.name(ctx).await?;
        if channel_name != "chatter-bot" {
            return Ok(());
        }

        let user_name = &msg.author.name;
        if user_name == "Clutha" {
            return Ok(());
        }

        let text = &msg.content;

        let state = self.channel_state(msg.channel_id);
        state.lock().await.process_user_text(text);

        println!("### {}", text);

        self.do_ai_response(ctx, msg).await?;

        Ok(())
    }

    async fn do_ai_response(&mut self, ctx: &Context, msg: &Message) -> CommandResult {
        let state = self.channel_state(msg.channel_id);
        let mut state = state.lock().await;

        let typing = msg.channel_id.start_typing(&ctx.http);

        let prompt = state.assemble_prompt();
        let result = self.gemini.generate_content(prompt).await?;

        state.process_model_text(&result);

        println!(">>> {}\n", result);

        let result_segments = prepare_response(result);
        for segment in result_segments {
            msg.channel_id.say(&ctx, segment).await?;
        }

        typing.stop();

        Ok(())
    }

    pub(crate) async fn set_prompt(
        &mut self,
        ctx: crate::commands::Context<'_>,
        prompt_name: &str,
    ) -> CommandResult {
        let mut path = PathBuf::new();
        path.push("prompts/");
        path.push(format!("{}.txt", prompt_name));
        let prompt = load_prompt(&path)?;

        let state = self.channel_state(ctx.channel_id());
        let mut state = state.lock().await;
        state.dialogue.append(&prompt.prompt);
        state.dialogue.append(&prompt.initial);

        let x = prompt.initial.parts.iter().last();
        if let Some(Part { role, text }) = x {
            if role == "model" {
                ctx.channel_id().say(&ctx, text).await?;
            }
        }

        Ok(())
    }

    pub(crate) fn channel_state(&self, channel_id: ChannelId) -> Arc<Mutex<State>> {
        let mut channels = self.channels.borrow_mut();
        channels.entry(channel_id)
            .or_insert_with(|| Arc::new(Mutex::new(State {
                dialogue: Dialogue::new(),
            }))).clone()
    }
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

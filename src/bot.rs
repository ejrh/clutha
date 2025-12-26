use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serenity::all::standard::CommandResult;
use serenity::all::{CacheHttp, Channel, ChannelId, ChannelType, Context, Message, MessageId};
use serenity::builder::CreateThread;
use tokio::sync::Mutex;
use tracing::info;

use crate::backend::Backend;
use crate::channel::{Mode, State};
use crate::dialogue::{Dialogue, Part};
use crate::prompt::{load_prompt, Prompt};

pub(crate) struct Bot {
    pub(crate) backend: Box<dyn Backend>,
    pub(crate) channels: Arc<Mutex<HashMap<ChannelId, Arc<Mutex<State>>>>>,
}

impl Bot {
    pub(crate) async fn handle_dialogue(&mut self, ctx: &Context, msg: &Message) -> CommandResult {
        let state = self.channel_state(ctx, msg.channel_id).await?;
        let mut state = state.lock().await;

        // TODO - some mentions are mentioning the role of the same name, and it would be
        //  nice to pick those up, too
        let mentions_me = msg.mentions_me(ctx).await.unwrap_or(false);

        if !self.should_process(ctx, msg, &state, mentions_me) {
            return Ok(());
        }

        let text = &msg.content;
        state.process_user_text(text);
        println!("### {}", text);

        if !self.should_respond(ctx, msg, &state, mentions_me) {
            return Ok(())
        }

        // We have to drop the lock on state, as the next function will acquire it again
        drop(state);

        self.do_ai_response(ctx, msg.channel_id, Some(&msg)).await
    }

    pub async fn do_ai_response(&mut self, ctx: &Context, channel_id: ChannelId, original_msg: Option<&Message>) -> CommandResult {
        let state = self.channel_state(ctx, channel_id).await?;
        let mut state = state.lock().await;

        let typing = channel_id.start_typing(&ctx.http);

        let prompt = state.assemble_prompt();
        let result = match self.backend.generate_content(prompt).await {
            Ok(result) => result,
            Err(err) => {
                channel_id.say(&ctx, format!("Error: {err:?}")).await?;
                return Err(err.into());
            }
        };

        let result_segments = prepare_response(&result);
        let mut dest_channel = channel_id;

        /* If the response length is large, put the response in a thread */
        let is_thread = matches!(
            channel_id.to_channel(ctx).await?.guild().map(|g| g.kind),
            Some(ChannelType::PublicThread | ChannelType::PrivateThread)
        );
        let total_len = result_segments.iter().map(|s| s.len()).sum::<usize>();
        let create_thread = !is_thread && total_len > 200 && original_msg.is_some();

        if create_thread {
            info!("creating thread");
            let msg = original_msg.unwrap();
            let mut first_line = result_segments[0].replace('\n', " ");
            //TODO truncate could panic if there is a multibyte character
            first_line.truncate(100);
            let r = channel_id.create_thread_from_message(ctx, msg.id, CreateThread::new(first_line)).await?;
            let thread_id = r.id;

            /* Create a new state for the thread, based on the channel state */
            let state2 = self.channel_state(ctx, thread_id).await?;
            let mut state2 = state2.lock().await;
            //state2.clone_from(state);
            state2.mode = Mode::Active;

            state2.process_model_text(&result);
            dest_channel = r.id;
        } else {
            state.process_model_text(&result);
        }

        println!(">>> {}\n", result);

        for segment in result_segments {
            dest_channel.say(&ctx, segment).await?;
        }

        typing.stop();

        Ok(())
    }

    fn should_process(&self, ctx: &Context, msg: &Message, state: &State, mentions_me: bool) -> bool {
        if msg.is_own(ctx) {
            return false;
        }

        match state.mode {
            Mode::Off => false,
            Mode::Passive => mentions_me,
            Mode::Lurking => true,
            Mode::Active => true,
        }
    }

    fn should_respond(&self, _ctx: &Context, _msg: &Message, state: &State, mentions_me: bool) -> bool {
        match state.mode {
            Mode::Off => false,
            Mode::Passive => mentions_me,
            Mode::Lurking => mentions_me,
            Mode::Active => true,
        }
    }

    pub(crate) async fn set_prompt(
        &mut self,
        ctx: &Context,
        channel_id: ChannelId,
        prompt_name: &str,
    ) -> CommandResult<bool> {
        let mut path = PathBuf::new();
        path.push("prompts/");
        path.push(format!("{}.txt", prompt_name));
        let prompt = load_prompt(&path)?;

        let state = self.channel_state(ctx, channel_id).await?;
        let mut state = state.lock().await;

        state.set_prompt(&prompt);

        /* Check if the last item in the prompt was from the user */
        let needs_response = match prompt.initial.parts.iter().last() {
            Some(Part { role, text }) => role == "user",
            None => false,
        };

        Ok(needs_response)
    }

    pub(crate) async fn channel_state(&self, cache: impl CacheHttp, channel_id: ChannelId) -> serenity::Result<Arc<Mutex<State>>> {
        let mut channels = self.channels.lock().await;
        if let Some(channel) = channels.get(&channel_id) { return Ok(channel.clone()) };

        let channel = self.new_channel_state(cache, channel_id).await?;
        let channel = Arc::new(Mutex::new(channel));
        channels.insert(channel_id, channel.clone());
        Ok(channel)
    }

    pub(crate) async fn new_channel_state(&self, cache: impl CacheHttp, channel_id: ChannelId) -> serenity::Result<State> {
        let channel = channel_id.to_channel(cache).await?;
        let mode = match &channel {
            Channel::Guild(gc) if gc.thread_metadata.is_none() => Mode::Active,
            Channel::Guild(_) => Mode::Lurking,
            Channel::Private(_) => Mode::Active,
            _ => Mode::Passive,
        };
        let prompt = load_prompt("prompts/default.txt")?;
        let mut state = State {
            mode,
            prompt: Prompt::default(),
            dialogue: Dialogue::new(),
        };
        state.set_prompt(&prompt);
        Ok(state)
    }
}

// This seems to be Discord's limit; make our limit slightly smaller to allow to overhead
const DISCORD_MAX_SEGMENT_SIZE: usize = 2000;
const MAX_SEGMENT_SIZE: usize = DISCORD_MAX_SEGMENT_SIZE - 100;

fn prepare_response(result: &str) -> Vec<String> {
    if result.len() < MAX_SEGMENT_SIZE {
        return vec![result.to_string()];
    }

    let groups = crate::dialogue::split_result(result, MAX_SEGMENT_SIZE);
    crate::dialogue::merge_groups(groups, MAX_SEGMENT_SIZE)
}

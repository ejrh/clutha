use itertools::Itertools;
use crate::dialogue::{Dialogue, MAXIMUM_DIALOGUE_LEN};
use crate::prompt::Prompt;

/// Channel mode; when does the bot respond to messages in a channel
#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    /// Ignores all non-command messages
    Off,
    /// Reads and responds only when mentioned
    Passive,
    /// Reads all but responds only when mentioned
    Lurking,
    /// Reads and responds to all
    Active,
}

impl TryFrom<&str> for Mode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        let mode = match value.as_str() {
            "off" => Mode::Off,
            "lurk" | "lurking" => Mode::Lurking,
            "passive" => Mode::Passive,
            "active" => Mode::Active,
            _ => { return Err(()) }
        };

        Ok(mode)
    }
}

pub(crate) struct State {
    pub(crate) mode: Mode,
    pub(crate) prompt: Prompt,
    pub(crate) dialogue: Dialogue,
}

impl State {
    pub(crate) fn process_user_text(&mut self, text: &str) {
        self.dialogue.push("user", text);
    }

    pub(crate) fn process_model_text(&mut self, text: &str) {
        self.dialogue.push("model", text);
    }

    pub(crate) fn set_prompt(&mut self, prompt: &Prompt) {
        self.dialogue.max_len = MAXIMUM_DIALOGUE_LEN - prompt.prompt.total_len;
        self.dialogue.append(&prompt.initial);
        self.prompt = prompt.clone();
    }

    pub(crate) fn assemble_prompt(&self) -> Vec<(String, String)> {
        let mut prompt = Vec::new();
        let combined_prompt = self.prompt.prompt.parts.iter().chain(self.dialogue.parts.iter());
        for (key, group) in combined_prompt
                .group_by(|p| &p.role).into_iter() {
            let text = group.map(|p| &p.text).join("\n\n");
            prompt.push((key.clone(), text));
        }
        prompt
    }

    pub(crate) fn reset_dialogue(&mut self) {
        self.dialogue.reset();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_assemble_prompt() {
        let mut state = State { mode: Mode::Passive, prompt: Prompt::default(), dialogue: Dialogue::new() };
        state.dialogue.push("user", "ab");
        state.dialogue.push("user", "cd");
        state.dialogue.push("model", "ef");
        state.dialogue.push("model", "gh");

        let prompt = state.assemble_prompt();
        let expected: Vec<(String, String)> = vec![
            ("user".into(), "ab\n\ncd".into()),
            ("model".into(), "ef\n\ngh".into()),
        ];
        assert_eq!(expected, prompt);
    }
}

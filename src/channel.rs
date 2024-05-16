use itertools::Itertools;
use crate::dialogue::Dialogue;

/// Channel mode; when does the bot respond to messages in a channel
#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    /// Ignores all non-command messages
    Off,
    /// Reads but does not respond
    Lurking,
    /// Responds when mentioned
    Passive,
    /// Responds to all
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
    pub(crate) dialogue: Dialogue,
}

impl State {
    pub(crate) fn process_user_text(&mut self, text: &str) {
        self.dialogue.push("user", text);
    }

    pub(crate) fn process_model_text(&mut self, text: &str) {
        self.dialogue.push("model", text);
    }

    pub(crate) fn assemble_prompt(&self) -> Vec<(String, String)> {
        let mut prompt = Vec::new();
        for (key, group) in self.dialogue.parts.iter()
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
        let mut state = State { mode: Mode::Passive, dialogue: Dialogue::new() };
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

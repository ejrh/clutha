use itertools::Itertools;
use crate::dialogue::Dialogue;

pub(crate) struct State {
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
        let mut state = State { dialogue: Dialogue::new() };
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

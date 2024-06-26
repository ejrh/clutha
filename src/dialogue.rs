use std::collections::VecDeque;
use std::io::BufRead;
use std::mem::take;

#[derive(Clone)]
pub(crate) struct Part {
    pub(crate) role: String,
    pub(crate) text: String,
}

#[derive(Clone, Default)]
pub(crate) struct Dialogue {
    pub(crate) parts: VecDeque<Part>,
    pub(crate) total_len: u64,
    pub(crate) max_len: u64,
}

pub(crate) const MAXIMUM_DIALOGUE_LEN: u64 = 1_000;

impl Dialogue {
    pub(crate) fn new() -> Dialogue {
        Dialogue {
            parts: VecDeque::new(),
            total_len: 0,
            max_len: MAXIMUM_DIALOGUE_LEN,
        }
    }

    pub(crate) fn push(&mut self, role: &str, text: &str) {
        let part = Part {
            role: role.to_string(),
            text: text.to_string(),
        };
        self.total_len += part.len();
        self.parts.push_back(part);
        self.truncate_to_size();
    }

    pub(crate) fn append(&mut self, other: &Dialogue) {
        for part in &other.parts {
            self.push(&part.role, &part.text);
        }
    }

    fn truncate_to_size(&mut self) {
        while self.total_len > self.max_len {
            let Some(part) = self.parts.pop_front() else {
                break;
            };
            self.total_len -= part.len();
        }
    }

    pub(crate) fn reset(&mut self) {
        self.parts.clear();
        self.total_len = 0;
    }
}

impl Part {
    fn len(&self) -> u64 {
        let text = self.text.trim();
        let words = text.split(' ').collect::<Vec<_>>();
        words.len() as u64
    }
}

pub(crate) fn split_result(result: String, max_size: usize) -> Vec<String> {
    /* Specific lines of the input are identified as split points:
        - Blank lines (outside of code blocks, and not following headings).
        - Code block lines starting ```; included with following group or current group
          depending on parity.
    */

    #[derive(Default)]
    struct Grouper {
        groups: Vec<String>,
        current_group: String,
    }

    impl Grouper {
        fn push(&mut self, line: &str) {
            self.current_group.push_str(line);
            self.current_group.push('\n');
        }

        fn end_group(&mut self) {
            let group = take(&mut self.current_group);
            if !group.trim().is_empty() {
                self.groups.push(group);
            }
        }
    }

    let mut grouper = Grouper::default();
    let mut after_heading = false;
    let mut in_code_block = false;

    for line in result.lines() {
        if line.is_empty() && !after_heading && !in_code_block {
            grouper.push(line);
            grouper.end_group();
        } else if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                grouper.end_group();
            }
            grouper.push(line);
            if !in_code_block {
                grouper.end_group();
            }
        } else {
            after_heading = line.starts_with('*') && line.ends_with('*');
            grouper.push(line);
        }
    }
    grouper.end_group();

    grouper.groups
}

pub(crate) fn merge_groups(groups: Vec<String>, max_size: usize) -> Vec<String> {
    let mut segments = Vec::new();
    let mut merged_group = String::new();
    for group in groups {
        if merged_group.len() + group.len() > max_size {
            if !merged_group.is_empty() {
                segments.push(take(&mut merged_group));
            }
            if group.len() > max_size {
                merged_group.push_str("(Group too big to send!)");
                continue;
            }
        }
        merged_group.push_str(&group);
    }
    if !merged_group.is_empty() {
        segments.push(merged_group);
    }

    segments
}

pub(crate) fn read_dialogue(f: &mut impl BufRead) -> Result<Dialogue, std::io::Error> {
    let mut dialogue = Dialogue::new();

    let mut dialogue_text = String::new();
    loop {
        let mut buf = String::new();
        let num_read = f.read_line(&mut buf)?;

        if num_read == 0 {
            break;
        }
        if buf.starts_with("---") {
            break;
        }

        dialogue_text.push_str(&buf);
    }

    for text in split_result(dialogue_text, 10000) {
        let (role, text) = if text.starts_with('>') {
            ("user", &text[2..])
        } else {
            ("model", text.as_str())
        };

        dialogue.push(role, &text);
    }

    Ok(dialogue)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dialogue_len_and_truncation() {
        let mut d = Dialogue::new();
        let big_str = "test ".repeat(400);
        let part = Part {
            role: "t".to_string(),
            text: big_str.clone(),
        };
        assert_eq!(400, part.len());
        d.push("t", &big_str.clone());
        assert_eq!(400, d.total_len);
        d.push("t", &big_str.clone());
        assert_eq!(800, d.total_len);
        d.push("t", &big_str.clone());
        assert_eq!(800, d.total_len);
    }

    #[test]
    fn test_split_result() {
        let input = "".to_string();
        let segments = split_result(input, 0);
        assert_eq!(0, segments.len());

        let input = "one line\n\n".to_string();
        let segments = split_result(input, 0);
        assert_eq!(vec!["one line\n\n"], segments);

        let input = "one\nsegment".to_string();
        let segments = split_result(input, 0);
        assert_eq!(vec!["one\nsegment\n"], segments);

        let input = "now\ntwo\n\nsegments".to_string();
        let segments = split_result(input, 0);
        assert_eq!(vec!["now\ntwo\n\n", "segments\n"], segments);

        let input = "line 1\n\n* heading *\n\nline 2".to_string();
        let segments = split_result(input, 0);
        assert_eq!(vec!["line 1\n\n", "* heading *\n\nline 2\n"], segments);

        let input = "* bullet point\n\ntext".to_string();
        let segments = split_result(input, 0);
        assert_eq!(vec!["* bullet point\n\n", "text\n"], segments);

        let input = "text\n```rust\nsome\n\ncode\n\nhere\n```\nmore text".to_string();
        let segments = split_result(input, 0);
        assert_eq!(
            vec![
                "text\n",
                "```rust\nsome\n\ncode\n\nhere\n```\n",
                "more text\n"
            ],
            segments
        );
    }

    #[test]
    fn test_merge_groups() {
        let groups = vec!["group1\n\n".into(), "group2".into()];
        let segments = merge_groups(groups.clone(), 10);
        assert_eq!(vec!["group1\n\n", "group2"], segments);

        let segments = merge_groups(groups, 20);
        assert_eq!(vec!["group1\n\ngroup2"], segments);
    }

    #[test]
    fn test_read_dialogue() {
        const TEST_DIALOGUE: &str = "> Hello
there\n
Hello back
to you";

        let mut stream = TEST_DIALOGUE.as_bytes();
        let dialogue = read_dialogue(&mut stream).unwrap();

        assert_eq!(2, dialogue.parts.len());

        let first_part = &dialogue.parts[0];
        assert_eq!("user", first_part.role);
        assert_eq!("Hello\nthere\n\n", first_part.text);

        let second_part = &dialogue.parts[1];
        assert_eq!("model", second_part.role);
        assert_eq!("Hello back\nto you\n", second_part.text);
    }
}

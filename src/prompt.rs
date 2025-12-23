use std::io::BufReader;
use std::path::Path;

use crate::dialogue::{read_dialogue, Dialogue};

#[derive(Clone, Default)]
pub(crate) struct Prompt {
    pub(crate) prompt: Dialogue,
    pub(crate) initial: Dialogue,
    pub(crate) filename: String,
}

pub(crate) fn load_prompt(path: impl AsRef<Path>) -> Result<Prompt, std::io::Error> {
    let filename = path.as_ref().as_os_str().to_str().unwrap_or("").to_owned();
    let f = std::fs::File::open(path)?;
    let mut f = BufReader::new(f);

    let prompt = read_dialogue(&mut f)?;
    let initial = read_dialogue(&mut f)?;

    Ok(Prompt {
        prompt,
        initial,
        filename,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_prompt() {
        let p = load_prompt("prompts/about.txt").unwrap();

        assert_eq!("prompts/about.txt", p.filename);
        assert_eq!(304, p.prompt.total_len);
        assert_eq!(2, p.initial.total_len);
    }
}

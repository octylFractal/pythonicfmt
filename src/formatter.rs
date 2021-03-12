use once_cell::sync::Lazy;
use regex::Regex;
use std::cmp::max;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Formatter {
    junk_column: usize,
}

// Note: we do set ops on this string, so it should stay small
// If it gets too large for some reason, it might be worth making a set
const JUNK_CHARS: &str = "{};";
static WS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\s").unwrap());
// Matches junk and whitespace from the end of the string
static END_JUNK_WS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(&*format!("[{}{}]+$", "\\s", regex::escape(JUNK_CHARS))).unwrap());

impl Formatter {
    pub fn junk_column(&mut self, junk_column: usize) -> &mut Self {
        self.junk_column = junk_column;
        self
    }

    // Thanks to &mut String, this could be optimized later
    // For now I'll be super un-optimal :)
    pub fn format(&self, content: &mut String) -> Result<()> {
        let mut lines = content
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        self.collapse_lines(&mut lines);
        for line in lines.iter_mut() {
            if let Some(junk_and_ws) = END_JUNK_WS_REGEX.find(&line.clone()) {
                let junk_str = WS_REGEX.replace_all(junk_and_ws.as_str(), "");
                if junk_str.is_empty() {
                    // We don't need to touch this, it's just trailing whitespace
                    continue;
                }
                let size_before = junk_and_ws.start();
                let space_count = max(0, self.junk_column - size_before);
                line.replace_range(
                    size_before..,
                    &*(format!("{}{}", " ".repeat(space_count), junk_str,)),
                );
            }
        }

        *content = lines.join("\n");

        Ok(())
    }

    // Merges junk and whitespace only lines to previous lines
    fn collapse_lines(&self, lines: &mut Vec<String>) {
        let mut index = 1;
        while index < lines.len() {
            let line = &mut lines[index];
            let line_without_ws = WS_REGEX.replace_all(&line, "").to_string();
            if line_without_ws.chars().all(|c| JUNK_CHARS.contains(c)) {
                lines[index - 1] += &*line_without_ws;
                lines.remove(index);
                index -= 1;
            }

            index += 1;
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Formatter { junk_column: 120 }
    }
}

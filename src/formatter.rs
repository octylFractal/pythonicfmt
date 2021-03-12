use once_cell::sync::Lazy;
use regex::Regex;
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
static BASE_JUNK_WS_REGEX: Lazy<String> =
    Lazy::new(|| format!("[{}{}]", "\\s", regex::escape(JUNK_CHARS)));
// Matches junk and whitespace from the start of the string
static START_JUNK_WS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&*format!(
        "^\\s+([{}]{}*)",
        regex::escape(JUNK_CHARS),
        &*BASE_JUNK_WS_REGEX,
    ))
    .unwrap()
});
// Matches junk and whitespace from the end of the string
static END_JUNK_WS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(&*format!("{}+$", &*BASE_JUNK_WS_REGEX)).unwrap());

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
        // Iterate backwards so all start-of-line is resolved prior to end-of-line
        let mut idx = lines.len() - 1;
        loop {
            self.process_line(&mut lines, idx);
            if idx == 0 {
                break;
            }
            idx -= 1;
        }

        *content = lines.join("\n");

        Ok(())
    }

    fn process_line(&self, lines: &mut Vec<String>, idx: usize) {
        let mut prev_line_modification: Option<String> = None;
        let line = &mut lines[idx];
        // Move start-of-line to previous line's end-of-line
        if let Some(captures) = START_JUNK_WS_REGEX.captures(&line.clone()) {
            let junk_and_ws = captures.get(1).unwrap();
            let junk_str = WS_REGEX.replace_all(junk_and_ws.as_str(), "");
            line.replace_range(junk_and_ws.range(), "");
            prev_line_modification = Some(junk_str.to_string());
        }
        // Move end-of-line outwards
        if let Some(junk_and_ws) = END_JUNK_WS_REGEX.find(&line.clone()) {
            let junk_str = WS_REGEX.replace_all(junk_and_ws.as_str(), "");
            if junk_str.is_empty() {
                // We don't need to touch this, it's just trailing whitespace
                return;
            }
            let size_before = junk_and_ws.start();
            let space_count = self.junk_column.saturating_sub(size_before);
            line.replace_range(
                size_before..,
                &*(format!("{}{}", " ".repeat(space_count), junk_str,)),
            );
        }

        if let Some(modification) = prev_line_modification {
            lines[idx - 1] += &*modification;
        }
    }

    // Merges junk and whitespace only lines to previous lines
    fn collapse_lines(&self, lines: &mut Vec<String>) {
        let mut index = 1;
        while index < lines.len() {
            let line = &mut lines[index];
            let line_without_ws = WS_REGEX.replace_all(&line, "").to_string();
            if !line_without_ws.is_empty()
                && line_without_ws.chars().all(|c| JUNK_CHARS.contains(c))
            {
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

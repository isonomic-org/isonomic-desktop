//! What never goes in a report.
//!
//! - Paths under the home folder become `~/…`.
//! - Any quoted text longer than [`QUOTED_LIMIT`] characters becomes
//!   `[text removed]`: a panic message can carry a value the person typed.
//! - A message is cut at [`MESSAGE_LIMIT`] characters.
//!
//! The person also sees the exact report before anything could be sent.

/// Quoted text longer than this is removed.
pub const QUOTED_LIMIT: usize = 40;
/// A report's message is cut at this many characters.
pub const MESSAGE_LIMIT: usize = 300;
const REMOVED: &str = "[text removed]";

/// Scrub one line of text.
pub fn line(text: &str, home: Option<&str>) -> String {
    let homed = match home.filter(|h| h.len() > 1) {
        Some(home) => text.replace(home, "~"),
        None => text.to_string(),
    };
    quoted(&homed)
}

/// Scrub a report's message: [`line`], then cut to [`MESSAGE_LIMIT`].
pub fn message(text: &str, home: Option<&str>) -> String {
    let scrubbed = line(text, home);
    if scrubbed.chars().count() <= MESSAGE_LIMIT {
        return scrubbed;
    }
    let cut: String = scrubbed.chars().take(MESSAGE_LIMIT).collect();
    format!("{cut}…")
}

/// Replace long quoted runs (`"…"` or `'…'`) with [`REMOVED`].
fn quoted(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' || c == '\'' {
            if let Some(len) = chars[i + 1..].iter().position(|&d| d == c) {
                if len > QUOTED_LIMIT {
                    out.push(c);
                    out.push_str(REMOVED);
                    out.push(c);
                    i += len + 2;
                    continue;
                }
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: Option<&str> = Some("/Users/omar");

    #[test]
    fn home_paths_become_a_tilde() {
        assert_eq!(
            line(
                "opened /Users/omar/Library/Application Support/Isonomic",
                HOME
            ),
            "opened ~/Library/Application Support/Isonomic"
        );
    }

    #[test]
    fn a_prompt_in_a_panic_message_is_removed() {
        let typed = "called `Option::unwrap()` on a value: \"Plan one day in Lisbon for my sister's birthday next week\"";
        let scrubbed = message(typed, HOME);
        assert!(!scrubbed.contains("Lisbon"), "{scrubbed}");
        assert!(scrubbed.contains("[text removed]"));
        // Short quoted words, such as a key name, stay.
        assert_eq!(line("missing key 'model'", HOME), "missing key 'model'");
    }

    #[test]
    fn a_long_message_is_cut() {
        let long = "x ".repeat(400);
        let cut = message(&long, None);
        assert_eq!(cut.chars().count(), MESSAGE_LIMIT + 1);
        assert!(cut.ends_with('…'));
    }

    #[test]
    fn an_unclosed_quote_is_left_as_text() {
        assert_eq!(line("it's fine", None), "it's fine");
    }
}

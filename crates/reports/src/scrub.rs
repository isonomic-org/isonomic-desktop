//! What never goes in a report.
//!
//! - Paths under the home folder become `~/…`.
//! - Any quoted text longer than [`QUOTED_LIMIT`] characters becomes
//!   `[text removed]`: a panic message can carry a value the person typed.
//! - A run of [`KEY_LENGTH`] or more key characters holding at least
//!   [`KEY_DIGITS`] digits becomes `[key removed]`: tokens, keys, digests
//!   and the person's ID never go in a report. Model and file names have
//!   too few digits to match.
//! - A message is cut at [`MESSAGE_LIMIT`] characters.
//!
//! The engine keeps the same rules (isonomic-capsule `crates/reports`).
//! Both move to the SDK's shared crate once it exists.
//!
//! The person also sees the exact report before anything could be sent.

/// Quoted text longer than this is removed.
pub const QUOTED_LIMIT: usize = 40;
/// A report's message is cut at this many characters.
pub const MESSAGE_LIMIT: usize = 300;
/// A run this long may be a key.
pub const KEY_LENGTH: usize = 24;
/// A run with this many digits is taken for a key.
pub const KEY_DIGITS: usize = 6;
const REMOVED: &str = "[text removed]";
const KEY_REMOVED: &str = "[key removed]";

/// Scrub one line of text.
pub fn line(text: &str, home: Option<&str>) -> String {
    let homed = match home.filter(|h| h.len() > 1) {
        Some(home) => text.replace(home, "~"),
        None => text.to_string(),
    };
    keys(&quoted(&homed))
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

fn key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '/' | '=')
}

/// Replace runs that look like keys with [`KEY_REMOVED`].
fn keys(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        let digits = run.chars().filter(|c| c.is_ascii_digit()).count();
        if run.chars().count() >= KEY_LENGTH && digits >= KEY_DIGITS {
            out.push_str(KEY_REMOVED);
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in text.chars() {
        if key_char(c) {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
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

    /// The engine's own cases, so the two scrubbers match.
    #[test]
    fn keys_tokens_and_ids_are_removed_but_names_stay() {
        let token = "token iso_tok_7f3a9c21b04e5d6f8a1b2c3d4e5f6a7b refused";
        assert_eq!(line(token, None), "token [key removed] refused");
        let id = "did:iso:3f9c0a7e1b2d4c5f8e6a7b9c0d1e2f3a3f9c0a7e1b2d4c5f8e6a7b9c0d1e2f3a";
        assert_eq!(line(id, None), "did:iso:[key removed]");
        let name = "loaded qwen3.5-4b-instruct-q4_k_m.gguf on llama-server b12387";
        assert_eq!(line(name, None), name);
    }
}

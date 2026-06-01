use std::{collections::HashMap, ops::Range};

fn is_abbreviation_boundary(c: char) -> bool {
    c.is_whitespace() || matches!(c, '(' | ')' | '{' | '}' | '|' | ';')
}

/// A successful abbreviation expansion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbbreviationExpansion {
    /// Byte range in the input line to replace.
    pub replace_range: Range<usize>,
    /// Text that replaces [`Self::replace_range`].
    pub replacement: String,
    /// Optional byte offset within [`Self::replacement`] where the cursor should
    /// be placed after expansion.
    pub cursor: Option<usize>,
}

/// The user action that requested abbreviation expansion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbbreviationTrigger {
    /// The user typed a space.
    Space,
    /// The user submitted the buffer with Enter/Submit.
    Submit,
}

/// A trait for fish-style abbreviation expansion.
///
/// Reedline calls [`Abbreviator::expand_at_cursor`] when the user presses Space
/// or Enter after typing a token. Before applying a candidate expansion,
/// Reedline asks the configured [`Highlighter`](crate::Highlighter) whether
/// expansion should be suppressed for that position.
///
/// `HashMap<String, String>` implements this trait for simple position-agnostic
/// expansion. Shells with a proper parser can implement this trait directly and
/// return the parser-selected replacement range.
pub trait Abbreviator: Send {
    /// Try to expand the token immediately before `cursor` in `line`.
    fn expand_at_cursor(
        &self,
        line: &str,
        cursor: usize,
        trigger: AbbreviationTrigger,
    ) -> Option<AbbreviationExpansion>;
}

impl Abbreviator for HashMap<String, String> {
    fn expand_at_cursor(
        &self,
        line: &str,
        cursor: usize,
        trigger: AbbreviationTrigger,
    ) -> Option<AbbreviationExpansion> {
        if !line.is_char_boundary(cursor) {
            return None;
        }

        let word_start = line[..cursor]
            .char_indices()
            .rev()
            .find(|(_, c)| is_abbreviation_boundary(*c))
            .map_or(0, |(i, c)| i + c.len_utf8());

        let text_after = &line[cursor..];
        if !text_after.is_empty() && !text_after.starts_with(is_abbreviation_boundary) {
            return None;
        }

        let token = &line[word_start..cursor];
        if token.is_empty() {
            return None;
        }

        let mut replacement = self.get(token)?.clone();
        if trigger == AbbreviationTrigger::Space {
            replacement.push(' ');
        }

        Some(AbbreviationExpansion {
            replace_range: word_start..cursor,
            replacement,
            cursor: None,
        })
    }
}

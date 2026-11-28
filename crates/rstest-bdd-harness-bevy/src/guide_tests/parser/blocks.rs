//! Block-start recognition for the fence scanner's lazy-continuation rule.
//!
//! `CommonMark` lets a list item's paragraph continue on an unindented line,
//! but a line that opens a new block ends the item instead. The scanner cannot
//! represent a lazy continuation, so [`starts_block`] decides which of the two
//! a line is: a new block ends the item normally, and anything else is rejected
//! rather than silently dropping the item's fence.
//!
//! The forms recognized are the ones this scanner models or must not misreport:
//! a list item, a block quote, a fence, an `ATX` heading, a thematic break, and
//! an HTML block.

use super::containers::{Line, list_marker};

/// Whether `text` is blank in `CommonMark`'s sense.
///
/// Only ASCII spaces and tabs count. `str::trim` would also accept Unicode
/// spaces such as the non-breaking space, which `CommonMark` reads as content.
pub(super) fn is_ascii_blank(text: &str) -> bool {
    text.bytes().all(|byte| matches!(byte, b' ' | b'\t'))
}

/// Whether `line` is blank once its block-quote prefixes are gone.
///
/// A quoted blank line is written `>` rather than left empty, so trimming the
/// raw text would call it content and read the next line as continuing a
/// paragraph that the blank line actually ended.
pub(super) fn is_blank_after_prefixes(line: Line<'_>) -> bool {
    let mut remaining = line.as_str();
    loop {
        remaining = remaining.trim_start_matches([' ', '\t']);
        let Some(after) = remaining.strip_prefix('>') else {
            break;
        };
        remaining = after;
    }
    is_ascii_blank(remaining)
}

/// Whether `line` begins a block that may interrupt a paragraph.
pub(super) fn starts_block(line: Line<'_>) -> bool {
    let Some(trimmed) = content_after_indent(line) else {
        return false;
    };
    opens_container(line) || opens_text_block(trimmed)
}

/// The line's text after up to three spaces, or `None` when indented deeper.
///
/// Four spaces begin an indented code block, which cannot interrupt a
/// paragraph.
fn content_after_indent(line: Line<'_>) -> Option<&str> {
    let text = line.as_str();
    let trimmed = text.trim_start_matches(' ');
    (text.len() - trimmed.len() <= 3).then_some(trimmed)
}

/// Whether `line` opens a container rather than paragraph text.
fn opens_container(line: Line<'_>) -> bool {
    list_marker(line).is_some() || line.strip_quote_marker().is_some()
}

/// Whether `trimmed` opens a leaf block that interrupts a paragraph.
fn opens_text_block(trimmed: &str) -> bool {
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    first == '<'
        || is_thematic_break(trimmed)
        || (matches!(first, '`' | '~') && starts_fence(trimmed))
        || (first == '#' && is_atx_heading(trimmed))
}

/// Whether `text` carries a fence delimiter of three or more characters.
fn starts_fence(text: &str) -> bool {
    let Some(character) = text.chars().next() else {
        return false;
    };
    text.chars().take_while(|c| *c == character).count() >= 3
}

/// Whether `text` is an `ATX` heading: one to six `#` then a space or the end.
fn is_atx_heading(text: &str) -> bool {
    let hashes = text.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return false;
    }
    let rest = text.get(hashes..).unwrap_or_default();
    rest.is_empty() || rest.starts_with([' ', '\t'])
}

/// Whether `text` is a thematic break of three or more matching marks.
pub(super) fn is_thematic_break(text: &str) -> bool {
    let Some(character) = text.chars().next().filter(|c| matches!(c, '-' | '*' | '_')) else {
        return false;
    };
    let marks = text.chars().filter(|c| *c == character).count();
    marks >= 3 && text.chars().all(|c| c == character || c == ' ')
}

//! List-item containment and marker recognition for the fence scanner.
//!
//! `CommonMark` measures indentation relative to a list item's content, so the
//! scanner must know which items contain the current line before it can tell a
//! fence from an indented code block. This module owns that stack and the
//! marker recognition that opens each item.

/// One open list item whose content begins `content_indent` spaces in.
struct ListItem {
    content_indent: usize,
}

/// Stack of list items containing the current line.
#[derive(Default)]
pub(super) struct ListContainers {
    items: Vec<ListItem>,
}

impl ListContainers {
    /// Matches open items against `line`, closing any it is dedented out of.
    ///
    /// A tab in the indentation of a line inside a list item is rejected
    /// rather than guessed at: partial tab-stop arithmetic would either hide
    /// a fence or invent one. The guide uses spaces, so failing closed keeps
    /// the checker honest for the layout it actually receives.
    pub(super) fn match_indent<'a>(&mut self, line: &'a str) -> Result<&'a str, String> {
        if line.trim().is_empty() {
            return Ok(line);
        }
        if leading_whitespace_has_tab(line) {
            return Err(
                "a tab-indented line inside a list item is not supported; use spaces".to_owned(),
            );
        }
        let mut rest = line;
        let mut matched = 0;
        for item in &self.items {
            let Some(after) = strip_indent(rest, item.content_indent) else {
                break;
            };
            rest = after;
            matched += 1;
        }
        self.items.truncate(matched);
        Ok(rest)
    }

    pub(super) fn len(&self) -> usize { self.items.len() }

    pub(super) fn push(&mut self, content_indent: usize) {
        self.items.push(ListItem { content_indent });
    }
}

/// Whether the leading whitespace of `line` contains a tab.
fn leading_whitespace_has_tab(line: &str) -> bool {
    line.chars()
        .take_while(|character| character.is_whitespace())
        .any(|character| character == '\t')
}

/// Consumes exactly `count` leading spaces.
fn strip_indent(text: &str, count: usize) -> Option<&str> {
    let mut rest = text;
    for _ in 0..count {
        rest = rest.strip_prefix(' ')?;
    }
    Some(rest)
}

/// Spaces a tab advances from `column`, with tab stops every four columns.
///
/// `CommonMark` measures indentation in columns, so a tab after a list marker
/// pads content to the next tab stop rather than by a fixed width. The low two
/// bits give the column's offset within its stop; the estate lint that denies
/// division and remainder operators is why the offset is taken that way.
fn tab_stop_padding(column: usize) -> usize { 4 - (column & 3) }

/// Recognizes a `CommonMark` list marker.
///
/// Returns the content indentation relative to `line` and the content that
/// follows the marker on the same line. Bullet markers are one character;
/// ordered markers are one to nine digits plus `.` or `)`. Either form must be
/// followed by whitespace or the end of the line, so `-foo` and `1.foo` are
/// not list items.
pub(super) fn list_marker(line: &str) -> Option<(usize, &str)> {
    let (marker_len, rest) = list_marker_kind(line)?;
    let (padding, content) = marker_padding(line, rest)?;
    Some((marker_len + padding, content))
}

/// Splits the padding after a marker from the item's content.
fn marker_padding<'a>(line: &str, rest: &'a str) -> Option<(usize, &'a str)> {
    if rest.is_empty() {
        // An empty item still reserves the marker's virtual trailing space.
        return Some((1, rest));
    }
    if let Some(after_tab) = rest.strip_prefix('\t') {
        let column = line.len() - rest.len();
        return Some((tab_stop_padding(column), after_tab));
    }
    space_padding(rest)
}

/// Splits space padding from the item's content.
fn space_padding(rest: &str) -> Option<(usize, &str)> {
    let spaces = rest.bytes().take_while(|byte| *byte == b' ').count();
    if spaces == 0 {
        return None;
    }
    // `CommonMark` uses one space of padding when five or more follow, so the
    // remaining spaces indent content inside the item.
    let padding = if spaces > 4 { 1 } else { spaces };
    Some((padding, rest.get(padding..)?))
}

/// Splits the marker itself from the text after it.
fn list_marker_kind(line: &str) -> Option<(usize, &str)> {
    let first = line.chars().next()?;
    if matches!(first, '-' | '+' | '*') {
        let rest = line.strip_prefix(first)?;
        return is_marker_terminator(rest).then_some((1, rest));
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if !(1..=9).contains(&digits) {
        return None;
    }
    let after_digits = line.get(digits..)?;
    let delimiter = after_digits.chars().next()?;
    if !matches!(delimiter, '.' | ')') {
        return None;
    }
    let rest = after_digits.get(delimiter.len_utf8()..)?;
    is_marker_terminator(rest).then_some((digits + 1, rest))
}

/// Whether a marker is followed by whitespace or the end of the line.
fn is_marker_terminator(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')
}

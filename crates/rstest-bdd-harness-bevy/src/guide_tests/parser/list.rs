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
    pub(super) fn match_indent<'a>(&mut self, line: Line<'a>) -> Result<Line<'a>, String> {
        if line.is_blank() {
            return Ok(line);
        }
        if line.has_tab_indent() {
            return Err(
                "a tab-indented line inside a list item is not supported; use spaces".to_owned(),
            );
        }
        let mut rest = line;
        let mut matched = 0;
        for item in &self.items {
            let Some(after) = rest.strip_spaces(item.content_indent) else {
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

/// A line fragment measured from a list item's content column.
///
/// Every helper below asks the same kind of question of the fragment: how many
/// spaces lead it, which marker opens it, what follows. Carrying those
/// operations on the fragment keeps bare strings out of the module's internal
/// calls, so a fragment from one container is not passed where another
/// container's fragment belongs.
#[derive(Clone, Copy)]
pub(super) struct Line<'a> {
    text: &'a str,
}

impl<'a> Line<'a> {
    pub(super) fn new(text: &'a str) -> Self { Self { text } }

    pub(super) fn as_str(self) -> &'a str { self.text }

    fn is_blank(self) -> bool { self.text.trim().is_empty() }

    fn is_empty(self) -> bool { self.text.is_empty() }

    fn len(self) -> usize { self.text.len() }

    fn first_char(self) -> Option<char> { self.text.chars().next() }

    /// Whether the leading whitespace contains a tab.
    fn has_tab_indent(self) -> bool {
        self.text
            .chars()
            .take_while(|character| character.is_whitespace())
            .any(|character| character == '\t')
    }

    fn strip_char(self, prefix: char) -> Option<Self> {
        self.text.strip_prefix(prefix).map(Self::new)
    }

    /// Consumes exactly `count` leading spaces.
    fn strip_spaces(self, count: usize) -> Option<Self> {
        let mut rest = self;
        for _ in 0..count {
            rest = rest.strip_char(' ')?;
        }
        Some(rest)
    }

    /// Splits at `count` bytes, failing when the fragment is shorter.
    fn skip_bytes(self, count: usize) -> Option<Self> { self.text.get(count..).map(Self::new) }

    fn leading_spaces(self) -> usize { self.text.bytes().take_while(|byte| *byte == b' ').count() }

    fn leading_digits(self) -> usize { self.text.bytes().take_while(u8::is_ascii_digit).count() }

    /// Whether a marker is followed by whitespace or the end of the line.
    fn is_marker_terminator(self) -> bool {
        self.is_empty() || self.text.starts_with(' ') || self.text.starts_with('\t')
    }
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
pub(super) fn list_marker(line: Line<'_>) -> Option<(usize, Line<'_>)> {
    let (marker_len, rest) = list_marker_kind(line)?;
    let (padding, content) = marker_padding(line, rest)?;
    Some((marker_len + padding, content))
}

/// Splits the padding after a marker from the item's content.
fn marker_padding<'a>(line: Line<'a>, rest: Line<'a>) -> Option<(usize, Line<'a>)> {
    if rest.is_empty() {
        // An empty item still reserves the marker's virtual trailing space.
        return Some((1, rest));
    }
    if let Some(after_tab) = rest.strip_char('\t') {
        let column = line.len() - rest.len();
        return Some((tab_stop_padding(column), after_tab));
    }
    space_padding(rest)
}

/// Splits space padding from the item's content.
fn space_padding(rest: Line<'_>) -> Option<(usize, Line<'_>)> {
    let spaces = rest.leading_spaces();
    if spaces == 0 {
        return None;
    }
    // `CommonMark` uses one space of padding when five or more follow, so the
    // remaining spaces indent content inside the item.
    let padding = if spaces > 4 { 1 } else { spaces };
    Some((padding, rest.skip_bytes(padding)?))
}

/// Splits the marker itself from the text after it.
fn list_marker_kind(line: Line<'_>) -> Option<(usize, Line<'_>)> {
    let first = line.first_char()?;
    if matches!(first, '-' | '+' | '*') {
        let rest = line.strip_char(first)?;
        return rest.is_marker_terminator().then_some((1, rest));
    }
    let digits = line.leading_digits();
    if !(1..=9).contains(&digits) {
        return None;
    }
    let after_digits = line.skip_bytes(digits)?;
    let delimiter = after_digits.first_char()?;
    if !matches!(delimiter, '.' | ')') {
        return None;
    }
    let rest = after_digits.skip_bytes(delimiter.len_utf8())?;
    rest.is_marker_terminator().then_some((digits + 1, rest))
}

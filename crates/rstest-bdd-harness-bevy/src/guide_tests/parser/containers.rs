//! Container tracking and list-marker recognition for the fence scanner.
//!
//! `CommonMark` nests block quotes and list items in the order they open, and
//! each level measures the next one's indentation from its own content column.
//! A list inside a block quote and a block quote inside a list therefore need
//! opposite prefix handling. The scanner keeps one stack in opening order, so a
//! line is matched against each container from outermost to innermost, and a
//! container that fails to match ends there together with everything inside it.

/// One open container with the identity a fence records.
#[derive(Clone, Copy)]
struct Entry {
    kind: Kind,
    /// Stable identity, unique for the parser's lifetime.
    ///
    /// A fence remembers the id of its innermost container. Because an id is
    /// never reused, comparing ids distinguishes "still inside the same
    /// container" from "a sibling container of the same depth", which a count
    /// alone cannot do.
    id: usize,
}

/// The kind of container an entry represents.
#[derive(Clone, Copy)]
enum Kind {
    /// A block quote, matched by a `>` marker on the line.
    Quote,
    /// A list item, matched by indentation to its content column.
    Item { content_indent: usize },
}

/// Stack of containers open at the current line, in opening order.
#[derive(Default)]
pub(super) struct Containers {
    entries: Vec<Entry>,
    next_id: usize,
}

impl Containers {
    /// Matches open containers against `line`, ending those it has left.
    ///
    /// A tab in the line's own indentation is rejected before any container is
    /// matched, because it would make the indentation each container measures
    /// ambiguous. Tabs exposed only after a container prefix are caught by the
    /// scanner as it strips the next prefix.
    pub(super) fn match_line<'a>(&mut self, line: Line<'a>) -> Result<Line<'a>, String> {
        let mut rest = line.reject_tab_indent()?;
        let matched = self.match_open_containers(&mut rest);
        self.entries.truncate(matched);
        Ok(rest)
    }

    /// Counts the open containers the line still belongs to.
    ///
    /// The first container the line has left ends the walk, because everything
    /// nested inside it is left with it. `rest` advances past each matched
    /// container so the next level measures from the correct column.
    fn match_open_containers(&self, rest: &mut Line<'_>) -> usize {
        let mut matched = 0;
        for entry in &self.entries {
            let Some(after) = match_entry(*entry, *rest) else {
                break;
            };
            *rest = after;
            matched += 1;
        }
        matched
    }

    /// The identity of the innermost open container, or zero at the top level.
    pub(super) fn innermost(&self) -> usize { self.entries.last().map_or(0, |entry| entry.id) }

    /// Container identities open at the current line, outermost first.
    pub(super) fn path(&self) -> Vec<usize> { self.entries.iter().map(|entry| entry.id).collect() }

    pub(super) fn quote_depth(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.kind, Kind::Quote))
            .count()
    }

    pub(super) fn list_depth(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.kind, Kind::Item { .. }))
            .count()
    }

    pub(super) fn open_quote(&mut self) { self.push(Kind::Quote); }

    pub(super) fn open_item(&mut self, content_indent: usize) {
        self.push(Kind::Item { content_indent });
    }

    fn push(&mut self, kind: Kind) {
        self.next_id += 1;
        self.entries.push(Entry {
            kind,
            id: self.next_id,
        });
    }
}

/// Matches one open container against `line`, consuming its prefix.
///
/// Returns the remaining fragment, or `None` when the line has left the
/// container. A blank line satisfies a list item without consuming anything:
/// the item stays open and resumes if a later line reaches its content column.
fn match_entry(entry: Entry, line: Line<'_>) -> Option<Line<'_>> {
    let Kind::Item { content_indent } = entry.kind else {
        return line.strip_quote_marker();
    };
    if line.is_blank() {
        return Some(line);
    }
    line.strip_spaces(content_indent)
}

/// A line fragment measured from a container's content column.
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

    pub(super) fn is_blank(self) -> bool { self.text.trim().is_empty() }

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

    /// Rejects a fragment whose indentation starts with a tab.
    ///
    /// `CommonMark` measures indentation in columns, and partial tab-stop
    /// arithmetic would either hide a fence or invent one. The guide uses
    /// spaces, so failing closed keeps the checker honest for the layout it
    /// actually receives. The check applies after every prefix strip, because
    /// a tab is harmless as a block quote's marker padding but ambiguous as
    /// the indentation of anything nested inside it.
    pub(super) fn reject_tab_indent(self) -> Result<Self, String> {
        if self.has_tab_indent() {
            return Err("a tab-indented line is not supported; use spaces".to_owned());
        }
        Ok(self)
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

    /// Strips a block-quote marker with up to three leading spaces.
    fn strip_quote_marker(self) -> Option<Self> {
        let leading = self.leading_spaces();
        if leading > 3 {
            return None;
        }
        let after = self.skip_bytes(leading)?.strip_char('>')?;
        Some(after.strip_quote_padding())
    }

    /// Strips the optional space or tab that may follow a block-quote marker.
    ///
    /// `CommonMark` locates content by column, with tab stops every four
    /// columns, so `>\t` begins a quoted line whose content is validly
    /// indented. Treating the tab as the marker's padding keeps a following
    /// fence delimiter visible to the checker; leaving it in place would hide
    /// the fence.
    pub(super) fn strip_quote_padding(self) -> Self {
        self.strip_char(' ')
            .or_else(|| self.strip_char('\t'))
            .unwrap_or(self)
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

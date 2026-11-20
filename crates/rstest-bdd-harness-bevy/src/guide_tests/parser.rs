//! `CommonMark` fence scanner for the users' guide.
//!
//! The scanner recognizes both fenced-code forms rustdoc accepts, backtick and
//! tilde, indented by up to three spaces relative to their container, at the
//! top level, inside block quotes, and inside list items. It also enforces the
//! documentation style guide's marker placement: every fence must be preceded
//! by a `tested-example` marker on the line immediately above it at the same
//! container level, blank lines may separate the two, prose may not, and a
//! contained fence must stay contained until it closes. A shallower line,
//! including a blank one, ends a block quote; a dedented line ends a list item
//! and leaves its fence unterminated.

#[path = "parser/list.rs"]
mod list;

use list::{Line, ListContainers, list_marker};

/// One fenced block and the marker immediately above it, if any.
#[derive(Debug)]
pub(super) struct Fence<'a> {
    pub(super) language: &'a str,
    pub(super) marker: Option<&'a str>,
}

/// A fence delimiter: the repeated character and the length of its run.
#[derive(Clone, Copy, Debug)]
struct Delimiter {
    character: char,
    length: usize,
}

impl Delimiter {
    /// Whether `self`, found on a candidate closing line, closes `opener`.
    ///
    /// `CommonMark` requires the same character and at least the opener's run
    /// length. A closing fence may carry only trailing spaces; any other text
    /// makes the line content rather than a boundary.
    fn closes(self, opener: Self, trailing: &str) -> bool {
        self.character == opener.character
            && self.length >= opener.length
            && trailing.trim().is_empty()
    }
}

/// An open fence awaiting its closing delimiter.
struct OpenFence<'a> {
    /// Block-quote nesting depth at the opener.
    ///
    /// Only a delimiter at the same depth closes the block; a quoted line
    /// inside a top-level fence is content. While the block is open, every
    /// line must keep at least this depth, because a shallower line ends the
    /// container and with it the fence.
    depth: usize,
    /// Number of list items containing the opener.
    ///
    /// A delimiter outside those items cannot close their fence. The count is
    /// deliberately a depth rather than the exact marker positions: a sibling
    /// item at the same depth is outside the fence's item, so the containment
    /// check rejects the dedent before a sibling's delimiter is considered.
    list_depth: usize,
    delimiter: Delimiter,
    language: &'a str,
    marker: Option<&'a str>,
}

/// Splits the guide into fenced blocks, enforcing marker placement.
///
/// Blank lines may separate a marker from its fence, as the Markdown
/// formatter inserts one. Prose may not: a marker separated from its fence by
/// prose could appear to cover a different block.
pub(super) fn parse_fences(text: &str) -> Result<Vec<Fence<'_>>, String> {
    FenceParser::default().parse(text)
}

/// Incremental parser state for [`parse_fences`].
///
/// The state is a struct rather than a group of locals so each transition owns
/// one small method, keeping the line dispatch readable.
#[derive(Default)]
struct FenceParser<'a> {
    fences: Vec<Fence<'a>>,
    pending_marker: Option<(&'a str, usize)>,
    open: Option<OpenFence<'a>>,
    containers: ListContainers,
}

impl<'a> FenceParser<'a> {
    fn parse(mut self, text: &'a str) -> Result<Vec<Fence<'a>>, String> {
        for (index, line) in text.lines().enumerate() {
            self.visit(line, index + 1)?;
        }
        self.finish()
    }

    /// Routes one line to the transition its syntax calls for.
    fn visit(&mut self, line: &'a str, line_number: usize) -> Result<(), String> {
        let scan = self.scan(line)?;
        if let Some(identifier) = scan.content.and_then(marker_identifier) {
            return self.open_marker(identifier, line_number);
        }
        // Containment is checked before delimiter processing: a shallower
        // delimiter line ends the container, so it must not be mistaken for a
        // closer (or for a new top-level fence) that the open block survives.
        if let Some(open) = &self.open {
            if open.depth > scan.depth {
                return Err(format!(
                    "the fenced block {:?} inside a block quote is not terminated",
                    open.language
                ));
            }
            if open.list_depth > scan.list_depth {
                return Err(format!(
                    "the fenced block {:?} inside a list item is not terminated",
                    open.language
                ));
            }
        }
        if self.fence_line(&scan) {
            return Ok(());
        }
        if line.trim().is_empty() {
            return Ok(());
        }
        self.reject_dangling_marker()
    }

    fn open_marker(&mut self, identifier: &'a str, line_number: usize) -> Result<(), String> {
        if self.open.is_some() {
            return Err(format!(
                "marker on line {line_number} appears inside a fenced block"
            ));
        }
        self.reject_dangling_marker()?;
        self.pending_marker = Some((identifier, line_number));
        Ok(())
    }

    /// Applies a fence opener or closer, reporting whether `line` was one.
    fn fence_line(&mut self, scan: &ContainerScan<'a>) -> bool {
        let Some(content) = scan.content else {
            return false;
        };
        let Some((delimiter, trailing)) = fence_delimiter(content) else {
            return false;
        };
        match self.open.take() {
            Some(open) if closes_open(scan, &open, delimiter, trailing) => {
                self.fences.push(Fence {
                    language: open.language,
                    marker: open.marker,
                });
            }
            Some(open) => self.open = Some(open),
            None => {
                self.open = Some(OpenFence {
                    depth: scan.depth,
                    list_depth: scan.list_depth,
                    delimiter,
                    language: trailing.trim(),
                    marker: self.pending_marker.take().map(|(identifier, _)| identifier),
                });
            }
        }
        true
    }

    /// Strips the containers around `line` and records list-item openings.
    fn scan(&mut self, line: &'a str) -> Result<ContainerScan<'a>, String> {
        let mut rest = self.containers.match_indent(Line::new(line))?.as_str();
        let mut depth = 0;
        loop {
            let trimmed = rest.trim_start_matches(' ');
            let leading = rest.len() - trimmed.len();
            // Four spaces relative to the current container begin an indented
            // code block, whose fence-like lines are literal text.
            if leading > 3 {
                return Ok(ContainerScan {
                    depth,
                    list_depth: self.containers.len(),
                    content: None,
                });
            }
            if let Some(after_quote) = trimmed.strip_prefix('>') {
                rest = strip_quote_padding(after_quote);
                depth += 1;
                continue;
            }
            // A marker inside an open fence is literal content, not a new
            // list item. Matching existing item indentation above still
            // detects the dedent that ends the fence's container.
            if self.open.is_none()
                && let Some((indent, content)) = list_marker(Line::new(trimmed))
            {
                self.containers.push(leading + indent);
                rest = content.as_str();
                continue;
            }
            return Ok(ContainerScan {
                depth,
                list_depth: self.containers.len(),
                content: Some(trimmed),
            });
        }
    }

    fn reject_dangling_marker(&self) -> Result<(), String> {
        match self.pending_marker {
            Some((_, earlier)) => Err(format!(
                "marker on line {earlier} is not followed by a fence"
            )),
            None => Ok(()),
        }
    }

    fn finish(self) -> Result<Vec<Fence<'a>>, String> {
        if let Some(open) = self.open {
            return Err(format!(
                "the fenced block {:?} is not terminated",
                open.language
            ));
        }
        self.reject_dangling_marker()?;
        Ok(self.fences)
    }
}

/// Whether a candidate delimiter closes `open` in `scan`'s container.
fn closes_open(
    scan: &ContainerScan<'_>,
    open: &OpenFence<'_>,
    delimiter: Delimiter,
    trailing: &str,
) -> bool {
    same_container(scan, open) && delimiter.closes(open.delimiter, trailing)
}

/// Whether `scan` is in the same block quote and list item as `open`.
fn same_container(scan: &ContainerScan<'_>, open: &OpenFence<'_>) -> bool {
    scan.depth == open.depth && scan.list_depth == open.list_depth
}

/// One line after its containers were stripped.
struct ContainerScan<'a> {
    /// Block-quote nesting depth.
    depth: usize,
    /// Number of list items containing the line.
    list_depth: usize,
    /// The line's content, or `None` when it is indented code.
    content: Option<&'a str>,
}

/// Recognizes a fence opener or closer in already-stripped content.
///
/// Returns the delimiter and the text after it. `CommonMark` permits up to three
/// leading spaces before the delimiter itself; [`FenceParser::scan`] removes
/// those and any list or block-quote containers first.
fn fence_delimiter(content: &str) -> Option<(Delimiter, &str)> {
    let character = content.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let length = content.chars().take_while(|c| *c == character).count();
    if length < 3 {
        return None;
    }
    // Delimiters are ASCII, so the run's character count is also its byte
    // length; the checked split keeps the helper free of panicking indexing.
    let (_, trailing) = content.split_at_checked(length)?;
    Some((Delimiter { character, length }, trailing))
}

/// Strips the optional space or tab that may follow a block-quote marker.
///
/// `CommonMark` locates content by column, with tab stops every four columns, so
/// `>\t` begins a quoted line whose content is validly indented. Treating the
/// tab as the marker's padding keeps a following fence delimiter visible to
/// the checker; leaving it in place would hide the fence.
fn strip_quote_padding(text: &str) -> &str {
    text.strip_prefix(' ')
        .or_else(|| text.strip_prefix('\t'))
        .unwrap_or(text)
}

/// Returns the identifier when `line` is a `tested-example` marker.
fn marker_identifier(line: &str) -> Option<&str> {
    line.strip_prefix("<!-- tested-example:")?
        .strip_suffix("-->")
        .map(str::trim)
}

#[cfg(test)]
#[path = "parser/tests.rs"]
mod tests;

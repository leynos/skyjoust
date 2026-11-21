//! `CommonMark` fence scanner for the users' guide.
//!
//! The scanner recognizes both fenced-code forms rustdoc accepts, backtick and
//! tilde, indented by up to three spaces relative to their container, at the
//! top level, inside block quotes, and inside list items. It also enforces the
//! documentation style guide's marker placement: every fence must be preceded
//! by a `tested-example` marker on the line immediately above it, in the same
//! container or one enclosing it. A marker in a sibling container—typically
//! the previous list item—does not carry over. Blank lines may separate the
//! two, prose may not, and a contained fence must stay contained until it
//! closes. A shallower line,
//! including a blank one, ends a block quote; a dedented line ends a list item
//! and leaves its fence unterminated.

#[path = "parser/containers.rs"]
mod containers;

use containers::{Containers, Line, list_marker};

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
    /// Identity of the innermost container holding the opener.
    ///
    /// Only a delimiter in that same container closes the block; a quoted line
    /// inside a top-level fence is content. A sibling container of equal depth
    /// is outside the fence's container, which a depth count alone could not
    /// distinguish.
    container: usize,
    /// Block-quote nesting depth at the opener, for diagnostics.
    quote_depth: usize,
    /// Number of list items containing the opener, for diagnostics.
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

/// A marker waiting for the fence it introduces.
struct PendingMarker<'a> {
    identifier: &'a str,
    line_number: usize,
    /// Container identities enclosing the marker, outermost first.
    ///
    /// A fence may inherit the marker only when this path is a prefix of the
    /// fence's own container path. A marker in a sibling list item shares the
    /// marker's depth but not its identity, so it must not label the fence.
    path: Vec<usize>,
}

/// Incremental parser state for [`parse_fences`].
///
/// The state is a struct rather than a group of locals so each transition owns
/// one small method, keeping the line dispatch readable.
#[derive(Default)]
struct FenceParser<'a> {
    fences: Vec<Fence<'a>>,
    pending_marker: Option<PendingMarker<'a>>,
    open: Option<OpenFence<'a>>,
    containers: Containers,
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
            let path = self.containers.path();
            return self.open_marker(identifier, line_number, path);
        }
        // Containment is checked before delimiter processing: a shallower
        // delimiter line ends the container, so it must not be mistaken for a
        // closer (or for a new top-level fence) that the open block survives.
        if let Some(open) = &self.open
            && !same_container(&scan, open)
        {
            if open.quote_depth > scan.quote_depth {
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
        if self.fence_line(&scan, line_number)? {
            return Ok(());
        }
        if is_blank_line(line, &scan) {
            return Ok(());
        }
        self.reject_dangling_marker()
    }

    fn open_marker(
        &mut self,
        identifier: &'a str,
        line_number: usize,
        path: Vec<usize>,
    ) -> Result<(), String> {
        if self.open.is_some() {
            return Err(format!(
                "marker on line {line_number} appears inside a fenced block"
            ));
        }
        self.reject_dangling_marker()?;
        self.pending_marker = Some(PendingMarker {
            identifier,
            line_number,
            path,
        });
        Ok(())
    }

    /// Applies a fence opener or closer, reporting whether `line` was one.
    fn fence_line(&mut self, scan: &ContainerScan<'a>, line_number: usize) -> Result<bool, String> {
        let Some(content) = scan.content else {
            return Ok(false);
        };
        let Some((delimiter, trailing)) = fence_delimiter(content) else {
            return Ok(false);
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
                let marker = self.take_marker(line_number)?;
                self.open = Some(OpenFence {
                    container: scan.container,
                    quote_depth: scan.quote_depth,
                    list_depth: scan.list_depth,
                    delimiter,
                    language: trailing.trim(),
                    marker,
                });
            }
        }
        Ok(true)
    }

    /// Claims the pending marker for a fence opening in the current containers.
    ///
    /// A marker in a sibling container belongs to another block—often the
    /// previous list item—so associating it here would attribute one example's
    /// identifier to another example's code. A marker in an enclosing container
    /// is still accepted: a top-level marker introduces a block-quoted example
    /// in the same way it introduces a top-level one.
    fn take_marker(&mut self, line_number: usize) -> Result<Option<&'a str>, String> {
        let fence_path = self.containers.path();
        match self.pending_marker.take() {
            None => Ok(None),
            Some(pending) if fence_path.starts_with(&pending.path) => Ok(Some(pending.identifier)),
            Some(pending) => Err(format!(
                "the marker on line {} is not in an enclosing container of the fenced block on \
                 line {line_number}",
                pending.line_number
            )),
        }
    }

    /// Strips the containers around `line` and records container openings.
    ///
    /// Containers are matched in their opening order, so the quote prefix of a
    /// block quote that contains a list is removed before the item's content
    /// indentation is measured, exactly as `CommonMark` nests them.
    fn scan(&mut self, line: &'a str) -> Result<ContainerScan<'a>, String> {
        let mut rest = self.containers.match_line(Line::new(line))?.as_str();
        loop {
            let trimmed = rest.trim_start_matches(' ');
            let leading = rest.len() - trimmed.len();
            // Four spaces relative to the current container begin an indented
            // code block, whose fence-like lines are literal text.
            if leading > 3 {
                return Ok(self.container_scan(None));
            }
            // A tab exposed by the prefix just stripped would hide the fence
            // from the container measurement below, so it is rejected here
            // rather than treated as ordinary content.
            let stripped = Line::new(trimmed).reject_tab_indent()?.as_str();
            if let Some(after_quote) = stripped.strip_prefix('>') {
                self.containers.open_quote();
                rest = Line::new(after_quote).strip_quote_padding().as_str();
                continue;
            }
            // A marker inside an open fence is literal content, not a new
            // list item. Matching existing item indentation above still
            // detects the dedent that ends the fence's container.
            if self.open.is_none()
                && let Some((indent, content)) = list_marker(Line::new(stripped))
            {
                self.containers.open_item(leading + indent);
                rest = content.as_str();
                continue;
            }
            return Ok(self.container_scan(Some(stripped)));
        }
    }

    /// Describes the container state and content of the current line.
    fn container_scan(&self, content: Option<&'a str>) -> ContainerScan<'a> {
        ContainerScan {
            container: self.containers.innermost(),
            quote_depth: self.containers.quote_depth(),
            list_depth: self.containers.list_depth(),
            content,
        }
    }

    fn reject_dangling_marker(&self) -> Result<(), String> {
        self.pending_marker.as_ref().map_or(Ok(()), |pending| {
            Err(format!(
                "marker on line {} is not followed by a fence",
                pending.line_number
            ))
        })
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

/// Whether `scan` is in the same container as `open`.
fn same_container(scan: &ContainerScan<'_>, open: &OpenFence<'_>) -> bool {
    scan.container == open.container
}

/// Whether `line` holds nothing but the container prefixes around it.
///
/// A blank line inside a block quote is written `>` rather than left empty, so
/// testing only the raw text would report a marker above it as dangling and
/// break the blank-line allowance for quoted examples. An indented code line
/// strips to no content at all; it is literal text, not a blank line, so that
/// case stays non-blank and still rejects a dangling marker.
fn is_blank_line(line: &str, scan: &ContainerScan<'_>) -> bool {
    line.trim().is_empty()
        || scan
            .content
            .is_some_and(|content| content.trim().is_empty())
}

/// One line after its containers were stripped.
struct ContainerScan<'a> {
    /// Identity of the innermost containing block, zero at the top level.
    container: usize,
    /// Block-quote nesting depth.
    quote_depth: usize,
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

/// Returns the identifier when `line` is a `tested-example` marker.
fn marker_identifier(line: &str) -> Option<&str> {
    line.strip_prefix("<!-- tested-example:")?
        .strip_suffix("-->")
        .map(str::trim)
}

#[cfg(test)]
#[path = "parser/tests.rs"]
mod tests;

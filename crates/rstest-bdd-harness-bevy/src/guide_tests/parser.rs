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

#[path = "parser/blocks.rs"]
mod blocks;
#[path = "parser/containers.rs"]
mod containers;
#[path = "parser/html.rs"]
mod html;
#[path = "parser/lexicon.rs"]
mod lexicon;

use blocks::{is_ascii_blank, is_thematic_break};
use containers::{Containers, Line, list_marker, opens_tab_padded_item};
use html::HtmlBlocks;
use lexicon::{Delimiter, fence_delimiter, marker_identifier};

/// One fenced block and the marker immediately above it, if any.
#[derive(Debug)]
pub(super) struct Fence<'a> {
    pub(super) language: &'a str,
    pub(super) marker: Option<&'a str>,
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
    html: HtmlBlocks,
}

impl<'a> FenceParser<'a> {
    fn parse(mut self, text: &'a str) -> Result<Vec<Fence<'a>>, String> {
        for (index, line) in text.lines().enumerate() {
            self.visit(line, index + 1)?;
        }
        self.finish()
    }

    /// Routes one line to the transition its syntax calls for.
    ///
    /// HTML-block tracking runs first and only outside an open fence, because
    /// a fence's own content is literal text whatever it looks like.
    fn visit(&mut self, line: &'a str, line_number: usize) -> Result<(), String> {
        let scan = self.scan(line)?;
        if self.open.is_none() && self.html.intercept(&scan, line_number)? {
            // An HTML block is content, so it separates a marker from its
            // fence exactly as prose does; only blanks may intervene.
            return self.reject_dangling_marker();
        }
        if self.record_marker(&scan, line_number)? {
            return Ok(());
        }
        self.reject_ended_container(&scan)?;
        if self.fence_line(&scan, line_number)? {
            return Ok(());
        }
        if is_blank_line(line, &scan) {
            return Ok(());
        }
        self.reject_dangling_marker()
    }

    /// Records `scan`'s marker, reporting whether the line carried one.
    ///
    /// Inside an open fence everything is literal content, including a line
    /// shaped like a marker: a guide that documents the marker convention must
    /// be able to show one, and `CommonMark` gives it no meaning there.
    fn record_marker(
        &mut self,
        scan: &ContainerScan<'a>,
        line_number: usize,
    ) -> Result<bool, String> {
        let Some(identifier) = scan.content.and_then(marker_identifier) else {
            return Ok(false);
        };
        if self.open.is_some() {
            return Ok(false);
        }
        let path = self.containers.path();
        self.open_marker(identifier, line_number, path)?;
        Ok(true)
    }

    /// Rejects a fence left behind by a container `scan` has ended.
    ///
    /// Containment is checked before delimiter processing: a shallower
    /// delimiter line ends the container, so it must not be mistaken for a
    /// closer (or for a new top-level fence) that the open block survives.
    fn reject_ended_container(&self, scan: &ContainerScan<'_>) -> Result<(), String> {
        let Some(open) = &self.open else {
            return Ok(());
        };
        if same_container(scan, open) {
            return Ok(());
        }
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
        Ok(())
    }

    fn open_marker(
        &mut self,
        identifier: &'a str,
        line_number: usize,
        path: Vec<usize>,
    ) -> Result<(), String> {
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
            match self.open_prefix(rest)? {
                Prefix::Container(after) => rest = after,
                Prefix::IndentedCode => {
                    return Ok(self.container_scan(indented_content(rest)));
                }
                Prefix::Content(content) => return Ok(self.container_scan(Some(content))),
            }
        }
    }

    /// Opens at most one container or quote from the front of `rest`.
    ///
    /// Four spaces relative to the current container begin an indented code
    /// block, whose fence-like lines are literal text. A line with nothing but
    /// whitespace left is not code—`CommonMark` does not start an indented code
    /// block on a blank line—so it keeps its content for `is_blank_line`.
    fn open_prefix(&mut self, rest: &'a str) -> Result<Prefix<'a>, String> {
        let trimmed = rest.trim_start_matches(' ');
        let leading = rest.len() - trimmed.len();
        // A tab exposed by the prefix just stripped would hide the fence from
        // the container measurement, so it is rejected rather than treated as
        // ordinary content.
        let stripped = Line::new(trimmed).reject_tab_indent()?.as_str();
        // Inside raw HTML a `>` or list marker is content, not a new container:
        // opening one would change the container identity and end the block.
        // Indentation is content too, because a terminator such as `]]>` may
        // carry up to any number of spaces and must stay visible to the
        // tracker; `CommonMark` starts no code block inside an HTML block.
        if self.html.is_open() {
            return Ok(Prefix::Content(stripped));
        }
        if leading > 3 {
            return Ok(Prefix::IndentedCode);
        }
        // A quoted line inside an open fence is literal content: opening a
        // quote there would end the fence's container and reject its content.
        if self.open.is_none()
            && let Some(after) = stripped.strip_prefix('>')
        {
            self.containers.open_quote();
            let quoted = Line::new(after).strip_quote_padding().as_str();
            return Ok(Prefix::Container(quoted));
        }
        Ok(self
            .open_item(stripped, leading)?
            .map_or(Prefix::Content(stripped), Prefix::Container))
    }

    /// Opens a list item at the front of `stripped`, returning its content.
    ///
    /// A marker inside an open fence is literal content, not a new item, and a
    /// thematic break wins over a list marker because `- - -` is a break rather
    /// than three nested items. A tab after a marker pads content to a column
    /// this scanner cannot compute from the stripped fragment, so it fails
    /// closed instead of measuring the wrong content column.
    fn open_item(&mut self, stripped: &'a str, leading: usize) -> Result<Option<&'a str>, String> {
        if self.open.is_some() || is_thematic_break(stripped) {
            return Ok(None);
        }
        if opens_tab_padded_item(Line::new(stripped)) {
            return Err("a tab after a list marker is not supported; use spaces".to_owned());
        }
        let Some((indent, content)) = list_marker(Line::new(stripped)) else {
            return Ok(None);
        };
        self.containers.open_item(leading + indent);
        Ok(Some(content.as_str()))
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
/// break the blank-line allowance for quoted examples. Whitespace left after
/// the containers is still blank, because `CommonMark` does not start an
/// indented code block on a blank line. An indented code line with any other
/// content strips to no content at all; it is literal text, not a blank line,
/// so that case stays non-blank and still rejects a dangling marker.
fn is_blank_line(line: &str, scan: &ContainerScan<'_>) -> bool {
    is_ascii_blank(line) || scan.content.is_some_and(is_ascii_blank)
}

/// Where one step of container stripping ended.
enum Prefix<'a> {
    /// A container opened; the line continues from its content.
    Container(&'a str),
    /// Four spaces began an indented code block.
    IndentedCode,
    /// No further container opened; this is the line's content.
    Content(&'a str),
}

/// The content an indented code line keeps when it is blank.
///
/// A whitespace-only line is not code—`CommonMark` starts no indented code
/// block on a blank line—so it keeps its text for `is_blank_line`.
fn indented_content(rest: &str) -> Option<&str> { is_ascii_blank(rest).then_some(rest) }

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

#[cfg(test)]
#[path = "parser/tests.rs"]
mod tests;

//! CommonMark fence scanner for the users' guide.
//!
//! The scanner recognizes both fenced-code forms rustdoc accepts, backtick and
//! tilde, indented by up to three spaces, at the top level and inside block
//! quotes. It also enforces the documentation style guide's marker placement:
//! every fence must be preceded by a `tested-example` marker with a top-level
//! line of its own, blank lines may separate the two, prose may not, and a
//! quoted fence must stay quoted until it closes. An unquoted line, including
//! a blank one, ends the block quote and leaves the fence unterminated.

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
    /// CommonMark requires the same character and at least the opener's run
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
        if let Some(identifier) = marker_identifier(line) {
            return self.open_marker(identifier, line_number);
        }
        // Containment is checked before delimiter processing: a shallower
        // delimiter line ends the container, so it must not be mistaken for a
        // closer (or for a new top-level fence) that the open block survives.
        if let Some(open) = &self.open
            && open.depth > quote_depth(line)
        {
            return Err(format!(
                "the fenced block {:?} inside a block quote is not terminated",
                open.language
            ));
        }
        if self.fence_line(line) {
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
    fn fence_line(&mut self, line: &'a str) -> bool {
        let Some((depth, delimiter, trailing)) = fence_delimiter(line) else {
            return false;
        };
        match self.open.take() {
            Some(open) if depth == open.depth && delimiter.closes(open.delimiter, trailing) => {
                self.fences.push(Fence {
                    language: open.language,
                    marker: open.marker,
                });
            }
            Some(open) => self.open = Some(open),
            None => {
                self.open = Some(OpenFence {
                    depth,
                    delimiter,
                    language: trailing.trim(),
                    marker: self.pending_marker.take().map(|(identifier, _)| identifier),
                });
            }
        }
        true
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

/// Recognizes a fence opener or closer.
///
/// Returns the block-quote depth, the delimiter, and the text after it.
/// CommonMark permits a fence inside a block quote and up to three leading
/// spaces before the delimiter itself.
fn fence_delimiter(line: &str) -> Option<(usize, Delimiter, &str)> {
    let (depth, rest) = fence_content(line)?;
    let character = rest.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let length = rest.chars().take_while(|c| *c == character).count();
    if length < 3 {
        return None;
    }
    // Delimiters are ASCII, so the run's character count is also its byte
    // length; the checked split keeps the helper free of panicking indexing.
    let (_, trailing) = rest.split_at_checked(length)?;
    Some((depth, Delimiter { character, length }, trailing))
}

/// The result of scanning a line's block-quote prefix.
enum QuoteScan<'a> {
    /// The line's content and the depth of the block quotes around it.
    Content { depth: usize, content: &'a str },
    /// The line begins an indented code block at this depth.
    IndentedCode { depth: usize },
}

impl<'a> QuoteScan<'a> {
    /// Returns the block-quote depth reached before content or indented code.
    fn depth(self) -> usize {
        match self {
            Self::Content { depth, .. } | Self::IndentedCode { depth } => depth,
        }
    }

    /// Returns the depth and content, unless the line is indented code.
    fn into_content(self) -> Option<(usize, &'a str)> {
        match self {
            Self::Content { depth, content } => Some((depth, content)),
            Self::IndentedCode { .. } => None,
        }
    }
}

/// Scans the block-quote markers prefixing `line`.
///
/// CommonMark locates content by column, so a line indented by four or more
/// spaces is an indented code block and carries no block-quote markers. The
/// scan stops at the first line that is neither indented code nor quoted.
fn scan_block_quote(line: &str) -> QuoteScan<'_> {
    let mut rest = line;
    let mut depth = 0;
    loop {
        let trimmed = rest.trim_start_matches(' ');
        if rest.len() - trimmed.len() > 3 {
            return QuoteScan::IndentedCode { depth };
        }
        let Some(after_quote) = trimmed.strip_prefix('>') else {
            return QuoteScan::Content {
                depth,
                content: trimmed,
            };
        };
        rest = strip_quote_padding(after_quote);
        depth += 1;
    }
}

/// Returns the block-quote depth and the text a fence delimiter would start.
///
/// `None` when the content is indented by four or more spaces: such a line is
/// an indented code block, whose text is literal rather than a fence.
fn fence_content(line: &str) -> Option<(usize, &str)> { scan_block_quote(line).into_content() }

/// Counts the block-quote markers that prefix `line`.
///
/// The count is unconditional, so an open quoted fence can tell when its
/// container has ended: an unquoted line, including a blank one, has depth
/// zero. A line indented by four or more spaces before a `>` is an indented
/// code block rather than a quote, so it does not deepen the count.
fn quote_depth(line: &str) -> usize { scan_block_quote(line).depth() }

/// Strips the optional space or tab that may follow a block-quote marker.
///
/// CommonMark locates content by column, with tab stops every four columns, so
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

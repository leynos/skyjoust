//! `CommonMark` HTML-block tracking for the fence scanner.
//!
//! A line beginning with an HTML tag opens an HTML block, and everything in
//! that block is raw HTML until a blank line or the end of its container.
//! `CommonMark` gives no meaning to a fence inside such a block, and rustdoc
//! runs the same parser, so an apparent Rust fence there is not an executable
//! example. The scanner therefore refuses the layout instead of recording a
//! fence the guide does not really contain; the author separates the HTML from
//! the example with a blank line, after which recognition resumes.
//!
//! A comment or declaration that closes on its own line—including the guide's
//! `tested-example` markers—does not open a block, and neither does an autolink.

use super::{ContainerScan, blocks::is_ascii_blank};

/// The scanner's action for one line while HTML blocks are tracked.
enum Verdict {
    /// The line is raw HTML; nothing else on it is Markdown.
    Content,
    /// The line holds a marker or fence delimiter inside an HTML block.
    Fence,
    /// The line is not inside an HTML block; scan it as Markdown.
    Markdown,
}

/// Whether an HTML block is open and where the next line belongs.
///
/// The opener's container is recorded because a block cannot outlive the block
/// quote or list item that holds it: a line outside that container ends the
/// HTML and is ordinary Markdown again.
#[derive(Default)]
pub(super) struct HtmlBlocks {
    open: Option<usize>,
}

impl HtmlBlocks {
    /// Rejects a fence or marker found inside an open HTML block.
    ///
    /// Returns whether the line was HTML content the caller must not scan.
    pub(super) fn intercept(
        &mut self,
        scan: &ContainerScan<'_>,
        line_number: usize,
    ) -> Result<bool, String> {
        match self.visit(scan) {
            Verdict::Fence => Err(format!(
                "the marker or fence on line {line_number} is inside an HTML block, where \
                 CommonMark and rustdoc give it no meaning; separate the example from the HTML \
                 with a blank line"
            )),
            Verdict::Content => Ok(true),
            Verdict::Markdown => Ok(false),
        }
    }

    /// Classifies the line by its stripped content and the state carried over.
    fn visit(&mut self, scan: &ContainerScan<'_>) -> Verdict {
        if let Some(container) = self.open {
            if scan.container != container {
                self.open = None;
                return Verdict::Markdown;
            }
            if is_blank_content(scan.content) {
                self.open = None;
                return Verdict::Markdown;
            }
            return if scan.content.is_some_and(is_fence_or_marker) {
                Verdict::Fence
            } else {
                Verdict::Content
            };
        }
        let Some(content) = scan.content else {
            return Verdict::Markdown;
        };
        if !starts_block(content) {
            return Verdict::Markdown;
        }
        self.open = Some(scan.container);
        Verdict::Content
    }
}

/// Whether the stripped content is a `CommonMark` blank line.
///
/// `None` is indented code, which is content and keeps a block open.
fn is_blank_content(content: Option<&str>) -> bool { content.is_some_and(is_ascii_blank) }

/// Whether `content` opens an HTML block that outlives its own line.
///
/// Only the forms that can continue past their first line matter: a comment
/// closed by `-->`, a processing instruction closed by `?>`, or a declaration
/// closed by `>` is complete on its line, so Markdown after it is unaffected.
fn starts_block(content: &str) -> bool {
    let trimmed = content.trim_start_matches(' ');
    let Some(rest) = trimmed.strip_prefix('<') else {
        return false;
    };
    if trimmed.starts_with("<!--") {
        return !trimmed.contains("-->");
    }
    if trimmed.starts_with("<?") {
        return !trimmed.contains("?>");
    }
    if trimmed.starts_with("<!") {
        return !trimmed.contains('>');
    }
    let name = rest.strip_prefix('/').unwrap_or(rest);
    has_tag_name(name)
}

/// Whether `text` begins an HTML tag name followed by a tag boundary.
///
/// The boundary test is what separates a tag from an autolink: `<details>`
/// opens a block, while `<https://example.org>` is paragraph content because a
/// tag name cannot be followed by `:`.
fn has_tag_name(text: &str) -> bool {
    let name = text
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        .count();
    if name == 0 {
        return false;
    }
    let rest = text.get(name..).unwrap_or_default();
    rest.is_empty() || rest.starts_with([' ', '\t', '>', '/'])
}

/// Whether `content` carries a marker or a fence delimiter.
///
/// Inside an HTML block either one would be recorded as Markdown by a scanner
/// that ignored the block, so both are rejected together. A delimiter's run
/// must repeat one character, so mixed punctuation in literal HTML is content.
fn is_fence_or_marker(content: &str) -> bool {
    let trimmed = content.trim_start_matches(' ');
    if trimmed.starts_with("<!-- tested-example:") {
        return true;
    }
    let Some(first) = trimmed
        .chars()
        .next()
        .filter(|character| matches!(character, '`' | '~'))
    else {
        return false;
    };
    trimmed
        .chars()
        .take_while(|character| *character == first)
        .count()
        >= 3
}

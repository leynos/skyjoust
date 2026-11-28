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

/// How an HTML block ends.
enum End {
    /// A blank line, as for a block-level tag or a complete tag on its line.
    Blank,
    /// A literal terminator, as for a comment, a declaration, or raw text.
    Marker(&'static str),
}

/// An open HTML block: its container and how it ends.
struct OpenBlock {
    container: usize,
    end: End,
}

/// Whether an HTML block is open and where the next line belongs.
///
/// The opener's container is recorded because a block cannot outlive the block
/// quote or list item that holds it: a line outside that container ends the
/// HTML and is ordinary Markdown again.
#[derive(Default)]
pub(super) struct HtmlBlocks {
    open: Option<OpenBlock>,
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

    /// Whether an HTML block is open.
    ///
    /// The scanner asks before opening a container, because a `>` or list
    /// marker inside raw HTML is content rather than a new container.
    pub(super) fn is_open(&self) -> bool { self.open.is_some() }

    /// Classifies a line while a block is open, ending the block if it does.
    fn visit_open(&mut self, scan: &ContainerScan<'_>) -> Verdict {
        let Some(block) = self.open.take() else {
            return Verdict::Markdown;
        };
        if scan.container != block.container {
            return Verdict::Markdown;
        }
        if ends_here(&block, scan.content) {
            // A terminating line is still part of the HTML: the `-->` or
            // closing tag that ends the block occurs on it, so anything else
            // there is literal text rather than a marker or fence.
            return match block.end {
                End::Blank => Verdict::Markdown,
                End::Marker(_) => Verdict::Content,
            };
        }
        self.open = Some(block);
        if scan.content.is_some_and(is_fence_or_marker) {
            Verdict::Fence
        } else {
            Verdict::Content
        }
    }

    /// Classifies the line by its stripped content and the state carried over.
    fn visit(&mut self, scan: &ContainerScan<'_>) -> Verdict {
        if self.open.is_some() {
            return self.visit_open(scan);
        }
        let Some(content) = scan.content else {
            return Verdict::Markdown;
        };
        let Some(end) = block_end(content) else {
            return Verdict::Markdown;
        };
        self.open = Some(OpenBlock {
            container: scan.container,
            end,
        });
        Verdict::Content
    }
}

/// Whether the stripped content is a `CommonMark` blank line.
///
/// `None` is indented code, which is content and keeps a block open.
fn is_blank_content(content: Option<&str>) -> bool { content.is_some_and(is_ascii_blank) }

/// Whether `block`'s end condition is satisfied by `content`.
///
/// A blank-terminated block ends at a blank line; a raw-text block such as
/// `<script>` ends only at its own closing tag, so a fence inside it stays
/// literal however many blank lines it holds.
fn ends_here(block: &OpenBlock, content: Option<&str>) -> bool {
    match block.end {
        End::Blank => is_blank_content(content),
        End::Marker(terminator) => {
            content.is_some_and(|text| contains_ignore_ascii_case(text, terminator))
        }
    }
}

/// Whether `text` contains `needle`, ignoring ASCII case.
///
/// HTML tag names are case-insensitive, so `</SCRIPT>` closes a `<script>`
/// block opened in any case.
fn contains_ignore_ascii_case(text: &str, needle: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

/// Whether `content` opens an HTML block that may interrupt a paragraph.
///
/// `CommonMark` lets the block-level forms interrupt a paragraph, but a
/// complete tag alone on its line does not: `<em>` there is paragraph text, so
/// a list item continues across it rather than ending.
pub(super) fn interrupts_paragraph(content: &str) -> bool {
    let Some(rest) = content.trim_start_matches(' ').strip_prefix('<') else {
        return false;
    };
    if rest.starts_with(['!', '?']) {
        return true;
    }
    let Some(tag) = tag_name(rest.strip_prefix('/').unwrap_or(rest)) else {
        return false;
    };
    INTERRUPTING_TAGS
        .iter()
        .any(|name| name.eq_ignore_ascii_case(tag))
}

/// The tag names whose HTML blocks may interrupt a paragraph.
///
/// The list combines `CommonMark`'s block tag names with its raw-text elements;
/// any other tag is inline and may not interrupt a paragraph.
const INTERRUPTING_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "pre",
    "script",
    "search",
    "section",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
];

/// How `content` opens an HTML block, when it opens one that outlives its line.
///
/// A form that closes on its own line—a comment with `-->`, a processing
/// instruction with `?>`, a declaration with `>`—does not open a block, so
/// Markdown after it is unaffected.
fn block_end(content: &str) -> Option<End> {
    let trimmed = content.trim_start_matches(' ');
    let rest = trimmed.strip_prefix('<')?;
    if let Some(inner) = trimmed.strip_prefix("<!--") {
        return (!inner.contains("-->")).then_some(End::Marker("-->"));
    }
    if let Some(inner) = trimmed.strip_prefix("<?") {
        return (!inner.contains("?>")).then_some(End::Marker("?>"));
    }
    if trimmed.starts_with("<!") {
        return (!trimmed.contains('>')).then_some(End::Marker(">"));
    }
    if let Some(closing) = rest.strip_prefix('/') {
        // A closing tag alone on its line is a block-level HTML form that a
        // blank line ends; it opens nothing that a later closing tag closes.
        return tag_name(closing).map(|_| End::Blank);
    }
    let tag = tag_name(rest)?;
    match raw_text_terminator(tag) {
        Some(End::Marker(terminator)) if contains_ignore_ascii_case(trimmed, terminator) => None,
        raw => Some(raw.unwrap_or(End::Blank)),
    }
}

/// The closing tag of a raw-text element, whose block ignores blank lines.
fn raw_text_terminator(tag: &str) -> Option<End> {
    match tag.to_ascii_lowercase().as_str() {
        "script" => Some(End::Marker("</script>")),
        "pre" => Some(End::Marker("</pre>")),
        "style" => Some(End::Marker("</style>")),
        "textarea" => Some(End::Marker("</textarea>")),
        _ => None,
    }
}

/// The tag name `text` begins, when a tag boundary follows it.
///
/// The name must start with an ASCII letter, so `<1>` is paragraph content. The
/// boundary test separates a tag from an autolink: `<details>` opens a block,
/// while `<https://example.org>` does not, because a tag name cannot be
/// followed by `:`.
fn tag_name(text: &str) -> Option<&str> {
    let length = text
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        .count();
    let name = text.get(..length)?;
    if !name.starts_with(|character: char| character.is_ascii_alphabetic()) {
        return None;
    }
    let rest = text.get(length..).unwrap_or_default();
    let boundary = rest.is_empty() || rest.starts_with([' ', '\t', '>', '/']);
    boundary.then_some(name)
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

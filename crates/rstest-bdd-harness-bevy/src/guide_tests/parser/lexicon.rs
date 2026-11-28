//! Fence and marker lexing for the guide scanner.
//!
//! Two lexical forms carry the scanner's contract: a fence delimiter, whose
//! character and run length decide which lines bound a block, and the
//! `tested-example` marker that labels one. Both are recognised on text whose
//! containers were already stripped, so the rules here concern only the
//! remaining fragment.

/// A fence delimiter: the repeated character and the length of its run.
#[derive(Clone, Copy, Debug)]
pub(super) struct Delimiter {
    pub(super) character: char,
    pub(super) length: usize,
}

impl Delimiter {
    /// Whether `self`, found on a candidate closing line, closes `opener`.
    ///
    /// `CommonMark` requires the same character and at least the opener's run
    /// length. A closing fence may carry only trailing spaces and tabs; any
    /// other text makes the line content rather than a boundary. The suffix is
    /// checked byte by byte because `str::trim` also strips Unicode spaces such
    /// as the non-breaking space, which `CommonMark` treats as content.
    pub(super) fn closes(self, opener: Self, trailing: &str) -> bool {
        self.character == opener.character
            && self.length >= opener.length
            && trailing.bytes().all(|byte| matches!(byte, b' ' | b'\t'))
    }
}

/// Recognizes a fence opener or closer in already-stripped content.
///
/// Returns the delimiter and the text after it. `CommonMark` permits up to three
/// leading spaces before the delimiter itself; the scanner removes those and any
/// list or block-quote containers first. A backtick fence's info string may not
/// contain a backtick, so `` ```rust``` `` is an inline code span rather than an
/// opener; treating it as one would let the following marker attach to a fence
/// the document never opened.
pub(super) fn fence_delimiter(content: &str) -> Option<(Delimiter, &str)> {
    let character = content.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let length = content.chars().take_while(|c| *c == character).count();
    if length < 3 {
        return None;
    }
    // Delimiters are ASCII, so the run's character count is also its byte
    // length; the checked split keeps the helper free of panicking indexing.
    let (_, trailing) = content.split_at_checked(length)?;
    if character == '`' && trailing.contains('`') {
        return None;
    }
    Some((Delimiter { character, length }, trailing))
}

/// Returns the identifier when `line` is a `tested-example` marker.
pub(super) fn marker_identifier(line: &str) -> Option<&str> {
    line.strip_prefix("<!-- tested-example:")?
        .strip_suffix("-->")
        .map(str::trim)
}

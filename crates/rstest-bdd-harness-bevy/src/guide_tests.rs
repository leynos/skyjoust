//! Validates the users' guide's fenced examples before rustdoc runs them.
//!
//! `lib.rs` includes the guide under `cfg(doctest)`, so rustdoc compiles every
//! Rust fence from the published text. Rustdoc ignores the `tested-example`
//! markers, so this module enforces the documentation style guide's loader
//! rules for that document: every fence, whatever its language, must be
//! preceded by a marker with a non-empty identifier, identifiers must be
//! unique, a marker must precede its fence with no intervening prose, and
//! every fence must be terminated. Fences are recognized in both CommonMark
//! forms rustdoc accepts, backtick and tilde, indented by up to three spaces,
//! at the top level and inside block quotes. A marker must itself be a
//! top-level line.

use std::collections::BTreeSet;

const GUIDE: &str = include_str!("../../../docs/users-guide.md");

/// One fenced block and the marker immediately above it, if any.
#[derive(Debug)]
struct Fence<'a> {
    language: &'a str,
    marker: Option<&'a str>,
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
    /// inside a top-level fence is content.
    depth: usize,
    delimiter: Delimiter,
    language: &'a str,
    marker: Option<&'a str>,
}

#[test]
fn every_fence_has_a_marker() {
    let fences = parse_fences(GUIDE).expect("the users' guide parses");

    assert!(
        !fences.is_empty(),
        "the users' guide must contain at least one fence, or the doctest loader has nothing to \
         check"
    );
    for fence in &fences {
        assert!(
            fence.marker.is_some(),
            "the {:?} fence in the users' guide has no tested-example marker",
            fence.language
        );
    }
}

#[test]
fn guide_contains_a_rust_example() {
    let fences = parse_fences(GUIDE).expect("the users' guide parses");

    assert!(
        fences
            .iter()
            .any(|fence| fence.language.starts_with("rust")),
        "the users' guide must contain at least one Rust fence, or the doctest loader checks \
         nothing"
    );
}

#[test]
fn tested_example_identifiers_are_unique_and_non_empty() {
    let fences = parse_fences(GUIDE).expect("the users' guide parses");
    let mut seen = BTreeSet::new();

    assert!(
        fences.iter().any(|fence| fence.marker.is_some()),
        "the users' guide must carry at least one tested-example marker"
    );
    for identifier in fences.iter().filter_map(|fence| fence.marker) {
        assert!(
            !identifier.is_empty(),
            "a tested-example marker in the users' guide has an empty identifier"
        );
        assert!(
            seen.insert(identifier),
            "the tested-example identifier {identifier:?} is used more than once"
        );
    }
}

#[test]
fn unmarked_tilde_fence_is_detected() {
    let fences = parse_fences("~~~rust\nlet value = 1;\n~~~\n").expect("the tilde fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a tilde fence must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the text after the delimiter names the language"
    );
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
    );
}

#[test]
fn unmarked_indented_fence_is_detected() {
    let fences =
        parse_fences("   ```rust\n   let value = 1;\n   ```\n").expect("the indented fence parses");

    assert_eq!(
        fences.len(),
        1,
        "an indented backtick fence must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
    );
}

#[test]
fn closer_with_a_different_character_does_not_close() {
    let error =
        parse_fences("~~~rust\n```\n").expect_err("a backtick line must not close a tilde fence");

    assert!(
        error.contains("not terminated"),
        "the error must report the unterminated fence, got {error:?}"
    );
}

#[test]
fn longer_closer_closes_the_fence() {
    let fences =
        parse_fences("```rust\nlet value = 1;\n`````\n").expect("a longer closer parses the block");

    assert_eq!(
        fences.len(),
        1,
        "a closer longer than its opener must still close the block"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the block keeps the opener's language"
    );
}

#[test]
fn unmarked_block_quoted_fence_is_detected() {
    let fences = parse_fences("> ~~~rust\n> let value = 1;\n> ~~~\n")
        .expect("the block-quoted fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a fence inside a block quote must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the info string inside the quote names the language"
    );
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
    );
}

#[test]
fn marker_above_a_block_quoted_fence_is_attached() {
    let fences = parse_fences(
        "<!-- tested-example: quoted-fence -->\n\n> ```rust\n> let value = 1;\n> ```\n",
    )
    .expect("the marked block-quoted fence parses");

    let marker = fences.first().and_then(|fence| fence.marker);
    assert_eq!(
        marker,
        Some("quoted-fence"),
        "a marker above a block-quoted fence must be attached to it"
    );
}

#[test]
fn quoted_delimiter_does_not_close_a_top_level_fence() {
    let fences = parse_fences("```text\n> ~~~\n```\n").expect("the top-level fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a quoted delimiter is content inside a top-level fence"
    );
    assert_eq!(
        fences
            .first()
            .map(|fence| fence.language)
            .expect("the fixture records exactly one fence"),
        "text",
        "the fence keeps its opener's language"
    );
}

/// Splits the guide into fenced blocks, enforcing marker placement.
///
/// Blank lines may separate a marker from its fence, as the Markdown
/// formatter inserts one. Prose may not: a marker separated from its fence by
/// prose could appear to cover a different block.
fn parse_fences(text: &str) -> Result<Vec<Fence<'_>>, String> { FenceParser::default().parse(text) }

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

/// Recognises a fence opener or closer.
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

/// Returns the block-quote depth and the text a fence delimiter would start.
///
/// `None` when the content is indented by four or more spaces: such a line is
/// an indented code block, whose text is literal rather than a fence.
fn fence_content(line: &str) -> Option<(usize, &str)> {
    let mut rest = line;
    let mut depth = 0;
    loop {
        let trimmed = rest.trim_start_matches(' ');
        if rest.len() - trimmed.len() > 3 {
            return None;
        }
        let Some(after_quote) = trimmed.strip_prefix('>') else {
            return Some((depth, trimmed));
        };
        rest = after_quote.strip_prefix(' ').unwrap_or(after_quote);
        depth += 1;
    }
}

/// Returns the identifier when `line` is a `tested-example` marker.
fn marker_identifier(line: &str) -> Option<&str> {
    line.strip_prefix("<!-- tested-example:")?
        .strip_suffix("-->")
        .map(str::trim)
}

//! Validates the users' guide's executable examples before rustdoc runs them.
//!
//! `lib.rs` includes the guide under `cfg(doctest)`, so rustdoc compiles every
//! Rust fence from the published text. Rustdoc ignores the `tested-example`
//! markers, so this module enforces the documentation style guide's loader
//! rules for that document: each Rust fence must be preceded by a marker with
//! a non-empty identifier, identifiers must be unique, a marker must precede
//! its fence with no intervening prose, and every fence must be terminated.

use std::collections::BTreeSet;

const GUIDE: &str = include_str!("../../../docs/users-guide.md");

/// One fenced block and the marker immediately above it, if any.
struct Fence<'a> {
    language: &'a str,
    marker: Option<&'a str>,
}

#[test]
fn every_rust_fence_has_a_marker() {
    let fences = parse_fences(GUIDE).expect("the users' guide parses");

    assert!(
        fences
            .iter()
            .any(|fence| fence.language.starts_with("rust")),
        "the users' guide must contain at least one Rust fence, or the doctest loader has nothing \
         to check"
    );
    for fence in &fences {
        if fence.language.starts_with("rust") {
            assert!(
                fence.marker.is_some(),
                "a Rust fence in the users' guide has no tested-example marker"
            );
        }
    }
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
    open_language: Option<&'a str>,
    open_marker: Option<&'a str>,
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
        if let Some(language) = line.strip_prefix("```") {
            self.toggle_fence(language.trim());
            return Ok(());
        }
        if line.trim().is_empty() {
            return Ok(());
        }
        self.reject_dangling_marker()
    }

    fn open_marker(&mut self, identifier: &'a str, line_number: usize) -> Result<(), String> {
        if self.open_language.is_some() {
            return Err(format!(
                "marker on line {line_number} appears inside a fenced block"
            ));
        }
        self.reject_dangling_marker()?;
        self.pending_marker = Some((identifier, line_number));
        Ok(())
    }

    fn toggle_fence(&mut self, language: &'a str) {
        if let Some(open) = self.open_language.take() {
            self.fences.push(Fence {
                language: open,
                marker: self.open_marker.take(),
            });
        } else {
            self.open_language = Some(language);
            self.open_marker = self.pending_marker.take().map(|(identifier, _)| identifier);
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
        if let Some(open) = self.open_language {
            return Err(format!("the fenced block {open:?} is not terminated"));
        }
        self.reject_dangling_marker()?;
        Ok(self.fences)
    }
}

/// Returns the identifier when `line` is a `tested-example` marker.
fn marker_identifier(line: &str) -> Option<&str> {
    line.strip_prefix("<!-- tested-example:")?
        .strip_suffix("-->")
        .map(str::trim)
}

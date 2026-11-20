//! Validates the users' guide's fenced examples before rustdoc runs them.
//!
//! `lib.rs` includes the guide under `cfg(doctest)`, so rustdoc compiles every
//! Rust fence from the published text. Rustdoc ignores the `tested-example`
//! markers, so this module enforces the documentation style guide's loader
//! rules for that document: every fence, whatever its language, must be
//! preceded by a marker with a non-empty identifier, identifiers must be
//! unique, and every fence must be terminated.
//!
//! The CommonMark scanner that recognizes fences, including their marker
//! placement and block-quote containment rules, lives in the [`parser`]
//! submodule; this module asserts the guide-wide properties over its output.

use std::collections::BTreeSet;

use parser::parse_fences;

#[path = "guide_tests/parser.rs"]
mod parser;

const GUIDE: &str = include_str!("../../../docs/users-guide.md");

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

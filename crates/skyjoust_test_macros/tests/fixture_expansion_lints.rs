//! Proves `allow_fixture_expansion_lints` suppresses `unused_braces` on an
//! `rstest` fixture whose body is a single expression.
//!
//! The crate-level `deny` is the point of this file. Without the attribute
//! under test, the single-expression fixture below fails to compile with
//! `error: unnecessary braces around block return value`. That the file
//! compiles at all is therefore the assertion; the runtime assertions merely
//! confirm the fixture still behaves normally afterwards.
#![deny(unused_braces)]

use rstest::{fixture, rstest};
use skyjoust_test_macros::allow_fixture_expansion_lints;

/// Single-expression fixture: the shape that trips `unused_braces`.
#[allow_fixture_expansion_lints]
#[fixture]
fn seed() -> u32 { 7 }

/// Multi-statement fixture: the shape that does *not* trip the lint.
///
/// This case is what rules out `#[expect(unused_braces)]` in the expansion —
/// an expectation would go unfulfilled here and warn.
#[allow_fixture_expansion_lints]
#[fixture]
fn doubled_seed() -> u32 {
    let base = 7;
    base * 2
}

#[rstest]
fn single_expression_fixture_is_injected(seed: u32) {
    assert_eq!(seed, 7, "fixture should supply its seed value");
}

#[rstest]
fn multi_statement_fixture_is_injected(doubled_seed: u32) {
    assert_eq!(
        doubled_seed, 14,
        "fixture should supply its doubled seed value"
    );
}

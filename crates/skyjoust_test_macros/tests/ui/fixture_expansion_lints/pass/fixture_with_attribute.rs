//! The attribute successfully suppresses lints from fixture expansion.

#![deny(unused_braces)]

use rstest::fixture;
use skyjoust_test_macros::allow_fixture_expansion_lints;

#[allow_fixture_expansion_lints]
#[fixture]
fn seed() -> u32 { 7 }

fn main() {}

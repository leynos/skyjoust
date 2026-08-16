//! A non-fixture function is rejected by the suppression attribute.

use skyjoust_test_macros::allow_fixture_expansion_lints;

#[allow_fixture_expansion_lints]
fn seed() -> u32 { 7 }

fn main() {}

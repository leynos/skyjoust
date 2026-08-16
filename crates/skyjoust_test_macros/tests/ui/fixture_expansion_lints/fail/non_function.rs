//! A non-function item is rejected by the suppression attribute.

use skyjoust_test_macros::allow_fixture_expansion_lints;

#[allow_fixture_expansion_lints]
struct Seed;

fn main() {}

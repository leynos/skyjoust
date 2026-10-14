//! Compile-fail fixture for a scenario bound to a missing feature file.

use rstest_bdd_macros::scenario;

#[scenario(path = "tests/ui/features/missing.feature", index = 0)]
fn missing_feature_file_fails_to_compile() {}

fn main() {}

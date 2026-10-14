//! Compile-pass fixture for the rstest fixture and step macros used by the crate.

use rstest::fixture;
use rstest_bdd_harness_bevy_test_macros::allow_fixture_expansion_lints;
use rstest_bdd_macros::{given, then, when};

#[allow_fixture_expansion_lints]
#[fixture]
fn app() -> u32 { 7 }

#[given("a fixture is available")]
fn given_fixture() {}

#[when("the scenario runs")]
fn when_scenario_runs() {}

#[then("the macros compile")]
fn then_macros_compile() {}

fn bdd_macro_contract_compiles(_app: u32) {}

fn main() {}

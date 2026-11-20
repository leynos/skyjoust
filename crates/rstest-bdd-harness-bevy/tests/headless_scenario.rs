//! Behavioural coverage proving `rstest-bdd` drives headless Bevy apps
//! through the public harness.
//!
//! Every scenario in this binary binds the same real macro surface a
//! downstream project uses: `harness = BevyHarness` selects the default
//! minimal profile, the `BareHarness` alias proves a type alias selects a
//! downstream profile, and `attributes = BevyAttributePolicy` keeps the
//! generated test free of a framework-specific runner attribute. The feature
//! text is included here so a change to any bound file rebuilds this binary.

// The steps live beside this binary's feature files rather than directly in
// `tests/`, so the path attribute keeps the layout the design documents.
#[path = "headless_scenario/steps.rs"]
mod steps;

use rstest_bdd_harness_bevy::{BareBevyProfile, BevyAttributePolicy, BevyHarness};
use rstest_bdd_macros::scenario;

/// Harness for the bare profile; a type alias is how a downstream project
/// selects a profile.
type BareHarness = BevyHarness<BareBevyProfile>;

/// Feature text the metadata step scans as its independent oracle.
pub(crate) const BEVY_SCENARIO_FEATURE: &str = include_str!("features/bevy_scenario.feature");
/// Keeps the 0.1.1 feature-file rebuild contract explicit at this boundary.
const _: &str = include_str!("features/headless_scenario.feature");
/// Keeps the failure-diagnostics feature-file rebuild contract explicit.
const _: &str = include_str!("features/bevy_panic.feature");

#[scenario(
    path = "tests/features/headless_scenario.feature",
    index = 0,
    harness = BevyHarness,
    attributes = BevyAttributePolicy
)]
fn minimal_app_advances_one_tick() {}

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 0,
    harness = BevyHarness,
    attributes = BevyAttributePolicy
)]
fn steps_receive_the_scenario_metadata() {}

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 1,
    harness = BevyHarness,
    attributes = BevyAttributePolicy
)]
fn steps_share_one_application_across_ticks() {}

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 2,
    harness = BevyHarness,
    attributes = BevyAttributePolicy
)]
fn advancing_zero_times_leaves_the_application_untouched() {}

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 3,
    harness = BareHarness,
    attributes = BevyAttributePolicy
)]
fn the_bare_profile_adds_no_plugins() {}

#[scenario(
    path = "tests/features/bevy_panic.feature",
    index = 0,
    harness = BevyHarness,
    attributes = BevyAttributePolicy
)]
#[should_panic(expected = "rstest-bdd-harness-bevy scenario panicked: phase=steps")]
fn a_failing_step_reports_its_scenario() {}

//! Behavioural coverage proving `rstest-bdd` drives a headless Bevy app.

use std::cell::RefCell;

use bevy::{app::App, diagnostic::FrameCount, time::TimePlugin};
use googletest::prelude::*;
use rstest::fixture;
use rstest_bdd::StepResult;
use rstest_bdd_harness_bevy::minimal_app;
use rstest_bdd_macros::{given, scenario, then, when};
use skyjoust_test_macros::allow_fixture_expansion_lints;

/// Gives `rustc` a rebuild dependency on the feature file; feature-file-only
/// edits do not otherwise invalidate the build.
const _: &str = include_str!("features/headless_scenario.feature");

/// Scenario-scoped headless application shared by the steps below.
#[allow_fixture_expansion_lints]
#[fixture]
fn app() -> RefCell<App> { RefCell::new(minimal_app()) }

#[given("a minimal headless Bevy application")]
fn given_minimal_app(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow()
        .map(|shared| assert_that!(shared.is_plugin_added::<TimePlugin>(), eq(true)))
        .map_err(|error| error.to_string())
}

#[when("the schedule advances once")]
fn when_schedule_advances_once(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow_mut()
        .map(|mut borrow| borrow.update())
        .map_err(|error| error.to_string())
}

#[then("the frame count reads 1")]
fn then_frame_count_reads_one(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow()
        .map(|shared| {
            let observed = shared.world().resource::<FrameCount>().0;
            assert_that!(observed, eq(1_u32));
        })
        .map_err(|error| error.to_string())
}

#[scenario(path = "tests/features/headless_scenario.feature", index = 0)]
fn minimal_app_advances_one_tick(app: RefCell<App>) {}

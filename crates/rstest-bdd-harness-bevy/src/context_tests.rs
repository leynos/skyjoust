//! Unit tests for the shared scenario handle.
//!
//! These tests pin the closure-based accessors, the clone semantics, and the
//! two diagnostics a step author can hit: a nested borrow and an application
//! whose `Main` schedule a panicking system removed.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
};

use bevy::{diagnostic::FrameCount, ecs::resource::Resource, prelude::Update};
use googletest::prelude::*;
use rstest::rstest;
use rstest_bdd_harness::ScenarioMetadata;

use super::{BORROWED_MESSAGE, BevyScenario, MISSING_MAIN_MESSAGE};
use crate::minimal_app;

/// A resource the accessor tests write and read back.
#[derive(Resource)]
struct Marker(u32);

fn test_metadata() -> ScenarioMetadata {
    ScenarioMetadata::new(
        "tests/features/demo.feature",
        "Demo scenario",
        12,
        Vec::new(),
    )
}

fn test_scenario() -> BevyScenario { BevyScenario::new(minimal_app(), test_metadata()) }

/// Extracts the text from a panic payload produced by `panic!` or
/// `panic_any`.
fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&'static str>().copied())
        .map_or_else(|| String::from("<non-string panic payload>"), str::to_owned)
}

#[gtest]
fn metadata_is_retained() {
    let scenario = test_scenario();
    expect_that!(
        scenario.metadata().feature_path(),
        eq("tests/features/demo.feature")
    );
    expect_that!(scenario.metadata().scenario_name(), eq("Demo scenario"));
    expect_that!(scenario.metadata().scenario_line(), eq(12));
}

#[gtest]
fn with_app_returns_the_closure_result_and_mutates_the_shared_app() {
    let scenario = test_scenario();
    let result = scenario.with_app(|app| {
        app.insert_resource(Marker(4));
        "configured"
    });
    expect_that!(result, eq("configured"));
    expect_that!(
        scenario.with_world(|world| world.resource::<Marker>().0),
        eq(4)
    );
}

#[gtest]
fn with_world_exposes_the_world_and_returns_the_closure_result() {
    let scenario = test_scenario();
    scenario.with_world(|world| world.insert_resource(Marker(2)));
    let observed = scenario.with_world(|world| world.resource::<Marker>().0);
    expect_that!(observed, eq(2));
}

#[gtest]
fn update_advances_exactly_one_frame() {
    let scenario = test_scenario();
    scenario.update();
    let frames = scenario.with_world(|world| world.resource::<FrameCount>().0);
    expect_that!(frames, eq(1));
}

#[gtest]
#[rstest]
#[case(0, 0)]
#[case(3, 3)]
fn update_times_advances_the_requested_number_of_frames(
    #[case] ticks: u32,
    #[case] expected_frames: u32,
) {
    let scenario = test_scenario();
    scenario.update_times(ticks);
    let frames = scenario.with_world(|world| world.resource::<FrameCount>().0);
    expect_that!(frames, eq(expected_frames));
}

#[gtest]
fn clones_share_one_application() {
    let scenario = test_scenario();
    let clone = scenario.clone();
    clone.update();
    let frames = scenario.with_world(|world| world.resource::<FrameCount>().0);
    expect_that!(frames, eq(1));
}

/// Asserts that a caught accessor panic carries the crate's full diagnostic.
fn expect_borrowed_panic(outcome: std::thread::Result<()>) {
    let payload = outcome.expect_err("the nested borrow must panic");
    let message = panic_message(payload.as_ref());
    expect_that!(message.starts_with(BORROWED_MESSAGE), eq(true));
}

#[gtest]
fn a_nested_borrow_panics_with_the_crate_prefix() {
    let scenario = test_scenario();
    expect_borrowed_panic(catch_unwind(AssertUnwindSafe(|| {
        scenario.with_app(|_app| scenario.update());
    })));
}

#[gtest]
fn a_cross_clone_update_inside_with_world_panics_with_the_crate_prefix() {
    let scenario = test_scenario();
    let clone = scenario.clone();
    expect_borrowed_panic(catch_unwind(AssertUnwindSafe(|| {
        scenario.with_world(|_world| clone.update());
    })));
}

#[gtest]
fn the_scenario_is_borrowable_after_a_caught_panic() {
    let scenario = test_scenario();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        scenario.with_app(|_app| scenario.update());
    }));
    expect_that!(outcome.is_err(), eq(true));
    scenario.update();
    let frames = scenario.with_world(|world| world.resource::<FrameCount>().0);
    expect_that!(frames, eq(1));
}

/// Builds a scenario whose first update panics in a system, removing `Main`.
fn scenario_with_broken_main() -> BevyScenario {
    let scenario = test_scenario();
    scenario.with_app(|app| {
        app.add_systems(Update, || panic!("system boom"));
    });
    let first_update = catch_unwind(AssertUnwindSafe(|| scenario.update()));
    expect_that!(first_update.is_err(), eq(true));
    scenario
}

#[gtest]
fn a_panicking_system_leaves_a_diagnosable_application() {
    let scenario = scenario_with_broken_main();
    let second_update = catch_unwind(AssertUnwindSafe(|| scenario.update()));
    let payload = second_update.expect_err("the second update must panic");
    let message = panic_message(payload.as_ref());
    expect_that!(message.contains(MISSING_MAIN_MESSAGE), eq(true));
}

#[gtest]
fn update_times_zero_still_rejects_a_missing_main_schedule() {
    let scenario = scenario_with_broken_main();
    let zero_update = catch_unwind(AssertUnwindSafe(|| scenario.update_times(0)));
    let payload = zero_update.expect_err("zero ticks must still check the Main schedule");
    let message = panic_message(payload.as_ref());
    expect_that!(message.contains(MISSING_MAIN_MESSAGE), eq(true));
}

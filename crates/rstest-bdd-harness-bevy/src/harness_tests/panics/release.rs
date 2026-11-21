//! Release rows for returning runs.
//!
//! The harness holds the application until it releases it: on the normal path
//! after cleanup, and after a configure error. Releasing it runs
//! profile-installed destructors, which are caller-supplied code. These rows
//! prove the release happens before the caller observes the outcome, and that a
//! destructor which panics is recorded as a secondary failure rather than
//! replacing the runner's value or the profile's own error. The panic routes
//! release the application through the same guard; their rows stay in the
//! parent module.

use std::{
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::ecs::resource::Resource;
use googletest::prelude::*;
use rstest_bdd_harness::{HarnessAdapter, ScenarioRunRequest, ScenarioRunner};

use super::super::{
    metadata,
    profiles::{ConfigureFailsWithDrop, Recording, SucceedsWithDrop},
    recorder::capture_errors,
};
use crate::{BevyHarness, BevyScenario};

/// Resource whose drop records that the application was released.
#[derive(Resource)]
struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); }
}

/// Builds a request whose runner inserts a [`DropFlag`] and calls `runner`.
///
/// The runner asserts the application is still alive before it hands control
/// to `runner`, so each test proves the flag it checks afterwards was set by
/// the harness rather than by the fixture.
fn request_with_drop_flag(
    flag: Arc<AtomicBool>,
    runner: impl FnOnce(BevyScenario) -> u32 + 'static,
) -> ScenarioRunRequest<'static, BevyScenario, u32> {
    ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.insert_resource(DropFlag(Arc::clone(&flag))));
            assert!(
                !flag.load(Ordering::SeqCst),
                "the application must still be alive inside the runner",
            );
            runner(scenario)
        }),
    )
}

#[gtest]
fn the_application_is_dropped_before_a_step_panic_reaches_the_caller() {
    let dropped = Arc::new(AtomicBool::new(false));
    let harness = BevyHarness::<Recording>::new();
    let request = request_with_drop_flag(Arc::clone(&dropped), |_scenario| -> u32 {
        panic_any("step boom")
    });

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let _payload = outcome.expect_err("the step panic must reach the caller");
    expect_that!(dropped.load(Ordering::SeqCst), eq(true));
    expect_that!(events.len(), eq(1));
}

#[gtest]
fn the_application_is_dropped_before_the_runner_result_returns() {
    let dropped = Arc::new(AtomicBool::new(false));
    let harness = BevyHarness::<Recording>::new();
    let request = request_with_drop_flag(Arc::clone(&dropped), |_scenario| 41);

    let (result, events) = capture_errors(|| harness.run(request));
    let value = result.expect("the recording profile must succeed");

    expect_that!(value, eq(41));
    expect_that!(dropped.load(Ordering::SeqCst), eq(true));
    expect_that!(events.len(), eq(0));
}

#[gtest]
fn a_panicking_drop_after_a_successful_run_is_recorded_not_raised() {
    let harness = BevyHarness::<SucceedsWithDrop>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|_scenario: BevyScenario| 9_u32),
    );

    let (result, events) = capture_errors(|| harness.run(request));
    let value = result.expect("a panicking application drop must not replace the runner value");

    expect_that!(value, eq(9));
    expect_that!(events.len(), eq(1));
    let record = events
        .first()
        .expect("the drop panic is the recorded event");
    expect_that!(record.phase.as_str(), eq("cleanup"));
    expect_that!(record.message.as_str(), eq("drop boom"));
}

#[gtest]
fn a_panicking_drop_after_a_configure_error_keeps_the_configure_error() {
    let harness = BevyHarness::<ConfigureFailsWithDrop>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|_scenario: BevyScenario| 0_u32),
    );

    let (result, events) = capture_errors(|| harness.run(request));
    let error = result.expect_err("the configure error must be returned");
    let text = error.to_string();

    expect_that!(text.as_str(), contains_substring("configure sentinel"));
    expect_that!(text.as_str(), not(contains_substring("drop boom")));
    expect_that!(events.len(), eq(1));
    let record = events
        .first()
        .expect("the drop panic is the recorded event");
    expect_that!(record.phase.as_str(), eq("configure"));
    expect_that!(record.message.as_str(), eq("drop boom"));
}

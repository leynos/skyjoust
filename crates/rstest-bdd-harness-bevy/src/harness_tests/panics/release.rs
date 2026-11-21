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
//!
//! The rows also cover the harness's other caller-owned values: the runner's
//! result, which the harness drops when cleanup fails, and the unrun runner,
//! which it drops when configuration fails. Both may own destructors that
//! panic, and both are released while a cleanup or configure outcome is
//! already settled.

use std::{
    any::Any,
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
    profiles::{
        CleanupFails,
        CleanupPanics,
        ConfigureFails,
        ConfigureFailsWithDrop,
        ConfigurePanics,
        Recording,
        SucceedsWithDrop,
    },
    recorder::{RecordedEvent, capture_errors},
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

/// Value whose destructor panics, returned by a runner that succeeded.
#[derive(Debug)]
struct PanicOnDropValue;

impl Drop for PanicOnDropValue {
    fn drop(&mut self) { panic_any("value boom"); }
}

/// Value whose destructor panics, captured by a runner that never runs.
struct PanicOnDropCapture;

impl Drop for PanicOnDropCapture {
    fn drop(&mut self) { panic_any("capture boom"); }
}

/// Builds a runner that owns a [`PanicOnDropCapture`] and is never run.
fn request_with_panicking_capture() -> ScenarioRunRequest<'static, BevyScenario, u32> {
    let capture = PanicOnDropCapture;
    ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |_scenario: BevyScenario| -> u32 {
            // The borrow forces the closure to capture the value, so its
            // destructor runs when the runner itself is dropped.
            let _ = &capture;
            0
        }),
    )
}

/// Downcasts a re-raised payload to the contract's `String` message.
fn raised_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .expect("the harness re-raises a String message")
        .as_str()
}

/// Asserts exactly one secondary record with `phase` and `error`.
fn expect_secondary_record(events: &[RecordedEvent], phase: &str, error: &str) {
    expect_that!(events.len(), eq(1));
    let record = events
        .first()
        .expect("the destructor panic is the recorded event");
    expect_that!(record.phase.as_str(), eq(phase));
    expect_that!(record.message.as_str(), eq(error));
}

#[gtest]
fn a_panicking_runner_value_is_recorded_after_a_cleanup_error() {
    let harness = BevyHarness::<CleanupFails>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|_scenario: BevyScenario| PanicOnDropValue),
    );

    let (result, events) = capture_errors(|| harness.run(request));
    let error = result.expect_err("the cleanup error must be returned");
    let text = error.to_string();

    expect_that!(text.as_str(), contains_substring("cleanup sentinel"));
    expect_that!(text.as_str(), not(contains_substring("value boom")));
    expect_secondary_record(&events, "cleanup", "value boom");
}

#[gtest]
fn a_panicking_runner_value_does_not_abort_a_cleanup_panic() {
    let harness = BevyHarness::<CleanupPanics>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|_scenario: BevyScenario| PanicOnDropValue),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the cleanup panic must reach the caller");
    let message = raised_message(payload.as_ref());
    expect_that!(message, contains_substring("phase=cleanup"));
    expect_that!(message, contains_substring("cleanup boom"));
    expect_that!(message, not(contains_substring("value boom")));
    expect_secondary_record(&events, "cleanup", "value boom");
}

#[gtest]
fn a_panicking_runner_capture_is_recorded_after_a_configure_error() {
    let harness = BevyHarness::<ConfigureFails>::new();
    let request = request_with_panicking_capture();

    let (result, events) = capture_errors(|| harness.run(request));
    let error = result.expect_err("the configure error must be returned");
    let text = error.to_string();

    expect_that!(text.as_str(), contains_substring("configure sentinel"));
    expect_that!(text.as_str(), not(contains_substring("capture boom")));
    expect_secondary_record(&events, "configure", "capture boom");
}

#[gtest]
fn a_panicking_runner_capture_does_not_abort_a_configure_panic() {
    let harness = BevyHarness::<ConfigurePanics>::new();
    let request = request_with_panicking_capture();

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the configure panic must reach the caller");
    let message = raised_message(payload.as_ref());
    expect_that!(message, contains_substring("phase=configure"));
    expect_that!(message, contains_substring("configure boom"));
    expect_that!(message, not(contains_substring("capture boom")));
    expect_secondary_record(&events, "configure", "capture boom");
}

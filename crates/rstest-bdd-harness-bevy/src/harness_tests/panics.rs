//! Panic-path rows of ADR 010's lifecycle table.
//!
//! Rows 2 and 5 prove that configure and cleanup panics are re-raised with
//! their own phase tags and without error records. Rows 6 to 8 prove that a
//! step panic is re-raised with phase `steps`, that cleanup still runs exactly
//! once, and that a cleanup failure becomes a `tracing::error!` record rather
//! than masking the step panic. The leak row proves the application is dropped
//! before the panic reaches the caller.

use std::{
    any::Any,
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::ecs::resource::Resource;
use googletest::prelude::*;
use rstest_bdd_harness::{HarnessAdapter, ScenarioRunRequest, ScenarioRunner};

use super::{
    metadata,
    profiles::{
        CleanupCalls,
        CleanupFails,
        CleanupPanics,
        ConfigurePanics,
        ConfigurePanicsWithDrop,
        PanicOnDrop,
        Recording,
        RunnerCalls,
    },
    recorder::capture_errors,
};
use crate::{BevyHarness, BevyScenario};

/// Resource whose drop records that the application was released.
#[derive(Resource)]
struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); }
}

/// Downcasts the re-raised payload to the contract's `String` message.
fn panic_text(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .expect("the harness re-raises a String message")
        .as_str()
}

/// Asserts the full phase-tagged message shape from ADR 010.
fn expect_phase_message(message: &str, phase: &str, payload: &str) {
    let expected_prefix =
        format!("rstest-bdd-harness-bevy scenario panicked: phase={phase}, harness=");
    assert!(
        message.starts_with(&expected_prefix),
        "the raised message must start with {expected_prefix:?}, got {message:?}",
    );
    assert!(
        message.contains("tests/features/demo.feature:12"),
        "the raised message must carry the feature path and line, got {message:?}",
    );
    assert!(
        message.contains("\"Demo scenario\""),
        "the raised message must quote the scenario name, got {message:?}",
    );
    assert!(
        message.contains("BevyHarness"),
        "the raised message must name the harness type, got {message:?}",
    );
    assert!(
        message.contains(payload),
        "the raised message must carry the payload text {payload:?}, got {message:?}",
    );
}

#[gtest]
fn a_configure_panic_is_raised_with_the_configure_phase() {
    let harness = BevyHarness::<ConfigurePanics>::new();
    let runner_called = Rc::new(RefCell::new(false));
    let flag = Rc::clone(&runner_called);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |_scenario: BevyScenario| -> u32 {
            *flag.borrow_mut() = true;
            0
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the configure panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "configure", "configure boom");
    expect_that!(*runner_called.borrow(), eq(false));
    expect_that!(events.len(), eq(0));
}

#[gtest]
fn a_cleanup_panic_is_raised_with_the_cleanup_phase() {
    let harness = BevyHarness::<CleanupPanics>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.resource_mut::<RunnerCalls>().0 += 1);
            *sink.borrow_mut() = Some(scenario);
            0
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the cleanup panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "cleanup", "cleanup boom");
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<RunnerCalls>().0),
        eq(1)
    );
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(events.len(), eq(0));
}

#[gtest]
fn a_step_panic_runs_cleanup_once_and_records_the_step_phase() {
    let harness = BevyHarness::<Recording>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.resource_mut::<RunnerCalls>().0 += 1);
            *sink.borrow_mut() = Some(scenario.clone());
            panic_any("step boom")
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the step panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "steps", "step boom");
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<RunnerCalls>().0),
        eq(1)
    );
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(events.len(), eq(1));
    let event = events.first().expect("the step panic leaves one record");
    expect_that!(event.phase.as_str(), eq("steps"));
    expect_that!(event.scenario_name.as_str(), eq("Demo scenario"));
    expect_that!(event.message.as_str(), contains_substring("step boom"));
}

#[gtest]
fn a_cleanup_error_after_a_step_panic_is_recorded_not_raised() {
    let harness = BevyHarness::<CleanupFails>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.resource_mut::<RunnerCalls>().0 += 1);
            *sink.borrow_mut() = Some(scenario.clone());
            panic_any("step boom")
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the step panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "steps", "step boom");
    assert!(
        !message.contains("cleanup sentinel"),
        "the cleanup error must not replace the step message, got {message:?}",
    );
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(events.len(), eq(2));
    let cleanup = events
        .get(1)
        .expect("the cleanup failure is the second record");
    expect_that!(cleanup.phase.as_str(), eq("cleanup"));
    expect_that!(cleanup.scenario_name.as_str(), eq("Demo scenario"));
    expect_that!(
        cleanup.message.as_str(),
        contains_substring("cleanup sentinel")
    );
}

#[gtest]
fn a_cleanup_panic_after_a_step_panic_is_recorded_not_raised() {
    let harness = BevyHarness::<CleanupPanics>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.resource_mut::<RunnerCalls>().0 += 1);
            *sink.borrow_mut() = Some(scenario.clone());
            panic_any("step boom")
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the step panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "steps", "step boom");
    assert!(
        !message.contains("cleanup boom"),
        "the cleanup panic must not replace the step message, got {message:?}",
    );
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(events.len(), eq(2));
    let cleanup = events
        .get(1)
        .expect("the cleanup panic is the second record");
    expect_that!(cleanup.phase.as_str(), eq("cleanup"));
    expect_that!(cleanup.message.as_str(), contains_substring("cleanup boom"));
}

#[gtest]
fn a_panicking_drop_after_a_step_panic_keeps_the_step_phase() {
    let harness = BevyHarness::<Recording>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.insert_resource(PanicOnDrop));
            panic_any("step boom")
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the step panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "steps", "step boom");
    assert!(
        !message.contains("drop boom"),
        "the drop panic must not replace the step message, got {message:?}",
    );
    expect_that!(events.len(), eq(2));
    let drop_record = events.get(1).expect("the drop panic is the second record");
    expect_that!(drop_record.phase.as_str(), eq("cleanup"));
    expect_that!(
        drop_record.message.as_str(),
        contains_substring("drop boom")
    );
}

#[gtest]
fn a_panicking_drop_after_a_configure_panic_keeps_the_configure_phase() {
    let harness = BevyHarness::<ConfigurePanicsWithDrop>::new();
    let runner_called = Rc::new(RefCell::new(false));
    let flag = Rc::clone(&runner_called);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |_scenario: BevyScenario| -> u32 {
            *flag.borrow_mut() = true;
            0
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let payload = outcome.expect_err("the configure panic must reach the caller");
    let message = panic_text(payload.as_ref());
    expect_phase_message(message, "configure", "configure boom");
    assert!(
        !message.contains("drop boom"),
        "the drop panic must not replace the configure message, got {message:?}",
    );
    expect_that!(*runner_called.borrow(), eq(false));
    expect_that!(events.len(), eq(1));
    let drop_record = events.first().expect("the drop panic leaves one record");
    expect_that!(drop_record.phase.as_str(), eq("configure"));
    expect_that!(
        drop_record.message.as_str(),
        contains_substring("drop boom")
    );
}

#[gtest]
fn the_application_is_dropped_before_a_step_panic_reaches_the_caller() {
    let dropped = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&dropped);
    let harness = BevyHarness::<Recording>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| -> u32 {
            scenario.with_world(|world| world.insert_resource(DropFlag(Arc::clone(&flag))));
            panic_any("step boom")
        }),
    );

    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));

    let _payload = outcome.expect_err("the step panic must reach the caller");
    expect_that!(dropped.load(Ordering::SeqCst), eq(true));
    expect_that!(events.len(), eq(1));
}

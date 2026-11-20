//! Hostile-payload rows of the panic path.
//!
//! A caught secondary payload is a `Box<dyn Any + Send>` supplied by
//! arbitrary caller code, so its `Drop` may itself panic. These rows prove the
//! harness transcribes and leaks each secondary payload: the primary
//! phase-tagged panic survives, the hostile text never replaces it, and the
//! secondary failure is recorded instead.

use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};

use googletest::prelude::*;
use rstest_bdd_harness::{HarnessAdapter, ScenarioRunRequest, ScenarioRunner};

use super::{
    super::{
        metadata,
        profiles::{
            CleanupPanicsWithHostilePayload,
            ConfigurePanicsWithHostileDrop,
            PanicOnDropWithHostilePayload,
            Recording,
        },
        recorder::{RecordedEvent, capture_errors},
    },
    ExpectedPhase,
    expect_phase_message,
    panic_text,
};
use crate::{BevyHarness, BevyProfile, BevyScenario};

/// Runs `harness` and returns the raised message and captured records.
fn run_catching<Profile: BevyProfile>(
    harness: BevyHarness<Profile>,
    panic_expectation: &str,
    runner: impl FnOnce(BevyScenario) -> u32 + 'static,
) -> (String, Vec<RecordedEvent>) {
    let request = ScenarioRunRequest::new(metadata(), ScenarioRunner::new(runner));
    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| harness.run(request))));
    let payload = outcome.expect_err(panic_expectation);
    (panic_text(payload.as_ref()).to_owned(), events)
}

/// Asserts a secondary failure is the record at `index`, after `index` others.
fn expect_secondary_record(events: &[RecordedEvent], index: usize, phase: &str) {
    expect_that!(events.len(), eq(index + 1));
    let record = events
        .get(index)
        .expect("the secondary failure is recorded");
    expect_that!(record.phase.as_str(), eq(phase));
    expect_that!(record.message.as_str(), eq("Box<dyn Any>"));
}

#[gtest]
fn a_hostile_cleanup_payload_cannot_replace_the_step_panic() {
    let (message, events) = run_catching(
        BevyHarness::<CleanupPanicsWithHostilePayload>::new(),
        "the step panic must reach the caller",
        |_scenario| -> u32 { panic_any("step boom") },
    );

    expect_phase_message(
        &message,
        &ExpectedPhase {
            phase: "steps",
            payload: "step boom",
        },
    );
    expect_that!(
        message.as_str(),
        not(contains_substring("payload drop boom"))
    );
    expect_secondary_record(&events, 1, "cleanup");
}

#[gtest]
fn a_hostile_drop_payload_cannot_replace_the_step_panic() {
    let (message, events) = run_catching(
        BevyHarness::<Recording>::new(),
        "the step panic must reach the caller",
        |scenario| -> u32 {
            scenario.with_world(|world| {
                world.insert_resource(PanicOnDropWithHostilePayload);
            });
            panic_any("step boom")
        },
    );

    expect_phase_message(
        &message,
        &ExpectedPhase {
            phase: "steps",
            payload: "step boom",
        },
    );
    expect_that!(
        message.as_str(),
        not(contains_substring("payload drop boom"))
    );
    expect_secondary_record(&events, 1, "cleanup");
}

#[gtest]
fn a_hostile_configure_drop_payload_cannot_replace_the_configure_panic() {
    let (message, events) = run_catching(
        BevyHarness::<ConfigurePanicsWithHostileDrop>::new(),
        "the configure panic must reach the caller",
        |_scenario| -> u32 { 0 },
    );

    expect_phase_message(
        &message,
        &ExpectedPhase {
            phase: "configure",
            payload: "configure boom",
        },
    );
    expect_that!(
        message.as_str(),
        not(contains_substring("payload drop boom"))
    );
    expect_secondary_record(&events, 0, "configure");
}

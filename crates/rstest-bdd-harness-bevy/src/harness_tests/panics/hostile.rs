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
            CleanupFailsWithHostileDisplay,
            CleanupFailsWithHostileDrop,
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

/// The record a hostile secondary failure must leave behind.
#[derive(Clone, Copy)]
struct SecondaryRecord<'a> {
    /// Position of the record among the captured events.
    index: usize,
    /// Phase the record must carry.
    phase: &'a str,
    /// Exact message text the record must carry.
    message: &'a str,
}

/// Asserts a secondary failure is the record at `expected.index`.
fn expect_secondary_record(events: &[RecordedEvent], expected: &SecondaryRecord<'_>) {
    let index = expected.index;
    expect_that!(events.len(), eq(index + 1));
    let record = events
        .get(index)
        .expect("the secondary failure is recorded");
    expect_that!(record.phase.as_str(), eq(expected.phase));
    expect_that!(record.message.as_str(), eq(expected.message));
}

/// The row-specific text a hostile step-panic case supplies.
///
/// Every such row panics the step the same way and expects the same primary
/// message, so only the hostile text and the record it must leave behind
/// differ between them. Carrying those two together keeps a caller from
/// transposing them, which would make the negative assertion check the wrong
/// string.
struct HostileRow<'a> {
    /// Text that must appear only in the recorded secondary failure.
    hostile_text: &'a str,
    /// Exact message the secondary record must carry.
    recorded_message: &'a str,
}

/// Asserts a hostile secondary failure leaves the step panic intact.
///
/// The step panics, then cleanup or the application drop fails in a way that
/// would unwind out of an unguarded handler. The caller must still receive the
/// steps-phase message, with the hostile text confined to the recorded
/// secondary event.
fn expect_hostile_secondary_after_step_panic<Profile: BevyProfile>(
    harness: BevyHarness<Profile>,
    row: &HostileRow<'_>,
) {
    let (message, events) = run_catching(
        harness,
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
    expect_that!(message.as_str(), not(contains_substring(row.hostile_text)));
    let expected = SecondaryRecord {
        index: 1,
        phase: "cleanup",
        message: row.recorded_message,
    };
    expect_secondary_record(&events, &expected);
}

#[gtest]
fn a_hostile_cleanup_payload_cannot_replace_the_step_panic() {
    expect_hostile_secondary_after_step_panic(
        BevyHarness::<CleanupPanicsWithHostilePayload>::new(),
        &HostileRow {
            hostile_text: "payload drop boom",
            recorded_message: "Box<dyn Any>",
        },
    );
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
    let expected = SecondaryRecord {
        index: 1,
        phase: "cleanup",
        message: "Box<dyn Any>",
    };
    expect_secondary_record(&events, &expected);
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
    let expected = SecondaryRecord {
        index: 0,
        phase: "configure",
        message: "Box<dyn Any>",
    };
    expect_secondary_record(&events, &expected);
}

#[gtest]
fn a_hostile_cleanup_error_display_cannot_replace_the_step_panic() {
    expect_hostile_secondary_after_step_panic(
        BevyHarness::<CleanupFailsWithHostileDisplay>::new(),
        &HostileRow {
            hostile_text: "display boom",
            recorded_message: "display boom",
        },
    );
}

#[gtest]
fn a_hostile_cleanup_error_drop_cannot_replace_the_step_panic() {
    expect_hostile_secondary_after_step_panic(
        BevyHarness::<CleanupFailsWithHostileDrop>::new(),
        &HostileRow {
            hostile_text: "error drop boom",
            recorded_message: "error drop boom",
        },
    );
}

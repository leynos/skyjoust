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
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::ecs::resource::Resource;
use googletest::prelude::*;
use rstest::rstest;
use rstest_bdd_harness::{HarnessAdapter, ScenarioRunRequest, ScenarioRunner};

use super::{
    super::{
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
    },
    ExpectedPhase,
    expect_phase_message,
    panic_text,
};
use crate::{BevyHarness, BevyProfile, BevyScenario};

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

/// Builds a request whose runner returns a value that panics on drop.
fn request_returning_panicking_value() -> ScenarioRunRequest<'static, BevyScenario, PanicOnDropValue>
{
    ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|_scenario: BevyScenario| PanicOnDropValue),
    )
}

/// Builds a request whose runner captures a value that panics on drop.
///
/// A configuration failure means the runner never runs, so the captured
/// value's destructor is the one released together with the runner.
fn request_with_panicking_capture() -> ScenarioRunRequest<'static, BevyScenario, PanicOnDropValue> {
    let capture = PanicOnDropCapture;
    ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |_scenario: BevyScenario| {
            // The borrow forces the closure to capture the value, so its
            // destructor runs when the runner itself is dropped.
            let _ = &capture;
            PanicOnDropValue
        }),
    )
}

/// The settled primary outcome a caller-value row expects.
#[derive(Clone, Copy)]
enum PrimaryOutcome {
    /// The wrapped profile error must carry this text.
    Error {
        /// Text the error message must contain.
        text: &'static str,
    },
    /// The raised phase-tagged message must carry this phase and payload.
    Panic {
        /// Phase tag the message prefix must carry.
        phase: &'static str,
        /// Payload text the message must carry.
        payload: &'static str,
    },
}

/// The row-specific values a caller-value guard row supplies.
struct CallerValueRow {
    /// Which settled outcome must reach the caller as primary.
    primary: PrimaryOutcome,
    /// Phase the one secondary record must carry.
    phase: &'static str,
    /// Exact message the one secondary record must carry.
    secondary: &'static str,
}

/// Asserts a caller-owned value panic cannot replace a settled outcome.
///
/// The request's runner either returns a value whose destructor panics or
/// captures one and never runs. The profile then fails configuration or
/// cleanup, so the value is released while that outcome is already settled.
/// The caller must observe the profile's error, or the phase-tagged panic,
/// with the destructor's text confined to one secondary record.
fn assert_caller_value_guarded<P: BevyProfile>(
    request: ScenarioRunRequest<'static, BevyScenario, PanicOnDropValue>,
    row: &CallerValueRow,
) {
    let (outcome, events) =
        capture_errors(|| catch_unwind(AssertUnwindSafe(|| BevyHarness::<P>::new().run(request))));

    let primary_text = match row.primary {
        PrimaryOutcome::Error { text } => {
            let Ok(result) = outcome else {
                panic!("the profile error must be returned, not a panic");
            };
            let error = result.expect_err("the profile error must be returned");
            let rendered = error.to_string();
            expect_that!(rendered.as_str(), contains_substring(text));
            rendered
        }
        PrimaryOutcome::Panic { phase, payload } => {
            let raised = outcome.expect_err("the phase-tagged panic must reach the caller");
            let message = panic_text(raised.as_ref()).to_owned();
            expect_phase_message(&message, &ExpectedPhase { phase, payload });
            message
        }
    };

    expect_that!(
        primary_text.as_str(),
        not(contains_substring(row.secondary))
    );
    expect_secondary_record(&events, row.phase, row.secondary);
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

/// One row of the caller-value guard table.
///
/// Every case releases a caller-owned value while a cleanup or configure
/// outcome is settled; they differ only in which request is run and which
/// outcome must stay primary. The case table keeps the shared rule in one
/// body instead of four near-identical ones.
#[derive(Clone, Copy)]
enum CallerCase {
    /// A returned value's destructor panics after a cleanup error.
    CleanupError,
    /// A returned value's destructor panics while a cleanup panic unwinds.
    CleanupPanic,
    /// A captured value's destructor panics after a configure error.
    ConfigureError,
    /// A captured value's destructor panics while a configure panic unwinds.
    ConfigurePanic,
}

/// Runs the four caller-value guard cases.
///
/// [`assert_caller_value_guarded`] takes the failing profile as a type
/// parameter, so the table dispatches each case to that profile's
/// instantiation rather than through a runtime value.
#[gtest]
#[rstest]
#[case::cleanup_error(CallerCase::CleanupError)]
#[case::cleanup_panic(CallerCase::CleanupPanic)]
#[case::configure_error(CallerCase::ConfigureError)]
#[case::configure_panic(CallerCase::ConfigurePanic)]
fn a_panicking_caller_value_cannot_replace_the_settled_outcome(#[case] case: CallerCase) {
    match case {
        CallerCase::CleanupError => assert_caller_value_guarded::<CleanupFails>(
            request_returning_panicking_value(),
            &CallerValueRow {
                primary: PrimaryOutcome::Error {
                    text: "cleanup sentinel",
                },
                phase: "cleanup",
                secondary: "value boom",
            },
        ),
        CallerCase::CleanupPanic => assert_caller_value_guarded::<CleanupPanics>(
            request_returning_panicking_value(),
            &CallerValueRow {
                primary: PrimaryOutcome::Panic {
                    phase: "cleanup",
                    payload: "cleanup boom",
                },
                phase: "cleanup",
                secondary: "value boom",
            },
        ),
        CallerCase::ConfigureError => assert_caller_value_guarded::<ConfigureFails>(
            request_with_panicking_capture(),
            &CallerValueRow {
                primary: PrimaryOutcome::Error {
                    text: "configure sentinel",
                },
                phase: "configure",
                secondary: "capture boom",
            },
        ),
        CallerCase::ConfigurePanic => assert_caller_value_guarded::<ConfigurePanics>(
            request_with_panicking_capture(),
            &CallerValueRow {
                primary: PrimaryOutcome::Panic {
                    phase: "configure",
                    payload: "configure boom",
                },
                phase: "configure",
                secondary: "capture boom",
            },
        ),
    }
}

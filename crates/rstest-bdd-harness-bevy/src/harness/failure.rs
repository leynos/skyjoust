//! Failure reporting and payload safety for the Bevy harness.
//!
//! Every path that leaves a scenario early ends here: this module builds the
//! phase-tagged panic message, runs cleanup on the panic route, records
//! secondary failures, and re-raises the original payload. Caught payloads and
//! profile errors are caller-supplied code whose `Drop` and `Display` may
//! panic, so transcription and disposal are guarded separately, and leaking a
//! payload is preferred to aborting the test binary.

use std::{
    any::{Any, type_name},
    fmt,
    panic::{self, AssertUnwindSafe},
};

use bevy::app::App;
use rstest_bdd_harness::{HarnessError, ScenarioMetadata};

use crate::{
    BevyScenario,
    harness::BevyHarness,
    panic::{Phase, payload_text, scenario_panic_message},
    profile::BevyProfile,
};

/// Re-raises a configure panic with the phase-tagged message.
pub(super) fn raise_configure_panic<P>(
    payload: Box<dyn Any + Send>,
    metadata: &ScenarioMetadata,
) -> ! {
    let message = message_for::<P>(Phase::Configure, payload.as_ref(), metadata);
    forget_then_raise(payload, message)
}

/// Runs cleanup on the panic path, recording rather than propagating failure.
///
/// A step panic must keep its identity, so a cleanup error or panic becomes a
/// second `tracing::error!` record and is never returned or raised.
fn cleanup_after_step_panic<P: BevyProfile>(scenario: &BevyScenario, metadata: &ScenarioMetadata) {
    match panic::catch_unwind(AssertUnwindSafe(|| P::cleanup(scenario, metadata))) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => record_cleanup_error_after_step_panic::<P>(error, metadata),
        Err(payload) => {
            let text = transcribe_and_forget(payload);
            record_error::<P>(
                Phase::Cleanup,
                "Bevy scenario cleanup panicked after a step panic",
                &text,
                metadata,
            );
        }
    }
}

/// Records a cleanup error after a step panic, guarding its own disposal.
fn record_cleanup_error_after_step_panic<P: BevyProfile>(
    error: HarnessError,
    metadata: &ScenarioMetadata,
) {
    let text = transcribe_profile_error(error);
    record_error::<P>(
        Phase::Cleanup,
        "Bevy scenario cleanup failed after a step panic",
        &text,
        metadata,
    );
}

/// Formats a profile error and disposes of it, surviving hostile behaviour.
///
/// A profile's error is caller-supplied, so both its `Display` and its `Drop`
/// may panic. Formatting and disposal are separate caught steps on purpose: an
/// error dropped while its own `Display` panic unwinds would panic a second
/// time there, and Rust aborts the process instead of unwinding to a catch
/// boundary, losing the step panic this path exists to preserve.
///
/// A panicking disposal is the later failure, so its text is the one recorded;
/// either way the step panic is unaffected.
fn transcribe_profile_error(error: HarnessError) -> String {
    let wrapped = wrap_profile_error(Phase::Cleanup, error);
    let formatted = match panic::catch_unwind(AssertUnwindSafe(|| wrapped.to_string())) {
        Ok(text) => text,
        Err(payload) => transcribe_and_forget(payload),
    };
    match panic::catch_unwind(AssertUnwindSafe(move || drop(wrapped))) {
        Ok(()) => formatted,
        Err(payload) => transcribe_and_forget(payload),
    }
}

/// Re-raises a step panic after cleanup and the guarded application drop.
pub(super) fn raise_step_panic<P: BevyProfile>(
    payload: Box<dyn Any + Send>,
    metadata: &ScenarioMetadata,
    scenario: BevyScenario,
) -> ! {
    let message = message_for::<P>(Phase::Steps, payload.as_ref(), metadata);
    record_error::<P>(Phase::Steps, "Bevy scenario panicked", &message, metadata);
    cleanup_after_step_panic::<P>(&scenario, metadata);
    drop_scenario_guarded::<P>(scenario, metadata);
    forget_then_raise(payload, message)
}

/// Re-raises a cleanup panic with its own phase tag and no error record.
pub(super) fn raise_cleanup_panic<P: BevyProfile>(
    payload: Box<dyn Any + Send>,
    metadata: &ScenarioMetadata,
    scenario: BevyScenario,
) -> ! {
    let message = message_for::<P>(Phase::Cleanup, payload.as_ref(), metadata);
    drop_scenario_guarded::<P>(scenario, metadata);
    forget_then_raise(payload, message)
}

/// Formats the phase-tagged message for the harness type `P`.
fn message_for<P>(phase: Phase, payload: &(dyn Any + Send), metadata: &ScenarioMetadata) -> String {
    scenario_panic_message(phase, payload, type_name::<BevyHarness<P>>(), metadata)
}

/// Drops the last scenario handle under a guard, recording a panicking drop.
///
/// Every route that releases the harness's own handle goes through this
/// function: the normal path after cleanup, a step panic, and a cleanup panic.
/// The scenario owns the application, so its release runs profile-installed
/// destructors, which are caller-supplied code.
pub(super) fn drop_scenario_guarded<P>(scenario: BevyScenario, metadata: &ScenarioMetadata) {
    drop_guarded::<P, _>(
        scenario,
        Phase::Cleanup,
        "Bevy scenario application drop panicked",
        metadata,
    );
}

/// Drops a partially configured application after the configure hook panicked.
pub(super) fn drop_app_after_configure_panic<P>(app: App, metadata: &ScenarioMetadata) {
    drop_guarded::<P, _>(
        app,
        Phase::Configure,
        "Bevy application drop panicked after a configure panic",
        metadata,
    );
}

/// Drops a partially configured application after the configure hook failed.
///
/// The profile's own error remains the reported failure; a destructor that
/// panics while the application is released is recorded as a secondary
/// diagnostic so it cannot replace that error.
pub(super) fn drop_app_after_configure_error<P>(app: App, metadata: &ScenarioMetadata) {
    drop_guarded::<P, _>(
        app,
        Phase::Configure,
        "Bevy application drop panicked after a configure error",
        metadata,
    );
}

/// Drops `value` under a guard, recording a panicking destructor.
///
/// The destructor is caller-supplied code. Running it unguarded during an
/// existing unwind would abort the process instead of reporting the scenario
/// failure, and running it unguarded on a returning path would let a
/// destructor panic replace an outcome the harness had already produced.
fn drop_guarded<P, T>(value: T, phase: Phase, summary: &str, metadata: &ScenarioMetadata) {
    if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(move || drop(value))) {
        let text = transcribe_and_forget(payload);
        record_error::<P>(phase, summary, &text, metadata);
    }
}

/// Transcribes a caught secondary payload and then leaks it.
///
/// The payload's `Drop` is arbitrary caller code. Running it here would let a
/// panicking destructor replace the phase-tagged panic this recovery path is
/// preserving. Leaking one payload on a path that is already failing is the
/// bounded cost of a stable diagnostic.
fn transcribe_and_forget(payload: Box<dyn Any + Send>) -> String {
    let text = payload_text(payload.as_ref());
    std::mem::forget(payload);
    text
}

/// Leaks the transcribed payload before raising, so its drop cannot double-panic.
fn forget_then_raise(payload: Box<dyn Any + Send>, message: String) -> ! {
    // The payload's text is already in `message`. An arbitrary payload `Drop`
    // must not run during the new unwind, because a panic there aborts the
    // process instead of reporting the scenario failure.
    std::mem::forget(payload);
    panic::panic_any(message)
}

/// Emits one structured error record for a scenario failure.
fn record_error<P>(phase: Phase, summary: &str, error: &str, metadata: &ScenarioMetadata) {
    tracing::error!(
        phase = %phase,
        harness_type = type_name::<BevyHarness<P>>(),
        feature_path = metadata.feature_path(),
        scenario_name = metadata.scenario_name(),
        scenario_line = metadata.scenario_line(),
        error = %error,
        "{}",
        summary
    );
}

/// A profile hook error tagged with its lifecycle phase.
#[derive(Debug)]
pub(super) struct ProfileHookError {
    pub(super) phase: Phase,
    pub(super) source: HarnessError,
}

impl fmt::Display for ProfileHookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Bevy profile {} hook failed: {}",
            self.phase, self.source
        )
    }
}

impl std::error::Error for ProfileHookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { Some(&self.source) }
}

/// Carries a phase-tagged hook error inside the upstream error enum.
pub(super) fn wrap_profile_error(phase: Phase, source: HarnessError) -> HarnessError {
    HarnessError::RuntimeBuildFailed(std::io::Error::other(ProfileHookError { phase, source }))
}

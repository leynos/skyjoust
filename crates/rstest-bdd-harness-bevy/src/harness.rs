//! Harness adapter that runs one headless Bevy application per scenario.
//!
//! [`BevyHarness`] owns the scenario lifecycle: it asks a [`BevyProfile`] to
//! configure a fresh `App`, hands the runner a cloneable [`BevyScenario`], and
//! releases scenario resources through the profile's cleanup hook. Profile
//! errors are wrapped with the phase that produced them so an `Err` from
//! cleanup is distinguishable from an `Err` from configuration. Panics from
//! any phase are re-raised with the phase-tagged message fixed by ADR 010, and
//! cleanup runs before a step panic is re-raised.

use std::{
    any::{Any, type_name},
    fmt,
    marker::PhantomData,
    panic::{self, AssertUnwindSafe},
};

use bevy::app::App;
use rstest_bdd_harness::{
    HarnessAdapter,
    HarnessError,
    HarnessResult,
    ScenarioMetadata,
    ScenarioRunRequest,
    ScenarioRunner,
};

use crate::{
    BevyScenario,
    panic::{Phase, payload_text, scenario_panic_message},
    profile::{BevyProfile, MinimalBevyProfile},
};

/// Runs one headless Bevy application per scenario with a profile-supplied
/// configuration.
///
/// The default profile, [`MinimalBevyProfile`], adds
/// [`bevy::prelude::MinimalPlugins`]. Generated tests select the harness with
/// `#[scenario(harness = BevyHarness)]`; a downstream profile is selected with
/// a type alias such as `type GameHarness = BevyHarness<GameProfile>;`.
///
/// # Examples
///
/// ```
/// use rstest_bdd_harness::{
///     HarnessAdapter,
///     ScenarioMetadata,
///     ScenarioRunRequest,
///     ScenarioRunner,
/// };
/// use rstest_bdd_harness_bevy::{BevyHarness, BevyScenario};
///
/// let request = ScenarioRunRequest::new(
///     ScenarioMetadata::new("tests/features/demo.feature", "Demo", 3, Vec::new()),
///     ScenarioRunner::new(|scenario: BevyScenario| {
///         scenario.update();
///         scenario.with_world(|world| world.resource::<bevy::diagnostic::FrameCount>().0)
///     }),
/// );
/// let harness: BevyHarness = BevyHarness::new();
/// let frames = harness
///     .run(request)
///     .expect("the minimal profile configures");
/// assert_eq!(frames, 1, "one update advances exactly one frame");
/// ```
pub struct BevyHarness<P = MinimalBevyProfile> {
    profile: PhantomData<fn() -> P>,
}

impl<P> BevyHarness<P> {
    /// Creates a harness for the profile type `P`.
    ///
    /// Generated scenario tests construct the harness through
    /// `Default::default()`; this constructor is equivalent and serves
    /// handwritten tests and doctests.
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::{BevyHarness, MinimalBevyProfile};
    ///
    /// let harness: BevyHarness<MinimalBevyProfile> = BevyHarness::new();
    /// let same: BevyHarness<MinimalBevyProfile> = BevyHarness::default();
    /// assert_eq!(
    ///     format!("{harness:?}"),
    ///     format!("{same:?}"),
    ///     "new and default build the same harness",
    /// );
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            profile: PhantomData,
        }
    }
}

impl<P> Default for BevyHarness<P> {
    fn default() -> Self { Self::new() }
}

impl<P> Clone for BevyHarness<P> {
    fn clone(&self) -> Self { *self }
}

impl<P> Copy for BevyHarness<P> {}

impl<P> fmt::Debug for BevyHarness<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BevyHarness")
            .field("profile", &type_name::<P>())
            .finish()
    }
}

impl<P: BevyProfile> HarnessAdapter for BevyHarness<P> {
    type Context = BevyScenario;

    fn run<T>(&self, request: ScenarioRunRequest<'_, BevyScenario, T>) -> HarnessResult<T> {
        let (metadata, runner) = request.into_parts();
        emit_start::<P>(&metadata);
        let scenario = match configure_app::<P>(&metadata) {
            Ok(scenario) => scenario,
            Err(ConfigureFailure::Failed(error)) => return Err(error),
            Err(ConfigureFailure::Panicked(payload)) => {
                raise_configure_panic::<P>(payload, &metadata)
            }
        };
        match run_steps(runner, &scenario) {
            StepOutcome::Returned(value) => match finish_scenario::<P>(&scenario) {
                CleanupOutcome::Completed => Ok(value),
                CleanupOutcome::Failed(error) => Err(error),
                CleanupOutcome::Panicked(payload) => {
                    raise_cleanup_panic::<P>(payload, &metadata, scenario)
                }
            },
            StepOutcome::Panicked(payload) => raise_step_panic::<P>(payload, &metadata, scenario),
        }
    }
}

/// Records the scenario identity before configuration can fail.
fn emit_start<P>(metadata: &ScenarioMetadata) {
    tracing::debug!(
        harness_type = type_name::<BevyHarness<P>>(),
        feature_path = metadata.feature_path(),
        scenario_name = metadata.scenario_name(),
        scenario_line = metadata.scenario_line(),
        "starting Bevy scenario"
    );
}

/// Why configuration did not produce a runnable scenario.
enum ConfigureFailure {
    /// The profile returned an error, wrapped with the configure phase.
    Failed(HarnessError),
    /// The profile panicked; the application has already been dropped.
    Panicked(Box<dyn Any + Send>),
}

/// Builds the application through the profile and wraps it for the runner.
///
/// The `App` is created before the catch boundary so a configure panic still
/// leaves the application owned here; it is dropped under a guard on that path
/// rather than while the original panic unwinds.
fn configure_app<P: BevyProfile>(
    metadata: &ScenarioMetadata,
) -> Result<BevyScenario, ConfigureFailure> {
    let mut app = App::new();
    match panic::catch_unwind(AssertUnwindSafe(|| P::configure(&mut app, metadata))) {
        Ok(Ok(())) => Ok(BevyScenario::new(app, metadata.clone())),
        Ok(Err(error)) => Err(ConfigureFailure::Failed(wrap_profile_error(
            Phase::Configure,
            error,
        ))),
        Err(payload) => {
            drop_app_guarded::<P>(app, metadata);
            Err(ConfigureFailure::Panicked(payload))
        }
    }
}

/// The runner's outcome, with a caught panic kept as its payload.
enum StepOutcome<T> {
    Returned(T),
    Panicked(Box<dyn Any + Send>),
}

/// Hands the runner a clone of the handle so the harness keeps its own.
fn run_steps<T>(
    runner: ScenarioRunner<'_, BevyScenario, T>,
    scenario: &BevyScenario,
) -> StepOutcome<T> {
    match panic::catch_unwind(AssertUnwindSafe(|| runner.run(scenario.clone()))) {
        Ok(value) => StepOutcome::Returned(value),
        Err(payload) => StepOutcome::Panicked(payload),
    }
}

/// The cleanup hook's outcome on the normal path.
enum CleanupOutcome {
    Completed,
    Failed(HarnessError),
    Panicked(Box<dyn Any + Send>),
}

/// Runs the profile cleanup hook after the runner returned.
fn finish_scenario<P: BevyProfile>(scenario: &BevyScenario) -> CleanupOutcome {
    match panic::catch_unwind(AssertUnwindSafe(|| {
        P::cleanup(scenario, scenario.metadata())
    })) {
        Ok(Ok(())) => CleanupOutcome::Completed,
        Ok(Err(error)) => CleanupOutcome::Failed(wrap_profile_error(Phase::Cleanup, error)),
        Err(payload) => CleanupOutcome::Panicked(payload),
    }
}

/// Re-raises a configure panic with the phase-tagged message.
fn raise_configure_panic<P>(payload: Box<dyn Any + Send>, metadata: &ScenarioMetadata) -> ! {
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
        Ok(Err(error)) => record_error::<P>(
            Phase::Cleanup,
            "Bevy scenario cleanup failed after a step panic",
            &wrap_profile_error(Phase::Cleanup, error).to_string(),
            metadata,
        ),
        Err(payload) => record_error::<P>(
            Phase::Cleanup,
            "Bevy scenario cleanup panicked after a step panic",
            &payload_text(payload.as_ref()),
            metadata,
        ),
    }
}

/// Re-raises a step panic after cleanup and the guarded application drop.
fn raise_step_panic<P: BevyProfile>(
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
fn raise_cleanup_panic<P: BevyProfile>(
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
fn drop_scenario_guarded<P>(scenario: BevyScenario, metadata: &ScenarioMetadata) {
    if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(move || drop(scenario))) {
        record_error::<P>(
            Phase::Cleanup,
            "Bevy scenario application drop panicked",
            &payload_text(payload.as_ref()),
            metadata,
        );
    }
}

/// Drops the partially configured application under a guard.
fn drop_app_guarded<P>(app: App, metadata: &ScenarioMetadata) {
    if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(move || drop(app))) {
        record_error::<P>(
            Phase::Configure,
            "Bevy application drop panicked after a configure panic",
            &payload_text(payload.as_ref()),
            metadata,
        );
    }
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
struct ProfileHookError {
    phase: Phase,
    source: HarnessError,
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
fn wrap_profile_error(phase: Phase, source: HarnessError) -> HarnessError {
    HarnessError::RuntimeBuildFailed(std::io::Error::other(ProfileHookError { phase, source }))
}

#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;

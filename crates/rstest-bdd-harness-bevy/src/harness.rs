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
    panic::Phase,
    profile::{BevyProfile, MinimalBevyProfile},
};

#[path = "harness/failure.rs"]
mod failure;

use failure::{
    drop_app_after_configure_error,
    drop_app_after_configure_panic,
    drop_runner_value_guarded,
    drop_scenario_guarded,
    drop_unrun_runner_guarded,
    raise_cleanup_panic,
    raise_configure_panic,
    raise_step_panic,
    wrap_profile_error,
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
/// A downstream crate runs a hand-built request against its own profile to
/// exercise the configuration its generated scenario tests will use. The
/// profile supplies only `configure`; the default cleanup hook is a no-op, and
/// the lifecycle tests cover a profile that overrides it.
///
/// ```
/// use bevy::{app::App, prelude::MinimalPlugins};
/// use rstest_bdd_harness::{
///     HarnessAdapter,
///     HarnessResult,
///     ScenarioMetadata,
///     ScenarioRunRequest,
///     ScenarioRunner,
/// };
/// use rstest_bdd_harness_bevy::{BevyHarness, BevyProfile, BevyScenario};
///
/// enum DemoProfile {}
///
/// impl BevyProfile for DemoProfile {
///     fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
///         app.add_plugins(MinimalPlugins);
///         Ok(())
///     }
/// }
///
/// let request = ScenarioRunRequest::new(
///     ScenarioMetadata::new("tests/features/demo.feature", "Demo", 3, Vec::new()),
///     ScenarioRunner::new(|scenario: BevyScenario| {
///         scenario.update();
///         scenario.with_world(|world| world.resource::<bevy::diagnostic::FrameCount>().0)
///     }),
/// );
/// let frames = BevyHarness::<DemoProfile>::new()
///     .run(request)
///     .expect("the demo profile configures");
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
    /// # Returns
    ///
    /// A harness that runs each scenario with the profile `P`.
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
            Err(failure) => return report_configure_failure::<P, T>(failure, runner, &metadata),
        };
        match run_steps(runner, &scenario) {
            StepOutcome::Returned(value) => finish_run::<P, T>(value, scenario, &metadata),
            StepOutcome::Panicked(payload) => raise_step_panic::<P>(payload, &metadata, scenario),
        }
    }
}

/// Reports a configure failure after releasing the runner it never called.
///
/// The runner captures caller-supplied values, so releasing it is caller code
/// running on a failure path. It is dropped under a guard before the failure
/// is reported: the configure error or phase-tagged panic stays the primary
/// outcome, and a destructor panic becomes a secondary record.
fn report_configure_failure<P: BevyProfile, T>(
    failure: ConfigureFailure,
    runner: ScenarioRunner<'_, BevyScenario, T>,
    metadata: &ScenarioMetadata,
) -> HarnessResult<T> {
    drop_unrun_runner_guarded::<P, T>(runner, metadata);
    match failure {
        ConfigureFailure::Failed(error) => Err(error),
        ConfigureFailure::Panicked(payload) => raise_configure_panic::<P>(payload, metadata),
    }
}

/// Finishes a returning run: cleanup, then the guarded release of the scenario.
///
/// The runner's value or the cleanup error is the primary result. The value is
/// dropped under a guard whenever cleanup did not complete normally, and the
/// scenario is released under a guard afterwards; both release caller-supplied
/// destructors, and a panic there must become a secondary record rather than
/// replace that result or abort the cleanup panic's unwind.
fn finish_run<P: BevyProfile, T>(
    value: T,
    scenario: BevyScenario,
    metadata: &ScenarioMetadata,
) -> HarnessResult<T> {
    let result = match finish_scenario::<P>(&scenario) {
        CleanupOutcome::Completed => Ok(value),
        CleanupOutcome::Failed(error) => {
            drop_runner_value_guarded::<P, T>(value, metadata);
            Err(error)
        }
        CleanupOutcome::Panicked(payload) => {
            drop_runner_value_guarded::<P, T>(value, metadata);
            raise_cleanup_panic::<P>(payload, metadata, scenario)
        }
    };
    drop_scenario_guarded::<P>(scenario, metadata);
    result
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
/// leaves the application owned here. Both failure paths release it under a
/// guard: a panicking destructor must neither unwind while the original panic
/// is unwinding nor replace the profile's own error.
fn configure_app<P: BevyProfile>(
    metadata: &ScenarioMetadata,
) -> Result<BevyScenario, ConfigureFailure> {
    let mut app = App::new();
    match panic::catch_unwind(AssertUnwindSafe(|| P::configure(&mut app, metadata))) {
        Ok(Ok(())) => Ok(BevyScenario::new(app, metadata.clone())),
        Ok(Err(error)) => {
            drop_app_after_configure_error::<P>(app, metadata);
            Err(ConfigureFailure::Failed(wrap_profile_error(
                Phase::Configure,
                error,
            )))
        }
        Err(payload) => {
            drop_app_after_configure_panic::<P>(app, metadata);
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

#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;

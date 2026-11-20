//! Harness adapter that runs one headless Bevy application per scenario.
//!
//! [`BevyHarness`] owns the scenario lifecycle: it asks a [`BevyProfile`] to
//! configure a fresh `App`, hands the runner a cloneable [`BevyScenario`], and
//! releases scenario resources through the profile's cleanup hook. Profile
//! errors are wrapped with the phase that produced them so an `Err` from
//! cleanup is distinguishable from an `Err` from configuration.

use std::{any::type_name, fmt, marker::PhantomData};

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
        let scenario = configure_app::<P>(metadata)?;
        let value = run_steps(runner, &scenario);
        finish_scenario::<P>(&scenario)?;
        Ok(value)
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

/// Builds the application through the profile and wraps it for the runner.
fn configure_app<P: BevyProfile>(metadata: ScenarioMetadata) -> HarnessResult<BevyScenario> {
    let mut app = App::new();
    P::configure(&mut app, &metadata)
        .map_err(|error| wrap_profile_error(Phase::Configure, error))?;
    Ok(BevyScenario::new(app, metadata))
}

/// Hands the runner a clone of the handle so the harness keeps its own.
fn run_steps<T>(runner: ScenarioRunner<'_, BevyScenario, T>, scenario: &BevyScenario) -> T {
    runner.run(scenario.clone())
}

/// Runs the profile cleanup hook on the normal path.
fn finish_scenario<P: BevyProfile>(scenario: &BevyScenario) -> HarnessResult<()> {
    P::cleanup(scenario, scenario.metadata())
        .map_err(|error| wrap_profile_error(Phase::Cleanup, error))
}

/// The lifecycle phase whose hook produced an error.
#[derive(Clone, Copy, Debug)]
enum Phase {
    Configure,
    Cleanup,
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Configure => "configure",
            Self::Cleanup => "cleanup",
        };
        formatter.write_str(name)
    }
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

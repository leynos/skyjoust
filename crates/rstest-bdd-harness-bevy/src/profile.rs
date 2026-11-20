//! Headless Bevy application configuration.
//!
//! This module owns the smallest plugin set a headless behavioural scenario
//! needs: `App` scaffolding with no window, renderer, asset, or audio stack.
//! `add_minimal_plugins` has exactly the shape design §5's `configure` calls,
//! so `MinimalBevyProfile::configure` becomes two lines that call it at
//! `0.1.2`.

use bevy::{app::App, prelude::MinimalPlugins};
use rstest_bdd_harness::{HarnessResult, ScenarioMetadata};

use crate::BevyScenario;

/// Adds the minimal headless plugin set to `app`.
///
/// [`MinimalPlugins`] provides task pools, time, the frame counter, and the
/// schedule runner — no window and no renderer, so the application advances
/// under `cargo test` without a display server.
///
/// # Example
///
/// ```
/// use bevy::app::App;
/// use rstest_bdd_harness_bevy::add_minimal_plugins;
///
/// let mut app = App::new();
/// add_minimal_plugins(&mut app);
/// app.update();
/// ```
pub fn add_minimal_plugins(app: &mut App) { app.add_plugins(MinimalPlugins); }

/// Builds a headless Bevy application carrying only the minimal plugin set.
///
/// This is the harness-free entry point for tests that want an application
/// without a scenario. Each call returns a fresh application whose frame count
/// starts at zero and whose time plugin reads wall-clock time.
///
/// # Example
///
/// ```
/// use bevy::diagnostic::FrameCount;
/// use rstest_bdd_harness_bevy::minimal_app;
///
/// let mut app = minimal_app();
/// app.update();
/// assert_eq!(app.world().resource::<FrameCount>().0, 1);
/// ```
#[must_use = "advance the returned application or inspect its world"]
pub fn minimal_app() -> App {
    let mut app = App::new();
    add_minimal_plugins(&mut app);
    app
}

/// Framework configuration for a [`crate::BevyHarness`] scenario.
///
/// A profile supplies the plugin set and any deterministic resources a game
/// or subsystem needs. Both hooks are associated functions, so the harness can
/// run a profile selected only as a type parameter.
///
/// Profiles that need Bevy's plugin finalization may call `App::finish` and
/// `App::cleanup` themselves, but a finished application can no longer accept
/// plugins; the harness therefore never finalizes, which keeps `add_plugins`
/// legal inside steps.
///
/// # Examples
///
/// ```
/// use bevy::{app::App, prelude::MinimalPlugins};
/// use rstest_bdd_harness::{HarnessResult, ScenarioMetadata};
/// use rstest_bdd_harness_bevy::BevyProfile;
///
/// enum HeadlessProfile {}
///
/// impl BevyProfile for HeadlessProfile {
///     fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
///         app.add_plugins(MinimalPlugins);
///         Ok(())
///     }
/// }
///
/// let mut app = App::new();
/// assert!(
///     HeadlessProfile::configure(&mut app, &ScenarioMetadata::default()).is_ok(),
///     "the profile configures a headless app",
/// );
/// ```
pub trait BevyProfile: 'static {
    /// Configures the fresh application before any step runs.
    ///
    /// # Errors
    ///
    /// Returns [`rstest_bdd_harness::HarnessError`] when the profile cannot
    /// build the runtime it promised. The harness wraps the error with the
    /// `configure` phase and returns it to the generated test.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::app::App;
    /// use rstest_bdd_harness::{HarnessResult, ScenarioMetadata};
    /// use rstest_bdd_harness_bevy::BevyProfile;
    ///
    /// enum EmptyProfile {}
    ///
    /// impl BevyProfile for EmptyProfile {
    ///     fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> { Ok(()) }
    /// }
    ///
    /// let mut app = App::new();
    /// assert!(
    ///     EmptyProfile::configure(&mut app, &ScenarioMetadata::default()).is_ok(),
    ///     "an empty profile succeeds",
    /// );
    /// ```
    fn configure(app: &mut App, metadata: &ScenarioMetadata) -> HarnessResult<()>;

    /// Releases scenario resources after the runner returns.
    ///
    /// The default implementation does nothing. A cleanup hook must not
    /// advance the application.
    ///
    /// # Errors
    ///
    /// Returns [`rstest_bdd_harness::HarnessError`] when release fails; the
    /// harness wraps and returns the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::app::App;
    /// use rstest_bdd_harness::{HarnessResult, ScenarioMetadata};
    /// use rstest_bdd_harness_bevy::{BevyProfile, BevyScenario};
    ///
    /// #[derive(bevy::prelude::Resource)]
    /// struct Released;
    ///
    /// enum CleaningProfile {}
    ///
    /// impl BevyProfile for CleaningProfile {
    ///     fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> { Ok(()) }
    ///
    ///     fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
    ///         scenario.with_world(|world| world.insert_resource(Released));
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn cleanup(_scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        Ok(())
    }
}

/// Profile that leaves `App::new()` untouched.
///
/// It exists for tests that prove the harness does not assume
/// [`MinimalPlugins`], and for callers that want to add their own plugins.
///
/// # Examples
///
/// ```
/// use bevy::app::App;
/// use rstest_bdd_harness::ScenarioMetadata;
/// use rstest_bdd_harness_bevy::{BareBevyProfile, BevyProfile};
///
/// let mut app = App::new();
/// assert!(
///     BareBevyProfile::configure(&mut app, &ScenarioMetadata::default()).is_ok(),
///     "the bare profile succeeds",
/// );
/// assert!(
///     !app.is_plugin_added::<bevy::time::TimePlugin>(),
///     "the bare profile adds no plugins",
/// );
/// ```
pub enum BareBevyProfile {}

impl BevyProfile for BareBevyProfile {
    fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> { Ok(()) }
}

/// Profile that adds [`MinimalPlugins`], the default compatibility line.
///
/// # Examples
///
/// ```
/// use bevy::app::App;
/// use rstest_bdd_harness::ScenarioMetadata;
/// use rstest_bdd_harness_bevy::{BevyProfile, MinimalBevyProfile};
///
/// let mut app = App::new();
/// assert!(
///     MinimalBevyProfile::configure(&mut app, &ScenarioMetadata::default()).is_ok(),
///     "the minimal profile succeeds",
/// );
/// assert!(
///     app.is_plugin_added::<bevy::time::TimePlugin>(),
///     "the minimal profile adds the time plugin",
/// );
/// ```
pub enum MinimalBevyProfile {}

impl BevyProfile for MinimalBevyProfile {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        add_minimal_plugins(app);
        Ok(())
    }
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;

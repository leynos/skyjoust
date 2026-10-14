//! Headless Bevy application configuration.
//!
//! This module owns the smallest plugin set a headless behavioural scenario
//! needs: `App` scaffolding with no window, renderer, asset, or audio stack.
//! `add_minimal_plugins` has exactly the shape design §5's `configure` calls,
//! so `MinimalBevyProfile::configure` becomes two lines that call it at
//! `0.5.1.2`.

use bevy::{app::App, prelude::MinimalPlugins};

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

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;

//! Headless Bevy application configuration.
//!
//! This module owns the minimal plugin set a headless behavioural scenario
//! needs: `App` scaffolding with no window, renderer, asset, or audio stack.
//! Roadmap task `0.5.1.2` turns `add_minimal_plugins` into
//! `MinimalBevyProfile::configure`.

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
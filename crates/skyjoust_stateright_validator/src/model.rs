//! Stateright model integration for the Skyjoust validator.

use serde::{Deserialize, Serialize};

use crate::state::SkyState;

/// Predicate used by a Stateright property over the model and current state.
pub type AlwaysProperty = fn(&SkyjoustInteractionModel, &SkyState) -> bool;

/// Configure the bounded Skyjoust interaction model explored by Stateright.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkyjoustInteractionModel {
    /// Exploration boundary. Increase for deeper interaction sequences, lower
    /// it for fast continuous integration smoke checks.
    pub max_depth: u16,
}

impl Default for SkyjoustInteractionModel {
    fn default() -> Self { Self { max_depth: 24 } }
}

impl SkyjoustInteractionModel {
    /// Return whether replay or exploration has exhausted the configured depth.
    pub(crate) fn depth_exhausted(&self, state: &SkyState) -> bool { state.depth >= self.max_depth }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

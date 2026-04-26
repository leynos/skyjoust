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
    pub max_depth: u8,
}

impl Default for SkyjoustInteractionModel {
    fn default() -> Self { Self { max_depth: 24 } }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

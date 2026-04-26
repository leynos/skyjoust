//! Stateright model integration for the Skyjoust validator.

use serde::{Deserialize, Serialize};
use stateright::{Model, Property};

use crate::{
    action_generation::available_actions,
    actions::SkyAction,
    properties::{ALWAYS_PROPERTIES, SOMETIMES_PROPERTIES},
    state::SkyState,
    transitions::transition,
};

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

impl Model for SkyjoustInteractionModel {
    type State = SkyState;
    type Action = SkyAction;

    fn init_states(&self) -> Vec<Self::State> { vec![SkyState::default()] }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        if state.depth < self.max_depth {
            available_actions(state, actions);
        }
    }

    fn next_state(&self, last: &Self::State, action: Self::Action) -> Option<Self::State> {
        if last.depth >= self.max_depth {
            None
        } else {
            transition(last, &action)
        }
    }

    fn properties(&self) -> Vec<Property<Self>> {
        let mut properties = Vec::new();
        for &(name, check) in ALWAYS_PROPERTIES {
            properties.push(Property::<Self>::always(name, check));
        }
        for &(name, check) in SOMETIMES_PROPERTIES {
            properties.push(Property::<Self>::sometimes(name, check));
        }
        properties
    }

    fn within_boundary(&self, state: &Self::State) -> bool { state.depth <= self.max_depth }
}

//! Adapter that connects the core Skyjoust model to Stateright traits.

use stateright::{Model, Property};

use crate::{
    action_generation::available_actions,
    actions::SkyAction,
    model::SkyjoustInteractionModel,
    properties::{ALWAYS_PROPERTIES, SOMETIMES_PROPERTIES},
    state::SkyState,
    transitions::transition,
};

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

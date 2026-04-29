//! Tests for model configuration defaults.

use stateright::Model;

use super::*;
use crate::{SkyAction, SkyState};

#[test]
fn default_model_depth_is_twenty_four() {
    assert_eq!(SkyjoustInteractionModel::default().max_depth, 24);
}

#[test]
fn actions_empty_at_depth_boundary() {
    let model = SkyjoustInteractionModel { max_depth: 3 };
    let state = SkyState {
        depth: 3,
        ..SkyState::default()
    };
    let mut actions = Vec::new();

    model.actions(&state, &mut actions);

    assert!(actions.is_empty());
}

#[test]
fn next_state_rejects_at_depth_boundary() {
    let model = SkyjoustInteractionModel { max_depth: 0 };

    assert_eq!(
        model.next_state(&SkyState::default(), SkyAction::AssetsLoaded),
        None
    );
}

#[test]
fn within_boundary_rejects_states_beyond_max_depth() {
    let model = SkyjoustInteractionModel { max_depth: 2 };
    let state = SkyState {
        depth: 3,
        ..SkyState::default()
    };

    assert!(!model.within_boundary(&state));
}

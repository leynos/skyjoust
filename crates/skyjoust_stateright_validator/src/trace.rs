//! Trace replay support for recorded Skyjoust action logs.

use serde::{Deserialize, Serialize};
use stateright::Model;

use crate::{
    actions::SkyAction,
    model::SkyjoustInteractionModel,
    properties::ALWAYS_PROPERTIES,
    state::SkyState,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceValidation {
    pub ok: bool,
    pub final_state: SkyState,
    pub failure: Option<TraceFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceFailure {
    pub step_index: usize,
    pub action: SkyAction,
    pub reason: String,
}

/// Replay a concrete engine event log against the same guards and invariants as
/// the Stateright model. This is useful for validating recorded gameplay traces
/// produced by the Bevy/Pixels runtime.
pub fn validate_trace<I>(model: &SkyjoustInteractionModel, trace: I) -> TraceValidation
where
    I: IntoIterator<Item = SkyAction>,
{
    let mut state = SkyState::default();

    for (step_index, action) in trace.into_iter().enumerate() {
        let next = match model.next_state(&state, action.clone()) {
            Some(next) => next,
            None => {
                return invalid_trace(
                    state,
                    step_index,
                    action,
                    "action was not legal from the current state",
                );
            }
        };

        if !model.within_boundary(&next) {
            return invalid_trace(
                next,
                step_index,
                action,
                "trace exceeded the configured exploration boundary",
            );
        }

        if let Some(name) = violated_invariant(model, &next) {
            return invalid_trace(
                next,
                step_index,
                action,
                &format!("violated invariant: {name}"),
            );
        }

        state = next;
    }

    TraceValidation {
        ok: true,
        final_state: state,
        failure: None,
    }
}

fn violated_invariant(model: &SkyjoustInteractionModel, state: &SkyState) -> Option<&'static str> {
    ALWAYS_PROPERTIES
        .iter()
        .find_map(|(name, check)| (!check(model, state)).then_some(*name))
}

fn invalid_trace(
    final_state: SkyState,
    step_index: usize,
    action: SkyAction,
    reason: &str,
) -> TraceValidation {
    TraceValidation {
        ok: false,
        final_state,
        failure: Some(TraceFailure {
            step_index,
            action,
            reason: reason.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    //! Tests for trace replay validation.

    use crate::{validate_trace, SkyAction, SkyjoustInteractionModel};

    #[test]
    fn known_good_keep_breach_trace_validates() {
        let model = SkyjoustInteractionModel::default();
        let result = validate_trace(
            &model,
            [
                SkyAction::AssetsLoaded,
                SkyAction::StartSkirmish,
                SkyAction::StartBattle,
                SkyAction::FinishConstructing,
                SkyAction::SpawnReady,
                SkyAction::CountdownDone,
                SkyAction::BombKeepBreach,
                SkyAction::VictoryCheck,
                SkyAction::ExportFinalScore,
                SkyAction::TallyRewards,
                SkyAction::CommitRewards,
            ],
        );
        assert!(result.ok, "{result:?}");
        assert!(result.final_state.rewards.committed);
    }

    #[test]
    fn rewards_cannot_commit_before_final_score() {
        let model = SkyjoustInteractionModel::default();
        let result = validate_trace(
            &model,
            [
                SkyAction::AssetsLoaded,
                SkyAction::StartSkirmish,
                SkyAction::CommitRewards,
            ],
        );
        assert!(!result.ok);
        assert!(result
            .failure
            .unwrap()
            .reason
            .contains("action was not legal"));
    }
}

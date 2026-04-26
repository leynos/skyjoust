//! Trace replay support for recorded Skyjoust action logs.

use serde::{Deserialize, Serialize};
use stateright::Model;

use crate::{
    actions::SkyAction,
    model::SkyjoustInteractionModel,
    properties::ALWAYS_PROPERTIES,
    state::SkyState,
};

/// Result returned after replaying a concrete action trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceValidation {
    /// Whether every action was legal and every invariant stayed true.
    pub ok: bool,
    /// State reached after the last replayed action, or at the failing step.
    pub final_state: SkyState,
    /// Failure details when replay stops before accepting the whole trace.
    pub failure: Option<TraceFailure>,
}

/// Description of the first illegal action or invariant failure in a trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceFailure {
    /// Zero-based index of the action that failed validation.
    pub step_index: usize,
    /// Action that could not be applied or that violated an invariant.
    pub action: SkyAction,
    /// Human-readable reason for the failure.
    pub reason: String,
}

/// Replay a concrete engine event log against the same guards, invariants, and
/// depth bound as the Stateright model. This is useful for validating recorded
/// gameplay traces produced by the Bevy/Pixels runtime.
///
/// ```
/// use skyjoust_stateright_validator::{
///     SkyAction,
///     SkyState,
///     SkyjoustInteractionModel,
///     TraceFailure,
///     TraceValidation,
///     validate_trace,
/// };
///
/// let model = SkyjoustInteractionModel { max_depth: 8 };
/// let trace = [SkyAction::AssetsLoaded, SkyAction::StartSkirmish];
/// let result: TraceValidation = validate_trace(&model, trace);
///
/// assert!(result.ok);
/// let final_state: SkyState = result.final_state.clone();
/// assert_eq!(final_state.depth, 2);
/// let failure: Option<TraceFailure> = result.failure;
/// assert!(failure.is_none());
/// ```
pub fn validate_trace<I>(model: &SkyjoustInteractionModel, trace: I) -> TraceValidation
where
    I: IntoIterator<Item = SkyAction>,
{
    let mut state = SkyState::default();

    for (step_index, action) in trace.into_iter().enumerate() {
        let next = match model.next_state(&state, action.clone()) {
            Some(next) => next,
            None => {
                let reason = if model.depth_exhausted(&state) {
                    "max depth reached / depth bound exhausted"
                } else {
                    "action was not legal from the current state"
                };
                return invalid_trace(state, step_index, action, reason);
            }
        };

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

    use crate::{SkyAction, SkyjoustInteractionModel, validate_trace};

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
        let failure = result
            .failure
            .expect("expected trace to have a failure with a reason");
        assert!(failure.reason.contains("action was not legal"));
    }

    #[test]
    fn longer_trace_validates_when_depth_is_raised() {
        let model = SkyjoustInteractionModel { max_depth: 40 };
        let result = validate_trace(&model, long_legal_trace());

        assert!(result.ok, "{result:?}");
        assert_eq!(result.final_state.depth, 31);
    }

    #[test]
    fn longer_trace_fails_with_default_depth() {
        let model = SkyjoustInteractionModel::default();
        let result = validate_trace(&model, long_legal_trace());

        assert!(!result.ok);
        assert_eq!(result.final_state.depth, model.max_depth);
        let failure = result
            .failure
            .expect("expected depth-bound trace to report failure");
        assert!(failure.reason.contains("depth bound exhausted"));
    }

    fn long_legal_trace() -> Vec<SkyAction> {
        serde_json::from_str(include_str!("../traces/long_legal_trace.json"))
            .expect("long legal trace fixture should deserialize")
    }
}

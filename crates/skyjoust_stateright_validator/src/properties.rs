//! Invariants and reachability properties for the validator model.

use crate::{
    model::{AlwaysProperty, SkyjoustInteractionModel},
    state::{
        CeremonyState,
        DuelState,
        MatchPhase,
        OrdnancePolicy,
        PlayerOrdnance,
        Rules,
        SkyState,
        Winner,
    },
};

pub const ALWAYS_PROPERTIES: &[(&str, AlwaysProperty)] = &[
    (
        "rewards_commit_requires_final_score",
        prop_rewards_commit_requires_final_score,
    ),
    (
        "rewards_open_requires_final_score",
        prop_rewards_open_requires_final_score,
    ),
    (
        "score_closed_after_final_snapshot",
        prop_score_closed_after_final_snapshot,
    ),
    (
        "no_score_write_after_final_snapshot",
        prop_no_score_write_after_final_snapshot,
    ),
    (
        "committed_rewards_leave_active_match",
        prop_committed_rewards_leave_active_match,
    ),
    (
        "duel_lock_only_during_duel",
        prop_duel_lock_only_during_duel,
    ),
    (
        "joust_only_disables_ordnance",
        prop_joust_only_disables_ordnance,
    ),
    (
        "truce_disables_friendly_fire",
        prop_truce_disables_friendly_fire,
    ),
    ("truce_break_is_penalized", prop_truce_break_is_penalized),
    (
        "laurels_only_after_tournament_completion",
        prop_laurels_only_after_tournament_completion,
    ),
    (
        "duel_reward_only_after_resolved_duel",
        prop_duel_reward_only_after_resolved_duel,
    ),
    (
        "temporary_rules_cleared_after_cooldown",
        prop_temporary_rules_cleared_after_cooldown,
    ),
    ("round_over_has_winner", prop_round_over_has_winner),
    (
        "warfront_not_mutated_during_match",
        prop_warfront_not_mutated_during_match,
    ),
];

pub const SOMETIMES_PROPERTIES: &[(&str, AlwaysProperty)] = &[
    (
        "can_breach_keep_and_commit_rewards",
        sometimes_breach_keep_and_commit_rewards,
    ),
    (
        "can_complete_tournament_and_get_laurels",
        sometimes_complete_tournament_and_get_laurels,
    ),
    (
        "can_resolve_duel_and_get_duel_rewards",
        sometimes_resolve_duel_and_get_rewards,
    ),
    (
        "can_break_truce_and_receive_infamy",
        sometimes_break_truce_and_receive_infamy,
    ),
    (
        "can_score_nonlethal_objective",
        sometimes_score_nonlethal_objective,
    ),
];

fn prop_rewards_commit_requires_final_score(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.committed || s.score.finalized
}

fn prop_rewards_open_requires_final_score(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.phase.is_open() || s.score.finalized
}

fn prop_score_closed_after_final_snapshot(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.score.finalized || !s.score.open
}

fn prop_no_score_write_after_final_snapshot(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.post_final_score_write
}

fn prop_committed_rewards_leave_active_match(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.committed || !s.is_match_active()
}

fn prop_duel_lock_only_during_duel(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rules.duel_lock
        || matches!(
            s.ceremony,
            CeremonyState::Duel(DuelState::ArenaLock)
                | CeremonyState::Duel(DuelState::DuelActive)
                | CeremonyState::Duel(DuelState::ResolveDuel)
                | CeremonyState::ConsequenceResolution
        )
}

fn prop_joust_only_disables_ordnance(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rules.joust_only
        || (s.rules.ordnance == OrdnancePolicy::Disabled
            && s.player_ordnance == PlayerOrdnance::Disabled)
}

fn prop_truce_disables_friendly_fire(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.truce_active || !s.rules.friendly_fire
}

fn prop_truce_break_is_penalized(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.truce_broken || (s.infamy > 0 && s.rewards.penalties > 0)
}

fn prop_laurels_only_after_tournament_completion(
    _: &SkyjoustInteractionModel,
    s: &SkyState,
) -> bool {
    (s.rewards.laurels == 0 && !s.rewards.tournament_bonus_granted) || s.tournament_completed
}

fn prop_duel_reward_only_after_resolved_duel(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.duel_bonus_granted || s.duel_resolved
}

fn prop_temporary_rules_cleared_after_cooldown(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    if matches!(s.ceremony, CeremonyState::Dormant | CeremonyState::Cooldown) && !s.truce_active {
        s.rules == Rules::baseline()
    } else {
        true
    }
}

fn prop_round_over_has_winner(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !matches!(
        s.match_phase,
        MatchPhase::RoundOver | MatchPhase::ResultsExported
    ) || s.winner != Winner::None
}

fn prop_warfront_not_mutated_during_match(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.warfront_mutated_during_match
}

fn sometimes_breach_keep_and_commit_rewards(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.objectives.keep_breached && s.rewards.committed && s.score.finalized
}

fn sometimes_complete_tournament_and_get_laurels(
    _: &SkyjoustInteractionModel,
    s: &SkyState,
) -> bool {
    s.tournament_completed && s.rewards.committed && s.rewards.laurels >= 3
}

fn sometimes_resolve_duel_and_get_rewards(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.duel_resolved && s.rewards.committed && s.rewards.influence >= 25
}

fn sometimes_break_truce_and_receive_infamy(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.truce_broken && s.rewards.committed && s.infamy > 0 && s.rewards.penalties > 0
}

fn sometimes_score_nonlethal_objective(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.objectives.outpost_controlled && s.score.red_score >= 200 && !s.objectives.keep_breached
}

#[cfg(test)]
mod tests {
    //! Tests for property helper semantics.

    use super::*;

    #[test]
    fn reward_phase_helper_matches_open_property() {
        let mut state = SkyState::default();
        state.rewards.phase = crate::state::RewardPhase::LedgerOpen;

        assert!(!prop_rewards_open_requires_final_score(
            &SkyjoustInteractionModel::default(),
            &state
        ));
    }
}

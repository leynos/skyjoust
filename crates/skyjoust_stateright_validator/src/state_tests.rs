//! Tests for state guard and mode helpers.

use super::*;

#[test]
fn tournament_cannot_start_when_ceremony_is_not_dormant() {
    let state = SkyState {
        match_phase: MatchPhase::NormalPlay,
        ceremony: CeremonyState::Duel(DuelState::ChallengeIssued),
        ..SkyState::default()
    };

    assert!(!state.can_start_tournament());
}

#[test]
fn enter_duel_mode_sets_duel_lock_joust_only_and_ordnance_disabled() {
    let mut state = SkyState::default();

    state.enter_duel_mode();

    assert!(state.rules.duel_lock);
    assert!(state.rules.joust_only);
    assert_eq!(state.player_ordnance, PlayerOrdnance::Disabled);
}

#[test]
fn clear_temporary_rules_restores_baseline_without_truce() {
    let mut state = SkyState {
        rules: Rules {
            duel_lock: true,
            joust_only: true,
            ordnance: OrdnancePolicy::Disabled,
            ..Rules::baseline()
        },
        player_ordnance: PlayerOrdnance::Disabled,
        ..SkyState::default()
    };

    state.clear_temporary_rules_if_safe();

    assert_eq!(state.rules, Rules::baseline());
    assert_eq!(state.player_ordnance, PlayerOrdnance::Ready);
}

#[test]
fn clear_temporary_rules_preserves_active_truce_override() {
    let mut state = SkyState {
        truce_active: true,
        rules: Rules {
            friendly_fire: false,
            duel_lock: true,
            joust_only: true,
            ordnance: OrdnancePolicy::Disabled,
            ..Rules::baseline()
        },
        ..SkyState::default()
    };

    state.clear_temporary_rules_if_safe();

    assert!(!state.rules.friendly_fire);
    assert!(!state.rules.duel_lock);
    assert!(!state.rules.joust_only);
    assert_eq!(state.rules.ordnance, OrdnancePolicy::Full);
    assert!(state.rules.allow_sudden_death);
}

#[test]
fn reset_for_match_start_zeroes_match_scoped_fields() {
    let mut state = SkyState {
        match_phase: MatchPhase::ResultsExported,
        ceremony: CeremonyState::Duel(DuelState::DuelActive),
        truce_active: true,
        truce_broken: true,
        tournament_rounds_won: 2,
        tournament_completed: true,
        duel_resolved: true,
        duel_consequence_active: true,
        post_final_score_write: true,
        warfront_mutated_during_match: true,
        rewards: RewardLedger {
            committed: true,
            phase: RewardPhase::Committed,
            ..RewardLedger::default()
        },
        score: ScoreLedger {
            finalized: true,
            red_score: 120,
            ..ScoreLedger::default()
        },
        ..SkyState::default()
    };

    state.reset_for_match_start();

    assert_eq!(state.match_phase, MatchPhase::Constructing);
    assert_eq!(state.ceremony, CeremonyState::Dormant);
    assert!(!state.score.open);
    assert_eq!(state.score.red_score, 0);
    assert_eq!(state.rewards.phase, RewardPhase::Dormant);
    assert!(!state.truce_active);
    assert!(!state.truce_broken);
    assert_eq!(state.tournament_rounds_won, 0);
    assert!(!state.tournament_completed);
    assert!(!state.duel_resolved);
    assert!(!state.duel_consequence_active);
    assert!(!state.post_final_score_write);
    assert!(!state.warfront_mutated_during_match);
}

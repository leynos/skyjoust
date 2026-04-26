//! Tests for state transition edge cases.

use super::*;

#[test]
fn warfront_mutation_flag_is_set_for_in_match_changes() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        warfront: WarfrontState::AwaitingBattleResult,
        ..SkyState::default()
    };
    let mut next = SkyState {
        warfront: WarfrontState::RewardCommit,
        ..last.clone()
    };

    mark_warfront_mutation_during_match(&last, &mut next);

    assert!(next.warfront_mutated_during_match);
}

#[test]
fn joust_requires_bracing_lance() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        lance: LanceState::Idle,
        score: crate::state::ScoreLedger {
            open: true,
            ..crate::state::ScoreLedger::default()
        },
        ..SkyState::default()
    };

    assert!(transition(
        &last,
        &SkyAction::Joust {
            winner: Team::Red,
            outcome: JoustOutcome::Knockback,
        },
    )
    .is_none());
}

#[test]
fn frozen_scoring_blocks_objective_atoms() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        score: crate::state::ScoreLedger {
            open: true,
            ..crate::state::ScoreLedger::default()
        },
        rules: crate::state::Rules {
            scoring_frozen: true,
            ..crate::state::Rules::baseline()
        },
        ..SkyState::default()
    };

    assert!(transition(&last, &SkyAction::CaptureOutpost).is_none());
    assert!(transition(&last, &SkyAction::ClaimShrine).is_none());
    assert!(transition(&last, &SkyAction::BlockSupplyRoute).is_none());
    assert!(transition(&last, &SkyAction::DeliverHostage).is_none());
    assert!(transition(&last, &SkyAction::BombKeepBreach).is_none());
}

#[test]
fn warfront_start_battle_reaches_battle_locked_before_match_start() {
    let preview = SkyState {
        app: AppState::WarfrontRunning,
        warfront: WarfrontState::BattlePreview,
        ..SkyState::default()
    };

    let locked =
        transition(&preview, &SkyAction::StartBattle).expect("battle preview should lock battle");
    assert_eq!(locked.app, AppState::WarfrontRunning);
    assert_eq!(locked.warfront, WarfrontState::BattleLocked);

    let started =
        transition(&locked, &SkyAction::StartBattle).expect("locked battle should start match");
    assert_eq!(started.app, AppState::MatchRunning);
    assert_eq!(started.warfront, WarfrontState::AwaitingBattleResult);
}

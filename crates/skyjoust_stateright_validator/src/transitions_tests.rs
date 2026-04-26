//! Tests for state transition edge cases.

use super::*;

#[test]
fn warfront_mutation_flag_is_set_for_in_match_changes() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        warfront: WarfrontState::AwaitingBattleResult,
        score: crate::state::ScoreLedger {
            finalized: true,
            ..crate::state::ScoreLedger::default()
        },
        rewards: crate::state::RewardLedger {
            phase: RewardPhase::Tallied,
            ..crate::state::RewardLedger::default()
        },
        ..SkyState::default()
    };

    let next = transition(&last, &SkyAction::CommitRewards).expect("reward commit should be legal");

    assert!(next.warfront_mutated_during_match);
    assert_eq!(next.warfront, WarfrontState::RewardCommit);
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

    assert!(
        transition(
            &last,
            &SkyAction::Joust {
                winner: Team::Red,
                outcome: JoustOutcome::Knockback,
            },
        )
        .is_none()
    );
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

    assert!(transition(&last, &SkyAction::CaptureOutpost { team: Team::Red }).is_none());
    assert!(transition(&last, &SkyAction::ClaimShrine { team: Team::Red }).is_none());
    assert!(transition(&last, &SkyAction::BlockSupplyRoute { team: Team::Red }).is_none());
    assert!(transition(&last, &SkyAction::DeliverHostage { team: Team::Red }).is_none());
    assert!(transition(&last, &SkyAction::BombKeepBreach { team: Team::Red }).is_none());
}

#[test]
fn objective_atoms_score_for_acting_team() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        score: crate::state::ScoreLedger {
            open: true,
            ..crate::state::ScoreLedger::default()
        },
        rules: crate::state::Rules {
            allow_sudden_death: true,
            ..crate::state::Rules::baseline()
        },
        ..SkyState::default()
    };

    let state = transition(&last, &SkyAction::CaptureOutpost { team: Team::Blue })
        .expect("blue outpost capture should score");

    assert_eq!(state.score.red_score, 0);
    assert_eq!(state.score.blue_score, 200);
    assert!(state.objectives.outpost_controlled);
}

#[test]
fn keep_breach_sets_winner_from_acting_team() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        score: crate::state::ScoreLedger {
            open: true,
            ..crate::state::ScoreLedger::default()
        },
        ..SkyState::default()
    };

    let state = transition(&last, &SkyAction::BombKeepBreach { team: Team::Blue })
        .expect("blue keep breach should score");

    assert_eq!(state.winner, Winner::Blue);
    assert_eq!(state.score.blue_score, 1000);
    assert!(state.score.victory_pending);
}

#[test]
fn timer_expired_resolves_round_when_sudden_death_disabled() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        rules: crate::state::Rules {
            allow_sudden_death: false,
            ..crate::state::Rules::baseline()
        },
        score: crate::state::ScoreLedger {
            open: true,
            red_score: 200,
            blue_score: 100,
            ..crate::state::ScoreLedger::default()
        },
        ..SkyState::default()
    };

    let state =
        transition(&last, &SkyAction::TimerExpired).expect("timer expiry should resolve round");

    assert_eq!(state.match_phase, MatchPhase::RoundOver);
    assert_eq!(state.winner, Winner::Red);
}

#[test]
fn timer_expired_enters_sudden_death_when_enabled() {
    let last = SkyState {
        match_phase: MatchPhase::NormalPlay,
        score: crate::state::ScoreLedger {
            open: true,
            ..crate::state::ScoreLedger::default()
        },
        ..SkyState::default()
    };

    let state = transition(&last, &SkyAction::TimerExpired).expect("timer expiry should be legal");

    assert_eq!(state.match_phase, MatchPhase::SuddenDeath);
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

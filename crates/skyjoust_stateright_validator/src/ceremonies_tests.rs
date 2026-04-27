//! Tests for ceremony transition handlers.

use rstest::{fixture, rstest};

use super::*;
use crate::state::{Rules, ScoreLedger};

fn active_state() -> SkyState {
    SkyState {
        match_phase: MatchPhase::NormalPlay,
        score: ScoreLedger {
            open: true,
            ..ScoreLedger::default()
        },
        ..SkyState::default()
    }
}

#[fixture]
fn duel_active_state() -> SkyState {
    SkyState {
        ceremony: CeremonyState::Duel(DuelState::DuelActive),
        match_phase: MatchPhase::EventOverride,
        ..active_state()
    }
}

#[test]
fn trigger_tournament_enters_arena_build_override() {
    let last = active_state();
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::TriggerTournament);

    assert_eq!(handled, Some(true));
    assert_eq!(
        state.ceremony,
        CeremonyState::Tournament(TournamentState::ArenaBuild)
    );
    assert_eq!(state.match_phase, MatchPhase::EventOverride);
}

#[test]
fn tournament_round_start_moves_from_arena_build_to_round_active() {
    let arena = SkyState {
        ceremony: CeremonyState::Tournament(TournamentState::ArenaBuild),
        match_phase: MatchPhase::EventOverride,
        ..active_state()
    };
    let mut registration = arena.clone();

    assert_eq!(
        handle_ceremonies(&arena, &mut registration, &SkyAction::ArenaReady),
        Some(true)
    );

    let mut round = registration.clone();
    assert_eq!(
        handle_ceremonies(&registration, &mut round, &SkyAction::TournamentRegistered,),
        Some(true)
    );
    assert_eq!(
        round.ceremony,
        CeremonyState::Tournament(TournamentState::RoundActive)
    );
}

#[test]
fn tournament_round_win_advances_to_round_complete() {
    let last = SkyState {
        ceremony: CeremonyState::Tournament(TournamentState::RoundActive),
        match_phase: MatchPhase::EventOverride,
        ..active_state()
    };
    let mut state = last.clone();

    let handled = handle_ceremonies(
        &last,
        &mut state,
        &SkyAction::TournamentRoundWon { winner: Team::Blue },
    );

    assert_eq!(handled, Some(true));
    assert_eq!(
        state.ceremony,
        CeremonyState::Tournament(TournamentState::RoundComplete)
    );
    assert_eq!(state.tournament_rounds_won, 1);
    assert_eq!(state.score.blue_score, 150);
}

#[rstest]
#[case(Team::Red, 350, 0)]
#[case(Team::Blue, 0, 350)]
fn duel_decisive_joust_scores_for_each_winning_team(
    mut duel_active_state: SkyState,
    #[case] winner: Team,
    #[case] red_score: i16,
    #[case] blue_score: i16,
) {
    duel_active_state.rules = Rules {
        duel_lock: true,
        joust_only: true,
        ordnance: crate::state::OrdnancePolicy::Disabled,
        ..Rules::baseline()
    };
    let mut state = duel_active_state.clone();

    let handled = handle_ceremonies(
        &duel_active_state,
        &mut state,
        &SkyAction::DuelDecisiveJoust {
            winner,
            outcome: JoustOutcome::CleanKill,
        },
    );

    assert_eq!(handled, Some(true));
    assert_eq!(state.score.red_score, red_score);
    assert_eq!(state.score.blue_score, blue_score);
}

#[test]
fn refuse_duel_records_refused_state() {
    let last = SkyState {
        ceremony: CeremonyState::Duel(DuelState::ChallengeIssued),
        match_phase: MatchPhase::NormalPlay,
        ..active_state()
    };
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::RefuseDuel);

    assert_eq!(handled, Some(true));
    assert_eq!(state.ceremony, CeremonyState::Duel(DuelState::Refused));
}

#[rstest]
#[case(Team::Red, -500, 0)]
#[case(Team::Blue, 0, -500)]
fn duel_interference_penalizes_only_offending_team(
    duel_active_state: SkyState,
    #[case] offender: Team,
    #[case] red_score: i16,
    #[case] blue_score: i16,
) {
    let mut state = duel_active_state.clone();

    let handled = handle_ceremonies(
        &duel_active_state,
        &mut state,
        &SkyAction::DuelInterference { offender },
    );

    assert_eq!(handled, Some(true));
    assert_eq!(state.infamy, 10);
    assert_eq!(state.score.red_score, red_score);
    assert_eq!(state.score.blue_score, blue_score);
}

#[test]
fn propose_treaty_advances_to_counter_offer() {
    let last = SkyState {
        ceremony: CeremonyState::Banquet(BanquetState::TermsOpen),
        ..active_state()
    };
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::ProposeTreaty);

    assert_eq!(handled, Some(true));
    assert_eq!(
        state.ceremony,
        CeremonyState::Banquet(BanquetState::CounterOffer)
    );
}

#[test]
fn start_wedding_truce_disables_friendly_fire() {
    let last = active_state();
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::StartWeddingTruce);

    assert_eq!(handled, Some(true));
    assert!(state.truce_active);
    assert!(!state.rules.friendly_fire);
}

#[test]
fn break_truce_restores_friendly_fire_and_records_infamy() {
    let last = SkyState {
        ceremony: CeremonyState::Wedding(WeddingState::TruceActive),
        truce_active: true,
        rules: Rules {
            friendly_fire: false,
            ..Rules::baseline()
        },
        ..active_state()
    };
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::BreakTruce);

    assert_eq!(handled, Some(true));
    assert!(!state.truce_active);
    assert!(state.rules.friendly_fire);
    assert_eq!(state.infamy, 50);
}

#[test]
fn match_scoped_ceremonies_reject_after_final_score() {
    let last = SkyState {
        score: ScoreLedger {
            finalized: true,
            ..ScoreLedger::default()
        },
        ..active_state()
    };
    let mut state = last.clone();

    assert_eq!(
        handle_ceremonies(&last, &mut state, &SkyAction::TriggerTournament),
        None
    );
}

#[test]
fn banquet_can_open_after_final_score() {
    let last = SkyState {
        app: AppState::WarfrontRunning,
        warfront: WarfrontState::StrategicChoice,
        score: ScoreLedger {
            finalized: true,
            ..ScoreLedger::default()
        },
        ..active_state()
    };
    let mut state = last.clone();

    let handled = handle_ceremonies(&last, &mut state, &SkyAction::OpenBanquet);

    assert_eq!(handled, Some(true));
    assert_eq!(state.warfront, WarfrontState::BanquetNegotiation);
    assert_eq!(
        state.ceremony,
        CeremonyState::Banquet(BanquetState::Seating)
    );
}

#[test]
fn banquet_cannot_open_during_active_ceremony() {
    let last = SkyState {
        app: AppState::WarfrontRunning,
        warfront: WarfrontState::StrategicChoice,
        ceremony: CeremonyState::ConsequenceResolution,
        ..active_state()
    };
    let mut state = last.clone();

    assert_eq!(
        handle_ceremonies(&last, &mut state, &SkyAction::OpenBanquet),
        None
    );
}

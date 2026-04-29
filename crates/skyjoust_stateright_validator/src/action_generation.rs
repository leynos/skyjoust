//! Action generation for each high-level validator concern.

use crate::{
    actions::{JoustOutcome, SkyAction, Team},
    state::{
        AppState,
        BanquetState,
        CeremonyState,
        DuelState,
        LanceState,
        MatchPhase,
        OrdnancePolicy,
        PlayerOrdnance,
        RewardPhase,
        SkyState,
        TournamentState,
        WarfrontState,
        WeddingState,
    },
};

/// Append every legal action available from `state`.
///
/// Parameters:
/// - `state` is the current validator snapshot.
/// - `actions` is extended in place with candidate `SkyAction` values.
///
/// Return semantics:
/// - The function returns no value; callers inspect the appended actions.
///
/// Preconditions:
/// - `actions` may already contain entries, and this function preserves them.
///
/// Side effects:
/// - Mutates only the supplied `actions` buffer.
pub(crate) fn available_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    push_app_actions(state, actions);
    push_match_actions(state, actions);
    push_ceremony_continuation_actions(state, actions);
    push_reward_actions(state, actions);
}

fn push_app_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.app {
        AppState::Boot => actions.push(SkyAction::AssetsLoaded),
        AppState::Title => {
            actions.push(SkyAction::StartSkirmish);
            actions.push(SkyAction::StartWarfront);
        }
        AppState::SkirmishSetup => actions.push(SkyAction::StartBattle),
        AppState::WarfrontSetup => actions.push(SkyAction::MapReady),
        AppState::WarfrontRunning => push_warfront_actions(state, actions),
        AppState::MatchRunning | AppState::Results => {}
    }
}

fn push_warfront_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.warfront {
        WarfrontState::StrategicChoice => {
            actions.push(SkyAction::SelectRegion);
            actions.push(SkyAction::OpenBanquet);
        }
        WarfrontState::BattlePreview | WarfrontState::BattleLocked => {
            actions.push(SkyAction::StartBattle);
        }
        _ => {}
    }
}

fn push_match_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.match_phase {
        MatchPhase::Constructing => actions.push(SkyAction::FinishConstructing),
        MatchPhase::SpawnWarmup => actions.push(SkyAction::SpawnReady),
        MatchPhase::Countdown => actions.push(SkyAction::CountdownDone),
        MatchPhase::NormalPlay | MatchPhase::EventOverride | MatchPhase::SuddenDeath => {
            push_live_match_actions(state, actions);
        }
        MatchPhase::RoundOver => actions.push(SkyAction::ExportFinalScore),
        MatchPhase::Paused | MatchPhase::ResultsExported | MatchPhase::Inactive => {}
    }
}

fn push_live_match_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.has_active_ceremony() {
        return;
    }

    if !state.score.victory_pending {
        push_active_match_actions(state, actions);
    }
    if state.score.victory_pending {
        actions.push(SkyAction::VictoryCheck);
    }
    if matches!(
        state.match_phase,
        MatchPhase::NormalPlay | MatchPhase::SuddenDeath
    ) {
        push_ceremony_start_actions(state, actions);
    }
}

fn push_active_match_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.score.events_accepted < 4 && !state.rules.duel_lock {
        push_lance_actions(state, actions);
        push_team_objective_actions(actions);
        push_ordnance_actions(state, actions);
    }
    actions.push(SkyAction::TimerExpired);
}

fn push_team_objective_actions(actions: &mut Vec<SkyAction>) {
    for team in [Team::Red, Team::Blue] {
        actions.push(SkyAction::CaptureOutpost { team });
        actions.push(SkyAction::ClaimShrine { team });
        actions.push(SkyAction::BlockSupplyRoute { team });
        actions.push(SkyAction::DeliverHostage { team });
    }
}

fn push_lance_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.lance {
        LanceState::Idle => actions.push(SkyAction::BracePressed),
        LanceState::Bracing => {
            push_joust_actions(actions);
            actions.push(SkyAction::BraceWindowExpired);
        }
        LanceState::Recovery | LanceState::Broken => {}
    }
}

fn push_joust_actions(actions: &mut Vec<SkyAction>) {
    for winner in [Team::Red, Team::Blue] {
        for outcome in [
            JoustOutcome::Knockback,
            JoustOutcome::Unhorse,
            JoustOutcome::Shatter,
            JoustOutcome::CleanKill,
        ] {
            actions.push(SkyAction::Joust { winner, outcome });
        }
    }
}

fn push_ordnance_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if can_spawn_ordnance(state) {
        for team in [Team::Red, Team::Blue] {
            actions.push(SkyAction::BombKeepBreach { team });
        }
    }
}

fn can_spawn_ordnance(state: &SkyState) -> bool {
    is_match_ordnance_enabled(state) && is_player_ordnance_ready(state)
}

fn is_match_ordnance_enabled(state: &SkyState) -> bool {
    matches!(
        state.rules.ordnance,
        OrdnancePolicy::Full | OrdnancePolicy::Limited
    ) && !state.rules.joust_only
}

fn is_player_ordnance_ready(state: &SkyState) -> bool {
    state.player_ordnance == PlayerOrdnance::Ready
}

fn push_ceremony_start_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.ceremony == CeremonyState::Dormant && !state.score.victory_pending {
        actions.push(SkyAction::TriggerTournament);
        actions.push(SkyAction::IssueDuel);
        actions.push(SkyAction::StartWeddingTruce);
    }
}

fn push_ceremony_continuation_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.ceremony {
        CeremonyState::Tournament(state) => push_tournament_actions(state, actions),
        CeremonyState::Duel(state) => push_duel_actions(state, actions),
        CeremonyState::Wedding(state) => push_wedding_actions(state, actions),
        CeremonyState::Banquet(state) => push_banquet_actions(state, actions),
        CeremonyState::ConsequenceResolution => actions.push(SkyAction::EventConsequencesRecorded),
        CeremonyState::Cooldown => actions.push(SkyAction::EventCooldownDone),
        CeremonyState::Dormant | CeremonyState::Queued(_) | CeremonyState::Prompt(_) => {}
    }
}

fn push_tournament_actions(state: TournamentState, actions: &mut Vec<SkyAction>) {
    match state {
        TournamentState::ArenaBuild => actions.push(SkyAction::ArenaReady),
        TournamentState::Registration => actions.push(SkyAction::TournamentRegistered),
        TournamentState::RoundActive => {
            for winner in [Team::Red, Team::Blue] {
                actions.push(SkyAction::TournamentRoundWon { winner });
            }
        }
        TournamentState::RoundComplete => actions.push(SkyAction::TournamentChampionDeclared),
        TournamentState::ChampionDeclared => {}
    }
}

fn push_duel_actions(state: DuelState, actions: &mut Vec<SkyAction>) {
    match state {
        DuelState::ChallengeIssued => {
            actions.push(SkyAction::AcceptDuel);
            actions.push(SkyAction::RefuseDuel);
        }
        DuelState::ArenaLock => actions.push(SkyAction::DuelReady),
        DuelState::DuelActive => {
            for winner in [Team::Red, Team::Blue] {
                actions.push(SkyAction::DuelDecisiveJoust {
                    winner,
                    outcome: JoustOutcome::CleanKill,
                });
            }
            actions.push(SkyAction::DuelInterference {
                offender: Team::Red,
            });
            actions.push(SkyAction::DuelInterference {
                offender: Team::Blue,
            });
        }
        DuelState::Refused | DuelState::ResolveDuel => {}
    }
}

fn push_wedding_actions(state: WeddingState, actions: &mut Vec<SkyAction>) {
    match state {
        WeddingState::TruceActive => {
            for team in [Team::Red, Team::Blue] {
                actions.push(SkyAction::CompleteJointObjective { team });
            }
            actions.push(SkyAction::BreakTruce);
            actions.push(SkyAction::ExpireTruce);
        }
        WeddingState::JointObjective => {
            actions.push(SkyAction::BreakTruce);
            actions.push(SkyAction::ExpireTruce);
        }
        WeddingState::AllianceProposed | WeddingState::Broken | WeddingState::Expired => {}
    }
}

fn push_banquet_actions(state: BanquetState, actions: &mut Vec<SkyAction>) {
    match state {
        BanquetState::Seating => actions.push(SkyAction::BanquetReady),
        BanquetState::TermsOpen => {
            actions.push(SkyAction::ProposeTreaty);
        }
        BanquetState::CounterOffer => {
            actions.push(SkyAction::AcceptTreaty);
            actions.push(SkyAction::RejectTreaty);
        }
        BanquetState::TreatySigned | BanquetState::Collapsed => {}
    }
}

fn push_reward_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.rewards.phase {
        RewardPhase::LedgerOpen => actions.push(SkyAction::TallyRewards),
        RewardPhase::Tallied => actions.push(SkyAction::CommitRewards),
        RewardPhase::Committed => push_committed_reward_actions(state, actions),
        RewardPhase::ReadyToSpend | RewardPhase::Dormant => {}
    }
}

fn push_committed_reward_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.warfront != WarfrontState::Inactive {
        actions.push(SkyAction::NextWarfrontTurn);
    } else {
        actions.push(SkyAction::ReturnToTitle);
    }
}

#[cfg(test)]
mod tests {
    //! Tests for action generation guards.

    use super::*;

    fn live_state(lance: LanceState) -> SkyState {
        SkyState {
            match_phase: MatchPhase::NormalPlay,
            lance,
            score: crate::state::ScoreLedger {
                open: true,
                ..crate::state::ScoreLedger::default()
            },
            ..SkyState::default()
        }
    }

    #[test]
    fn active_match_actions_follow_lance_lifecycle() {
        let mut idle_actions = Vec::new();
        push_active_match_actions(&live_state(LanceState::Idle), &mut idle_actions);
        assert!(idle_actions.contains(&SkyAction::BracePressed));
        assert!(
            !idle_actions
                .iter()
                .any(|action| matches!(action, SkyAction::Joust { .. }))
        );

        let mut bracing_actions = Vec::new();
        push_active_match_actions(&live_state(LanceState::Bracing), &mut bracing_actions);
        assert!(bracing_actions.contains(&SkyAction::BraceWindowExpired));
        for winner in [Team::Red, Team::Blue] {
            for outcome in [
                JoustOutcome::Knockback,
                JoustOutcome::Unhorse,
                JoustOutcome::Shatter,
                JoustOutcome::CleanKill,
            ] {
                assert!(bracing_actions.contains(&SkyAction::Joust { winner, outcome }));
            }
        }

        for team in [Team::Red, Team::Blue] {
            assert!(bracing_actions.contains(&SkyAction::CaptureOutpost { team }));
            assert!(bracing_actions.contains(&SkyAction::ClaimShrine { team }));
            assert!(bracing_actions.contains(&SkyAction::BlockSupplyRoute { team }));
            assert!(bracing_actions.contains(&SkyAction::DeliverHostage { team }));
            assert!(bracing_actions.contains(&SkyAction::BombKeepBreach { team }));
        }
    }

    #[test]
    fn live_match_actions_suppress_gameplay_during_ceremony() {
        let state = SkyState {
            ceremony: CeremonyState::Tournament(TournamentState::RoundActive),
            ..live_state(LanceState::Bracing)
        };
        let mut actions = Vec::new();

        push_live_match_actions(&state, &mut actions);

        assert!(actions.is_empty());
    }

    #[test]
    fn tournament_actions_include_both_round_winners() {
        let mut actions = Vec::new();

        push_tournament_actions(TournamentState::RoundActive, &mut actions);

        assert!(actions.contains(&SkyAction::TournamentRoundWon { winner: Team::Red }));
        assert!(actions.contains(&SkyAction::TournamentRoundWon { winner: Team::Blue }));
    }

    #[test]
    fn duel_actions_include_both_clean_kill_winners() {
        let mut actions = Vec::new();

        push_duel_actions(DuelState::DuelActive, &mut actions);

        assert!(actions.contains(&SkyAction::DuelDecisiveJoust {
            winner: Team::Red,
            outcome: JoustOutcome::CleanKill,
        }));
        assert!(actions.contains(&SkyAction::DuelDecisiveJoust {
            winner: Team::Blue,
            outcome: JoustOutcome::CleanKill,
        }));
    }
}

//! Ceremony transition handlers for tournaments, duels, weddings, and banquets.

use crate::{
    actions::{JoustOutcome, SkyAction, Team},
    scoring::{ScoreAtom, apply_dishonour_penalty, apply_joust_score, apply_objective_score},
    state::{
        AppState,
        BanquetState,
        CeremonyState,
        DuelState,
        MatchPhase,
        SkyState,
        TournamentState,
        WarfrontState,
        WeddingState,
    },
};

/// Apply a ceremony-scoped transition when `action` belongs to an event flow.
///
/// Parameters:
/// - `last` is the immutable source state used for guard checks.
/// - `state` is the already-cloned destination state to mutate.
/// - `action` is the candidate action to handle.
///
/// Return semantics:
/// - Returns `Some(true)` when a ceremony handler accepts the action.
/// - Returns `None` when the action is not a legal ceremony transition.
///
/// Preconditions:
/// - `state` must start as the caller's clone of `last`.
///
/// Side effects:
/// - Mutates ceremony, rule, score, reward, truce, and Warfront fields that belong to the accepted
///   ceremony transition.
pub(crate) fn handle_ceremonies(
    last: &SkyState,
    state: &mut SkyState,
    action: &SkyAction,
) -> Option<bool> {
    handle_tournament(last, state, action)
        .or_else(|| handle_duel(last, state, action))
        .or_else(|| handle_wedding(last, state, action))
        .or_else(|| handle_banquet(last, state, action))
        .or_else(|| handle_ceremony_common(last, state, action))
}

fn handle_tournament(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    if last.score.finalized {
        return None;
    }

    match action {
        SkyAction::TriggerTournament => start_tournament(last, state)?,
        SkyAction::ArenaReady => {
            advance_tournament_arena(last, state)?;
        }
        SkyAction::TournamentRegistered => {
            advance_tournament_registration(last, state)?;
        }
        SkyAction::TournamentRoundWon { winner } => {
            record_tournament_round_win(last, state, *winner)?;
        }
        SkyAction::TournamentChampionDeclared => declare_tournament_champion(last, state)?,
        _ => return None,
    }
    Some(true)
}

fn handle_duel(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    if last.score.finalized {
        return None;
    }

    match action {
        SkyAction::IssueDuel => {
            guard(last.can_issue_duel())?;
            state.ceremony = CeremonyState::Duel(DuelState::ChallengeIssued);
        }
        SkyAction::AcceptDuel => accept_duel(last, state)?,
        SkyAction::DuelReady => {
            guard(last.ceremony == CeremonyState::Duel(DuelState::ArenaLock))?;
            state.ceremony = CeremonyState::Duel(DuelState::DuelActive);
        }
        SkyAction::DuelDecisiveJoust { winner, outcome } => {
            resolve_duel(last, state, *winner, *outcome)?
        }
        SkyAction::DuelInterference { offender } => {
            guard(last.ceremony == CeremonyState::Duel(DuelState::DuelActive))?;
            apply_dishonour_penalty(state, *offender);
        }
        _ => return None,
    }
    Some(true)
}

fn handle_wedding(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    match action {
        SkyAction::StartWeddingTruce => start_wedding_truce(last, state)?,
        SkyAction::CompleteJointObjective { team } => {
            complete_joint_objective(last, state, *team)?;
        }
        SkyAction::BreakTruce => break_truce(last, state)?,
        SkyAction::ExpireTruce => expire_truce(last, state)?,
        _ => return None,
    }
    Some(true)
}

fn handle_banquet(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    match action {
        SkyAction::OpenBanquet => open_banquet(last, state)?,
        SkyAction::BanquetReady => {
            open_banquet_terms(last, state)?;
        }
        SkyAction::ProposeTreaty => propose_treaty(last, state)?,
        SkyAction::AcceptTreaty => accept_treaty(last, state)?,
        SkyAction::RejectTreaty => reject_treaty(last, state)?,
        _ => return None,
    }
    Some(true)
}

fn handle_ceremony_common(
    last: &SkyState,
    state: &mut SkyState,
    action: &SkyAction,
) -> Option<bool> {
    match action {
        SkyAction::EventConsequencesRecorded => record_event_consequences(last, state)?,
        SkyAction::EventCooldownDone => {
            finish_event_cooldown(last, state)?;
        }
        _ => return None,
    }
    Some(true)
}

fn start_tournament(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.can_start_tournament())?;
    state.ceremony = CeremonyState::Tournament(TournamentState::ArenaBuild);
    state.match_phase = MatchPhase::EventOverride;
    state.enter_tournament_mode();
    Some(())
}

fn advance_tournament_arena(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Tournament(TournamentState::ArenaBuild))?;
    state.ceremony = CeremonyState::Tournament(TournamentState::Registration);
    Some(())
}

fn advance_tournament_registration(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Tournament(TournamentState::Registration))?;
    state.ceremony = CeremonyState::Tournament(TournamentState::RoundActive);
    Some(())
}

fn record_tournament_round_win(last: &SkyState, state: &mut SkyState, winner: Team) -> Option<()> {
    guard(last.ceremony == CeremonyState::Tournament(TournamentState::RoundActive))?;
    apply_joust_score(state, winner, JoustOutcome::Unhorse);
    state.tournament_rounds_won = state.tournament_rounds_won.saturating_add(1);
    state.ceremony = CeremonyState::Tournament(TournamentState::RoundComplete);
    Some(())
}

fn declare_tournament_champion(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Tournament(TournamentState::RoundComplete))?;
    guard(last.tournament_rounds_won > 0)?;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.tournament_completed = true;
    state.match_phase = MatchPhase::NormalPlay;
    Some(())
}

fn accept_duel(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Duel(DuelState::ChallengeIssued))?;
    state.ceremony = CeremonyState::Duel(DuelState::ArenaLock);
    state.match_phase = MatchPhase::EventOverride;
    state.enter_duel_mode();
    Some(())
}

fn resolve_duel(
    last: &SkyState,
    state: &mut SkyState,
    winner: Team,
    outcome: JoustOutcome,
) -> Option<()> {
    guard(last.ceremony == CeremonyState::Duel(DuelState::DuelActive))?;
    guard(matches!(
        outcome,
        JoustOutcome::Unhorse | JoustOutcome::Shatter | JoustOutcome::CleanKill
    ))?;
    apply_joust_score(state, winner, outcome);
    state.duel_resolved = true;
    state.duel_consequence_active = true;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.match_phase = MatchPhase::NormalPlay;
    Some(())
}

fn start_wedding_truce(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.can_start_wedding_truce())?;
    state.ceremony = CeremonyState::Wedding(WeddingState::TruceActive);
    state.match_phase = MatchPhase::EventOverride;
    state.enter_wedding_truce_mode();
    Some(())
}

fn complete_joint_objective(last: &SkyState, state: &mut SkyState, team: Team) -> Option<()> {
    guard(last.score.open && !last.score.finalized && !last.rules.scoring_frozen)?;
    guard(last.ceremony == CeremonyState::Wedding(WeddingState::TruceActive))?;
    state.ceremony = CeremonyState::Wedding(WeddingState::JointObjective);
    apply_objective_score(state, team, ScoreAtom::HostageDeliver);
    Some(())
}

fn break_truce(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(!last.score.finalized)?;
    guard(last.in_wedding_truce())?;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.truce_active = false;
    state.truce_broken = true;
    state.rules.friendly_fire = true;
    state.infamy += 50;
    state.rewards.penalties = state.rewards.penalties.saturating_add(1);
    state.match_phase = MatchPhase::NormalPlay;
    Some(())
}

fn expire_truce(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(!last.score.finalized)?;
    guard(last.in_wedding_truce())?;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.truce_active = false;
    state.match_phase = MatchPhase::NormalPlay;
    Some(())
}

fn open_banquet(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(
        last.app == AppState::WarfrontRunning && last.warfront == WarfrontState::StrategicChoice,
    )?;
    state.warfront = WarfrontState::BanquetNegotiation;
    state.ceremony = CeremonyState::Banquet(BanquetState::Seating);
    Some(())
}

fn open_banquet_terms(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Banquet(BanquetState::Seating))?;
    state.ceremony = CeremonyState::Banquet(BanquetState::TermsOpen);
    Some(())
}

fn propose_treaty(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Banquet(BanquetState::TermsOpen))?;
    state.ceremony = CeremonyState::Banquet(BanquetState::CounterOffer);
    Some(())
}

fn accept_treaty(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Banquet(BanquetState::CounterOffer))?;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.treaty_signed = true;
    state.warfront = WarfrontState::StrategicChoice;
    Some(())
}

fn reject_treaty(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Banquet(BanquetState::CounterOffer))?;
    state.ceremony = CeremonyState::ConsequenceResolution;
    state.warfront = WarfrontState::StrategicChoice;
    state.infamy += 5;
    Some(())
}

fn record_event_consequences(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::ConsequenceResolution)?;
    state.clear_temporary_rules_if_safe();
    state.duel_consequence_active = false;
    state.ceremony = CeremonyState::Cooldown;
    Some(())
}

fn finish_event_cooldown(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.ceremony == CeremonyState::Cooldown)?;
    guard(!last.rules.duel_lock && !last.rules.joust_only)?;
    state.ceremony = CeremonyState::Dormant;
    Some(())
}

/// Convert a boolean guard into `Option<()>` for `?`-based validation.
///
/// Returns `Some(())` when `condition` is true and `None` when it is false, so
/// callers can write `guard(condition)?;` inside functions that return
/// `Option<T>`.
///
/// # Examples
///
/// ```
/// fn only_even(value: u8) -> Option<u8> {
///     fn guard(condition: bool) -> Option<()> { condition.then_some(()) }
///
///     guard(value % 2 == 0)?;
///     Some(value)
/// }
///
/// assert_eq!(only_even(4), Some(4));
/// assert_eq!(only_even(5), None);
/// ```
fn guard(condition: bool) -> Option<()> { if condition { Some(()) } else { None } }

#[cfg(test)]
#[path = "ceremonies_tests.rs"]
mod tests;

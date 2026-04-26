//! State transition handlers for the Skyjoust validator model.

use crate::{
    actions::{JoustOutcome, SkyAction, Team},
    ceremonies::handle_ceremonies,
    scoring::{
        apply_joust_score,
        apply_objective_score,
        decide_winner,
        tally_rewards,
        update_recovery_from_outcome,
        ScoreAtom,
    },
    state::{
        AppState,
        LanceState,
        MatchPhase,
        OrdnancePolicy,
        PlayerOrdnance,
        RewardPhase,
        SkyState,
        WarfrontState,
        Winner,
    },
};

/// Apply one high-level action to a state snapshot.
///
/// Parameters:
/// - `last` is the immutable source state used for all guard checks.
/// - `action` is the candidate transition to attempt.
///
/// Return semantics:
/// - Returns `Some(SkyState)` with depth incremented when the transition is legal.
/// - Returns `None` when no transition handler accepts the action.
///
/// Preconditions:
/// - The caller is responsible for enforcing exploration depth limits.
///
/// Side effects:
/// - Has no external side effects, but the returned state records accepted score, reward, ceremony,
///   and Warfront mutations. In debug builds, setting `SKYJOUST_VALIDATOR_DEBUG` prints attempted
///   actions to stderr.
pub(crate) fn transition(last: &SkyState, action: &SkyAction) -> Option<SkyState> {
    if cfg!(debug_assertions) && std::env::var("SKYJOUST_VALIDATOR_DEBUG").is_ok() {
        eprintln!(
            "skyjoust validator transition: depth={} action={action:?}",
            last.depth
        );
    }

    let mut state = last.clone();
    state.depth += 1;

    let handled = handle_app_flow(last, &mut state, action)
        .or_else(|| handle_ceremonies(last, &mut state, action))
        .or_else(|| handle_gameplay(last, &mut state, action))
        .or_else(|| handle_scoring_and_rewards(last, &mut state, action))?;

    if handled {
        mark_warfront_mutation_during_match(last, &mut state);
        Some(state)
    } else {
        None
    }
}

fn handle_app_flow(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    match action {
        SkyAction::AssetsLoaded => {
            guard(last.app == AppState::Boot)?;
            state.app = AppState::Title;
        }
        SkyAction::StartSkirmish => {
            guard(last.app == AppState::Title)?;
            state.app = AppState::SkirmishSetup;
        }
        SkyAction::StartWarfront => {
            guard(last.app == AppState::Title)?;
            state.app = AppState::WarfrontSetup;
            state.warfront = WarfrontState::GenerateOrLoad;
        }
        SkyAction::MapReady => {
            map_ready(last, state)?;
        }
        SkyAction::SelectRegion => {
            select_region(last, state)?;
        }
        SkyAction::StartBattle => {
            start_battle(last, state)?;
        }
        SkyAction::FinishConstructing => {
            guard(last.match_phase == MatchPhase::Constructing)?;
            state.match_phase = MatchPhase::SpawnWarmup;
        }
        SkyAction::SpawnReady => {
            guard(last.match_phase == MatchPhase::SpawnWarmup)?;
            state.match_phase = MatchPhase::Countdown;
        }
        SkyAction::CountdownDone => {
            guard(last.match_phase == MatchPhase::Countdown)?;
            state.match_phase = MatchPhase::NormalPlay;
            state.score.open = true;
        }
        _ => return None,
    }
    Some(true)
}

fn map_ready(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.app == AppState::WarfrontSetup)?;
    guard(last.warfront == WarfrontState::GenerateOrLoad)?;
    state.app = AppState::WarfrontRunning;
    state.warfront = WarfrontState::StrategicChoice;
    Some(())
}

fn select_region(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.app == AppState::WarfrontRunning)?;
    guard(last.warfront == WarfrontState::StrategicChoice)?;
    state.warfront = WarfrontState::BattlePreview;
    Some(())
}

fn handle_gameplay(last: &SkyState, state: &mut SkyState, action: &SkyAction) -> Option<bool> {
    match action {
        SkyAction::BracePressed => {
            guard(last.is_match_active() && last.lance == LanceState::Idle)?;
            state.lance = LanceState::Bracing;
        }
        SkyAction::BraceWindowExpired => {
            guard(last.lance == LanceState::Bracing)?;
            state.lance = LanceState::Recovery;
        }
        SkyAction::Joust { winner, outcome } => joust(last, state, *winner, *outcome)?,
        SkyAction::CaptureOutpost => capture_outpost(last, state)?,
        SkyAction::ClaimShrine => claim_shrine(last, state)?,
        SkyAction::BlockSupplyRoute => block_supply_route(last, state)?,
        SkyAction::DeliverHostage => deliver_hostage(last, state)?,
        SkyAction::BombKeepBreach => bomb_keep_breach(last, state)?,
        SkyAction::TimerExpired => {
            guard(last.is_match_active())?;
            state.match_phase = MatchPhase::SuddenDeath;
        }
        SkyAction::VictoryCheck => {
            guard(last.is_match_active() && last.score.victory_pending)?;
            state.match_phase = MatchPhase::RoundOver;
            if state.winner == Winner::None {
                state.winner = decide_winner(&state.score);
            }
        }
        _ => return None,
    }
    Some(true)
}

fn handle_scoring_and_rewards(
    last: &SkyState,
    state: &mut SkyState,
    action: &SkyAction,
) -> Option<bool> {
    match action {
        SkyAction::ExportFinalScore => export_final_score(last, state)?,
        SkyAction::TallyRewards => {
            guard(last.rewards.phase == RewardPhase::LedgerOpen && last.score.finalized)?;
            tally_rewards(state);
        }
        SkyAction::CommitRewards => commit_rewards(last, state)?,
        SkyAction::NextWarfrontTurn => {
            guard(
                last.rewards.phase == RewardPhase::Committed
                    && last.warfront != WarfrontState::Inactive,
            )?;
            state.app = AppState::WarfrontRunning;
            state.match_phase = MatchPhase::Inactive;
            state.warfront = WarfrontState::StrategicChoice;
            state.rewards.phase = RewardPhase::ReadyToSpend;
        }
        SkyAction::ReturnToTitle => {
            guard(last.rewards.phase == RewardPhase::Committed || last.app == AppState::Results)?;
            *state = SkyState::default();
            state.depth = last.depth.saturating_add(1);
            state.app = AppState::Title;
        }
        _ => return None,
    }
    Some(true)
}

fn start_battle(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(matches!(
        last.app,
        AppState::SkirmishSetup | AppState::WarfrontRunning
    ))?;
    if last.app == AppState::WarfrontRunning {
        if last.can_start_warfront_battle() {
            state.warfront = WarfrontState::BattleLocked;
            return Some(());
        }
        guard(last.warfront == WarfrontState::BattleLocked)?;
        state.warfront = WarfrontState::AwaitingBattleResult;
    }
    state.app = AppState::MatchRunning;
    state.reset_for_match_start();
    Some(())
}

fn joust(last: &SkyState, state: &mut SkyState, winner: Team, outcome: JoustOutcome) -> Option<()> {
    guard(last.is_match_active() && last.score.open && !last.rules.scoring_frozen)?;
    guard(last.lance == LanceState::Bracing)?;
    guard(!last.rules.duel_lock)?;
    apply_joust_score(state, winner, outcome);
    state.lance = LanceState::Recovery;
    update_recovery_from_outcome(state, winner, outcome);
    Some(())
}

fn capture_outpost(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(can_score_gameplay(last))?;
    guard(last.is_match_active() && !last.objectives.outpost_controlled && !last.rules.duel_lock)?;
    state.objectives.outpost_controlled = true;
    apply_objective_score(state, Team::Red, ScoreAtom::OutpostCapture);
    Some(())
}

fn claim_shrine(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(can_score_gameplay(last))?;
    guard(last.is_match_active() && !last.objectives.shrine_claimed && !last.rules.duel_lock)?;
    state.objectives.shrine_claimed = true;
    apply_objective_score(state, Team::Red, ScoreAtom::ShrineClaim);
    Some(())
}

fn block_supply_route(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(can_score_gameplay(last))?;
    guard(
        last.is_match_active() && !last.objectives.supply_route_blocked && !last.rules.duel_lock,
    )?;
    state.objectives.supply_route_blocked = true;
    apply_objective_score(state, Team::Red, ScoreAtom::SupplyRouteBlock);
    Some(())
}

fn deliver_hostage(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(can_score_gameplay(last))?;
    guard(last.is_match_active() && !last.objectives.hostage_delivered && !last.rules.duel_lock)?;
    state.objectives.hostage_delivered = true;
    apply_objective_score(state, Team::Red, ScoreAtom::HostageDeliver);
    Some(())
}

fn bomb_keep_breach(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.is_match_active())?;
    guard(can_score_gameplay(last))?;
    guard(matches!(
        last.rules.ordnance,
        OrdnancePolicy::Full | OrdnancePolicy::Limited
    ))?;
    guard(last.player_ordnance == PlayerOrdnance::Ready && !last.rules.joust_only)?;
    state.player_ordnance = PlayerOrdnance::Cooldown;
    state.objectives.keep_breached = true;
    apply_objective_score(state, Team::Red, ScoreAtom::KeepBreach);
    state.score.victory_pending = true;
    state.winner = Winner::Red;
    Some(())
}

fn can_score_gameplay(state: &SkyState) -> bool { state.score.open && !state.rules.scoring_frozen }

fn export_final_score(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.match_phase == MatchPhase::RoundOver)?;
    state.match_phase = MatchPhase::ResultsExported;
    state.app = AppState::Results;
    state.clear_temporary_rules_if_safe();
    state.ceremony = crate::state::CeremonyState::Dormant;
    state.duel_consequence_active = false;
    state.score.open = false;
    state.score.finalized = true;
    state.rewards.phase = RewardPhase::LedgerOpen;
    state.rewards.pending_delta = true;
    Some(())
}

fn commit_rewards(last: &SkyState, state: &mut SkyState) -> Option<()> {
    guard(last.rewards.phase == RewardPhase::Tallied && last.score.finalized)?;
    guard(last.match_phase != MatchPhase::Paused)?;
    state.rewards.phase = RewardPhase::Committed;
    state.rewards.committed = true;
    state.rewards.pending_delta = false;
    if last.warfront == WarfrontState::AwaitingBattleResult {
        state.warfront = WarfrontState::RewardCommit;
    }
    Some(())
}

fn mark_warfront_mutation_during_match(last: &SkyState, state: &mut SkyState) {
    if last.is_in_match_or_building_match() && last.warfront != state.warfront {
        state.warfront_mutated_during_match = true;
    }
}

/// Convert a boolean guard into `Option<()>` for `?`-based validation.
///
/// Returns `Some(())` when `condition` is true and `None` when it is false, so
/// callers can write `guard(condition)?;` inside functions that return
/// `Option<T>`.
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
fn guard(condition: bool) -> Option<()> {
    if condition {
        Some(())
    } else {
        None
    }
}

#[cfg(test)]
#[path = "transitions_tests.rs"]
mod tests;

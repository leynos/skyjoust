//! Scoring, morale, recovery, and reward helper logic.

use crate::{
    actions::{JoustOutcome, Team},
    state::{RecoveryState, RewardPhase, ScoreLedger, SkyState, Winner},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ScoreAtom {
    OutpostCapture,
    ShrineClaim,
    SupplyRouteBlock,
    HostageDeliver,
    KeepBreach,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoraleEffect {
    ScoringTeam(i16),
    Opponent(i16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScoreComparison {
    RedLeads,
    BlueLeads,
    Tied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScoreDelta {
    score: i16,
    glory: i16,
    morale_effect: MoraleEffect,
}

/// Apply the score, glory, and morale effect for one joust outcome.
///
/// Parameters:
/// - `s` is the match state to update.
/// - `winner` selects the team receiving points and glory.
/// - `outcome` selects the score atom emitted by the contact.
///
/// Return semantics:
/// - Returns no value; callers inspect the mutated state.
///
/// Preconditions:
/// - Callers must already have checked that scoring is legal for the current match phase.
///
/// Side effects:
/// - Opens the score ledger, increments accepted events, adjusts morale, and may mark victory
///   pending. If the score was finalized, records a rejected post-final write instead.
pub(crate) fn apply_joust_score(s: &mut SkyState, winner: Team, outcome: JoustOutcome) {
    if reject_finalized_score_write(s) {
        return;
    }
    let delta = match outcome {
        JoustOutcome::Knockback => ScoreDelta {
            score: 80,
            glory: 10,
            morale_effect: MoraleEffect::Opponent(-1),
        },
        JoustOutcome::Unhorse => ScoreDelta {
            score: 150,
            glory: 20,
            morale_effect: MoraleEffect::Opponent(-3),
        },
        JoustOutcome::Shatter => ScoreDelta {
            score: 220,
            glory: 30,
            morale_effect: MoraleEffect::Opponent(-4),
        },
        JoustOutcome::CleanKill => ScoreDelta {
            score: 350,
            glory: 45,
            morale_effect: MoraleEffect::Opponent(-7),
        },
    };
    apply_score_delta(s, winner, delta);
}

/// Apply score and morale effects for a non-joust objective atom.
///
/// Parameters:
/// - `s` is the match state to update.
/// - `winner` selects the team credited with the objective.
/// - `atom` selects the objective scoring rule.
///
/// Return semantics:
/// - Returns no value; callers inspect the mutated state.
///
/// Preconditions:
/// - Callers must already have checked objective legality and scoring openness.
///
/// Side effects:
/// - Mutates score, glory, morale, event count, and possible victory state. If the score was
///   finalized, records a rejected post-final write instead.
pub(crate) fn apply_objective_score(s: &mut SkyState, winner: Team, atom: ScoreAtom) {
    if reject_finalized_score_write(s) {
        return;
    }
    let delta = match atom {
        ScoreAtom::OutpostCapture => ScoreDelta {
            score: 200,
            glory: 0,
            morale_effect: MoraleEffect::ScoringTeam(2),
        },
        ScoreAtom::ShrineClaim => ScoreDelta {
            score: 120,
            glory: 0,
            morale_effect: MoraleEffect::ScoringTeam(1),
        },
        ScoreAtom::SupplyRouteBlock => ScoreDelta {
            score: 160,
            glory: 0,
            morale_effect: MoraleEffect::Opponent(-2),
        },
        ScoreAtom::HostageDeliver => ScoreDelta {
            score: 250,
            glory: 0,
            morale_effect: MoraleEffect::ScoringTeam(3),
        },
        ScoreAtom::KeepBreach => ScoreDelta {
            score: 1000,
            glory: 100,
            morale_effect: MoraleEffect::Opponent(-999),
        },
    };
    apply_score_delta(s, winner, delta);
}

/// Deduct points and record infamy for a dishonour violation.
///
/// Parameters:
/// - `s` is the match state to update.
/// - `offender` is the team that receives the point deduction.
///
/// Return semantics:
/// - Returns no value; callers inspect the mutated state.
///
/// Preconditions:
/// - Callers must determine the offending team from the active rule context.
///
/// Side effects:
/// - Increments infamy and reward penalties, deducts points from the offender, and marks the score
///   ledger as having a pending delta.
pub(crate) fn apply_dishonour_penalty(s: &mut SkyState, offender: Team) {
    s.infamy += 10;
    s.rewards.penalties = s.rewards.penalties.saturating_add(1);
    match offender {
        Team::Red => {
            s.score.red_score -= 500;
        }
        Team::Blue => {
            s.score.blue_score -= 500;
        }
    }
    s.score.pending_delta = true;
}

/// Update local recovery state from a joust result.
///
/// Parameters:
/// - `s` is the match state to update.
/// - `winner` identifies which team won the contact.
/// - `outcome` selects the recovery severity.
///
/// Return semantics:
/// - Returns no value; callers inspect `s.recovery`.
///
/// Preconditions:
/// - The model treats Blue winning as the local rider losing the exchange.
///
/// Side effects:
/// - Mutates only `s.recovery`.
pub(crate) fn update_recovery_from_outcome(s: &mut SkyState, winner: Team, outcome: JoustOutcome) {
    let local_lost = winner == Team::Blue;
    s.recovery = match (local_lost, outcome) {
        (true, JoustOutcome::Knockback) => RecoveryState::Stunned,
        (true, JoustOutcome::Unhorse) => RecoveryState::Unhorsed,
        (true, JoustOutcome::Shatter | JoustOutcome::CleanKill) => RecoveryState::Dead,
        (false, _) => RecoveryState::Alive,
    };
}

/// Classify the final winner from the score ledger.
///
/// Parameters:
/// - `score` is the finalized or pending score ledger to compare.
///
/// Return semantics:
/// - Returns `Winner::Red`, `Winner::Blue`, or `Winner::TieBreak`.
///
/// Preconditions:
/// - The caller should only use this once the match is ready to resolve.
///
/// Side effects:
/// - None.
pub(crate) fn decide_winner(score: &ScoreLedger) -> Winner {
    match compare_scores(score) {
        ScoreComparison::RedLeads => Winner::Red,
        ScoreComparison::BlueLeads => Winner::Blue,
        ScoreComparison::Tied => Winner::TieBreak,
    }
}

/// Convert a finalized score snapshot into pending reward ledger entries.
///
/// Parameters:
/// - `s` is the state containing the finalized score and event flags.
///
/// Return semantics:
/// - Returns no value; callers inspect `s.rewards`.
///
/// Preconditions:
/// - Callers must ensure the reward ledger is open and score finalized.
///
/// Side effects:
/// - Moves rewards to `Tallied`, leaves the match inactive, and applies victory, tournament, duel,
///   treaty, and truce-break reward modifiers.
pub(crate) fn tally_rewards(s: &mut SkyState) {
    s.rewards.phase = RewardPhase::Tallied;
    s.rewards.pending_delta = true;
    s.match_phase = crate::state::MatchPhase::Inactive;

    s.rewards.glory += 20;
    s.rewards.coin += 10;
    match s.winner {
        Winner::Red => {
            s.rewards.glory += 60 + s.score.red_glory;
            s.rewards.coin += 40;
            s.rewards.influence += 10;
        }
        Winner::Blue => {
            s.rewards.glory += 15 + s.score.blue_glory;
            s.rewards.coin += 15;
        }
        Winner::TieBreak | Winner::None => {
            s.rewards.glory += 15;
            s.rewards.coin += 15;
        }
    }

    if s.tournament_completed {
        s.rewards.laurels = s.rewards.laurels.saturating_add(3);
        s.rewards.glory += 100;
        s.rewards.tournament_bonus_granted = true;
    }
    if s.duel_resolved {
        s.rewards.glory += 50;
        s.rewards.influence += 25;
        s.rewards.duel_bonus_granted = true;
    }
    if s.treaty_signed {
        s.rewards.influence += 40;
    }
    if s.truce_broken {
        s.rewards.influence -= 50;
        s.rewards.coin -= 30;
        if s.rewards.penalties == 0 {
            s.rewards.penalties = 1;
        }
    }
}

fn reject_finalized_score_write(s: &mut SkyState) -> bool {
    if s.score.finalized {
        s.post_final_score_write = true;
        true
    } else {
        false
    }
}

fn apply_score_delta(s: &mut SkyState, winner: Team, delta: ScoreDelta) {
    s.score.open = true;
    s.score.pending_delta = true;
    s.score.events_accepted = s.score.events_accepted.saturating_add(1);
    match winner {
        Team::Red => apply_red_score_delta(s, delta),
        Team::Blue => apply_blue_score_delta(s, delta),
    }
}

fn apply_red_score_delta(s: &mut SkyState, delta: ScoreDelta) {
    s.score.red_score += delta.score;
    s.score.red_glory += delta.glory;
    match delta.morale_effect {
        MoraleEffect::ScoringTeam(delta) => s.score.red_morale += delta,
        MoraleEffect::Opponent(delta) => s.score.blue_morale += delta,
    }
    if s.score.blue_morale <= 0 {
        s.score.victory_pending = true;
        s.winner = Winner::Red;
    }
}

fn apply_blue_score_delta(s: &mut SkyState, delta: ScoreDelta) {
    s.score.blue_score += delta.score;
    s.score.blue_glory += delta.glory;
    match delta.morale_effect {
        MoraleEffect::ScoringTeam(delta) => s.score.blue_morale += delta,
        MoraleEffect::Opponent(delta) => s.score.red_morale += delta,
    }
    if s.score.red_morale <= 0 {
        s.score.victory_pending = true;
        s.winner = Winner::Blue;
    }
}

fn compare_scores(score: &ScoreLedger) -> ScoreComparison {
    match score.red_score.cmp(&score.blue_score) {
        std::cmp::Ordering::Greater => ScoreComparison::RedLeads,
        std::cmp::Ordering::Less => ScoreComparison::BlueLeads,
        std::cmp::Ordering::Equal => ScoreComparison::Tied,
    }
}

#[cfg(test)]
mod tests {
    //! Tests for team-aware scoring helper semantics.

    use super::*;

    #[test]
    fn objective_team_morale_buffs_scoring_team() {
        let mut state = SkyState::default();

        apply_objective_score(&mut state, Team::Red, ScoreAtom::OutpostCapture);

        assert_eq!(state.score.red_morale, 12);
        assert_eq!(state.score.blue_morale, 10);
    }

    #[test]
    fn objective_target_morale_penalizes_opponent() {
        let mut state = SkyState::default();

        apply_objective_score(&mut state, Team::Red, ScoreAtom::SupplyRouteBlock);

        assert_eq!(state.score.red_morale, 10);
        assert_eq!(state.score.blue_morale, 8);
    }

    #[test]
    fn dishonour_penalty_hits_offending_team() {
        let mut state = SkyState::default();

        apply_dishonour_penalty(&mut state, Team::Blue);

        assert_eq!(state.score.red_score, 0);
        assert_eq!(state.score.blue_score, -500);
    }
}

//! Score and reward ledger types for the Skyjoust validator model.

use serde::{Deserialize, Serialize};

/// Score, glory, morale, and finalization state for the current match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independently toggled ledger state, not a set of related options"
)]
pub struct ScoreLedger {
    /// Whether the ledger is open to accept score events.
    pub open: bool,
    /// Whether the ledger has been finalized for the match.
    pub finalized: bool,
    /// Whether a score delta is pending application.
    pub pending_delta: bool,
    /// Number of score events accepted so far.
    pub events_accepted: u8,
    /// Red team's accumulated score.
    pub red_score: i16,
    /// Blue team's accumulated score.
    pub blue_score: i16,
    /// Red team's accumulated glory.
    pub red_glory: i16,
    /// Blue team's accumulated glory.
    pub blue_glory: i16,
    /// Red team's accumulated morale.
    pub red_morale: i16,
    /// Blue team's accumulated morale.
    pub blue_morale: i16,
    /// Whether a victory check is pending.
    pub victory_pending: bool,
}

impl Default for ScoreLedger {
    fn default() -> Self {
        Self {
            open: false,
            finalized: false,
            pending_delta: false,
            events_accepted: 0,
            red_score: 0,
            blue_score: 0,
            red_glory: 0,
            blue_glory: 0,
            red_morale: 10,
            blue_morale: 10,
            victory_pending: false,
        }
    }
}

/// Reward payout state derived from a finalized score snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independently toggled ledger state, not a set of related options"
)]
pub struct RewardLedger {
    /// Reward ledger lifecycle phase.
    pub phase: RewardPhase,
    /// Whether a reward delta is pending application.
    pub pending_delta: bool,
    /// Whether the tallied rewards have been committed.
    pub committed: bool,
    /// Accumulated glory reward.
    pub glory: i16,
    /// Accumulated coin reward.
    pub coin: i16,
    /// Accumulated influence reward.
    pub influence: i16,
    /// Accumulated laurels reward.
    pub laurels: u8,
    /// Accumulated penalties applied to the reward payout.
    pub penalties: u8,
    /// Whether the tournament bonus has been granted.
    pub tournament_bonus_granted: bool,
    /// Whether the duel bonus has been granted.
    pub duel_bonus_granted: bool,
}

impl Default for RewardLedger {
    fn default() -> Self {
        Self {
            phase: RewardPhase::Dormant,
            pending_delta: false,
            committed: false,
            glory: 0,
            coin: 0,
            influence: 0,
            laurels: 0,
            penalties: 0,
            tournament_bonus_granted: false,
            duel_bonus_granted: false,
        }
    }
}

/// Reward ledger lifecycle phase used to gate payout transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardPhase {
    /// No reward ledger is open.
    Dormant,
    /// The reward ledger is open to accept reward events.
    LedgerOpen,
    /// Rewards have been tallied from the final score.
    Tallied,
    /// Tallied rewards have been committed.
    Committed,
    /// Committed rewards are ready to spend.
    ReadyToSpend,
}

impl RewardPhase {
    /// Return whether this phase has opened the reward ledger.
    ///
    /// Parameters:
    /// - `self` is the phase to classify.
    ///
    /// Return semantics:
    /// - Returns `false` for `Dormant`.
    /// - Returns `true` once rewards are open, tallied, committed, or spendable.
    ///
    /// Preconditions:
    /// - None.
    ///
    /// Side effects:
    /// - None.
    pub(crate) const fn is_open(self) -> bool {
        matches!(
            self,
            Self::LedgerOpen | Self::Tallied | Self::Committed | Self::ReadyToSpend
        )
    }
}

#[cfg(test)]
mod tests {
    //! Tests for score and reward ledgers.

    use super::*;

    #[test]
    fn reward_phase_is_open_after_dormant() {
        assert!(!RewardPhase::Dormant.is_open());
        assert!(RewardPhase::LedgerOpen.is_open());
        assert!(RewardPhase::Tallied.is_open());
        assert!(RewardPhase::Committed.is_open());
        assert!(RewardPhase::ReadyToSpend.is_open());
    }

    #[test]
    fn score_ledger_default_starts_morale_at_ten() {
        let ledger = ScoreLedger::default();

        assert_eq!(ledger.red_morale, 10);
        assert_eq!(ledger.blue_morale, 10);
    }

    #[test]
    fn reward_ledger_default_starts_dormant() {
        assert_eq!(RewardLedger::default().phase, RewardPhase::Dormant);
    }
}

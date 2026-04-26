//! Score and reward ledger types for the Skyjoust validator model.

use serde::{Deserialize, Serialize};

/// Score, glory, morale, and finalization state for the current match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScoreLedger {
    pub open: bool,
    pub finalized: bool,
    pub pending_delta: bool,
    pub events_accepted: u8,
    pub red_score: i16,
    pub blue_score: i16,
    pub red_glory: i16,
    pub blue_glory: i16,
    pub red_morale: i16,
    pub blue_morale: i16,
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
pub struct RewardLedger {
    pub phase: RewardPhase,
    pub pending_delta: bool,
    pub committed: bool,
    pub glory: i16,
    pub coin: i16,
    pub influence: i16,
    pub laurels: u8,
    pub penalties: u8,
    pub tournament_bonus_granted: bool,
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
    Dormant,
    LedgerOpen,
    Tallied,
    Committed,
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
    pub(crate) fn is_open(self) -> bool {
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

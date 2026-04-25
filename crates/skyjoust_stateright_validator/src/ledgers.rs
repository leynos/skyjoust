//! Score and reward ledger types for the Skyjoust validator model.

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardPhase {
    Dormant,
    LedgerOpen,
    Tallied,
    Committed,
    ReadyToSpend,
}

impl RewardPhase {
    pub(crate) fn is_open(self) -> bool {
        matches!(
            self,
            Self::LedgerOpen | Self::Tallied | Self::Committed | Self::ReadyToSpend
        )
    }
}

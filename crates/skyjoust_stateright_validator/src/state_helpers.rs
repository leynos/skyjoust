//! Helper methods for `SkyState`.

use crate::state::{
    AppState,
    CeremonyState,
    LanceState,
    MatchPhase,
    ObjectiveSnapshot,
    OrdnancePolicy,
    PlayerOrdnance,
    RecoveryState,
    RewardLedger,
    Rules,
    ScoreLedger,
    SkyState,
    WarfrontState,
    Winner,
};

impl SkyState {
    /// Return true when the match can accept active gameplay actions.
    ///
    /// ```
    /// use skyjoust_stateright_validator::{MatchPhase, SkyState};
    ///
    /// let state = SkyState {
    ///     match_phase: MatchPhase::NormalPlay,
    ///     ..SkyState::default()
    /// };
    /// assert!(state.is_match_active());
    /// ```
    pub fn is_match_active(&self) -> bool {
        matches!(
            self.match_phase,
            MatchPhase::NormalPlay | MatchPhase::EventOverride | MatchPhase::SuddenDeath
        )
    }

    /// Return true while the match lifecycle is constructing, active, or exporting.
    ///
    /// ```
    /// use skyjoust_stateright_validator::{MatchPhase, SkyState};
    ///
    /// let state = SkyState {
    ///     match_phase: MatchPhase::Constructing,
    ///     ..SkyState::default()
    /// };
    /// assert!(state.is_in_match_or_building_match());
    /// ```
    pub fn is_in_match_or_building_match(&self) -> bool {
        matches!(
            self.match_phase,
            MatchPhase::Constructing
                | MatchPhase::SpawnWarmup
                | MatchPhase::Countdown
                | MatchPhase::NormalPlay
                | MatchPhase::EventOverride
                | MatchPhase::SuddenDeath
                | MatchPhase::Paused
                | MatchPhase::RoundOver
                | MatchPhase::ResultsExported
        )
    }

    /// Return true when a ceremony flow owns the current interaction state.
    ///
    /// ```
    /// use skyjoust_stateright_validator::{CeremonyState, DuelState, SkyState};
    ///
    /// let state = SkyState {
    ///     ceremony: CeremonyState::Duel(DuelState::DuelActive),
    ///     ..SkyState::default()
    /// };
    /// assert!(state.has_active_ceremony());
    /// ```
    pub fn has_active_ceremony(&self) -> bool {
        matches!(
            self.ceremony,
            CeremonyState::Tournament(_)
                | CeremonyState::Duel(_)
                | CeremonyState::Wedding(_)
                | CeremonyState::Banquet(_)
                | CeremonyState::ConsequenceResolution
        )
    }

    /// Return true when tournament ceremony activation is legal.
    ///
    /// Parameters: `self` is the current state snapshot.
    /// Return semantics: true means the match is active and no ceremony owns it.
    /// Preconditions: none.
    /// Side effects: none.
    pub(crate) fn can_start_tournament(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    /// Return true when a duel challenge can be issued.
    ///
    /// Parameters: `self` is the current state snapshot.
    /// Return semantics: true means active match play is ceremony-free.
    /// Preconditions: none.
    /// Side effects: none.
    pub(crate) fn can_issue_duel(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    /// Return true when a wedding-truce ceremony can start.
    ///
    /// Parameters: `self` is the current state snapshot.
    /// Return semantics: true means active match play is ceremony-free.
    /// Preconditions: none.
    /// Side effects: none.
    pub(crate) fn can_start_wedding_truce(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    /// Return true when the selected Warfront battle can be locked.
    ///
    /// Parameters: `self` is the current state snapshot.
    /// Return semantics: true means Warfront is running from battle preview.
    /// Preconditions: none.
    /// Side effects: none.
    pub(crate) fn can_start_warfront_battle(&self) -> bool {
        self.app == AppState::WarfrontRunning && self.warfront == WarfrontState::BattlePreview
    }

    /// Return true when a wedding ceremony currently enforces a truce.
    ///
    /// Parameters: `self` is the current state snapshot.
    /// Return semantics: true means `truce_active` and wedding state agree.
    /// Preconditions: none.
    /// Side effects: none.
    pub(crate) fn in_wedding_truce(&self) -> bool {
        self.truce_active && matches!(self.ceremony, CeremonyState::Wedding(_))
    }

    /// Apply tournament-only temporary rules.
    ///
    /// Parameters: `self` is the destination state to mutate.
    /// Return semantics: no return value.
    /// Preconditions: callers have accepted a tournament-start transition.
    /// Side effects: disables ordnance and forces joust-only player rules.
    pub(crate) fn enter_tournament_mode(&mut self) {
        self.rules.ordnance = OrdnancePolicy::Disabled;
        self.rules.joust_only = true;
        self.player_ordnance = PlayerOrdnance::Disabled;
    }

    /// Apply duel-only temporary rules.
    ///
    /// Parameters: `self` is the destination state to mutate.
    /// Return semantics: no return value.
    /// Preconditions: callers have accepted a duel-start transition.
    /// Side effects: enables duel lock, disables ordnance, and forces
    /// joust-only player rules.
    pub(crate) fn enter_duel_mode(&mut self) {
        self.rules.duel_lock = true;
        self.rules.ordnance = OrdnancePolicy::Disabled;
        self.rules.joust_only = true;
        self.player_ordnance = PlayerOrdnance::Disabled;
    }

    /// Apply wedding-truce temporary rules.
    ///
    /// Parameters: `self` is the destination state to mutate.
    /// Return semantics: no return value.
    /// Preconditions: callers have accepted a wedding-truce transition.
    /// Side effects: activates truce state and disables friendly fire.
    pub(crate) fn enter_wedding_truce_mode(&mut self) {
        self.truce_active = true;
        self.rules.friendly_fire = false;
    }

    /// Reset per-match fields before entering construction.
    ///
    /// Parameters: `self` is the state being reused for the next match.
    /// Return semantics: no return value.
    /// Preconditions: the app flow has accepted a battle-start transition.
    /// Side effects: resets match, score, reward, ceremony, rule, objective,
    /// recovery, and event-derived flags while preserving campaign context.
    pub(crate) fn reset_for_match_start(&mut self) {
        self.match_phase = MatchPhase::Constructing;
        self.ceremony = CeremonyState::Dormant;
        self.rules = Rules::baseline();
        self.player_ordnance = PlayerOrdnance::Ready;
        self.lance = LanceState::Idle;
        self.recovery = RecoveryState::Alive;
        self.objectives = ObjectiveSnapshot::default();
        self.score = ScoreLedger::default();
        self.rewards = RewardLedger::default();
        self.winner = Winner::None;
        self.truce_active = false;
        self.truce_broken = false;
        self.tournament_rounds_won = 0;
        self.tournament_completed = false;
        self.duel_resolved = false;
        self.duel_consequence_active = false;
        self.post_final_score_write = false;
        self.warfront_mutated_during_match = false;
    }

    /// Restore baseline rules unless a live truce still owns an override.
    ///
    /// Parameters: `self` is the state whose temporary rules are being cleared.
    /// Return semantics: no return value.
    /// Preconditions: callers are leaving ceremony consequence handling.
    /// Side effects: resets rules and recalculates player ordnance readiness.
    pub(crate) fn clear_temporary_rules_if_safe(&mut self) {
        self.rules = if self.truce_active {
            Rules {
                friendly_fire: false,
                ..Rules::baseline()
            }
        } else {
            Rules::baseline()
        };
        self.player_ordnance =
            if self.rules.joust_only || self.rules.ordnance == OrdnancePolicy::Disabled {
                PlayerOrdnance::Disabled
            } else {
                PlayerOrdnance::Ready
            };
    }
}

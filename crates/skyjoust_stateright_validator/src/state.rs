//! State types and guard helpers for the Skyjoust validator model.

use serde::{Deserialize, Serialize};

pub use crate::ledgers::{RewardLedger, RewardPhase, ScoreLedger};

/// Complete validator snapshot for app, match, ceremony, scoring, and rewards.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkyState {
    pub depth: u8,
    pub app: AppState,
    pub warfront: WarfrontState,
    pub match_phase: MatchPhase,
    pub ceremony: CeremonyState,
    pub rules: Rules,
    pub player_ordnance: PlayerOrdnance,
    pub lance: LanceState,
    pub recovery: RecoveryState,
    pub objectives: ObjectiveSnapshot,
    pub score: ScoreLedger,
    pub rewards: RewardLedger,
    pub winner: Winner,
    pub truce_active: bool,
    pub truce_broken: bool,
    pub tournament_rounds_won: u8,
    pub tournament_completed: bool,
    pub duel_resolved: bool,
    pub duel_consequence_active: bool,
    pub treaty_signed: bool,
    pub infamy: i16,
    pub post_final_score_write: bool,
    pub warfront_mutated_during_match: bool,
}

impl Default for SkyState {
    fn default() -> Self {
        Self {
            depth: 0,
            app: AppState::Boot,
            warfront: WarfrontState::Inactive,
            match_phase: MatchPhase::Inactive,
            ceremony: CeremonyState::Dormant,
            rules: Rules::baseline(),
            player_ordnance: PlayerOrdnance::Ready,
            lance: LanceState::Idle,
            recovery: RecoveryState::Alive,
            objectives: ObjectiveSnapshot::default(),
            score: ScoreLedger::default(),
            rewards: RewardLedger::default(),
            winner: Winner::None,
            truce_active: false,
            truce_broken: false,
            tournament_rounds_won: 0,
            tournament_completed: false,
            duel_resolved: false,
            duel_consequence_active: false,
            treaty_signed: false,
            infamy: 0,
            post_final_score_write: false,
            warfront_mutated_during_match: false,
        }
    }
}

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

    pub(crate) fn can_start_tournament(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    pub(crate) fn can_issue_duel(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    pub(crate) fn can_start_wedding_truce(&self) -> bool {
        self.is_match_active() && self.ceremony == CeremonyState::Dormant
    }

    pub(crate) fn can_start_warfront_battle(&self) -> bool {
        self.app == AppState::WarfrontRunning && self.warfront == WarfrontState::BattlePreview
    }

    pub(crate) fn in_wedding_truce(&self) -> bool {
        self.truce_active && matches!(self.ceremony, CeremonyState::Wedding(_))
    }

    pub(crate) fn enter_tournament_mode(&mut self) {
        self.rules.ordnance = OrdnancePolicy::Disabled;
        self.rules.joust_only = true;
        self.player_ordnance = PlayerOrdnance::Disabled;
    }

    pub(crate) fn enter_duel_mode(&mut self) {
        self.rules.duel_lock = true;
        self.rules.ordnance = OrdnancePolicy::Disabled;
        self.rules.joust_only = true;
        self.player_ordnance = PlayerOrdnance::Disabled;
    }

    pub(crate) fn enter_wedding_truce_mode(&mut self) {
        self.truce_active = true;
        self.rules.friendly_fire = false;
    }

    pub(crate) fn reset_for_match_start(&mut self) {
        self.match_phase = MatchPhase::Constructing;
        self.ceremony = CeremonyState::Dormant;
        self.rules = Rules::baseline();
        self.player_ordnance = PlayerOrdnance::Ready;
        self.lance = LanceState::Idle;
        self.recovery = RecoveryState::Alive;
        self.objectives = ObjectiveSnapshot::default();
        self.score = ScoreLedger {
            open: true,
            ..ScoreLedger::default()
        };
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

/// Top-level application mode explored by the validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppState {
    Boot,
    Title,
    SkirmishSetup,
    WarfrontSetup,
    WarfrontRunning,
    MatchRunning,
    Results,
}

/// Warfront campaign state relevant to match handoff and reward commits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarfrontState {
    Inactive,
    GenerateOrLoad,
    StrategicChoice,
    BattlePreview,
    BattleLocked,
    AwaitingBattleResult,
    ApplyBattleResult,
    RewardCommit,
    BanquetNegotiation,
    SeasonComplete,
}

/// Match lifecycle phase used to gate scoring, rewards, and Warfront mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchPhase {
    Inactive,
    Constructing,
    SpawnWarmup,
    Countdown,
    NormalPlay,
    EventOverride,
    SuddenDeath,
    Paused,
    RoundOver,
    ResultsExported,
}

/// Ceremony state nested under tournament, duel, wedding, and banquet flows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CeremonyState {
    Dormant,
    Queued(EventKind),
    Prompt(EventKind),
    Tournament(TournamentState),
    Duel(DuelState),
    Wedding(WeddingState),
    Banquet(BanquetState),
    ConsequenceResolution,
    Cooldown,
}

/// Ceremony categories that can be queued or prompted before activation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    Tournament,
    Duel,
    WeddingAlliance,
    Banquet,
}

/// Tournament sub-state used while temporary tournament rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TournamentState {
    ArenaBuild,
    Registration,
    RoundActive,
    ChampionDeclared,
}

/// Duel sub-state used while duel lock and joust-only rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DuelState {
    ChallengeIssued,
    ArenaLock,
    DuelActive,
    ResolveDuel,
}

/// Wedding alliance sub-state used for truce and joint-objective modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeddingState {
    AllianceProposed,
    TruceActive,
    JointObjective,
    Broken,
    Expired,
}

/// Banquet negotiation sub-state used for Warfront treaty modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BanquetState {
    Seating,
    TermsOpen,
    TreatySigned,
    Collapsed,
}

/// Current temporary and baseline rules that gate legal match actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rules {
    pub ordnance: OrdnancePolicy,
    pub friendly_fire: bool,
    pub duel_lock: bool,
    pub scoring_frozen: bool,
    pub joust_only: bool,
}

impl Rules {
    pub const fn baseline() -> Self {
        Self {
            ordnance: OrdnancePolicy::Full,
            friendly_fire: true,
            duel_lock: false,
            scoring_frozen: false,
            joust_only: false,
        }
    }
}

/// Ordnance availability policy for match and ceremony rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrdnancePolicy {
    Full,
    Limited,
    Disabled,
}

/// Player ordnance lifecycle state used by legal ordnance action checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerOrdnance {
    Ready,
    Cooldown,
    ResupplyNeeded,
    Disabled,
}

/// Lance lifecycle state used to gate brace windows and joust contacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanceState {
    Idle,
    Bracing,
    Recovery,
    Broken,
}

/// Rider recovery state after collisions, unhorsing, and respawn windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryState {
    Alive,
    Stunned,
    Unhorsed,
    Dead,
    Respawning,
}

/// Objective flags that can emit score atoms during a match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub keep_breached: bool,
    pub outpost_controlled: bool,
    pub shrine_claimed: bool,
    pub supply_route_blocked: bool,
    pub hostage_delivered: bool,
}

/// Winner classification exported with the final score snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Winner {
    None,
    Red,
    Blue,
    TieBreak,
}

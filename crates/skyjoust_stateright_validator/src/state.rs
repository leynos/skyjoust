//! State types and guard helpers for the Skyjoust validator model.

use serde::{Deserialize, Serialize};

pub use crate::ledgers::{RewardLedger, RewardPhase, ScoreLedger};

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
            treaty_signed: false,
            infamy: 0,
            post_final_score_write: false,
            warfront_mutated_during_match: false,
        }
    }
}

impl SkyState {
    pub fn is_match_active(&self) -> bool {
        matches!(
            self.match_phase,
            MatchPhase::NormalPlay | MatchPhase::EventOverride | MatchPhase::SuddenDeath
        )
    }

    pub fn is_in_match_or_building_match(&self) -> bool {
        matches!(
            self.match_phase,
            MatchPhase::Constructing
                | MatchPhase::SpawnWarmup
                | MatchPhase::Countdown
                | MatchPhase::NormalPlay
                | MatchPhase::EventOverride
                | MatchPhase::SuddenDeath
                | MatchPhase::RoundOver
                | MatchPhase::ResultsExported
        )
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchPhase {
    Inactive,
    Constructing,
    SpawnWarmup,
    Countdown,
    NormalPlay,
    EventOverride,
    SuddenDeath,
    RoundOver,
    ResultsExported,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    Tournament,
    Duel,
    WeddingAlliance,
    Banquet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TournamentState {
    ArenaBuild,
    Registration,
    RoundActive,
    ChampionDeclared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DuelState {
    ChallengeIssued,
    ArenaLock,
    DuelActive,
    ResolveDuel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeddingState {
    AllianceProposed,
    TruceActive,
    JointObjective,
    Broken,
    Expired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BanquetState {
    Seating,
    TermsOpen,
    TreatySigned,
    Collapsed,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrdnancePolicy {
    Full,
    Limited,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerOrdnance {
    Ready,
    Cooldown,
    ResupplyNeeded,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanceState {
    Idle,
    Bracing,
    Recovery,
    Broken,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryState {
    Alive,
    Stunned,
    Unhorsed,
    Dead,
    Respawning,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub keep_breached: bool,
    pub outpost_controlled: bool,
    pub shrine_claimed: bool,
    pub supply_route_blocked: bool,
    pub hostage_delivered: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Winner {
    None,
    Red,
    Blue,
    TieBreak,
}

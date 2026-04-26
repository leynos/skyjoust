//! State types and guard helpers for the Skyjoust validator model.

pub use crate::ledgers::{RewardLedger, RewardPhase, ScoreLedger};

/// Complete validator snapshot for app, match, ceremony, scoring, and rewards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

/// Top-level application mode explored by the validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    Tournament,
    Duel,
    WeddingAlliance,
    Banquet,
}

/// Tournament sub-state used while temporary tournament rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TournamentState {
    ArenaBuild,
    Registration,
    RoundActive,
    RoundComplete,
    ChampionDeclared,
}

/// Duel sub-state used while duel lock and joust-only rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DuelState {
    ChallengeIssued,
    ArenaLock,
    DuelActive,
    ResolveDuel,
}

/// Wedding alliance sub-state used for truce and joint-objective modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeddingState {
    AllianceProposed,
    TruceActive,
    JointObjective,
    Broken,
    Expired,
}

/// Banquet negotiation sub-state used for Warfront treaty modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BanquetState {
    Seating,
    TermsOpen,
    CounterOffer,
    TreatySigned,
    Collapsed,
}

/// Current temporary and baseline rules that gate legal match actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rules {
    pub ordnance: OrdnancePolicy,
    pub friendly_fire: bool,
    pub duel_lock: bool,
    pub scoring_frozen: bool,
    pub joust_only: bool,
    pub allow_sudden_death: bool,
}

impl Rules {
    pub const fn baseline() -> Self {
        Self {
            ordnance: OrdnancePolicy::Full,
            friendly_fire: true,
            duel_lock: false,
            scoring_frozen: false,
            joust_only: false,
            allow_sudden_death: true,
        }
    }
}

/// Ordnance availability policy for match and ceremony rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrdnancePolicy {
    Full,
    Limited,
    Disabled,
}

/// Player ordnance lifecycle state used by legal ordnance action checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerOrdnance {
    Ready,
    Cooldown,
    ResupplyNeeded,
    Disabled,
}

/// Lance lifecycle state used to gate brace windows and joust contacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LanceState {
    Idle,
    Bracing,
    Recovery,
    Broken,
}

/// Rider recovery state after collisions, unhorsing, and respawn windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecoveryState {
    Alive,
    Stunned,
    Unhorsed,
    Dead,
    Respawning,
}

/// Objective flags that can emit score atoms during a match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ObjectiveSnapshot {
    pub keep_breached: bool,
    pub outpost_controlled: bool,
    pub shrine_claimed: bool,
    pub supply_route_blocked: bool,
    pub hostage_delivered: bool,
}

/// Winner classification exported with the final score snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Winner {
    None,
    Red,
    Blue,
    TieBreak,
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

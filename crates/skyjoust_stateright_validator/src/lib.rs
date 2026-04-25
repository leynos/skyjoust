//! Stateright validator for Project Skyjoust's high-level engine interactions.
//!
//! The model is intentionally smaller than the runtime engine.  It preserves the
//! interactions we care about proving at the design level:
//!
//! * match lifecycle gates scoring,
//! * ceremony events override and then restore rules,
//! * player actions can only emit legal scoring atoms,
//! * objectives can close the match,
//! * rewards can only commit from a finalized score snapshot,
//! * warfront state only mutates through result and reward handoff.
//!
//! Use [`SkyjoustInteractionModel`] with Stateright for exhaustive exploration, or
//! use [`validate_trace`] to replay an engine event log against the same contract.

use serde::{Deserialize, Serialize};
use stateright::{Model, Property};

pub type AlwaysProperty = fn(&SkyjoustInteractionModel, &SkyState) -> bool;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkyjoustInteractionModel {
    /// Exploration boundary.  Increase for deeper interaction sequences, lower it
    /// for fast CI smoke checks.
    pub max_depth: u8,
}

impl Default for SkyjoustInteractionModel {
    fn default() -> Self {
        Self { max_depth: 24 }
    }
}

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

    fn clear_temporary_rules_if_safe(&mut self) {
        self.rules = if self.truce_active {
            Rules {
                friendly_fire: false,
                ..Rules::baseline()
            }
        } else {
            Rules::baseline()
        };
        self.player_ordnance = if self.rules.joust_only || self.rules.ordnance == OrdnancePolicy::Disabled {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub keep_breached: bool,
    pub outpost_controlled: bool,
    pub shrine_claimed: bool,
    pub supply_route_blocked: bool,
    pub hostage_delivered: bool,
}

impl Default for ObjectiveSnapshot {
    fn default() -> Self {
        Self {
            keep_breached: false,
            outpost_controlled: false,
            shrine_claimed: false,
            supply_route_blocked: false,
            hostage_delivered: false,
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Winner {
    None,
    Red,
    Blue,
    TieBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JoustOutcome {
    Knockback,
    Unhorse,
    Shatter,
    CleanKill,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkyAction {
    AssetsLoaded,
    StartSkirmish,
    StartWarfront,
    MapReady,
    SelectRegion,
    StartBattle,
    FinishConstructing,
    SpawnReady,
    CountdownDone,

    TriggerTournament,
    ArenaReady,
    TournamentRegistered,
    TournamentRoundWon,
    TournamentChampionDeclared,

    IssueDuel,
    AcceptDuel,
    DuelReady,
    DuelDecisiveJoust { winner: Team, outcome: JoustOutcome },
    DuelInterference,

    StartWeddingTruce,
    CompleteJointObjective,
    BreakTruce,
    ExpireTruce,

    OpenBanquet,
    BanquetReady,
    ProposeTreaty,
    AcceptTreaty,
    RejectTreaty,

    EventConsequencesRecorded,
    EventCooldownDone,

    BracePressed,
    BraceWindowExpired,
    Joust { winner: Team, outcome: JoustOutcome },
    CaptureOutpost,
    ClaimShrine,
    BlockSupplyRoute,
    DeliverHostage,
    BombKeepBreach,
    TimerExpired,
    VictoryCheck,

    ExportFinalScore,
    TallyRewards,
    CommitRewards,
    NextWarfrontTurn,
    ReturnToTitle,
}

impl Model for SkyjoustInteractionModel {
    type State = SkyState;
    type Action = SkyAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![SkyState::default()]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        if state.depth >= self.max_depth {
            return;
        }

        match state.app {
            AppState::Boot => {
                actions.push(SkyAction::AssetsLoaded);
            }
            AppState::Title => {
                actions.push(SkyAction::StartSkirmish);
                actions.push(SkyAction::StartWarfront);
            }
            AppState::SkirmishSetup => {
                actions.push(SkyAction::StartBattle);
            }
            AppState::WarfrontSetup => {
                actions.push(SkyAction::MapReady);
            }
            AppState::WarfrontRunning => match state.warfront {
                WarfrontState::StrategicChoice => {
                    actions.push(SkyAction::SelectRegion);
                    actions.push(SkyAction::OpenBanquet);
                }
                WarfrontState::BattlePreview => {
                    actions.push(SkyAction::StartBattle);
                }
                _ => {}
            },
            AppState::MatchRunning | AppState::Results => {}
        }

        match state.match_phase {
            MatchPhase::Constructing => actions.push(SkyAction::FinishConstructing),
            MatchPhase::SpawnWarmup => actions.push(SkyAction::SpawnReady),
            MatchPhase::Countdown => actions.push(SkyAction::CountdownDone),
            MatchPhase::NormalPlay | MatchPhase::EventOverride | MatchPhase::SuddenDeath => {
                if !state.score.victory_pending {
                    push_active_match_actions(state, actions);
                }
                if state.score.victory_pending {
                    actions.push(SkyAction::VictoryCheck);
                }
                if state.match_phase == MatchPhase::NormalPlay || state.match_phase == MatchPhase::SuddenDeath {
                    push_ceremony_start_actions(state, actions);
                }
            }
            MatchPhase::RoundOver => actions.push(SkyAction::ExportFinalScore),
            MatchPhase::ResultsExported | MatchPhase::Inactive => {}
        }

        push_ceremony_continuation_actions(state, actions);

        match state.rewards.phase {
            RewardPhase::LedgerOpen => actions.push(SkyAction::TallyRewards),
            RewardPhase::Tallied => actions.push(SkyAction::CommitRewards),
            RewardPhase::Committed => {
                if state.warfront != WarfrontState::Inactive {
                    actions.push(SkyAction::NextWarfrontTurn);
                } else {
                    actions.push(SkyAction::ReturnToTitle);
                }
            }
            RewardPhase::ReadyToSpend | RewardPhase::Dormant => {}
        }
    }

    fn next_state(&self, last: &Self::State, action: Self::Action) -> Option<Self::State> {
        if last.depth >= self.max_depth {
            return None;
        }

        let mut s = last.clone();
        s.depth += 1;

        let legal = match action {
            SkyAction::AssetsLoaded => {
                guard(last.app == AppState::Boot)?;
                s.app = AppState::Title;
                true
            }
            SkyAction::StartSkirmish => {
                guard(last.app == AppState::Title)?;
                s.app = AppState::SkirmishSetup;
                true
            }
            SkyAction::StartWarfront => {
                guard(last.app == AppState::Title)?;
                s.app = AppState::WarfrontSetup;
                s.warfront = WarfrontState::GenerateOrLoad;
                true
            }
            SkyAction::MapReady => {
                guard(last.app == AppState::WarfrontSetup && last.warfront == WarfrontState::GenerateOrLoad)?;
                s.app = AppState::WarfrontRunning;
                s.warfront = WarfrontState::StrategicChoice;
                true
            }
            SkyAction::SelectRegion => {
                guard(last.app == AppState::WarfrontRunning && last.warfront == WarfrontState::StrategicChoice)?;
                s.warfront = WarfrontState::BattlePreview;
                true
            }
            SkyAction::StartBattle => {
                guard(matches!(last.app, AppState::SkirmishSetup | AppState::WarfrontRunning))?;
                if last.app == AppState::WarfrontRunning {
                    guard(last.warfront == WarfrontState::BattlePreview)?;
                    s.warfront = WarfrontState::BattleLocked;
                }
                s.app = AppState::MatchRunning;
                s.match_phase = MatchPhase::Constructing;
                s.score = ScoreLedger { open: true, ..ScoreLedger::default() };
                s.rewards = RewardLedger::default();
                true
            }
            SkyAction::FinishConstructing => {
                guard(last.match_phase == MatchPhase::Constructing)?;
                s.match_phase = MatchPhase::SpawnWarmup;
                if s.warfront == WarfrontState::BattleLocked {
                    s.warfront = WarfrontState::AwaitingBattleResult;
                }
                true
            }
            SkyAction::SpawnReady => {
                guard(last.match_phase == MatchPhase::SpawnWarmup)?;
                s.match_phase = MatchPhase::Countdown;
                true
            }
            SkyAction::CountdownDone => {
                guard(last.match_phase == MatchPhase::Countdown)?;
                s.match_phase = MatchPhase::NormalPlay;
                s.score.open = true;
                true
            }

            SkyAction::TriggerTournament => {
                guard(last.is_match_active() && last.ceremony == CeremonyState::Dormant)?;
                s.ceremony = CeremonyState::Tournament(TournamentState::ArenaBuild);
                s.match_phase = MatchPhase::EventOverride;
                s.rules.ordnance = OrdnancePolicy::Disabled;
                s.rules.joust_only = true;
                s.player_ordnance = PlayerOrdnance::Disabled;
                true
            }
            SkyAction::ArenaReady => {
                guard(last.ceremony == CeremonyState::Tournament(TournamentState::ArenaBuild))?;
                s.ceremony = CeremonyState::Tournament(TournamentState::Registration);
                true
            }
            SkyAction::TournamentRegistered => {
                guard(last.ceremony == CeremonyState::Tournament(TournamentState::Registration))?;
                s.ceremony = CeremonyState::Tournament(TournamentState::RoundActive);
                true
            }
            SkyAction::TournamentRoundWon => {
                guard(last.ceremony == CeremonyState::Tournament(TournamentState::RoundActive))?;
                apply_joust_score(&mut s, Team::Red, JoustOutcome::Unhorse);
                s.tournament_rounds_won = s.tournament_rounds_won.saturating_add(1);
                true
            }
            SkyAction::TournamentChampionDeclared => {
                guard(last.ceremony == CeremonyState::Tournament(TournamentState::RoundActive))?;
                guard(last.tournament_rounds_won > 0)?;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.tournament_completed = true;
                s.match_phase = MatchPhase::NormalPlay;
                true
            }

            SkyAction::IssueDuel => {
                guard(last.is_match_active() && last.ceremony == CeremonyState::Dormant)?;
                s.ceremony = CeremonyState::Duel(DuelState::ChallengeIssued);
                true
            }
            SkyAction::AcceptDuel => {
                guard(last.ceremony == CeremonyState::Duel(DuelState::ChallengeIssued))?;
                s.ceremony = CeremonyState::Duel(DuelState::ArenaLock);
                s.match_phase = MatchPhase::EventOverride;
                s.rules.duel_lock = true;
                s.rules.ordnance = OrdnancePolicy::Disabled;
                s.rules.joust_only = true;
                s.player_ordnance = PlayerOrdnance::Disabled;
                true
            }
            SkyAction::DuelReady => {
                guard(last.ceremony == CeremonyState::Duel(DuelState::ArenaLock))?;
                s.ceremony = CeremonyState::Duel(DuelState::DuelActive);
                true
            }
            SkyAction::DuelDecisiveJoust { winner, outcome } => {
                guard(last.ceremony == CeremonyState::Duel(DuelState::DuelActive))?;
                guard(matches!(outcome, JoustOutcome::Unhorse | JoustOutcome::Shatter | JoustOutcome::CleanKill))?;
                apply_joust_score(&mut s, winner, outcome);
                s.duel_resolved = true;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.match_phase = MatchPhase::NormalPlay;
                true
            }
            SkyAction::DuelInterference => {
                guard(last.ceremony == CeremonyState::Duel(DuelState::DuelActive))?;
                apply_dishonour_penalty(&mut s);
                true
            }

            SkyAction::StartWeddingTruce => {
                guard(last.is_match_active() && last.ceremony == CeremonyState::Dormant)?;
                s.ceremony = CeremonyState::Wedding(WeddingState::TruceActive);
                s.match_phase = MatchPhase::EventOverride;
                s.truce_active = true;
                s.rules.friendly_fire = false;
                true
            }
            SkyAction::CompleteJointObjective => {
                guard(last.ceremony == CeremonyState::Wedding(WeddingState::TruceActive))?;
                s.ceremony = CeremonyState::Wedding(WeddingState::JointObjective);
                apply_objective_score(&mut s, Team::Red, ScoreAtom::HostageDeliver);
                true
            }
            SkyAction::BreakTruce => {
                guard(last.truce_active && matches!(last.ceremony, CeremonyState::Wedding(_)))?;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.truce_active = false;
                s.truce_broken = true;
                s.rules.friendly_fire = true;
                s.infamy += 50;
                s.rewards.penalties = s.rewards.penalties.saturating_add(1);
                s.match_phase = MatchPhase::NormalPlay;
                true
            }
            SkyAction::ExpireTruce => {
                guard(last.truce_active && matches!(last.ceremony, CeremonyState::Wedding(_)))?;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.truce_active = false;
                s.match_phase = MatchPhase::NormalPlay;
                true
            }

            SkyAction::OpenBanquet => {
                guard(last.app == AppState::WarfrontRunning && last.warfront == WarfrontState::StrategicChoice)?;
                s.warfront = WarfrontState::BanquetNegotiation;
                s.ceremony = CeremonyState::Banquet(BanquetState::Seating);
                true
            }
            SkyAction::BanquetReady => {
                guard(last.ceremony == CeremonyState::Banquet(BanquetState::Seating))?;
                s.ceremony = CeremonyState::Banquet(BanquetState::TermsOpen);
                true
            }
            SkyAction::ProposeTreaty => {
                guard(last.ceremony == CeremonyState::Banquet(BanquetState::TermsOpen))?;
                true
            }
            SkyAction::AcceptTreaty => {
                guard(last.ceremony == CeremonyState::Banquet(BanquetState::TermsOpen))?;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.treaty_signed = true;
                s.warfront = WarfrontState::StrategicChoice;
                true
            }
            SkyAction::RejectTreaty => {
                guard(last.ceremony == CeremonyState::Banquet(BanquetState::TermsOpen))?;
                s.ceremony = CeremonyState::ConsequenceResolution;
                s.warfront = WarfrontState::StrategicChoice;
                s.infamy += 5;
                true
            }

            SkyAction::EventConsequencesRecorded => {
                guard(last.ceremony == CeremonyState::ConsequenceResolution)?;
                s.clear_temporary_rules_if_safe();
                s.ceremony = CeremonyState::Cooldown;
                true
            }
            SkyAction::EventCooldownDone => {
                guard(last.ceremony == CeremonyState::Cooldown)?;
                guard(!last.rules.duel_lock && !last.rules.joust_only)?;
                s.ceremony = CeremonyState::Dormant;
                true
            }

            SkyAction::BracePressed => {
                guard(last.is_match_active() && last.lance == LanceState::Idle)?;
                s.lance = LanceState::Bracing;
                true
            }
            SkyAction::BraceWindowExpired => {
                guard(last.lance == LanceState::Bracing)?;
                s.lance = LanceState::Recovery;
                true
            }
            SkyAction::Joust { winner, outcome } => {
                guard(last.is_match_active() && last.score.open && !last.rules.scoring_frozen)?;
                guard(!last.rules.duel_lock)?;
                apply_joust_score(&mut s, winner, outcome);
                s.lance = LanceState::Recovery;
                update_recovery_from_outcome(&mut s, winner, outcome);
                true
            }
            SkyAction::CaptureOutpost => {
                guard(last.is_match_active() && !last.objectives.outpost_controlled && !last.rules.duel_lock)?;
                s.objectives.outpost_controlled = true;
                apply_objective_score(&mut s, Team::Red, ScoreAtom::OutpostCapture);
                true
            }
            SkyAction::ClaimShrine => {
                guard(last.is_match_active() && !last.objectives.shrine_claimed && !last.rules.duel_lock)?;
                s.objectives.shrine_claimed = true;
                apply_objective_score(&mut s, Team::Red, ScoreAtom::ShrineClaim);
                true
            }
            SkyAction::BlockSupplyRoute => {
                guard(last.is_match_active() && !last.objectives.supply_route_blocked && !last.rules.duel_lock)?;
                s.objectives.supply_route_blocked = true;
                apply_objective_score(&mut s, Team::Red, ScoreAtom::SupplyRouteBlock);
                true
            }
            SkyAction::DeliverHostage => {
                guard(last.is_match_active() && !last.objectives.hostage_delivered && !last.rules.duel_lock)?;
                s.objectives.hostage_delivered = true;
                apply_objective_score(&mut s, Team::Red, ScoreAtom::HostageDeliver);
                true
            }
            SkyAction::BombKeepBreach => {
                guard(last.is_match_active())?;
                guard(matches!(last.rules.ordnance, OrdnancePolicy::Full | OrdnancePolicy::Limited))?;
                guard(last.player_ordnance == PlayerOrdnance::Ready && !last.rules.joust_only)?;
                s.player_ordnance = PlayerOrdnance::Cooldown;
                s.objectives.keep_breached = true;
                apply_objective_score(&mut s, Team::Red, ScoreAtom::KeepBreach);
                s.score.victory_pending = true;
                s.winner = Winner::Red;
                true
            }
            SkyAction::TimerExpired => {
                guard(last.is_match_active())?;
                s.match_phase = MatchPhase::SuddenDeath;
                true
            }
            SkyAction::VictoryCheck => {
                guard(last.is_match_active() && last.score.victory_pending)?;
                s.match_phase = MatchPhase::RoundOver;
                if s.winner == Winner::None {
                    s.winner = decide_winner(&s.score);
                }
                true
            }

            SkyAction::ExportFinalScore => {
                guard(last.match_phase == MatchPhase::RoundOver)?;
                s.match_phase = MatchPhase::ResultsExported;
                s.app = AppState::Results;
                s.score.open = false;
                s.score.finalized = true;
                s.rewards.phase = RewardPhase::LedgerOpen;
                s.rewards.pending_delta = true;
                if s.warfront == WarfrontState::AwaitingBattleResult {
                    s.warfront = WarfrontState::RewardCommit;
                }
                true
            }
            SkyAction::TallyRewards => {
                guard(last.rewards.phase == RewardPhase::LedgerOpen && last.score.finalized)?;
                tally_rewards(&mut s);
                true
            }
            SkyAction::CommitRewards => {
                guard(last.rewards.phase == RewardPhase::Tallied && last.score.finalized)?;
                s.rewards.phase = RewardPhase::Committed;
                s.rewards.committed = true;
                s.rewards.pending_delta = false;
                if s.is_in_match_or_building_match() {
                    // The runtime should only mutate the warfront after this point.
                    s.warfront_mutated_during_match = false;
                }
                true
            }
            SkyAction::NextWarfrontTurn => {
                guard(last.rewards.phase == RewardPhase::Committed && last.warfront != WarfrontState::Inactive)?;
                s.app = AppState::WarfrontRunning;
                s.match_phase = MatchPhase::Inactive;
                s.warfront = WarfrontState::StrategicChoice;
                s.rewards.phase = RewardPhase::ReadyToSpend;
                true
            }
            SkyAction::ReturnToTitle => {
                guard(last.rewards.phase == RewardPhase::Committed || last.app == AppState::Results)?;
                s = SkyState::default();
                s.depth = last.depth.saturating_add(1);
                s.app = AppState::Title;
                true
            }
        };

        if legal {
            Some(s)
        } else {
            None
        }
    }

    fn properties(&self) -> Vec<Property<Self>> {
        let mut properties = Vec::new();
        for (name, check) in ALWAYS_PROPERTIES {
            properties.push(Property::<Self>::always(*name, *check));
        }
        for (name, check) in SOMETIMES_PROPERTIES {
            properties.push(Property::<Self>::sometimes(*name, *check));
        }
        properties
    }

    fn within_boundary(&self, state: &Self::State) -> bool {
        state.depth <= self.max_depth
    }
}

fn push_active_match_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.score.events_accepted < 4 && !state.rules.duel_lock {
        actions.push(SkyAction::BracePressed);
        actions.push(SkyAction::Joust { winner: Team::Red, outcome: JoustOutcome::Knockback });
        actions.push(SkyAction::Joust { winner: Team::Red, outcome: JoustOutcome::Unhorse });
        actions.push(SkyAction::Joust { winner: Team::Red, outcome: JoustOutcome::Shatter });
        actions.push(SkyAction::CaptureOutpost);
        actions.push(SkyAction::ClaimShrine);
        actions.push(SkyAction::BlockSupplyRoute);
        actions.push(SkyAction::DeliverHostage);
        if matches!(state.rules.ordnance, OrdnancePolicy::Full | OrdnancePolicy::Limited)
            && state.player_ordnance == PlayerOrdnance::Ready
            && !state.rules.joust_only
        {
            actions.push(SkyAction::BombKeepBreach);
        }
    }
    actions.push(SkyAction::TimerExpired);
}

fn push_ceremony_start_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    if state.ceremony == CeremonyState::Dormant && !state.score.victory_pending {
        actions.push(SkyAction::TriggerTournament);
        actions.push(SkyAction::IssueDuel);
        actions.push(SkyAction::StartWeddingTruce);
    }
}

fn push_ceremony_continuation_actions(state: &SkyState, actions: &mut Vec<SkyAction>) {
    match state.ceremony {
        CeremonyState::Tournament(TournamentState::ArenaBuild) => actions.push(SkyAction::ArenaReady),
        CeremonyState::Tournament(TournamentState::Registration) => actions.push(SkyAction::TournamentRegistered),
        CeremonyState::Tournament(TournamentState::RoundActive) => {
            if state.tournament_rounds_won < 2 {
                actions.push(SkyAction::TournamentRoundWon);
            }
            if state.tournament_rounds_won > 0 {
                actions.push(SkyAction::TournamentChampionDeclared);
            }
        }
        CeremonyState::Duel(DuelState::ChallengeIssued) => actions.push(SkyAction::AcceptDuel),
        CeremonyState::Duel(DuelState::ArenaLock) => actions.push(SkyAction::DuelReady),
        CeremonyState::Duel(DuelState::DuelActive) => {
            actions.push(SkyAction::DuelDecisiveJoust { winner: Team::Red, outcome: JoustOutcome::CleanKill });
            actions.push(SkyAction::DuelInterference);
        }
        CeremonyState::Wedding(WeddingState::TruceActive) => {
            actions.push(SkyAction::CompleteJointObjective);
            actions.push(SkyAction::BreakTruce);
            actions.push(SkyAction::ExpireTruce);
        }
        CeremonyState::Wedding(WeddingState::JointObjective) => {
            actions.push(SkyAction::BreakTruce);
            actions.push(SkyAction::ExpireTruce);
        }
        CeremonyState::Banquet(BanquetState::Seating) => actions.push(SkyAction::BanquetReady),
        CeremonyState::Banquet(BanquetState::TermsOpen) => {
            actions.push(SkyAction::ProposeTreaty);
            actions.push(SkyAction::AcceptTreaty);
            actions.push(SkyAction::RejectTreaty);
        }
        CeremonyState::ConsequenceResolution => actions.push(SkyAction::EventConsequencesRecorded),
        CeremonyState::Cooldown => actions.push(SkyAction::EventCooldownDone),
        CeremonyState::Dormant
        | CeremonyState::Queued(_)
        | CeremonyState::Prompt(_)
        | CeremonyState::Tournament(TournamentState::ChampionDeclared)
        | CeremonyState::Duel(DuelState::ResolveDuel)
        | CeremonyState::Wedding(WeddingState::AllianceProposed)
        | CeremonyState::Wedding(WeddingState::Broken)
        | CeremonyState::Wedding(WeddingState::Expired)
        | CeremonyState::Banquet(BanquetState::TreatySigned)
        | CeremonyState::Banquet(BanquetState::Collapsed) => {}
    }
}

fn guard(condition: bool) -> Option<()> {
    if condition { Some(()) } else { None }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ScoreAtom {
    OutpostCapture,
    ShrineClaim,
    SupplyRouteBlock,
    HostageDeliver,
    KeepBreach,
}

fn apply_joust_score(s: &mut SkyState, winner: Team, outcome: JoustOutcome) {
    if s.score.finalized {
        s.post_final_score_write = true;
        return;
    }
    let (score, glory, morale_delta) = match outcome {
        JoustOutcome::Knockback => (80, 10, 1),
        JoustOutcome::Unhorse => (150, 20, 3),
        JoustOutcome::Shatter => (220, 30, 4),
        JoustOutcome::CleanKill => (350, 45, 7),
    };
    apply_score_delta(s, winner, score, glory, morale_delta);
}

fn apply_objective_score(s: &mut SkyState, winner: Team, atom: ScoreAtom) {
    if s.score.finalized {
        s.post_final_score_write = true;
        return;
    }
    let (score, glory, morale_delta) = match atom {
        ScoreAtom::OutpostCapture => (200, 0, -2),
        ScoreAtom::ShrineClaim => (120, 0, -1),
        ScoreAtom::SupplyRouteBlock => (160, 0, 2),
        ScoreAtom::HostageDeliver => (250, 0, -3),
        ScoreAtom::KeepBreach => (1000, 100, 999),
    };
    apply_score_delta(s, winner, score, glory, morale_delta);
}

fn apply_score_delta(s: &mut SkyState, winner: Team, score: i16, glory: i16, morale_delta_target: i16) {
    s.score.open = true;
    s.score.pending_delta = true;
    s.score.events_accepted = s.score.events_accepted.saturating_add(1);
    match winner {
        Team::Red => {
            s.score.red_score += score;
            s.score.red_glory += glory;
            s.score.blue_morale -= morale_delta_target;
            if s.score.blue_morale <= 0 {
                s.score.victory_pending = true;
                s.winner = Winner::Red;
            }
        }
        Team::Blue => {
            s.score.blue_score += score;
            s.score.blue_glory += glory;
            s.score.red_morale -= morale_delta_target;
            if s.score.red_morale <= 0 {
                s.score.victory_pending = true;
                s.winner = Winner::Blue;
            }
        }
    }
}

fn update_recovery_from_outcome(s: &mut SkyState, winner: Team, outcome: JoustOutcome) {
    let local_lost = winner == Team::Blue;
    s.recovery = match (local_lost, outcome) {
        (true, JoustOutcome::Knockback) => RecoveryState::Stunned,
        (true, JoustOutcome::Unhorse) => RecoveryState::Unhorsed,
        (true, JoustOutcome::Shatter | JoustOutcome::CleanKill) => RecoveryState::Dead,
        (false, _) => RecoveryState::Alive,
    };
}

fn apply_dishonour_penalty(s: &mut SkyState) {
    s.infamy += 10;
    s.rewards.penalties = s.rewards.penalties.saturating_add(1);
    s.score.red_score -= 500;
    s.score.pending_delta = true;
}

fn decide_winner(score: &ScoreLedger) -> Winner {
    if score.red_score > score.blue_score {
        Winner::Red
    } else if score.blue_score > score.red_score {
        Winner::Blue
    } else {
        Winner::TieBreak
    }
}

fn tally_rewards(s: &mut SkyState) {
    s.rewards.phase = RewardPhase::Tallied;
    s.rewards.pending_delta = true;

    // Base participation + result reward.
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

pub const ALWAYS_PROPERTIES: &[(&str, AlwaysProperty)] = &[
    ("rewards_commit_requires_final_score", prop_rewards_commit_requires_final_score),
    ("rewards_open_requires_final_score", prop_rewards_open_requires_final_score),
    ("score_closed_after_final_snapshot", prop_score_closed_after_final_snapshot),
    ("no_score_write_after_final_snapshot", prop_no_score_write_after_final_snapshot),
    ("committed_rewards_leave_active_match", prop_committed_rewards_leave_active_match),
    ("duel_lock_only_during_duel", prop_duel_lock_only_during_duel),
    ("joust_only_disables_ordnance", prop_joust_only_disables_ordnance),
    ("truce_disables_friendly_fire", prop_truce_disables_friendly_fire),
    ("truce_break_is_penalized", prop_truce_break_is_penalized),
    ("laurels_only_after_tournament_completion", prop_laurels_only_after_tournament_completion),
    ("duel_reward_only_after_resolved_duel", prop_duel_reward_only_after_resolved_duel),
    ("temporary_rules_cleared_after_cooldown", prop_temporary_rules_cleared_after_cooldown),
    ("round_over_has_winner", prop_round_over_has_winner),
    ("warfront_not_mutated_during_match", prop_warfront_not_mutated_during_match),
];

pub const SOMETIMES_PROPERTIES: &[(&str, AlwaysProperty)] = &[
    ("can_breach_keep_and_commit_rewards", sometimes_breach_keep_and_commit_rewards),
    ("can_complete_tournament_and_get_laurels", sometimes_complete_tournament_and_get_laurels),
    ("can_resolve_duel_and_get_duel_rewards", sometimes_resolve_duel_and_get_rewards),
    ("can_break_truce_and_receive_infamy", sometimes_break_truce_and_receive_infamy),
    ("can_score_nonlethal_objective", sometimes_score_nonlethal_objective),
];

fn prop_rewards_commit_requires_final_score(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.committed || s.score.finalized
}

fn prop_rewards_open_requires_final_score(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !matches!(s.rewards.phase, RewardPhase::LedgerOpen | RewardPhase::Tallied | RewardPhase::Committed | RewardPhase::ReadyToSpend)
        || s.score.finalized
}

fn prop_score_closed_after_final_snapshot(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.score.finalized || !s.score.open
}

fn prop_no_score_write_after_final_snapshot(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.post_final_score_write
}

fn prop_committed_rewards_leave_active_match(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.committed || !s.is_match_active()
}

fn prop_duel_lock_only_during_duel(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rules.duel_lock
        || matches!(
            s.ceremony,
            CeremonyState::Duel(DuelState::ArenaLock)
                | CeremonyState::Duel(DuelState::DuelActive)
                | CeremonyState::Duel(DuelState::ResolveDuel)
                | CeremonyState::ConsequenceResolution
        )
}

fn prop_joust_only_disables_ordnance(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rules.joust_only
        || (s.rules.ordnance == OrdnancePolicy::Disabled && s.player_ordnance == PlayerOrdnance::Disabled)
}

fn prop_truce_disables_friendly_fire(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.truce_active || !s.rules.friendly_fire
}

fn prop_truce_break_is_penalized(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.truce_broken || (s.infamy > 0 && s.rewards.penalties > 0)
}

fn prop_laurels_only_after_tournament_completion(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    (s.rewards.laurels == 0 && !s.rewards.tournament_bonus_granted) || s.tournament_completed
}

fn prop_duel_reward_only_after_resolved_duel(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.rewards.duel_bonus_granted || s.duel_resolved
}

fn prop_temporary_rules_cleared_after_cooldown(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    if matches!(s.ceremony, CeremonyState::Dormant | CeremonyState::Cooldown) && !s.truce_active {
        s.rules == Rules::baseline()
    } else {
        true
    }
}

fn prop_round_over_has_winner(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !matches!(s.match_phase, MatchPhase::RoundOver | MatchPhase::ResultsExported) || s.winner != Winner::None
}

fn prop_warfront_not_mutated_during_match(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    !s.warfront_mutated_during_match
}

fn sometimes_breach_keep_and_commit_rewards(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.objectives.keep_breached && s.rewards.committed && s.score.finalized
}

fn sometimes_complete_tournament_and_get_laurels(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.tournament_completed && s.rewards.committed && s.rewards.laurels >= 3
}

fn sometimes_resolve_duel_and_get_rewards(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.duel_resolved && s.rewards.committed && s.rewards.influence >= 25
}

fn sometimes_break_truce_and_receive_infamy(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.truce_broken && s.rewards.committed && s.infamy > 0 && s.rewards.penalties > 0
}

fn sometimes_score_nonlethal_objective(_: &SkyjoustInteractionModel, s: &SkyState) -> bool {
    s.objectives.outpost_controlled && s.score.red_score >= 200 && !s.objectives.keep_breached
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceValidation {
    pub ok: bool,
    pub final_state: SkyState,
    pub failure: Option<TraceFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceFailure {
    pub step_index: usize,
    pub action: SkyAction,
    pub reason: String,
}

/// Replay a concrete engine event log against the same guards and invariants as
/// the Stateright model.  This is useful for validating recorded gameplay traces
/// produced by the Bevy/Pixels runtime.
pub fn validate_trace<I>(model: &SkyjoustInteractionModel, trace: I) -> TraceValidation
where
    I: IntoIterator<Item = SkyAction>,
{
    let mut state = SkyState::default();

    for (step_index, action) in trace.into_iter().enumerate() {
        let next = match model.next_state(&state, action.clone()) {
            Some(next) => next,
            None => {
                return TraceValidation {
                    ok: false,
                    final_state: state,
                    failure: Some(TraceFailure {
                        step_index,
                        action,
                        reason: "action was not legal from the current state".to_string(),
                    }),
                };
            }
        };

        if !model.within_boundary(&next) {
            return TraceValidation {
                ok: false,
                final_state: next,
                failure: Some(TraceFailure {
                    step_index,
                    action,
                    reason: "trace exceeded the configured exploration boundary".to_string(),
                }),
            };
        }

        for (name, check) in ALWAYS_PROPERTIES {
            if !(check)(model, &next) {
                return TraceValidation {
                    ok: false,
                    final_state: next,
                    failure: Some(TraceFailure {
                        step_index,
                        action,
                        reason: format!("violated invariant: {name}"),
                    }),
                };
            }
        }

        state = next;
    }

    TraceValidation { ok: true, final_state: state, failure: None }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn known_good_keep_breach_trace_validates() {
        let model = SkyjoustInteractionModel::default();
        let result = validate_trace(&model, [
            SkyAction::AssetsLoaded,
            SkyAction::StartSkirmish,
            SkyAction::StartBattle,
            SkyAction::FinishConstructing,
            SkyAction::SpawnReady,
            SkyAction::CountdownDone,
            SkyAction::BombKeepBreach,
            SkyAction::VictoryCheck,
            SkyAction::ExportFinalScore,
            SkyAction::TallyRewards,
            SkyAction::CommitRewards,
        ]);
        assert!(result.ok, "{result:?}");
        assert!(result.final_state.rewards.committed);
    }

    #[test]
    fn rewards_cannot_commit_before_final_score() {
        let model = SkyjoustInteractionModel::default();
        let result = validate_trace(&model, [SkyAction::AssetsLoaded, SkyAction::StartSkirmish, SkyAction::CommitRewards]);
        assert!(!result.ok);
        assert!(result.failure.unwrap().reason.contains("action was not legal"));
    }
}

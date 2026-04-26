//! Serde adapter implementations for state domain types.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::state::{
    AppState,
    BanquetState,
    CeremonyState,
    DuelState,
    EventKind,
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
    TournamentState,
    WarfrontState,
    WeddingState,
    Winner,
};

#[derive(Serialize, Deserialize)]
struct SkyStateDto {
    depth: u16,
    app: AppState,
    warfront: WarfrontState,
    match_phase: MatchPhase,
    ceremony: CeremonyState,
    rules: Rules,
    player_ordnance: PlayerOrdnance,
    lance: LanceState,
    recovery: RecoveryState,
    objectives: ObjectiveSnapshot,
    score: ScoreLedger,
    rewards: RewardLedger,
    winner: Winner,
    truce_active: bool,
    truce_broken: bool,
    tournament_rounds_won: u8,
    tournament_completed: bool,
    duel_resolved: bool,
    duel_consequence_active: bool,
    treaty_signed: bool,
    infamy: i16,
    post_final_score_write: bool,
    warfront_mutated_during_match: bool,
}

impl From<SkyState> for SkyStateDto {
    fn from(state: SkyState) -> Self {
        Self {
            depth: state.depth,
            app: state.app,
            warfront: state.warfront,
            match_phase: state.match_phase,
            ceremony: state.ceremony,
            rules: state.rules,
            player_ordnance: state.player_ordnance,
            lance: state.lance,
            recovery: state.recovery,
            objectives: state.objectives,
            score: state.score,
            rewards: state.rewards,
            winner: state.winner,
            truce_active: state.truce_active,
            truce_broken: state.truce_broken,
            tournament_rounds_won: state.tournament_rounds_won,
            tournament_completed: state.tournament_completed,
            duel_resolved: state.duel_resolved,
            duel_consequence_active: state.duel_consequence_active,
            treaty_signed: state.treaty_signed,
            infamy: state.infamy,
            post_final_score_write: state.post_final_score_write,
            warfront_mutated_during_match: state.warfront_mutated_during_match,
        }
    }
}

impl From<SkyStateDto> for SkyState {
    fn from(dto: SkyStateDto) -> Self {
        Self {
            depth: dto.depth,
            app: dto.app,
            warfront: dto.warfront,
            match_phase: dto.match_phase,
            ceremony: dto.ceremony,
            rules: dto.rules,
            player_ordnance: dto.player_ordnance,
            lance: dto.lance,
            recovery: dto.recovery,
            objectives: dto.objectives,
            score: dto.score,
            rewards: dto.rewards,
            winner: dto.winner,
            truce_active: dto.truce_active,
            truce_broken: dto.truce_broken,
            tournament_rounds_won: dto.tournament_rounds_won,
            tournament_completed: dto.tournament_completed,
            duel_resolved: dto.duel_resolved,
            duel_consequence_active: dto.duel_consequence_active,
            treaty_signed: dto.treaty_signed,
            infamy: dto.infamy,
            post_final_score_write: dto.post_final_score_write,
            warfront_mutated_during_match: dto.warfront_mutated_during_match,
        }
    }
}

impl Serialize for SkyState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SkyStateDto::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SkyState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SkyStateDto::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Serialize, Deserialize)]
struct RulesDto {
    ordnance: OrdnancePolicy,
    friendly_fire: bool,
    duel_lock: bool,
    scoring_frozen: bool,
    joust_only: bool,
    allow_sudden_death: bool,
}

impl Serialize for Rules {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RulesDto {
            ordnance: self.ordnance,
            friendly_fire: self.friendly_fire,
            duel_lock: self.duel_lock,
            scoring_frozen: self.scoring_frozen,
            joust_only: self.joust_only,
            allow_sudden_death: self.allow_sudden_death,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rules {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dto = RulesDto::deserialize(deserializer)?;
        Ok(Self {
            ordnance: dto.ordnance,
            friendly_fire: dto.friendly_fire,
            duel_lock: dto.duel_lock,
            scoring_frozen: dto.scoring_frozen,
            joust_only: dto.joust_only,
            allow_sudden_death: dto.allow_sudden_death,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ObjectiveSnapshotDto {
    keep_breached: bool,
    outpost_controlled: bool,
    shrine_claimed: bool,
    supply_route_blocked: bool,
    hostage_delivered: bool,
}

impl Serialize for ObjectiveSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ObjectiveSnapshotDto {
            keep_breached: self.keep_breached,
            outpost_controlled: self.outpost_controlled,
            shrine_claimed: self.shrine_claimed,
            supply_route_blocked: self.supply_route_blocked,
            hostage_delivered: self.hostage_delivered,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ObjectiveSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dto = ObjectiveSnapshotDto::deserialize(deserializer)?;
        Ok(Self {
            keep_breached: dto.keep_breached,
            outpost_controlled: dto.outpost_controlled,
            shrine_claimed: dto.shrine_claimed,
            supply_route_blocked: dto.supply_route_blocked,
            hostage_delivered: dto.hostage_delivered,
        })
    }
}

#[derive(Serialize, Deserialize)]
enum CeremonyStateDto {
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

impl From<CeremonyState> for CeremonyStateDto {
    fn from(state: CeremonyState) -> Self {
        match state {
            CeremonyState::Dormant => Self::Dormant,
            CeremonyState::Queued(kind) => Self::Queued(kind),
            CeremonyState::Prompt(kind) => Self::Prompt(kind),
            CeremonyState::Tournament(inner) => Self::Tournament(inner),
            CeremonyState::Duel(inner) => Self::Duel(inner),
            CeremonyState::Wedding(inner) => Self::Wedding(inner),
            CeremonyState::Banquet(inner) => Self::Banquet(inner),
            CeremonyState::ConsequenceResolution => Self::ConsequenceResolution,
            CeremonyState::Cooldown => Self::Cooldown,
        }
    }
}

impl From<CeremonyStateDto> for CeremonyState {
    fn from(dto: CeremonyStateDto) -> Self {
        match dto {
            CeremonyStateDto::Dormant => Self::Dormant,
            CeremonyStateDto::Queued(kind) => Self::Queued(kind),
            CeremonyStateDto::Prompt(kind) => Self::Prompt(kind),
            CeremonyStateDto::Tournament(inner) => Self::Tournament(inner),
            CeremonyStateDto::Duel(inner) => Self::Duel(inner),
            CeremonyStateDto::Wedding(inner) => Self::Wedding(inner),
            CeremonyStateDto::Banquet(inner) => Self::Banquet(inner),
            CeremonyStateDto::ConsequenceResolution => Self::ConsequenceResolution,
            CeremonyStateDto::Cooldown => Self::Cooldown,
        }
    }
}

impl Serialize for CeremonyState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CeremonyStateDto::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CeremonyState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        CeremonyStateDto::deserialize(deserializer).map(Self::from)
    }
}

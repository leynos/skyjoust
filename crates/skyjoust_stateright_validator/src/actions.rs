//! Action and small domain enums used by the Skyjoust validator model.

/// Team identifier used for score, morale, and penalty routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    Red,
    Blue,
}

/// Joust result categories that can become score atoms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JoustOutcome {
    Knockback,
    Unhorse,
    Shatter,
    CleanKill,
}

/// High-level actions explored by the Stateright model and trace validator.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    TournamentRoundWon { winner: Team },
    TournamentChampionDeclared,

    IssueDuel,
    AcceptDuel,
    RefuseDuel,
    DuelReady,
    DuelDecisiveJoust { winner: Team, outcome: JoustOutcome },
    DuelInterference { offender: Team },

    StartWeddingTruce,
    CompleteJointObjective { team: Team },
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
    CaptureOutpost { team: Team },
    ClaimShrine { team: Team },
    BlockSupplyRoute { team: Team },
    DeliverHostage { team: Team },
    BombKeepBreach { team: Team },
    TimerExpired,
    VictoryCheck,

    ExportFinalScore,
    TallyRewards,
    CommitRewards,
    NextWarfrontTurn,
    ReturnToTitle,
}

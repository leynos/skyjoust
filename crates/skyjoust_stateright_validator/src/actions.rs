//! Action and small domain enums used by the Skyjoust validator model.

/// Team identifier used for score, morale, and penalty routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    /// The red team.
    Red,
    /// The blue team.
    Blue,
}

/// Joust result categories that can become score atoms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JoustOutcome {
    /// A joust contact that only pushed the loser back.
    Knockback,
    /// A joust contact that unhorsed the loser.
    Unhorse,
    /// A joust contact that shattered the loser's lance.
    Shatter,
    /// A joust contact that eliminated the loser outright.
    CleanKill,
}

/// High-level actions explored by the Stateright model and trace validator.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SkyAction {
    /// Game assets finished loading.
    AssetsLoaded,
    /// The player started a skirmish match.
    StartSkirmish,
    /// The player started a Warfront campaign.
    StartWarfront,
    /// The Warfront map finished generating or loading.
    MapReady,
    /// The player selected a Warfront region for the next battle.
    SelectRegion,
    /// The player locked in the previewed battle.
    StartBattle,
    /// Match construction finished and spawn warmup can begin.
    FinishConstructing,
    /// Spawn warmup finished and the countdown can begin.
    SpawnReady,
    /// The pre-match countdown finished.
    CountdownDone,

    /// A tournament ceremony was triggered.
    TriggerTournament,
    /// The tournament arena finished building.
    ArenaReady,
    /// Tournament registration completed.
    TournamentRegistered,
    /// A tournament round was won.
    TournamentRoundWon {
        /// The team that won the round.
        winner: Team,
    },
    /// A tournament champion was declared.
    TournamentChampionDeclared,

    /// A duel was issued.
    IssueDuel,
    /// A duel challenge was accepted.
    AcceptDuel,
    /// A duel challenge was refused.
    RefuseDuel,
    /// The duel arena finished locking down.
    DuelReady,
    /// The duel was resolved by a decisive joust.
    DuelDecisiveJoust {
        /// The team that won the decisive joust.
        winner: Team,
        /// The outcome category of the decisive joust.
        outcome: JoustOutcome,
    },
    /// A team interfered with the active duel.
    DuelInterference {
        /// The team that interfered.
        offender: Team,
    },

    /// A wedding-alliance truce was started.
    StartWeddingTruce,
    /// A joint objective under the wedding alliance was completed.
    CompleteJointObjective {
        /// The team that completed the joint objective.
        team: Team,
    },
    /// The wedding-alliance truce was broken.
    BreakTruce,
    /// The wedding-alliance truce expired naturally.
    ExpireTruce,

    /// A banquet negotiation was opened.
    OpenBanquet,
    /// The banquet finished seating and terms can be proposed.
    BanquetReady,
    /// A treaty was proposed at the banquet.
    ProposeTreaty,
    /// The proposed treaty was accepted.
    AcceptTreaty,
    /// The proposed treaty was rejected.
    RejectTreaty,

    /// Ceremony consequences finished recording.
    EventConsequencesRecorded,
    /// The post-ceremony cooldown finished.
    EventCooldownDone,

    /// The player pressed the lance brace control.
    BracePressed,
    /// The lance brace window expired without a joust contact.
    BraceWindowExpired,
    /// A joust contact resolved between two riders.
    Joust {
        /// The team that won the joust contact.
        winner: Team,
        /// The outcome category of the joust contact.
        outcome: JoustOutcome,
    },
    /// A team captured the outpost objective.
    CaptureOutpost {
        /// The team that captured the outpost.
        team: Team,
    },
    /// A team claimed the shrine objective.
    ClaimShrine {
        /// The team that claimed the shrine.
        team: Team,
    },
    /// A team blocked the supply route objective.
    BlockSupplyRoute {
        /// The team that blocked the supply route.
        team: Team,
    },
    /// A team delivered the hostage objective.
    DeliverHostage {
        /// The team that delivered the hostage.
        team: Team,
    },
    /// A team breached the keep objective with a bomb.
    BombKeepBreach {
        /// The team that breached the keep.
        team: Team,
    },
    /// The match timer expired.
    TimerExpired,
    /// A victory condition check was requested.
    VictoryCheck,

    /// The final score snapshot was exported.
    ExportFinalScore,
    /// Warfront rewards were tallied from the final score.
    TallyRewards,
    /// Tallied rewards were committed to the campaign state.
    CommitRewards,
    /// The Warfront campaign advanced to the next turn.
    NextWarfrontTurn,
    /// The player returned to the title screen.
    ReturnToTitle,
}

//! Name tables and lookups shared by the `SkyAction` serde adapters.

use crate::actions::SkyAction;

pub(super) const UNIT_ACTION_NAMES: &[&str] = &[
    "AssetsLoaded",
    "StartSkirmish",
    "StartWarfront",
    "MapReady",
    "SelectRegion",
    "StartBattle",
    "FinishConstructing",
    "SpawnReady",
    "CountdownDone",
    "TriggerTournament",
    "ArenaReady",
    "TournamentRegistered",
    "TournamentChampionDeclared",
    "IssueDuel",
    "AcceptDuel",
    "RefuseDuel",
    "DuelReady",
    "StartWeddingTruce",
    "BreakTruce",
    "ExpireTruce",
    "OpenBanquet",
    "BanquetReady",
    "ProposeTreaty",
    "AcceptTreaty",
    "RejectTreaty",
    "EventConsequencesRecorded",
    "EventCooldownDone",
    "BracePressed",
    "BraceWindowExpired",
    "TimerExpired",
    "VictoryCheck",
    "ExportFinalScore",
    "TallyRewards",
    "CommitRewards",
    "NextWarfrontTurn",
    "ReturnToTitle",
];

pub(super) const TAGGED_ACTION_NAMES: &[&str] = &[
    "DuelDecisiveJoust",
    "DuelInterference",
    "Joust",
    "TournamentRoundWon",
    "CompleteJointObjective",
    "CaptureOutpost",
    "ClaimShrine",
    "BlockSupplyRoute",
    "DeliverHostage",
    "BombKeepBreach",
];

/// Return the unit-name string for `action`, or `None` for a tagged variant.
///
/// `Serialize for SkyAction` dispatches every tagged variant to
/// `serialize_tagged`/`serialize_team_action` before it ever reaches this
/// function, so the `None` arm is not expected to run in practice. It
/// returns `None` rather than panicking so a future caller — or a bug in
/// that dispatch — gets a value to turn into a proper serialization error
/// instead of an unconditional panic.
pub(super) const fn unit_action_name(action: &SkyAction) -> Option<&'static str> {
    Some(match action {
        SkyAction::AssetsLoaded => "AssetsLoaded",
        SkyAction::StartSkirmish => "StartSkirmish",
        SkyAction::StartWarfront => "StartWarfront",
        SkyAction::MapReady => "MapReady",
        SkyAction::SelectRegion => "SelectRegion",
        SkyAction::StartBattle => "StartBattle",
        SkyAction::FinishConstructing => "FinishConstructing",
        SkyAction::SpawnReady => "SpawnReady",
        SkyAction::CountdownDone => "CountdownDone",
        SkyAction::TriggerTournament => "TriggerTournament",
        SkyAction::ArenaReady => "ArenaReady",
        SkyAction::TournamentRegistered => "TournamentRegistered",
        SkyAction::TournamentChampionDeclared => "TournamentChampionDeclared",
        SkyAction::IssueDuel => "IssueDuel",
        SkyAction::AcceptDuel => "AcceptDuel",
        SkyAction::RefuseDuel => "RefuseDuel",
        SkyAction::DuelReady => "DuelReady",
        SkyAction::StartWeddingTruce => "StartWeddingTruce",
        SkyAction::BreakTruce => "BreakTruce",
        SkyAction::ExpireTruce => "ExpireTruce",
        SkyAction::OpenBanquet => "OpenBanquet",
        SkyAction::BanquetReady => "BanquetReady",
        SkyAction::ProposeTreaty => "ProposeTreaty",
        SkyAction::AcceptTreaty => "AcceptTreaty",
        SkyAction::RejectTreaty => "RejectTreaty",
        SkyAction::EventConsequencesRecorded => "EventConsequencesRecorded",
        SkyAction::EventCooldownDone => "EventCooldownDone",
        SkyAction::BracePressed => "BracePressed",
        SkyAction::BraceWindowExpired => "BraceWindowExpired",
        SkyAction::TimerExpired => "TimerExpired",
        SkyAction::VictoryCheck => "VictoryCheck",
        SkyAction::ExportFinalScore => "ExportFinalScore",
        SkyAction::TallyRewards => "TallyRewards",
        SkyAction::CommitRewards => "CommitRewards",
        SkyAction::NextWarfrontTurn => "NextWarfrontTurn",
        SkyAction::ReturnToTitle => "ReturnToTitle",
        SkyAction::DuelDecisiveJoust { .. }
        | SkyAction::DuelInterference { .. }
        | SkyAction::Joust { .. }
        | SkyAction::TournamentRoundWon { .. }
        | SkyAction::CompleteJointObjective { .. }
        | SkyAction::CaptureOutpost { .. }
        | SkyAction::ClaimShrine { .. }
        | SkyAction::BlockSupplyRoute { .. }
        | SkyAction::DeliverHostage { .. }
        | SkyAction::BombKeepBreach { .. } => return None,
    })
}

pub(super) fn unit_action_from_name(name: &str) -> Option<SkyAction> {
    Some(match name {
        "AssetsLoaded" => SkyAction::AssetsLoaded,
        "StartSkirmish" => SkyAction::StartSkirmish,
        "StartWarfront" => SkyAction::StartWarfront,
        "MapReady" => SkyAction::MapReady,
        "SelectRegion" => SkyAction::SelectRegion,
        "StartBattle" => SkyAction::StartBattle,
        "FinishConstructing" => SkyAction::FinishConstructing,
        "SpawnReady" => SkyAction::SpawnReady,
        "CountdownDone" => SkyAction::CountdownDone,
        "TriggerTournament" => SkyAction::TriggerTournament,
        "ArenaReady" => SkyAction::ArenaReady,
        "TournamentRegistered" => SkyAction::TournamentRegistered,
        "TournamentChampionDeclared" => SkyAction::TournamentChampionDeclared,
        "IssueDuel" => SkyAction::IssueDuel,
        "AcceptDuel" => SkyAction::AcceptDuel,
        "RefuseDuel" => SkyAction::RefuseDuel,
        "DuelReady" => SkyAction::DuelReady,
        "StartWeddingTruce" => SkyAction::StartWeddingTruce,
        "BreakTruce" => SkyAction::BreakTruce,
        "ExpireTruce" => SkyAction::ExpireTruce,
        "OpenBanquet" => SkyAction::OpenBanquet,
        "BanquetReady" => SkyAction::BanquetReady,
        "ProposeTreaty" => SkyAction::ProposeTreaty,
        "AcceptTreaty" => SkyAction::AcceptTreaty,
        "RejectTreaty" => SkyAction::RejectTreaty,
        "EventConsequencesRecorded" => SkyAction::EventConsequencesRecorded,
        "EventCooldownDone" => SkyAction::EventCooldownDone,
        "BracePressed" => SkyAction::BracePressed,
        "BraceWindowExpired" => SkyAction::BraceWindowExpired,
        "TimerExpired" => SkyAction::TimerExpired,
        "VictoryCheck" => SkyAction::VictoryCheck,
        "ExportFinalScore" => SkyAction::ExportFinalScore,
        "TallyRewards" => SkyAction::TallyRewards,
        "CommitRewards" => SkyAction::CommitRewards,
        "NextWarfrontTurn" => SkyAction::NextWarfrontTurn,
        "ReturnToTitle" => SkyAction::ReturnToTitle,
        _ => return None,
    })
}

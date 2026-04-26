//! Serde adapter implementations for action domain types.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::actions::{JoustOutcome, SkyAction, Team};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DuelJoustDto {
    winner: Team,
    outcome: JoustOutcome,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OffenderDto {
    offender: Team,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TeamDto {
    team: Team,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WinnerDto {
    winner: Team,
}

impl Serialize for SkyAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::DuelDecisiveJoust { winner, outcome } => serialize_tagged(
                serializer,
                "DuelDecisiveJoust",
                DuelJoustDto {
                    winner: *winner,
                    outcome: *outcome,
                },
            ),
            Self::DuelInterference { offender } => serialize_tagged(
                serializer,
                "DuelInterference",
                OffenderDto {
                    offender: *offender,
                },
            ),
            Self::Joust { winner, outcome } => serialize_tagged(
                serializer,
                "Joust",
                DuelJoustDto {
                    winner: *winner,
                    outcome: *outcome,
                },
            ),
            Self::TournamentRoundWon { winner } => serialize_tagged(
                serializer,
                "TournamentRoundWon",
                WinnerDto { winner: *winner },
            ),
            Self::CompleteJointObjective { team } => {
                serialize_team_action(serializer, "CompleteJointObjective", *team)
            }
            Self::CaptureOutpost { team } => {
                serialize_team_action(serializer, "CaptureOutpost", *team)
            }
            Self::ClaimShrine { team } => serialize_team_action(serializer, "ClaimShrine", *team),
            Self::BlockSupplyRoute { team } => {
                serialize_team_action(serializer, "BlockSupplyRoute", *team)
            }
            Self::DeliverHostage { team } => {
                serialize_team_action(serializer, "DeliverHostage", *team)
            }
            Self::BombKeepBreach { team } => {
                serialize_team_action(serializer, "BombKeepBreach", *team)
            }
            Self::AssetsLoaded
            | Self::StartSkirmish
            | Self::StartWarfront
            | Self::MapReady
            | Self::SelectRegion
            | Self::StartBattle
            | Self::FinishConstructing
            | Self::SpawnReady
            | Self::CountdownDone
            | Self::TriggerTournament
            | Self::ArenaReady
            | Self::TournamentRegistered
            | Self::TournamentChampionDeclared
            | Self::IssueDuel
            | Self::AcceptDuel
            | Self::DuelReady
            | Self::StartWeddingTruce
            | Self::BreakTruce
            | Self::ExpireTruce
            | Self::OpenBanquet
            | Self::BanquetReady
            | Self::ProposeTreaty
            | Self::AcceptTreaty
            | Self::RejectTreaty
            | Self::EventConsequencesRecorded
            | Self::EventCooldownDone
            | Self::BracePressed
            | Self::BraceWindowExpired
            | Self::TimerExpired
            | Self::VictoryCheck
            | Self::ExportFinalScore
            | Self::TallyRewards
            | Self::CommitRewards
            | Self::NextWarfrontTurn
            | Self::ReturnToTitle => serializer.serialize_str(unit_action_name(self)),
        }
    }
}

impl<'de> Deserialize<'de> for SkyAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(name) => unit_action_from_name(&name)
                .ok_or_else(|| D::Error::unknown_variant(name.as_str(), UNIT_ACTION_NAMES)),
            serde_json::Value::Object(object) if object.len() == 1 => {
                let (name, payload) = object
                    .into_iter()
                    .next()
                    .ok_or_else(|| D::Error::custom("expected tagged SkyAction"))?;
                match name.as_str() {
                    "DuelDecisiveJoust" => {
                        let dto: DuelJoustDto =
                            serde_json::from_value(payload).map_err(D::Error::custom)?;
                        Ok(Self::DuelDecisiveJoust {
                            winner: dto.winner,
                            outcome: dto.outcome,
                        })
                    }
                    "DuelInterference" => {
                        let dto: OffenderDto =
                            serde_json::from_value(payload).map_err(D::Error::custom)?;
                        Ok(Self::DuelInterference {
                            offender: dto.offender,
                        })
                    }
                    "Joust" => {
                        let dto: DuelJoustDto =
                            serde_json::from_value(payload).map_err(D::Error::custom)?;
                        Ok(Self::Joust {
                            winner: dto.winner,
                            outcome: dto.outcome,
                        })
                    }
                    "TournamentRoundWon" => {
                        let dto: WinnerDto =
                            serde_json::from_value(payload).map_err(D::Error::custom)?;
                        Ok(Self::TournamentRoundWon { winner: dto.winner })
                    }
                    "CompleteJointObjective" => deserialize_team_action(payload, |team| {
                        Self::CompleteJointObjective { team }
                    })
                    .map_err(D::Error::custom),
                    "CaptureOutpost" => {
                        deserialize_team_action(payload, |team| Self::CaptureOutpost { team })
                            .map_err(D::Error::custom)
                    }
                    "ClaimShrine" => {
                        deserialize_team_action(payload, |team| Self::ClaimShrine { team })
                            .map_err(D::Error::custom)
                    }
                    "BlockSupplyRoute" => {
                        deserialize_team_action(payload, |team| Self::BlockSupplyRoute { team })
                            .map_err(D::Error::custom)
                    }
                    "DeliverHostage" => {
                        deserialize_team_action(payload, |team| Self::DeliverHostage { team })
                            .map_err(D::Error::custom)
                    }
                    "BombKeepBreach" => {
                        deserialize_team_action(payload, |team| Self::BombKeepBreach { team })
                            .map_err(D::Error::custom)
                    }
                    _ => Err(D::Error::unknown_variant(
                        name.as_str(),
                        TAGGED_ACTION_NAMES,
                    )),
                }
            }
            _ => Err(D::Error::custom(
                "expected a SkyAction string or tagged object",
            )),
        }
    }
}

const UNIT_ACTION_NAMES: &[&str] = &[
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

const TAGGED_ACTION_NAMES: &[&str] = &[
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

fn unit_action_name(action: &SkyAction) -> &'static str {
    match action {
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
        | SkyAction::BombKeepBreach { .. } => {
            unreachable!("tagged action variants are serialized before unit names")
        }
    }
}

fn unit_action_from_name(name: &str) -> Option<SkyAction> {
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

fn serialize_team_action<S>(
    serializer: S,
    name: &'static str,
    team: Team,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize_tagged(serializer, name, TeamDto { team })
}

fn deserialize_team_action<F>(
    payload: serde_json::Value,
    build: F,
) -> Result<SkyAction, serde_json::Error>
where
    F: FnOnce(Team) -> SkyAction,
{
    let dto: TeamDto = serde_json::from_value(payload)?;
    Ok(build(dto.team))
}

fn serialize_tagged<S, T>(serializer: S, name: &'static str, payload: T) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    use serde::ser::SerializeMap;

    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry(name, &payload)?;
    map.end()
}

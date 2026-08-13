//! Serde adapter implementations for action domain types.

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::Error as _,
    ser::Error as SerError,
};

use super::action_names::{
    TAGGED_ACTION_NAMES,
    UNIT_ACTION_NAMES,
    unit_action_from_name,
    unit_action_name,
};
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
    // One arm per `SkyAction` variant is the clearest way to express this
    // dispatch; splitting it further would trade a single obvious match for
    // several small functions that all need to be read together anyway.
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive match arm per SkyAction variant is clearer than an artificial \
                  split"
    )]
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
            | Self::RefuseDuel
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
            | Self::ReturnToTitle => {
                let name = unit_action_name(self).ok_or_else(|| {
                    S::Error::custom("tagged action variant reached the unit-name serializer")
                })?;
                serializer.serialize_str(name)
            }
        }
    }
}

impl<'de> Deserialize<'de> for SkyAction {
    // Mirrors the exhaustive match in `Serialize for SkyAction`: one arm per
    // tagged variant name is the clearest way to express this dispatch.
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive match arm per tagged SkyAction variant is clearer than an \
                  artificial split"
    )]
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

#[cfg(test)]
mod tests {
    //! Round-trip tests for the `SkyAction` serde adapter.

    use rstest::rstest;

    use super::*;
    use crate::actions::JoustOutcome;

    #[rstest]
    #[case::assets_loaded(SkyAction::AssetsLoaded, "\"AssetsLoaded\"")]
    #[case::return_to_title(SkyAction::ReturnToTitle, "\"ReturnToTitle\"")]
    fn unit_action_round_trips_through_json(#[case] action: SkyAction, #[case] json: &str) {
        let serialized = serde_json::to_string(&action).expect("unit action should serialize");

        assert_eq!(serialized, json);
        assert_eq!(
            serde_json::from_str::<SkyAction>(&serialized)
                .expect("serialized unit action should deserialize"),
            action
        );
    }

    #[rstest]
    #[case::team_action(
        SkyAction::CaptureOutpost { team: Team::Red },
        "{\"CaptureOutpost\":{\"team\":\"Red\"}}"
    )]
    #[case::joust(
        SkyAction::Joust { winner: Team::Blue, outcome: JoustOutcome::CleanKill },
        "{\"Joust\":{\"winner\":\"Blue\",\"outcome\":\"CleanKill\"}}"
    )]
    fn tagged_action_round_trips_through_json(#[case] action: SkyAction, #[case] json: &str) {
        let serialized = serde_json::to_string(&action).expect("tagged action should serialize");

        assert_eq!(serialized, json);
        assert_eq!(
            serde_json::from_str::<SkyAction>(&serialized)
                .expect("serialized tagged action should deserialize"),
            action
        );
    }
}

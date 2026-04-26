//! Serde adapter implementations for unit-like enum domain types.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

use crate::{
    actions::{JoustOutcome, Team},
    state::{
        AppState,
        BanquetState,
        DuelState,
        EventKind,
        LanceState,
        MatchPhase,
        OrdnancePolicy,
        PlayerOrdnance,
        RecoveryState,
        TournamentState,
        WarfrontState,
        WeddingState,
        Winner,
    },
};

macro_rules! string_enum_serde {
    ($type:ty, {$($variant:ident => $name:literal),+ $(,)?}) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let value = match self {
                    $(Self::$variant => $name,)+
                };
                serializer.serialize_str(value)
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct EnumVisitor;

                impl Visitor<'_> for EnumVisitor {
                    type Value = $type;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str("a Skyjoust enum variant string")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            $($name => Ok(<$type>::$variant),)+
                            _ => Err(E::unknown_variant(value, &[$($name),+])),
                        }
                    }
                }

                deserializer.deserialize_str(EnumVisitor)
            }
        }
    };
}

string_enum_serde!(Team, { Red => "Red", Blue => "Blue" });
string_enum_serde!(JoustOutcome, {
    Knockback => "Knockback",
    Unhorse => "Unhorse",
    Shatter => "Shatter",
    CleanKill => "CleanKill",
});
string_enum_serde!(AppState, {
    Boot => "Boot",
    Title => "Title",
    SkirmishSetup => "SkirmishSetup",
    WarfrontSetup => "WarfrontSetup",
    WarfrontRunning => "WarfrontRunning",
    MatchRunning => "MatchRunning",
    Results => "Results",
});
string_enum_serde!(WarfrontState, {
    Inactive => "Inactive",
    GenerateOrLoad => "GenerateOrLoad",
    StrategicChoice => "StrategicChoice",
    BattlePreview => "BattlePreview",
    BattleLocked => "BattleLocked",
    AwaitingBattleResult => "AwaitingBattleResult",
    ApplyBattleResult => "ApplyBattleResult",
    RewardCommit => "RewardCommit",
    BanquetNegotiation => "BanquetNegotiation",
    SeasonComplete => "SeasonComplete",
});
string_enum_serde!(MatchPhase, {
    Inactive => "Inactive",
    Constructing => "Constructing",
    SpawnWarmup => "SpawnWarmup",
    Countdown => "Countdown",
    NormalPlay => "NormalPlay",
    EventOverride => "EventOverride",
    SuddenDeath => "SuddenDeath",
    Paused => "Paused",
    RoundOver => "RoundOver",
    ResultsExported => "ResultsExported",
});
string_enum_serde!(EventKind, {
    Tournament => "Tournament",
    Duel => "Duel",
    WeddingAlliance => "WeddingAlliance",
    Banquet => "Banquet",
});
string_enum_serde!(TournamentState, {
    ArenaBuild => "ArenaBuild",
    Registration => "Registration",
    RoundActive => "RoundActive",
    RoundComplete => "RoundComplete",
    ChampionDeclared => "ChampionDeclared",
});
string_enum_serde!(DuelState, {
    ChallengeIssued => "ChallengeIssued",
    ArenaLock => "ArenaLock",
    DuelActive => "DuelActive",
    ResolveDuel => "ResolveDuel",
});
string_enum_serde!(WeddingState, {
    AllianceProposed => "AllianceProposed",
    TruceActive => "TruceActive",
    JointObjective => "JointObjective",
    Broken => "Broken",
    Expired => "Expired",
});
string_enum_serde!(BanquetState, {
    Seating => "Seating",
    TermsOpen => "TermsOpen",
    TreatySigned => "TreatySigned",
    Collapsed => "Collapsed",
});
string_enum_serde!(OrdnancePolicy, {
    Full => "Full",
    Limited => "Limited",
    Disabled => "Disabled",
});
string_enum_serde!(PlayerOrdnance, {
    Ready => "Ready",
    Cooldown => "Cooldown",
    ResupplyNeeded => "ResupplyNeeded",
    Disabled => "Disabled",
});
string_enum_serde!(LanceState, {
    Idle => "Idle",
    Bracing => "Bracing",
    Recovery => "Recovery",
    Broken => "Broken",
});
string_enum_serde!(RecoveryState, {
    Alive => "Alive",
    Stunned => "Stunned",
    Unhorsed => "Unhorsed",
    Dead => "Dead",
    Respawning => "Respawning",
});
string_enum_serde!(Winner, {
    None => "None",
    Red => "Red",
    Blue => "Blue",
    TieBreak => "TieBreak",
});

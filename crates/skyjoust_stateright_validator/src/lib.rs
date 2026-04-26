//! Stateright validator for Project Skyjoust's high-level engine interactions.
//!
//! The model is intentionally smaller than the runtime engine. It preserves the
//! interactions we care about proving at the design level:
//!
//! - match lifecycle gates scoring,
//! - ceremony events override and then restore rules,
//! - player actions can only emit legal scoring atoms,
//! - objectives can close the match,
//! - rewards can only commit from a finalized score snapshot,
//! - warfront state only mutates through result and reward handoff.
//!
//! Use [`SkyjoustInteractionModel`] with Stateright for exhaustive exploration,
//! or use [`validate_trace`] to replay an engine event log against the same
//! contract.

mod action_generation;
mod actions;
mod ceremonies;
mod ledgers;
mod model;
mod properties;
mod scoring;
mod serde_impls;
mod state;
mod state_helpers;
mod stateright_adapter;
mod trace;
mod transitions;

pub use actions::{JoustOutcome, SkyAction, Team};
pub use model::{AlwaysProperty, SkyjoustInteractionModel};
pub use properties::{ALWAYS_PROPERTIES, SOMETIMES_PROPERTIES};
pub use state::{
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
    RewardPhase,
    Rules,
    ScoreLedger,
    SkyState,
    TournamentState,
    WarfrontState,
    WeddingState,
    Winner,
};
pub use trace::{validate_trace, TraceFailure, TraceValidation};

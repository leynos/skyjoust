//! State types and guard helpers for the Skyjoust validator model.

pub use crate::ledgers::{RewardLedger, RewardPhase, ScoreLedger};

/// Complete validator snapshot for app, match, ceremony, scoring, and rewards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independently toggled piece of match state, not a set of related \
              options"
)]
pub struct SkyState {
    /// Number of transitions explored to reach this state, used to bound search depth.
    pub depth: u16,
    /// Top-level application mode.
    pub app: AppState,
    /// Warfront campaign state.
    pub warfront: WarfrontState,
    /// Match lifecycle phase.
    pub match_phase: MatchPhase,
    /// Ceremony state nested under tournament, duel, wedding, and banquet flows.
    pub ceremony: CeremonyState,
    /// Current temporary and baseline rules gating legal match actions.
    pub rules: Rules,
    /// Player ordnance lifecycle state.
    pub player_ordnance: PlayerOrdnance,
    /// Lance lifecycle state.
    pub lance: LanceState,
    /// Rider recovery state.
    pub recovery: RecoveryState,
    /// Objective flags that can emit score atoms during a match.
    pub objectives: ObjectiveSnapshot,
    /// Accumulated per-team score.
    pub score: ScoreLedger,
    /// Accumulated Warfront reward ledger.
    pub rewards: RewardLedger,
    /// Winner classification exported with the final score snapshot.
    pub winner: Winner,
    /// Whether a wedding-alliance truce is currently active.
    pub truce_active: bool,
    /// Whether the wedding-alliance truce has been broken.
    pub truce_broken: bool,
    /// Number of tournament rounds won so far.
    pub tournament_rounds_won: u8,
    /// Whether the tournament has completed.
    pub tournament_completed: bool,
    /// Whether the active duel has been resolved.
    pub duel_resolved: bool,
    /// Whether duel consequences are still being applied.
    pub duel_consequence_active: bool,
    /// Whether a banquet treaty has been signed.
    pub treaty_signed: bool,
    /// Accumulated infamy score, positive or negative.
    pub infamy: i16,
    /// Whether the final score has already been written post-match.
    pub post_final_score_write: bool,
    /// Whether the Warfront state was mutated during the current match.
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
    /// The application is booting.
    Boot,
    /// The title screen is showing.
    Title,
    /// The player is configuring a skirmish match.
    SkirmishSetup,
    /// The player is configuring a Warfront campaign.
    WarfrontSetup,
    /// A Warfront campaign is running between matches.
    WarfrontRunning,
    /// A match is running.
    MatchRunning,
    /// Match or campaign results are showing.
    Results,
}

/// Warfront campaign state relevant to match handoff and reward commits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WarfrontState {
    /// No Warfront campaign is active.
    Inactive,
    /// The campaign map is being generated or loaded.
    GenerateOrLoad,
    /// The player is choosing a strategic action.
    StrategicChoice,
    /// The next battle is being previewed.
    BattlePreview,
    /// The next battle has been locked in.
    BattleLocked,
    /// Waiting for the battle result to be applied.
    AwaitingBattleResult,
    /// The battle result is being applied to the campaign.
    ApplyBattleResult,
    /// Rewards from the battle are being committed.
    RewardCommit,
    /// A banquet negotiation is underway.
    BanquetNegotiation,
    /// The Warfront season has completed.
    SeasonComplete,
}

/// Match lifecycle phase used to gate scoring, rewards, and Warfront mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MatchPhase {
    /// No match is active.
    Inactive,
    /// The match is being constructed.
    Constructing,
    /// Players are spawning and warming up.
    SpawnWarmup,
    /// The pre-match countdown is running.
    Countdown,
    /// Normal match play is underway.
    NormalPlay,
    /// A ceremony event is overriding normal play.
    EventOverride,
    /// Sudden death is active.
    SuddenDeath,
    /// The match is paused.
    Paused,
    /// The round has ended.
    RoundOver,
    /// Match results have been exported.
    ResultsExported,
}

/// Ceremony state nested under tournament, duel, wedding, and banquet flows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CeremonyState {
    /// No ceremony is active or queued.
    Dormant,
    /// A ceremony of the given kind is queued.
    Queued(EventKind),
    /// A ceremony of the given kind is being prompted to the player.
    Prompt(EventKind),
    /// A tournament ceremony is active.
    Tournament(TournamentState),
    /// A duel ceremony is active.
    Duel(DuelState),
    /// A wedding-alliance ceremony is active.
    Wedding(WeddingState),
    /// A banquet ceremony is active.
    Banquet(BanquetState),
    /// Ceremony consequences are being resolved.
    ConsequenceResolution,
    /// The post-ceremony cooldown is running.
    Cooldown,
}

/// Ceremony categories that can be queued or prompted before activation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// A tournament ceremony.
    Tournament,
    /// A duel ceremony.
    Duel,
    /// A wedding-alliance ceremony.
    WeddingAlliance,
    /// A banquet ceremony.
    Banquet,
}

/// Tournament sub-state used while temporary tournament rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TournamentState {
    /// The tournament arena is being built.
    ArenaBuild,
    /// Players are registering for the tournament.
    Registration,
    /// A tournament round is active.
    RoundActive,
    /// A tournament round has completed.
    RoundComplete,
    /// The tournament champion has been declared.
    ChampionDeclared,
}

/// Duel sub-state used while duel lock and joust-only rules are active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DuelState {
    /// A duel challenge has been issued.
    ChallengeIssued,
    /// The duel challenge was refused.
    Refused,
    /// The duel arena is locking down.
    ArenaLock,
    /// The duel is active.
    DuelActive,
    /// The duel is being resolved.
    ResolveDuel,
}

/// Wedding alliance sub-state used for truce and joint-objective modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeddingState {
    /// A wedding alliance has been proposed.
    AllianceProposed,
    /// The wedding-alliance truce is active.
    TruceActive,
    /// A joint objective under the alliance is active.
    JointObjective,
    /// The wedding alliance has been broken.
    Broken,
    /// The wedding alliance has expired.
    Expired,
}

/// Banquet negotiation sub-state used for Warfront treaty modelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BanquetState {
    /// Attendees are being seated.
    Seating,
    /// Treaty terms are open for negotiation.
    TermsOpen,
    /// A counter-offer has been made.
    CounterOffer,
    /// The treaty has been signed.
    TreatySigned,
    /// The banquet negotiation collapsed without a treaty.
    Collapsed,
}

/// Current temporary and baseline rules that gate legal match actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independently toggled rule, not a set of related options"
)]
pub struct Rules {
    /// Ordnance availability policy.
    pub ordnance: OrdnancePolicy,
    /// Whether friendly fire is enabled.
    pub friendly_fire: bool,
    /// Whether duel locking is active, restricting play to the duellists.
    pub duel_lock: bool,
    /// Whether scoring is temporarily frozen.
    pub scoring_frozen: bool,
    /// Whether only joust actions are permitted.
    pub joust_only: bool,
    /// Whether sudden death is permitted at round end.
    pub allow_sudden_death: bool,
}

impl Rules {
    /// Return the baseline match rules used before temporary ceremony modifiers.
    ///
    /// Baseline rules enable full ordnance and friendly fire, leave duel locking
    /// disabled, keep scoring live, do not force joust-only play, and allow
    /// sudden death.
    ///
    /// ```
    /// use skyjoust_stateright_validator::{OrdnancePolicy, Rules};
    ///
    /// let rules = Rules::baseline();
    ///
    /// assert_eq!(rules.ordnance, OrdnancePolicy::Full);
    /// assert!(rules.friendly_fire);
    /// assert!(!rules.duel_lock);
    /// assert!(rules.allow_sudden_death);
    /// ```
    #[must_use]
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
    /// All ordnance is available.
    Full,
    /// Ordnance is available but constrained.
    Limited,
    /// Ordnance is disabled entirely.
    Disabled,
}

/// Player ordnance lifecycle state used by legal ordnance action checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerOrdnance {
    /// Ordnance is ready to fire.
    Ready,
    /// Ordnance is cooling down after use.
    Cooldown,
    /// Ordnance is depleted and needs resupply.
    ResupplyNeeded,
    /// Ordnance is disabled by rules or ceremony state.
    Disabled,
}

/// Lance lifecycle state used to gate brace windows and joust contacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LanceState {
    /// The lance is idle and ready.
    Idle,
    /// The lance is braced for a joust contact.
    Bracing,
    /// The lance is recovering after a contact.
    Recovery,
    /// The lance is broken and unusable.
    Broken,
}

/// Rider recovery state after collisions, unhorsing, and respawn windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecoveryState {
    /// The rider is alive and active.
    Alive,
    /// The rider is stunned after a contact.
    Stunned,
    /// The rider has been unhorsed.
    Unhorsed,
    /// The rider is dead.
    Dead,
    /// The rider is respawning.
    Respawning,
}

/// Objective flags that can emit score atoms during a match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independently toggled objective, not a set of related options"
)]
pub struct ObjectiveSnapshot {
    /// Whether the keep objective has been breached.
    pub keep_breached: bool,
    /// Whether the outpost objective is controlled.
    pub outpost_controlled: bool,
    /// Whether the shrine objective has been claimed.
    pub shrine_claimed: bool,
    /// Whether the supply route objective has been blocked.
    pub supply_route_blocked: bool,
    /// Whether the hostage objective has been delivered.
    pub hostage_delivered: bool,
}

/// Winner classification exported with the final score snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Winner {
    /// No winner has been decided.
    None,
    /// The red team won.
    Red,
    /// The blue team won.
    Blue,
    /// The match was decided by a tie-break.
    TieBreak,
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

//! Reusable headless Bevy harness adapter for `rstest-bdd` behavioural tests.
//!
//! This crate incubates in the Skyjoust workspace but depends on no Skyjoust or
//! Lille code, so it can move to its own repository as a directory move plus a
//! dependency rewire. Game-specific setup belongs in downstream profile types,
//! never here.

mod context;
mod harness;
mod panic;
mod policy;
mod profile;

pub use bevy;
pub use context::BevyScenario;
pub use harness::BevyHarness;
pub use policy::BevyAttributePolicy;
pub use profile::{
    BareBevyProfile,
    BevyProfile,
    MinimalBevyProfile,
    add_minimal_plugins,
    minimal_app,
};
pub use rstest_bdd_harness::{
    AttributePolicy,
    HarnessAdapter,
    HarnessError,
    HarnessResult,
    ScenarioMetadata,
    ScenarioRunRequest,
    ScenarioRunner,
    StdScenarioRunRequest,
    StdScenarioRunner,
    TestAttribute,
};
pub use tracing;

#[cfg(test)]
#[path = "guide_tests.rs"]
mod guide_tests;

/// Compiles the Rust examples published in the Skyjoust users' guide.
///
/// The item is hidden and exists only while rustdoc collects doctests, so the
/// guide's examples are exercised from the published text rather than a copied
/// fixture. The path is relative to this crate's `src` directory.
#[cfg(doctest)]
#[doc = include_str!("../../../docs/users-guide.md")]
pub struct UsersGuideExamples;

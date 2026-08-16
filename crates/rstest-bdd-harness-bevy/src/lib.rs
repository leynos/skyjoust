//! Reusable headless Bevy harness scaffolding for `rstest-bdd` behavioural
//! tests.
//!
//! This crate incubates in the Skyjoust workspace but depends on no Skyjoust or
//! Lille code, so it can move to its own repository as a directory move plus a
//! dependency rewire. Game-specific setup belongs in downstream profile types,
//! never here.

mod profile;

pub use bevy;
pub use rstest_bdd_harness::{
    AttributePolicy, HarnessAdapter, HarnessError, HarnessResult, ScenarioMetadata,
    ScenarioRunRequest, ScenarioRunner, StdScenarioRunRequest, StdScenarioRunner, TestAttribute,
};
pub use tracing;
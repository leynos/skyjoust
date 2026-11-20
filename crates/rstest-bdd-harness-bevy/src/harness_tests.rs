//! Lifecycle tests for the Bevy harness adapter.
//!
//! The normal-path rows of ADR 010's lifecycle table run here: a configure
//! error, a returning runner with cleanup success, and a returning runner with
//! cleanup failure. The panic rows arrive with the panic path at EP-M3.

use std::{cell::RefCell, rc::Rc};

use bevy::diagnostic::FrameCount;
use googletest::prelude::*;
use rstest_bdd_harness::{HarnessAdapter, ScenarioMetadata, ScenarioRunRequest, ScenarioRunner};

use super::BevyHarness;
use crate::{BareBevyProfile, BevyScenario};

#[path = "harness_tests/profiles.rs"]
mod profiles;

use profiles::{
    CleanupCalls,
    CleanupFails,
    ConfigureFails,
    FinalizeProbe,
    FixedTicks,
    FixedTimestep,
    NotSendProfile,
    Recording,
    RunnerCalls,
};

fn metadata() -> ScenarioMetadata {
    ScenarioMetadata::new(
        "tests/features/demo.feature",
        "Demo scenario",
        12,
        Vec::new(),
    )
}

#[gtest]
fn a_configure_error_is_wrapped_and_skips_the_runner() {
    let harness = BevyHarness::<ConfigureFails>::new();
    let runner_called = Rc::new(RefCell::new(false));
    let flag = Rc::clone(&runner_called);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |_scenario: BevyScenario| {
            *flag.borrow_mut() = true;
            0_u32
        }),
    );

    let error = harness.run(request).expect_err("configure must fail");

    expect_that!(*runner_called.borrow(), eq(false));
    expect_that!(
        error.to_string(),
        eq(concat!(
            "failed to build runtime: Bevy profile configure hook failed: ",
            "failed to build runtime: configure sentinel"
        ))
    );
}

#[gtest]
fn a_returning_runner_result_is_propagated_and_cleanup_runs_once() {
    let harness = BevyHarness::<Recording>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| {
            scenario.with_world(|world| {
                world.resource_mut::<RunnerCalls>().0 += 1;
            });
            scenario.update();
            *sink.borrow_mut() = Some(scenario);
            7_u32
        }),
    );

    let value = harness
        .run(request)
        .expect("the recording profile must succeed");

    expect_that!(value, eq(7));
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<RunnerCalls>().0),
        eq(1)
    );
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(
        scenario.with_world(|world| world.resource::<FrameCount>().0),
        eq(1)
    );
}

#[gtest]
fn a_cleanup_error_is_wrapped_after_the_runner_ran() {
    let harness = BevyHarness::<CleanupFails>::new();
    let stashed: Rc<RefCell<Option<BevyScenario>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&stashed);
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(move |scenario: BevyScenario| {
            scenario.update();
            *sink.borrow_mut() = Some(scenario);
            11_u32
        }),
    );

    let error = harness.run(request).expect_err("cleanup must fail");

    expect_that!(
        error.to_string(),
        eq(concat!(
            "failed to build runtime: Bevy profile cleanup hook failed: ",
            "failed to build runtime: cleanup sentinel"
        ))
    );
    let scenario = stashed
        .borrow()
        .clone()
        .expect("the runner stashed its scenario");
    expect_that!(
        scenario.with_world(|world| world.resource::<CleanupCalls>().0),
        eq(1)
    );
    expect_that!(
        scenario.with_world(|world| world.resource::<FrameCount>().0),
        eq(1)
    );
}

#[gtest]
fn the_harness_never_finalizes_plugins() {
    let harness = BevyHarness::<FinalizeProbe>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|scenario: BevyScenario| {
            scenario.update();
            scenario.update();
            scenario.update();
            scenario.with_world(|world| world.resource::<FrameCount>().0)
        }),
    );

    let frames = harness
        .run(request)
        .expect("the probe plugin must not be finalized");

    expect_that!(frames, eq(3));
}

#[gtest]
fn fixed_timesteps_run_one_fixed_update_per_update() {
    let harness = BevyHarness::<FixedTimestep>::new();
    let request = ScenarioRunRequest::new(
        metadata(),
        ScenarioRunner::new(|scenario: BevyScenario| {
            // Bevy's first update records the clock's first instant and
            // contributes a zero delta, so fixed ticks start on the second
            // update. Measure the steady state, which is the contract a
            // deterministic profile depends on.
            scenario.update();
            let before = scenario.with_world(|world| world.resource::<FixedTicks>().0);
            scenario.update();
            scenario.update();
            let after = scenario.with_world(|world| world.resource::<FixedTicks>().0);
            (before, after)
        }),
    );

    let (before, after) = harness
        .run(request)
        .expect("the fixed profile must succeed");

    expect_that!(after - before, eq(2));
}

#[gtest]
fn the_harness_is_send_and_sync_for_a_non_send_profile() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<BevyHarness<NotSendProfile>>();
}

#[gtest]
fn new_and_default_harnesses_are_equivalent() {
    let from_new = BevyHarness::<Recording>::new();
    let from_default = BevyHarness::<Recording>::default();

    let new_debug = format!("{from_new:?}");
    let default_debug = format!("{from_default:?}");

    expect_that!(new_debug.as_str(), eq(default_debug.as_str()));

    let new_result = from_new
        .run(ScenarioRunRequest::new(
            metadata(),
            ScenarioRunner::new(|scenario: BevyScenario| {
                scenario.update();
                5_u32
            }),
        ))
        .expect("the new harness must run");
    let default_result = from_default
        .run(ScenarioRunRequest::new(
            metadata(),
            ScenarioRunner::new(|scenario: BevyScenario| {
                scenario.update();
                5_u32
            }),
        ))
        .expect("the default harness must run");

    expect_that!(new_result, eq(default_result));
}

#[gtest]
fn request_metadata_reaches_the_scenario() {
    let expected = metadata();
    let harness = BevyHarness::<BareBevyProfile>::new();
    let request = ScenarioRunRequest::new(
        expected.clone(),
        ScenarioRunner::new(|scenario: BevyScenario| scenario.metadata().clone()),
    );

    let observed = harness.run(request).expect("the bare profile must succeed");

    expect_that!(observed, eq(&expected));
}

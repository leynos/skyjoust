//! Step definitions shared by the harness-backed Bevy scenarios.
//!
//! Every step takes the harness-provided [`BevyScenario`] through the
//! reserved `rstest_bdd_harness_context` fixture, so the steps exercise the
//! same public handle a downstream project receives. A step that can fail
//! returns [`StepResult`] rather than panicking, letting the scenario report
//! the diagnostic instead of an anonymous panic. The metadata step scans the
//! feature text included by the test binary, so its oracle is independent of
//! the harness that supplied the metadata.

use bevy::{
    diagnostic::FrameCount,
    ecs::{resource::Resource, system::ResMut},
    prelude::Update,
    time::TimePlugin,
};
use rstest_bdd::StepResult;
use rstest_bdd_harness_bevy::BevyScenario;
use rstest_bdd_macros::{given, then, when};

use crate::BEVY_SCENARIO_FEATURE;

/// Manifest-relative path every Bevy scenario feature in this binary uses.
const FEATURE_PATH_SUFFIX: &str = "tests/features/bevy_scenario.feature";

/// Scenario the metadata step is bound to; asserting it rejects metadata from
/// any other declaration that happens to be internally consistent.
const EXPECTED_METADATA_SCENARIO: &str = "Steps receive the scenario metadata from the harness";

/// Counts `Update` ticks observed by the toy system.
#[derive(Resource, Default)]
struct ToyCounter(u32);

/// Increments the toy counter once per `Update` schedule run.
fn increment_toy_counter(mut counter: ResMut<ToyCounter>) { counter.0 += 1; }

#[given("a minimal headless Bevy application")]
fn given_minimal_app(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    require_time_plugin(scenario, true)
}

#[given("a scenario driven by the minimal Bevy harness")]
fn given_minimal_harness(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    require_time_plugin(scenario, true)
}

#[given("a scenario driven by the bare Bevy harness")]
fn given_bare_harness(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    require_time_plugin(scenario, false)
}

#[given("a toy counter incremented on every Update tick")]
fn given_toy_counter(#[from(rstest_bdd_harness_context)] scenario: &BevyScenario) {
    scenario.with_app(|app| {
        app.init_resource::<ToyCounter>();
        app.add_systems(Update, increment_toy_counter);
    });
}

#[when("the schedule advances once")]
fn when_schedule_advances_once(#[from(rstest_bdd_harness_context)] scenario: &BevyScenario) {
    scenario.update();
}

#[when("the schedule advances {count:u32} times")]
fn when_schedule_advances(#[from(rstest_bdd_harness_context)] scenario: &BevyScenario, count: u32) {
    scenario.update_times(count);
}

#[when("a step fails with {message:string}")]
fn when_step_fails(message: String) -> StepResult<(), String> { Err(message) }

#[then("the frame count reads 1")]
fn then_frame_count_reads_one(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    require_frame_count(scenario, 1)
}

#[then("the frame count reads {expected:u32}")]
fn then_frame_count_reads(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
    expected: u32,
) -> StepResult<(), String> {
    require_frame_count(scenario, expected)
}

#[then("the toy counter reads {expected:u32}")]
fn then_toy_counter_reads(
    #[from(rstest_bdd_harness_context)] scenario: BevyScenario,
    expected: u32,
) -> StepResult<(), String> {
    require_counter(
        toy_count(&scenario),
        expected,
        "toy counter",
        "the scenario has no toy counter",
    )
}

#[then("the application has no frame counter")]
fn then_no_frame_counter(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    frame_count(scenario).map_or(Ok(()), |observed| {
        Err(format!(
            "the application must have no frame counter, but it read {observed}"
        ))
    })
}

#[then("the scenario metadata matches its declaration in the feature file")]
fn then_metadata_matches(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    let metadata = scenario.metadata();
    if metadata.scenario_name() != EXPECTED_METADATA_SCENARIO {
        return Err(format!(
            "the scenario name must read {EXPECTED_METADATA_SCENARIO:?}, but it read {:?}",
            metadata.scenario_name()
        ));
    }
    let Some(declared_line) = declared_scenario_line(EXPECTED_METADATA_SCENARIO) else {
        return Err(format!(
            "the feature file does not declare the scenario {EXPECTED_METADATA_SCENARIO:?}"
        ));
    };
    if metadata.scenario_line() != declared_line {
        return Err(format!(
            "the scenario line must read {declared_line}, but it read {}",
            metadata.scenario_line()
        ));
    }
    if !metadata.feature_path().ends_with(FEATURE_PATH_SUFFIX) {
        return Err(format!(
            "the feature path must end with {FEATURE_PATH_SUFFIX}, but it read {:?}",
            metadata.feature_path()
        ));
    }
    Ok(())
}

/// Fails unless the profile installed the time plugin when `required`.
fn require_time_plugin(scenario: &BevyScenario, required: bool) -> StepResult<(), String> {
    let has_time_plugin = scenario.with_app(|app| app.is_plugin_added::<TimePlugin>());
    match (has_time_plugin, required) {
        (true, true) | (false, false) => Ok(()),
        (false, true) => Err("the minimal profile must add the time plugin".to_owned()),
        (true, false) => Err("the bare profile must not add the time plugin".to_owned()),
    }
}

/// Reads the frame counter when the profile installed one.
fn frame_count(scenario: &BevyScenario) -> Option<u32> {
    scenario.with_world(|world| world.get_resource::<FrameCount>().map(|count| count.0))
}

/// Reads the toy counter when a step installed it.
fn toy_count(scenario: &BevyScenario) -> Option<u32> {
    scenario.with_world(|world| world.get_resource::<ToyCounter>().map(|count| count.0))
}

/// Fails unless the frame counter reads `expected`.
fn require_frame_count(scenario: &BevyScenario, expected: u32) -> StepResult<(), String> {
    require_counter(
        frame_count(scenario),
        expected,
        "frame count",
        "the application has no frame counter",
    )
}

/// Fails unless an observed counter equals `expected`, reporting a missing
/// counter separately from a wrong value.
fn require_counter(
    observed: Option<u32>,
    expected: u32,
    label: &str,
    missing: &str,
) -> StepResult<(), String> {
    match observed {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "the {label} must read {expected}, but it read {actual}"
        )),
        None => Err(missing.to_owned()),
    }
}

/// Returns the one-based line declaring `Scenario: {name}` in the feature.
fn declared_scenario_line(name: &str) -> Option<u32> {
    let declaration = format!("Scenario: {name}");
    BEVY_SCENARIO_FEATURE
        .lines()
        .position(|line| line.trim() == declaration)
        .and_then(|index| u32::try_from(index + 1).ok())
}

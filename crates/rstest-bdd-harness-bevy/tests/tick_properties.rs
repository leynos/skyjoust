//! Property coverage for tick additivity.
//!
//! Advancing an application by a sequence of `update` and `update_times`
//! calls must leave the engine frame counter and a toy `Update` counter equal
//! to the total number of frames the sequence requested. The generated
//! sequences are built without filtering, and a named table pins the empty
//! sequence and zero-count edge cases that a generator may rarely draw.

use bevy::{
    diagnostic::FrameCount,
    ecs::{resource::Resource, system::ResMut},
    prelude::Update,
};
use proptest::{collection::vec, prelude::*};
use rstest::rstest;
use rstest_bdd_harness::{HarnessAdapter, ScenarioMetadata, ScenarioRunRequest, ScenarioRunner};
use rstest_bdd_harness_bevy::{BevyHarness, BevyScenario, MinimalBevyProfile, minimal_app};

/// Counts `Update` ticks observed by the toy system.
#[derive(Resource, Default)]
struct ToyCounter(u32);

/// Increments the toy counter once per `Update` schedule run.
fn increment_toy_counter(mut counter: ResMut<ToyCounter>) { counter.0 += 1; }

/// One advancing operation a scenario can perform.
#[derive(Clone, Copy, Debug)]
enum Op {
    /// Advance exactly one frame.
    Update,
    /// Advance `count` frames.
    UpdateTimes(u32),
}

impl Op {
    /// Returns the number of frames the operation advances.
    const fn frames(self) -> u32 {
        match self {
            Self::Update => 1,
            Self::UpdateTimes(count) => count,
        }
    }
}

/// One observation after an operation: both counters must agree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Observation {
    frame_count: u32,
    toy_count: u32,
}

/// Generates operation sequences by construction, without filtering.
fn op_sequences() -> impl Strategy<Value = Vec<Op>> {
    vec(
        prop_oneof![Just(Op::Update), (0_u32..=8).prop_map(Op::UpdateTimes)],
        0..=16,
    )
}

/// Applies one operation to the scenario application.
fn apply(scenario: &BevyScenario, op: Op) {
    match op {
        Op::Update => scenario.update(),
        Op::UpdateTimes(count) => scenario.update_times(count),
    }
}

/// Reads both counters through the public scenario accessor.
fn observe(scenario: &BevyScenario) -> Observation {
    scenario.with_world(|world| {
        let Some(frame_count) = world.get_resource::<FrameCount>() else {
            panic!("the minimal profile must provide a frame counter");
        };
        let Some(toy) = world.get_resource::<ToyCounter>() else {
            panic!("the scenario must install the toy counter");
        };
        Observation {
            frame_count: frame_count.0,
            toy_count: toy.0,
        }
    })
}

/// Runs the operations through the real harness and returns one observation
/// per operation.
fn run_sequence(ops: &[Op]) -> Vec<Observation> {
    let request = ScenarioRunRequest::new(
        ScenarioMetadata::new(
            "tests/features/bevy_scenario.feature",
            "tick additivity",
            1,
            Vec::new(),
        ),
        ScenarioRunner::new(|scenario: BevyScenario| {
            scenario.with_app(|app| {
                app.init_resource::<ToyCounter>();
                app.add_systems(Update, increment_toy_counter);
            });
            ops.iter()
                .map(|op| {
                    apply(&scenario, *op);
                    observe(&scenario)
                })
                .collect::<Vec<_>>()
        }),
    );
    match BevyHarness::<MinimalBevyProfile>::new().run(request) {
        Ok(observations) => observations,
        Err(error) => panic!("the minimal harness failed to run the sequence: {error}"),
    }
}

/// Computes the observation each operation must produce.
fn expected_observations(ops: &[Op]) -> Vec<Observation> {
    ops.iter()
        .scan(0_u32, |running, op| {
            *running += op.frames();
            Some(Observation {
                frame_count: *running,
                toy_count: *running,
            })
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn frame_count_tracks_update_calls(ticks in 0_u32..=32) {
        let mut app = minimal_app();
        (0..ticks).for_each(|_| app.update());
        prop_assert_eq!(
            app.world().resource::<FrameCount>().0,
            ticks,
            "the frame count must equal the number of update calls"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn counters_track_a_sequence_of_advances(ops in op_sequences()) {
        prop_assert_eq!(
            run_sequence(&ops),
            expected_observations(&ops),
            "each observation must equal the frames advanced so far"
        );
    }
}

#[rstest]
#[case::empty_sequence(vec![])]
#[case::lone_zero_advance(vec![Op::UpdateTimes(0)])]
#[case::zero_advance_first(vec![Op::UpdateTimes(0), Op::Update])]
fn tick_additivity_holds_for_named_sequences(#[case] ops: Vec<Op>) {
    assert_eq!(
        run_sequence(&ops),
        expected_observations(&ops),
        "the named sequence {ops:?} must advance exactly its requested frames"
    );
}

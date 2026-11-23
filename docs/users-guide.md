# Skyjoust user's guide

This guide is for operators and integrators who run the Skyjoust validator
tools against the high-level interaction contract.

On Linux, install the `mold` linker before running any `cargo` command in this
guide: the repository's Cargo configuration links Linux builds with it, so a
build without `mold` fails at the link step.

## 1. Validator contract with `cargo test`

The validator contract runs the Rust unit tests, integration tests, trace
replay tests, and the bounded Stateright depth-first search (DFS) property
check. It should run before changing the state graph, action set, trace format,
or scoring rules.

<!-- tested-example: validator-contract-command -->

```sh
cargo test -p skyjoust-stateright-validator
```

Success means the explored states satisfied the `always` invariants and reached
the expected `sometimes` examples. A failure usually points to one of three
causes:

- an action became legal from the wrong state,
- a transition changed score, rewards, or Warfront data too early,
- a reachability path disappeared after a guard change.

Worked example:

<!-- tested-example: validator-contract-output -->

```plaintext
running 1 test
test exhaustive_high_level_interaction_contract ... ok
```

In continuous integration (CI), a failed contract run should be read as a
behavioural regression unless the graph specification and validator contract
were intentionally changed together.

## 2. Stateright Explorer

The Explorer starts an interactive browser view of the bounded interaction
model. It shows reachable states, enabled actions, invariant failures, and
counterexample paths.

<!-- tested-example: stateright-explorer-command -->

```sh
cargo run -p skyjoust-stateright-validator --example serve_explorer
```

Open `http://localhost:3000/` after the process reports that the Explorer is
serving. The state graph view is useful for inspecting why a transition is
reachable. Counterexample paths list the exact action sequence that violates a
property.

Worked example:

<!-- tested-example: stateright-explorer-output -->

```plaintext
attempting to serve Stateright Explorer at localhost:3000
Stateright Explorer is serving Project Skyjoust at http://localhost:3000/
```

The Explorer is a developer-facing diagnostic surface. It does not replace the
automated `cargo test` contract check.

## 3. Trace validation

The `validate_trace` binary replays a concrete JSON action array against the
same guards and invariants as the model checker. Standard input carries the
trace, standard output carries the JSON result, and standard error carries
optional diagnostics.

<!-- tested-example: trace-validation-command -->

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace \
  < crates/skyjoust_stateright_validator/traces/keep_breach_reward_commit.json
```

The output has three top-level fields:

| Field         | Meaning                                                                |
| ------------- | ---------------------------------------------------------------------- |
| `ok`          | `true` when every action was legal and every invariant held.           |
| `final_state` | The last accepted state, or the state at the failing step.             |
| `failure`     | `null` on success, or `step_index`, `action`, and `reason` on failure. |

Exit code `0` means the trace validated. Exit code `2` means the trace was
well-formed JSON but failed the Skyjoust interaction contract.

Worked example:

<!-- tested-example: trace-validation-pipe-command -->

```sh
printf '%s\n' '["AssetsLoaded","StartSkirmish","CommitRewards"]' \
  | cargo run -p skyjoust-stateright-validator --bin validate_trace
```

Expected result:

<!-- tested-example: trace-validation-result -->

```json
{
  "ok": false,
  "final_state": {
    "depth": 2,
    "app": "SkirmishSetup",
    "warfront": "Inactive",
    "match_phase": "Inactive",
    "ceremony": "Dormant",
    "rules": {
      "ordnance": "Full",
      "friendly_fire": true,
      "duel_lock": false,
      "scoring_frozen": false,
      "joust_only": false,
      "allow_sudden_death": true
    },
    "player_ordnance": "Ready",
    "lance": "Idle",
    "recovery": "Alive",
    "objectives": {
      "keep_breached": false,
      "outpost_controlled": false,
      "shrine_claimed": false,
      "supply_route_blocked": false,
      "hostage_delivered": false
    },
    "score": {
      "open": false,
      "finalized": false,
      "pending_delta": false,
      "events_accepted": 0,
      "red_score": 0,
      "blue_score": 0,
      "red_glory": 0,
      "blue_glory": 0,
      "red_morale": 10,
      "blue_morale": 10,
      "victory_pending": false
    },
    "rewards": {
      "phase": "Dormant",
      "pending_delta": false,
      "committed": false,
      "glory": 0,
      "coin": 0,
      "influence": 0,
      "laurels": 0,
      "penalties": 0,
      "tournament_bonus_granted": false,
      "duel_bonus_granted": false
    },
    "winner": "None",
    "truce_active": false,
    "truce_broken": false,
    "tournament_rounds_won": 0,
    "tournament_completed": false,
    "duel_resolved": false,
    "duel_consequence_active": false,
    "treaty_signed": false,
    "infamy": 0,
    "post_final_score_write": false,
    "warfront_mutated_during_match": false
  },
  "failure": {
    "step_index": 2,
    "action": "CommitRewards",
    "reason": "action was not legal from the current state"
  }
}
```

Longer traces can raise the replay depth bound:

<!-- tested-example: trace-validation-max-depth-command -->

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace -- --max-depth 64 < trace.json
```

Verbose diagnostics print each replayed action and the final summary to
standard error:

<!-- tested-example: trace-validation-verbose-command -->

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace -- --verbose < trace.json
```

## 4. Use the headless Bevy test helpers

`rstest-bdd-harness-bevy` provides a `rstest-bdd` harness for tests that need a
headless Bevy application. The crate is currently workspace-only
(`publish = false`), so tests in this workspace's root package use a path
development dependency. Add a direct `rstest-bdd-harness` dependency as well:
the scenario macro uses the base harness API path for non-first-party
harnesses, and steps need its `StepResult` type.

<!-- tested-example: bevy-harness-dev-dependencies -->

```toml
[dev-dependencies]
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
rstest-bdd-harness = "0.6.0"
rstest-bdd-harness-bevy = { path = "crates/rstest-bdd-harness-bevy" }
```

The crate supports Bevy `0.20.1`. Select the default minimal profile with
`harness = BevyHarness`, or a downstream profile through a type alias such as
`type GameHarness = BevyHarness<GameProfile>;`. A scenario step requests the
shared application through the reserved harness-context fixture. The consuming
package supplies the feature file: create
`tests/features/bevy_scenario.feature` with the scenario this example binds
(the repository's copy is
`crates/rstest-bdd-harness-bevy/tests/features/bevy_scenario.feature`), where
`index = 1` selects its second scenario, "Steps share one application across
ticks".

<!-- tested-example: bevy-harness-scenario -->

```rust
use rstest_bdd::StepResult;
use rstest_bdd_harness_bevy::{
    BevyHarness, BevyScenario,
    bevy::{
        diagnostic::FrameCount,
        ecs::{resource::Resource, system::ResMut},
        prelude::Update,
        time::TimePlugin,
    },
};
use rstest_bdd_macros::{given, scenario, then, when};

/// Counts the `Update` ticks observed by the toy system.
#[derive(Resource, Default)]
struct ToyCounter(u32);

/// Increments the toy counter once per `Update` schedule run.
fn increment_toy_counter(mut counter: ResMut<ToyCounter>) { counter.0 += 1; }

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 1,
    harness = BevyHarness,
    attributes = rstest_bdd_harness_bevy::BevyAttributePolicy
)]
fn steps_share_one_application_across_ticks() {}

#[given("a scenario driven by the minimal Bevy harness")]
fn given_minimal_harness(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) -> StepResult<(), String> {
    if scenario.with_app(|app| app.is_plugin_added::<TimePlugin>()) {
        Ok(())
    } else {
        Err("the minimal profile must add the time plugin".to_owned())
    }
}

#[given("a toy counter incremented on every Update tick")]
fn given_toy_counter(#[from(rstest_bdd_harness_context)] scenario: &BevyScenario) {
    scenario.with_app(|app| {
        app.init_resource::<ToyCounter>();
        app.add_systems(Update, increment_toy_counter);
    });
}

#[when("the schedule advances {count:u32} times")]
fn when_schedule_advances(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
    count: u32,
) -> StepResult<(), String> {
    scenario.update_times(count);
    Ok(())
}

#[then("the toy counter reads {expected:u32}")]
fn then_toy_counter_reads(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
    expected: u32,
) -> StepResult<(), String> {
    match scenario.with_world(|world| world.get_resource::<ToyCounter>().map(|c| c.0)) {
        Some(observed) if observed == expected => Ok(()),
        Some(observed) => Err(format!(
            "the toy counter must read {expected}, but it read {observed}"
        )),
        None => Err("the scenario has no toy counter".to_owned()),
    }
}

#[then("the frame count reads {expected:u32}")]
fn then_frame_count_reads(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
    expected: u32,
) -> StepResult<(), String> {
    let observed = scenario.with_world(|world| world.resource::<FrameCount>().0);
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "the frame count must read {expected}, but it read {observed}"
        ))
    }
}

```

The example defines every step its selected scenario uses, so a package that
copies it together with the feature file gets a scenario that runs. Rustdoc
compiles this fence from the published text, and the repository's behavioural
suite runs the same scenario against the same feature, so the example cannot
drift from working behaviour.

The harness does not advance the app by itself: call `scenario.update()` or
`scenario.update_times(count)` explicitly. A count of zero still checks that
the application can advance. Fallible steps return `StepResult` and let the
harness report the scenario diagnostic; they do not panic or call `.expect()`.
Profile hooks return `HarnessResult`, and a hook that constructs an error uses
the idiom `HarnessError::RuntimeBuildFailed(std::io::Error::other(message))`.

`BevyScenario` clones share one application. A clone must not outlive the
scenario that produced it, and steps should request `&BevyScenario`. A profile's
`cleanup` hook releases scenario resources and must not advance the
application. The harness never calls `App::run`, `App::finish`, or
`App::cleanup`, so `add_plugins` remains legal inside steps; a profile that
finalizes the app itself gives up that ability. For deterministic time, install
`TimeUpdateStrategy::FixedTimesteps(n)` in the profile's `configure` hook.

`add_minimal_plugins` adds Bevy's minimal plugin set to an existing `App`, and
`minimal_app` creates a fresh application with that set already installed. Both
avoid window, renderer, asset, and audio plugins and remain available for tests
that do not need a scenario. A test body reads:

<!-- tested-example: minimal-headless-app -->

```rust
use rstest_bdd_harness_bevy::{bevy::diagnostic::FrameCount, minimal_app};

let mut app = minimal_app();
app.update();
assert_eq!(
    app.world().resource::<FrameCount>().0,
    1,
    "one update advances exactly one frame"
);
```

For the crate's full test setup and extension guidance, see the
[crate README](../crates/rstest-bdd-harness-bevy/README.md) and
[harness design](rstest-bdd-harness-bevy-design.md). The
[Skyjoust adoption note](rstest-bdd-harness-bevy-adoption.md) explains the
workspace-only test dependency and migration boundary. The
[`rstest-bdd` v0.6.0 user's guide](rstest-bdd-v0.6.0-users-guide.md) covers the
framework's usage; the
[v0.6.0 migration guide](rstest-bdd-v0.6.0-migration-guide.md) records changes
from v0.5.0. They were imported from the upstream
[user's guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/users-guide.md)
and
[migration guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/v0-6-0-migration-guide.md).

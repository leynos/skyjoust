# Adopt the Bevy BDD helpers in Skyjoust

This note covers the in-tree helper scaffold for Skyjoust tests. It is not a
published dependency or a runtime requirement.

## Add the test dependency

For BDD tests in the root `skyjoust` package, keep the harness and its test
framework dependencies in `[dev-dependencies]`:

```toml
[dev-dependencies]
rstest = "0.28"
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
rstest-bdd-harness = "0.6.0"
rstest-bdd-harness-bevy = { path = "crates/rstest-bdd-harness-bevy" }
```

Use a path relative to the consuming package for other workspace members. Do
not add the harness to `[dependencies]`: it is workspace-only and declares
`publish = false`. External consumers should wait until a separately approved
extraction and release.

## Import the headless helpers

The crate supports Bevy `0.20.1`. Bevy appears in its public API, so keep
consumers on that workspace-compatible line. The crate disables Bevy's default
features and enables `std`; its minimal profile avoids window, renderer, asset,
and audio plugins.

Use `minimal_app` for a fresh application, or `add_minimal_plugins` when a test
already owns an `App`. The test controls schedule advancement with explicit
`app.update()` calls:

```rust
use rstest_bdd_harness_bevy::{
    add_minimal_plugins,
    bevy::{app::App, diagnostic::FrameCount},
    minimal_app,
};

let mut configured = App::new();
add_minimal_plugins(&mut configured);
configured.update();

let mut scenario_app = minimal_app();
scenario_app.update();
assert_eq!(scenario_app.world().resource::<FrameCount>().0, 1);
```

Import Bevy types through the crate's `bevy` re-export when that helps keep
tests on the harness's compatibility line. Use scenario and step attributes from
`rstest-bdd-macros` and `StepResult` from `rstest-bdd`; the imported
[v0.6.0 user's guide](rstest-bdd-v0.6.0-users-guide.md) covers their usage.

## Drive a scenario through the harness

Roadmap task `0.1.2` adds the `rstest-bdd` harness. Select the default minimal
profile with `harness = BevyHarness`, or a downstream profile through a type
alias, and request the shared application with the reserved
`rstest_bdd_harness_context` fixture:

```rust
use rstest_bdd_harness_bevy::{BevyHarness, BevyScenario, BevyProfile};
use rstest_bdd_macros::scenario;

pub enum SkyjoustProfile {}

impl BevyProfile for SkyjoustProfile {
    fn configure(
        app: &mut bevy::app::App,
        _metadata: &rstest_bdd_harness::ScenarioMetadata,
    ) -> rstest_bdd_harness::HarnessResult<()> {
        app.add_plugins(bevy::prelude::MinimalPlugins);
        Ok(())
    }
}

pub type SkyjoustHarness = BevyHarness<SkyjoustProfile>;

#[scenario(
    path = "tests/features/bevy_scenario.feature",
    index = 1,
    harness = SkyjoustHarness,
    attributes = rstest_bdd_harness_bevy::BevyAttributePolicy
)]
fn steps_share_one_application_across_ticks() {}

#[when("the schedule advances once")]
fn when_schedule_advances(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
) {
    scenario.update();
}
```

The harness never advances the app on its own. Steps call `scenario.update()` or
`scenario.update_times(count)` explicitly. Game-specific plugins belong in the
downstream profile's `configure` hook, and its `cleanup` hook must release
scenario resources without advancing the application. See the
[user's guide](users-guide.md#4-use-the-headless-bevy-test-helpers) for the
full rules.

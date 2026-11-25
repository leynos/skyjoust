# `rstest-bdd-harness-bevy`

A reusable Bevy harness adapter for `rstest-bdd` behavioural tests. This crate
incubates in the Skyjoust workspace but depends on no Skyjoust or Lille code,
so it can move to its own repository as a directory move plus a dependency
rewire. Game-specific setup belongs in the downstream profile types defined by
each consumer.

Roadmap task `0.1.2` delivers the public harness API: `BevyScenario`,
`BevyProfile`, `BevyHarness`, `BareBevyProfile`, `MinimalBevyProfile`, and
`BevyAttributePolicy`. A generated scenario selects the harness with
`harness = BevyHarness`, or a type alias when it selects a downstream profile,
and its steps receive the shared application through the reserved
`rstest_bdd_harness_context` fixture. See
[the harness design](../../docs/rstest-bdd-harness-bevy-design.md), [ADR 009](../../docs/adrs/adr-0009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md)
for the extraction boundary, and
[ADR 010](../../docs/adrs/adr-0010-bevy-harness-failure-contract.md) for the
failure contract.

## Run the tests

```bash
cargo test -p rstest-bdd-harness-bevy
```

The suite has these layers:

- library unit tests inside `src/` for the scenario context, harness lifecycle,
  panic-message formatter, attribute policy, and profile functions (44 tests);
- behavioural scenarios in `tests/headless_scenario.rs` bound through the real
  `#[scenario]` macro surface, covering metadata retention, state shared across
  explicit advances, zero advances, the bare profile, and the augmented failure
  diagnostic (6 tests);
- a 64-case property over sequences of `update` and `update_times` calls, plus
  named boundary cases for empty and zero-count sequences, in
  `tests/tick_properties.rs` (5 tests);
- the extraction-contract manifest tripwire in `tests/extraction_boundary.rs`
  (4 tests);
- `trybuild` compile-pass and compile-fail fixtures in `tests/macro_ui.rs`
  (1 test that drives the fixture binaries).

The crate also runs 16 doctests. All seven local commit gates passed on the
`0.1.2` milestone tree: `check-fmt`, `lint`, `typecheck`, `test`, `spelling`,
`markdownlint`, and `nixie`.

Fixture lint handling is provided by the nested, test-only workspace member
`rstest-bdd-harness-bevy-test-macros` package. It owns the
`allow_fixture_expansion_lints` attribute used for `rstest` fixture expansion;
it is not part of this library's runtime API and moves with the harness source.

The crate uses the published stable `rstest-bdd` 0.6.0 family with implicit
caret requirements. Review the
[v0.6.0 user's guide](../../docs/rstest-bdd-v0.6.0-users-guide.md) for
framework usage and the
[migration guide](../../docs/rstest-bdd-v0.6.0-migration-guide.md) for changes
since v0.5.0. Their source is the upstream
[user's guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/users-guide.md)
and
[migration guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/v0-6-0-migration-guide.md).

## Files

| Path                                   | Responsibility                                                                                                   |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `src/context.rs`                       | `BevyScenario`: the shared `App` handle, metadata, closure-based access, and explicit advancement.               |
| `src/harness.rs`                       | `BevyHarness`: the lifecycle, phase-tagged error wrapping, and panic re-raise fixed by ADR 010.                  |
| `src/panic.rs`                         | The pure panic-message formatter and payload classification.                                                     |
| `src/policy.rs`                        | `BevyAttributePolicy`, the forward-compatible marker policy.                                                     |
| `src/profile.rs`                       | `BevyProfile`, `BareBevyProfile`, `MinimalBevyProfile`, `add_minimal_plugins`, and `minimal_app`.                |
| `src/*_tests.rs`, `src/harness_tests/` | Sibling unit tests, including the recording subscriber and tripwire profiles.                                    |
| `src/snapshots/`                       | `insta` snapshots of the phase-tagged message format.                                                            |
| `tests/headless_scenario.rs`           | The `#[scenario]` bindings, including the type alias selecting the bare profile and the `should_panic` scenario. |
| `tests/headless_scenario/steps.rs`     | Step definitions; reached through `#[path]` from the integration root.                                           |
| `tests/features/`                      | The `headless_scenario`, `bevy_scenario`, and `bevy_panic` Gherkin specifications.                               |
| `tests/tick_properties.rs`             | Frame-count and sequence-additivity properties.                                                                  |
| `tests/extraction_boundary.rs`         | The manifest tripwire that rejects game crates.                                                                  |
| `tests/macro_ui.rs`, `tests/ui/`       | Compile-pass and compile-fail macro contract fixtures.                                                           |
| `test-macros/`                         | The private proc-macro package used to scope fixture-expansion lint handling to this crate's tests.              |

*Table 1: Harness crate layout.*

The unit tests sit beside their modules inside `src/`, following the
repository's sibling-test idiom; they ship with the crate's source archive when
the crate is published after extraction.

## Extending

The profile type is the single extension seam. Every game-specific plugin,
resource, and cleanup hook lives in a downstream implementation of
`BevyProfile`, never in this crate:

```rust
use rstest_bdd_harness::{HarnessResult, ScenarioMetadata};
use rstest_bdd_harness_bevy::{
    BevyHarness, BevyProfile,
    bevy::{app::App, prelude::MinimalPlugins},
};

pub enum GameProfile {}

impl BevyProfile for GameProfile {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        Ok(())
    }
}

pub type GameHarness = BevyHarness<GameProfile>;
```

A profile's `cleanup` runs after a successful scenario and during unwinding
after a step panic; it must release scenario resources without advancing the
application. The harness never calls `App::run`, `App::finish`, or
`App::cleanup`, so `add_plugins` remains legal inside steps. A profile that
calls `finish` or `cleanup` itself gives up that ability. Deterministic time is
a downstream technique: a profile may install
`TimeUpdateStrategy::FixedTimesteps(n)` before the runner starts.

`BevyScenario` clones share one application. A clone must not outlive the
scenario that produced it, and step functions should request `&BevyScenario`.
Fallible steps return `StepResult` rather than panicking so the harness can
report the scenario diagnostic.

## Bevy compatibility

Bevy types appear in this crate's public signatures — `minimal_app` returns
`bevy::app::App` — so a Bevy minor bump is a breaking change here. The
workspace resolves one Bevy line across the harness-facing graph; changing it
is an ADR-level decision.

| Bevy release | Status                                                                            |
| ------------ | --------------------------------------------------------------------------------- |
| `0.20.1`     | Current compatibility line; `default-features = false` with `features = ["std"]`. |
| `0.19.1`     | Superseded by the `0.20.1` line after the workspace upgrade.                      |
| `0.17.3`     | Historical probe baseline only.                                                   |

*Table 2: Bevy compatibility of the harness crate.*

The crate declares no Bevy renderer, window, asset, or audio feature; the
resolved graph in a workspace build is a workspace-wide property, because Cargo
unifies features across members built in one invocation.

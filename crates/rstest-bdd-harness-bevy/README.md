# `rstest-bdd-harness-bevy`

A reusable Bevy harness adapter for `rstest-bdd` behavioural tests. This crate
incubates in the Skyjoust workspace but depends on no Skyjoust or Lille code,
so it can move to its own repository as a directory move plus a dependency
rewire. Game-specific setup belongs in the downstream profile types defined by
each consumer.

This milestone ships the scaffold and its profile functions: a headless Bevy
`0.19.1` application driven from a Gherkin scenario through the ordinary
`cargo test` harness. The harness types (`BevyScenario`, `BevyProfile`,
`BevyHarness`, and their companions) belong to roadmap task `0.5.1.2`. See
[the harness design](../../docs/rstest-bdd-harness-bevy-design.md) and
[ADR 007](../../docs/adr/007-in-tree-incubation-of-the-bevy-bdd-harness-crate.md).

## Run the tests

```bash
cargo test -p rstest-bdd-harness-bevy
```

Thirteen tests pass: five unit tests for the profile functions, a Gherkin
scenario that advances one headless update tick, a 32-case property test over
tick counts, four extraction-contract tripwire tests, and two doctests. The
behavioural scenario appears as `minimal_app_advances_one_tick`.

## Files

- `src/profile.rs` — `add_minimal_plugins` and `minimal_app`, the two public
  functions of this milestone.
- `tests/headless_scenario.rs` — the `rstest-bdd` binding with the
  `include_str!` feature-file rebuild guard.
- `tests/features/headless_scenario.feature` — the Gherkin specification.
- `tests/tick_properties.rs` — the frame-count tick invariant over `0..=32`.
- `tests/extraction_boundary.rs` — the manifest tripwire that rejects game
  crates.

The unit tests in `src/profile_tests.rs` sit beside their module inside `src/`,
following the repository's sibling-test idiom; they ship with the crate's
source archive when the crate is published after extraction.

## Extending

Task `0.5.1.2` adds `BevyScenario`, `BevyProfile`, `BevyHarness`,
`BareBevyProfile`, `MinimalBevyProfile`, and `BevyAttributePolicy`. The profile
type is the single extension seam: every game-specific plugin, resource, and
cleanup hook lives in a downstream implementation of it, never in this crate.

## Bevy compatibility

Bevy types appear in this crate's public signatures — `minimal_app` returns
`bevy::app::App` — so a Bevy minor bump is a breaking change here. The
workspace resolves one Bevy line across the harness-facing graph; changing it
is an ADR-level decision.

| Bevy release | Status                                                                            |
| ------------ | --------------------------------------------------------------------------------- |
| `0.19.1`     | Current compatibility line; `default-features = false` with `features = ["std"]`. |
| `0.17.3`     | Historical probe baseline only; superseded by the `0.19.1` release.               |

*Table 1: Bevy compatibility of the harness crate.*

The crate declares no Bevy renderer, window, asset, or audio feature; the
resolved graph in a workspace build is a workspace-wide property, because Cargo
unifies features across members built in one invocation.

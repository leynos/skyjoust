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
[ADR 009](../../docs/adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md).

## Run the tests

```bash
cargo test -p rstest-bdd-harness-bevy
```

The suite covers five unit cases for the profile functions, a Gherkin scenario
that advances one headless update tick, a 32-case property test over tick
counts, four extraction-contract tripwire tests, and two doctests. The
behavioural scenario appears as `minimal_app_advances_one_tick`. The broader
crate suite passed against stable `rstest-bdd` 0.6.0. The six local gates and
acceptance checks passed on the implementation source tree; later changes were
documentation-only. Final acceptance requires hosted CI, including a cold
coverage run, and review.

Fixture lint handling is provided by the nested, test-only workspace member
`rstest-bdd-harness-bevy-test-macros` package. It owns the
`allow_fixture_expansion_lints` attribute used for `rstest` fixture expansion;
it is not part of this library's runtime API and moves with the harness source.
The `trybuild` pass fixture covers an `rstest` fixture and `rstest-bdd` step
macros; it does not expand a valid `#[scenario]` or exercise `_app` implicit
fixture normalization. The valid scenario binding remains compiled and
exercised by `tests/headless_scenario.rs`. The compile-fail fixture binds a
missing scenario feature file, testing the `rstest-bdd` contract rather than
Rust's generic `include_str!` diagnostic. The focused `trybuild` target passed
against stable 0.6.0 as part of the completed local validation. The stable
feature-file freshness A/B also passed: macro auto-tracking rebuilds after
feature-only edits with or without the explicit declaration. A hosted cold
coverage run is required to confirm the timeout mitigation.

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

- `src/profile.rs` — `add_minimal_plugins` and `minimal_app`, the two public
  functions of this milestone.
- `test-macros/` — the private proc-macro package used to scope fixture
  expansion lint handling to this crate's tests.
- `tests/headless_scenario.rs` — the `rstest-bdd` binding and explicit
  `include_str!` declaration. Stable 0.6.0 also tracks bound feature files as
  Cargo rebuild dependencies. The completed stable A/B showed that edits
  rebuild with or without the declaration; it remains because the task contract
  requires the documentary, belt-and-braces dependency.
- `tests/macro_ui.rs` and `tests/ui/` — compile-pass and compile-fail macro
  contract fixtures.
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

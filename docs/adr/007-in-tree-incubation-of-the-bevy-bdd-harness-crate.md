# 007: In-tree incubation of the Bevy BDD harness crate

Status: Accepted

Date: 2026-08-17

Accepted: 2026-08-17

## Context

Project Skyjoust is a Cargo workspace: the root `skyjoust` package, the
`skyjoust_stateright_validator` crate, and the `skyjoust_test_macros` crate
added by [ADR 006](006-test-macro-crate-for-fixture-expansion-lints.md).
Roadmap phase `0.5` needs behaviour-driven tests that drive a deterministic,
headless Bevy entity-component system (ECS) application from Gherkin scenarios
through the ordinary `cargo test` harness. The reusable vehicle is a new crate,
`rstest-bdd-harness-bevy`, that plugs a headless Bevy application into the
`rstest-bdd` harness contract.

Bevy types cross the harness boundary: downstream profile types configure the
same `bevy::app::App` the harness drives, and roadmap `1.2.2` brings Bevy into
the runtime. The harness crate therefore cannot make an independent Bevy
version choice. The version it builds against is a workspace-wide coupling.

The `rstest-bdd` version situation forces the dependency strategy. The
`0.6.0-beta3` release was published to crates.io on 2026-07-07 and carries the
harness API this work targets: `HarnessAdapter`, `ScenarioRunRequest`, and the
reserved `rstest_bdd_harness_context` fixture key. Upstream `main` has since
moved to an unpublished `0.6.0-beta4` that changes `StepContext::borrow_mut` to
take `&self`, adds a `policy_conformance` module and a `testing` feature, and
renames generated panic text from `initialise` to `initialize`. A caret
requirement on the pre-release line admits `beta4`, so the workspace must pin
exactly until a deliberate compatibility pass validates a later release.

The harness crate must stay free of Skyjoust and Lille code so it can later
move to its own repository. The roadmap names the extraction target as
`leynos/rstest-bdd-harness-bevy`.

## Decision

### Workspace membership

Add `crates/rstest-bdd-harness-bevy` as a workspace member, growing the
workspace to four members: the root package plus three crates. This does not
reopen [ADR 002](002-crate-layout-and-public-api.md)'s deferral of runtime
crate splits. That decision governs *runtime* functionality, which must stay as
modules inside the runtime crate; the harness is a tooling-facing test adapter,
in the same category as the exception ADR 002 already grants the validator
crate.

### The extension seam

State the crate boundary as an extension-seam rule rather than hexagonal
taxonomy: *the profile type is the single extension seam. Every game-specific
plugin, resource, and cleanup hook lives in a downstream implementation of it,
never in this crate.* The harness trait is an extension point owned by
`rstest-bdd`, not a port this crate defines against a domain it owns, so
ports-and-adapters language would strain the design. The durable consequence —
the crate holds no game domain logic — is what keeps game code out of the
extracted repository.

### Extraction contract and its true cost

The crate must never depend on a Skyjoust gameplay crate or any Lille crate,
directly or transitively. Test-only tooling is a separate category:
`skyjoust_test_macros` is a permitted development dependency, on the same
footing as `rstest`, and is deliberately excluded from the extracted
repository. The boundary guard matches dependency *names* exactly rather than
by substring, so the permitted `skyjoust-test-macros` near-match passes.

Extraction is more than a directory move plus a dependency rewire. It also
requires a *configuration transplant*, because workspace lint inheritance does
not survive the move. The extracted repository needs the `[workspace.lints]`
tables, `clippy.toml`, `.rustfmt.toml`, `rust-toolchain.toml`, and the Whitaker
Dylint wiring copied out before it can pass the same gates.

### Dependency strategy

- Depend on the published `rstest-bdd` family crates, not git dependencies
  against `main`. `0.6.0-beta3` is published, so the roadmap's conditional
  git-dependency instruction no longer applies.
- Pin every `rstest-bdd` family crate exactly to `=0.6.0-beta3`, a
  maintainer-approved temporary exception to the workspace's implicit-caret
  mandate. A caret on a pre-release is not a pin: it admits `0.6.0-beta4`,
  which upstream has made source-breaking, and Dependabot automerges daily here.
  `Cargo.lock` is load-bearing for this family until an explicitly validated
  `0.6.0` final returns the workspace to implicit caret syntax.
- Write ordinary stable requirements with implicit caret syntax, such as
  `bevy = { version = "0.19.1", default-features = false, features = ["std"] }`.
  The `std` feature is required: without it, `bevy_platform` selects a fallback
  `Instant` that passes `_rdtsc()` straight to `Duration::from_nanos`, so
  `Time<Real>` advances at roughly the timestamp counter frequency rather than
  wall-clock.
- Target Bevy `0.19.1` for both the harness and the future Skyjoust runtime.
  One Bevy line must resolve across the harness-facing graph (`EP-REQ-003`).
- Declare `tracing` directly and re-export it separately, rather than
  re-exporting it through `rstest-bdd-harness`, so this crate's public API does
  not expose a version it does not control.

### Bevy version ownership

The Bevy release is a workspace-wide coupling, not a local choice. The
maintainer owns the bump. A change is an ADR-level decision that requires a
compatibility pass across every consumer, because Bevy types appear in this
crate's public signatures and in downstream profile types.

### The headless guarantee

The crate declares no Bevy renderer, window, asset, or audio features —
`default-features = false` with only `features = ["std"]` enabled. State the
guarantee accurately: *this crate declares no such features; the resolved graph
in a workspace build is a workspace-wide property*, because Cargo unifies
features across members built in one invocation. `make test` and `make lint`
build `--workspace --all-features`. Any other member that later adds `bevy`
must also declare `default-features = false`, or the headless guarantee is lost
for everyone.

### Dependency-sourcing options

| Source                                                        | Resolution                         | Stability                        | Risk                                   | Outcome                                         |
| ------------------------------------------------------------- | ---------------------------------- | -------------------------------- | -------------------------------------- | ----------------------------------------------- |
| Git dependency on `rstest-bdd` `main`                         | Unpinned head, `0.6.0-beta4` API   | Drifts with upstream commits     | Source-breaking changes; vendored GPUI | Rejected: `beta3` is published                  |
| Caret `^0.6.0-beta3`                                          | Admits `beta4` and `0.6.0` final   | Depends on Dependabot behaviour  | Automerged breaking upgrade            | Rejected: a caret on a pre-release is not a pin |
| Exact `=0.6.0-beta3` (**chosen**)                             | Pinned                             | Reproducible `Cargo.lock`        | Requires a deliberate upgrade pass     | Chosen until an explicitly validated `0.6.0`    |
| Bevy `0.19.1` caret with `default-features = false` and `std` | One Bevy line across the workspace | Cargo lock keeps it reproducible | Workspace build cost                   | Chosen; the runtime couples to the same release |

*Table 1: Dependency-sourcing options for the `rstest-bdd` family and Bevy.*

## Consequences

The workspace carries a fourth member whose library depends on Bevy with
default features disabled and pulls the `rstest-bdd` harness contract into
every workspace build. `make test` and `make lint` each compile the added
`0.19.1` graph, and the CI caches grow correspondingly; the measured cost is
recorded in the ExecPlan's `Outcomes & retrospective`.

[ADR 002](002-crate-layout-and-public-api.md)'s Consequences told readers to
expect exactly two crates. That statement is superseded by this record, and a
forward pointer is added there so the two documents do not disagree.

The `rstest-bdd` family stays exactly pinned for now. A published `0.6.0` final
is a migration trigger, not proof of compatibility: validate the whole
behavioural suite against it before returning to implicit caret syntax. In the
meantime, a bare `cargo update` or Dependabot automerge resolves nothing for a
pinned pre-release, so `Cargo.lock` churn cannot admit the breaking `beta4`.

Roadmap task `0.5.1.1` delivers the crate scaffold and its first tests; the
harness types themselves (`BevyScenario`, `BevyHarness`, `BevyProfile`, and
their companions) belong to task `0.5.1.2`. Extraction to
`leynos/rstest-bdd-harness-bevy` stays a directory move plus a dependency
rewire, but only after the configuration transplant documented above, and only
after Skyjoust and Lille each carry one headless scenario through their normal
gates (roadmap `0.5.1.5`).

Downstream adoption: Skyjoust and Lille each define a profile type outside this
crate that adds `MinimalPlugins` and their own plugins. Game-specific setup
therefore never reaches the reusable crate, and the single Bevy release line is
the only coupling the extracted repository inherits.

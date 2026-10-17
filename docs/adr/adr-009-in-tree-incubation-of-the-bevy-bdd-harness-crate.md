# ADR 009: In-tree incubation of the Bevy BDD harness crate

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

The initial implementation targeted the published `0.6.0-beta3` API and used
exact prerelease pins because later beta versions could break compatibility.
The published `0.6.0` final is now the target. Its migration guide records the
custom harness return contract (`HarnessResult<T>`), fallible fixture and step
behaviour, a Rust 1.88 minimum, and automatic Cargo rebuild tracking for bound
feature files. The source and test suite must complete the compatibility pass
and acceptance gates against this final before the implementation is reported
accepted; the guide import alone is not validation.

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

The crate and its standalone test graph must not depend on any Skyjoust or
Lille package, including test-only tooling. Fixture lint handling is owned
inside the harness crate's extraction boundary by the nested workspace member
`rstest-bdd-harness-bevy-test-macros`; it must not use the sibling
`skyjoust-test-macros` path dependency. The manifest guard checks normal and
development dependencies and retains a synthetic `skyjoust-test-macros`
near-match canary to ensure matching remains exact. That canary does not
declare the sibling as a dependency of the harness crate.

Compile-time coverage for the `rstest-bdd` macros used by the integration test
is part of this crate's acceptance scope: include representative compile-pass
and compile-fail UI fixtures with checked diagnostics. This is distinct from a
trybuild test for Rust's built-in `include_str!` missing-file error, which is
not a crate-defined macro contract and is not required.

Extraction is more than a directory move plus a dependency rewire. It also
requires a *configuration transplant*, because workspace lint inheritance does
not survive the move. The extracted repository needs the `[workspace.lints]`
tables, `clippy.toml`, `.rustfmt.toml`, `rust-toolchain.toml`, and the Whitaker
Dylint wiring copied out before it can pass the same gates.

### Dependency strategy

- Depend on the published stable `rstest-bdd` 0.6.0 family, not git dependencies
  against `main`.
- Use implicit-caret requirements such as `"0.6.0"` for the stable family,
  returning to the workspace's ordinary versioning policy. The former exact
  `=0.6.0-beta3` pins were a temporary prerelease safeguard; they are retained
  below as historical rationale, not the current decision. Compatibility and
  acceptance evidence for the stable release remains pending.
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

| Source / requirement                                          | Resolution                         | Stability                        | Risk                                   | Outcome                                                  |
| ------------------------------------------------------------- | ---------------------------------- | -------------------------------- | -------------------------------------- | -------------------------------------------------------- |
| Git dependency on `rstest-bdd` `main`                         | Unpinned head                      | Drifts with upstream commits     | Unreviewed API changes                 | Rejected: stable 0.6.0 is published                      |
| Exact `=0.6.0-beta3` (historical)                             | Pinned prerelease                  | Reproducible lockfile            | Stays on prerelease API                | Superseded by 2026-10-12 decision; compatibility pending |
| Implicit caret `0.6.0` (**chosen**)                           | Stable 0.6.x compatible releases   | Workspace versioning convention  | API changes need review and validation | Selected; acceptance gates remain pending                |
| Bevy `0.19.1` caret with `default-features = false` and `std` | One Bevy line across the workspace | Cargo lock keeps it reproducible | Workspace build cost                   | Chosen; runtime couples to the same release              |

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

The `rstest-bdd` family now targets stable `0.6.0` with implicit-caret
requirements. The lockfile records the resolved versions but does not replace
the compatibility pass or the pending gates. The imported
[v0.6.0 user's guide](../rstest-bdd-v0.6.0-users-guide.md) describes the
published API, and the
[migration guide](../rstest-bdd-v0.6.0-migration-guide.md) records changes
since v0.5.0. Their exact upstream sources are the
[user's guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/users-guide.md)
and
[migration guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/v0-6-0-migration-guide.md).

Roadmap task `0.1.1` delivers the crate scaffold and its first tests; the
harness types themselves (`BevyScenario`, `BevyHarness`, `BevyProfile`, and
their companions) belong to task `0.1.2`. Extraction to
`leynos/rstest-bdd-harness-bevy` stays a directory move plus a dependency
rewire, but only after the configuration transplant documented above, and only
after Skyjoust and Lille each carry one headless scenario through their normal
gates (roadmap `0.1.5`).

Downstream adoption: Skyjoust and Lille each define a profile type outside this
crate that adds `MinimalPlugins` and their own plugins. Game-specific setup
therefore never reaches the reusable crate, and the single Bevy release line is
the only coupling the extracted repository inherits.

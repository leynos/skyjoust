# Skyjoust developer's guide

This guide is for maintainers changing the Skyjoust runtime contract,
Stateright validator, trace tools, and accompanying specifications.

## 1. Normative references

The validator must stay synchronized with these source documents:

- [Product requirements](skyjoust-product-requirements.md)
- [Technical design](skyjoust-technical-design.md)
- [State graph specification](skyjoust-state-graphs.yaml)
- [Validator contract](../crates/skyjoust_stateright_validator/spec/validator_contract.yaml)
- [Architecture decision records](adr/)

## 2. Runtime crate and module boundary

[ADR 002](adr/002-crate-layout-and-public-api.md) is the source of truth for
the workspace shape. The accepted decision is one runtime crate with strict
internal modules, beside the separate `skyjoust_stateright_validator` crate;
[ADR 009](adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md) adds
a third member crate, `rstest-bdd-harness-bevy`, as tooling in the same
category as the validator exception. Maintainers should add new runtime
functionality as a module inside the runtime crate, not as a new crate.

The runtime modules and their responsibilities follow the technical design's
runtime ownership table: `game_app`, `core`, `sim`, `terrain`, `stategraphs`,
`render`, `audio`, `ui`, and `assets`. The dependency direction is one-way:

```plaintext
game_app -> subsystem modules -> core
```

Lower-level modules must not call higher-level orchestration or adapters.
Domain code must not depend on adapters, renderer code, audio backends, process
setup, or window lifecycle glue.

A module may be promoted to its own crate only when at least one of these
conditions holds:

- an API is reused across the runtime, developer tooling, or the validator
  crate;
- a boundary is stable enough to test and release independently of the
  runtime crate.

Record any such extraction in a follow-up ADR before changing `Cargo.toml`
workspace members.

### 2.1. The `rstest-bdd-harness-bevy` crate

The harness crate plugs a headless Bevy application into the `rstest-bdd`
harness contract. Its boundary rule is stated in
[ADR 009](adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md) as
an extension seam: *the profile type is the single extension seam. Every
game-specific plugin, resource, and cleanup hook lives in a downstream
implementation of it, never in this crate.* Skyjoust and Lille each define a
profile type outside the crate once the first deterministic runtime resources
exist (roadmap `0.1.4`); the crate itself never acquires game modules.

The crate incubates here but keeps its extraction contract: no Skyjoust or
Lille dependency, directly or transitively. `tests/extraction_boundary.rs`
trips on a directly declared game crate, and
`cargo tree -p rstest-bdd-harness-bevy -e normal,dev` remains the authority for
the transitive half.

Writing behavioural scenarios taught this task two estate-specific traps and
one Bevy trap.

- The `allow-expect-in-tests` setting covers `#[test]` functions only.
  `rstest-bdd` step functions are macro-registered free functions, so
  `.expect()` inside a step fails `make lint`. Fallible steps return
  `StepResult` and propagate `RefCell` borrow failures with `try_borrow` and
  `try_borrow_mut`, mapping errors instead of panicking. See
  `tests/headless_scenario.rs` for the pattern.
- The integration test retains an explicit `include_str!` binding for its
  feature file. Stable `rstest-bdd` 0.6.0 also tracks bound feature files as
  Cargo rebuild dependencies. The stable freshness A/B confirmed that a
  feature-only edit triggers recompilation and the expected `Step not found`
  failure both with and without the declaration. Automatic macro tracking
  provides the rebuild dependency; the explicit declaration remains because the
  task contract requires it, as a documentary, belt-and-braces dependency. The
  earlier beta3 A/B result is historical.
- `googletest`'s `expect_that!` panics without a `#[gtest]` test context, so
  unit tests annotate `#[gtest]` above `#[rstest]`. `rstest-bdd` step functions
  have no such context and use `assert_that!`, which panics directly.
- Bevy's `MinimalPlugins` includes `ScheduleRunnerPlugin`, whose own `run`
  method loops forever. Scenarios must call `App::update` explicitly (via the
  fixture or a step) and must never let the schedule runner drive the loop.

The fixture-expansion lint trap is documented in §7.3; it applies to the
behavioural fixture exactly as the search-based tests in that section describe.

Compile-based UI test binaries run `trybuild`, which starts nested Cargo
compilations. For Nextest-based coverage, binaries named with the `_ui` suffix
use the dedicated `compile-ui` test group: that group runs one test at a time
and gives each test a finite 90-minute slow-time budget. Other tests retain the
default 180-second slow-time budget. The workspace-wide timeout is 190 minutes,
which contains both serialized UI runners, their five-second termination grace
periods, and an ordinary-test tail. The policy applies to development and
coverage runs that use Nextest; `cargo test` does not apply Nextest groups or
timeouts. The root `nextest_policy` suite guards the runner inventory, scoped
limits, and containment by the CI watchdogs.

`make test-prepare` runs `cargo fetch --locked` against the shared default
Cargo cache before timed tests start. `make test` depends on that preparation
and runs the test binaries and doctests offline, including nested trybuild
processes. All trybuild dependencies come from workspace dependencies. Both the
pull-request coverage job and main-branch coverage workflow prepare
dependencies before invoking coverage. Offline execution avoids network delays;
it does not eliminate Cargo's cache locks or compiler-admission waits. Cargo
parallelism remains under caller control.

Both CI coverage callers set a finite 13,200-second Cargo watchdog, allowing 30
minutes for cold compilation beyond the 190-minute Nextest run. Their
240-minute step watchdogs leave time for cleanup and reporting. The ordinary
test step has the same outer bound. On pull requests, the `coverage-ratchet`
job depends on `build-test`, so coverage runs after the Cranelift suite in a
separate job with its own explicit 360-minute timeout. This keeps the two
240-minute step budgets within separate six-hour job limits; setup, lint, and
reporting still use time within their respective job budgets. GitHub terminates
a hosted job at six hours
([Actions limits](https://docs.github.com/en/enterprise-cloud@latest/actions/reference/limits)).
These are conservative containment budgets following Cuprum's timeout
treatment,[^cargo-queue-budgets] not measured execution-time guarantees.
Earlier local runs reached the 600-second and 20-minute limits while nested
Cargo waited for package-cache access and compiled dependencies. A later run
passed under the 60-minute UI budget, but a cold run still timed out after 60
minutes: trybuild spent 49m21s compiling its first pass fixture while Cargo
reported cache-lock waits and compiler queues of up to 464. The 90-minute
per-test budget retains a finite bound while covering that measured cold
compile and additional admission waits.

The repository relies on the installed compiler wrappers for shared admission.
A healthy pool can have every slot occupied and hundreds of requests queued.
Start each gate once and let its compiler requests wait their turn; a busy pool
does not require a quiet-window preflight or a duplicate build. Keep the shared
default Cargo cache and leave Cargo parallelism under caller control. The
compile-test budget includes nested compiler-admission and cache-lock waits.
Separating allocation, preparation, and execution timing further would require
a supported upstream workload-admission contract, as proposed in Cuprum's
resource-coordination follow-up.[^build-resource-coordination]

Cargo can cache a failed compiler-version probe in the effective target
directory's `.rustc_info.json`. After admission recovers, an immediate retry
can replay the cached `rustc -vV` failure, including old admission-wait
messages. Inspect that generated file's `outputs` for an entry with
`success: false` before treating those messages as a fresh failure. Preserve
the failed metadata for diagnosis, remove only that generated file, and retry
the gate through the normal compiler wrapper. Keep the shared Cargo cache and
admission settings.

## 3. Validator module structure

The `skyjoust-stateright-validator` crate keeps domain logic in small modules:

| File                    | Responsibility                                                           |
| ----------------------- | ------------------------------------------------------------------------ |
| `actions.rs`            | Domain action and small enum definitions used by traces and transitions. |
| `action_generation.rs`  | Legal action enumeration for each state during model exploration.        |
| `ceremonies.rs`         | Tournament, duel, wedding, banquet, and consequence transitions.         |
| `ledgers.rs`            | Score and reward ledger state.                                           |
| `model.rs`              | Core bounded model configuration.                                        |
| `properties.rs`         | `always` invariants and `sometimes` reachability checks.                 |
| `scoring.rs`            | Score atoms, morale changes, winner selection, and reward tallying.      |
| `serde_impls/`          | Serialization adapters for domain types, one module per adapter group.   |
| `state.rs`              | Core state snapshot, state enums, and guard helpers.                     |
| `stateright_adapter.rs` | Stateright `Model` implementation for the core model.                    |
| `trace.rs`              | Concrete JSON trace replay and validation output types.                  |
| `transitions.rs`        | Top-level transition dispatcher and gameplay/reward transitions.         |

The binary `src/bin/validate_trace.rs` is process glue. The Explorer example in
`examples/serve_explorer.rs` is diagnostic glue.

### 3.1. `serde_impls/action_names.rs` boundary

`serde_impls/action_names.rs` owns the canonical JSON name tables and lookups
for unit `SkyAction` variants: `UNIT_ACTION_NAMES`, `TAGGED_ACTION_NAMES`,
`unit_action_name`, and `unit_action_from_name`. `unit_action_name` returns
`Option<&'static str>`, `None` for a tagged variant, so the one call site in
`serde_impls/actions.rs` can turn that into a `serde::ser::Error` rather than
panicking; `unit_action_from_name` returns `Option<SkyAction>`, `None` for an
unrecognized name. These exist to back the `SkyAction` serde adapter in
`serde_impls/actions.rs`, the module's only permitted consumer — its
`pub(super)` visibility enforces that at compile time. Domain modules and
external callers must not depend on this private serialization detail; go
through `Serialize`/`Deserialize` for `SkyAction` instead.

Composition rule: when a `SkyAction` variant changes, update the
serializer/deserializer dispatch in `serde_impls/actions.rs` and the name
tables or lookup functions in `action_names.rs` together — whether that is a
name-table entry for a unit variant or a `serialize_tagged`/
`serialize_team_action` call for a payload-carrying one. Reuse policy: this
module is scoped to `SkyAction` serde name conversion only; give another wire
format or domain type its own adapter-owned mapping rather than extending this
one.

## 4. Public application programming interface

The crate root re-exports the public surface used by tests, tools, and future
runtime integration:

| Symbol                                         | Purpose                                                              |
| ---------------------------------------------- | -------------------------------------------------------------------- |
| `SkyjoustInteractionModel`                     | Configures the bounded interaction model.                            |
| `SkyState`                                     | Carries the complete model snapshot.                                 |
| `SkyAction`                                    | Represents one replayable high-level action.                         |
| `TraceValidation`                              | Reports trace replay success or failure.                             |
| `TraceFailure`                                 | Describes the first failed replay step.                              |
| `validate_trace`                               | Replays concrete actions against transition guards and invariants.   |
| State enums                                    | Expose app, match, ceremony, Warfront, objective, and reward phases. |
| `ALWAYS_PROPERTIES` and `SOMETIMES_PROPERTIES` | Expose property tables for diagnostics.                              |

Serde support is intentionally isolated in the `serde_impls` module; domain
modules do not derive serialization traits directly.

## 5. Extending the model

Model changes should be made in this order:

1. Add the `SkyAction` variant in `actions.rs`.
2. Add action generation in `action_generation.rs` so Stateright can explore
   the new transition only from legal states.
3. Add transition handling in `transitions.rs` or the appropriate feature
   module, such as `ceremonies.rs`.
4. Add or update invariants and reachability checks in `properties.rs`.
5. Update the canonical graph bundle in `docs/skyjoust-state-graphs.yaml`,
   then run `make generate-state-graphs` to regenerate
   `docs/skyjoust-state-graphs.json`.
6. Update
   `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`.
7. Add a focused unit test and, when structured output, user-interface output,
   diagnostics, or JSON contracts change, an `insta` snapshot test with
   meaningful, stable assertions.
8. Add `trybuild` coverage when the change introduces compile-time behaviour,
   such as macro expansion, trait bounds, feature-gated APIs, or compile-pass
   and compile-fail contracts.

Guard helpers should stay pure and side-effect free. Transition helpers may
mutate only the cloned destination state supplied by the caller.

## 6. Local validation

Run the full Rust gates before committing code:

```sh
make check-fmt
make check-state-graphs
make lint
make typecheck
make test
cargo doc --no-deps --workspace
```

`make typecheck` runs `cargo check --workspace --all-targets --all-features`
with `RUSTFLAGS` composed from the caller's value, `-D warnings` and the
standard flags (`-Zthreads=8`, plus `mold` on Linux), so warnings fail the gate.

Run Markdown checks after documentation changes:

```sh
make fmt
make markdownlint
make nixie
git diff --check
```

### 6.1. Markdown lint configuration

`.markdownlint-cli2.jsonc` is the single Markdown lint configuration. The
`markdownlint-cli2 --fix` step in `make fmt`, the `make markdownlint` target,
and the `markdownlint-cli2` action in CI all read it, so one policy governs
formatting and checking alike.

It enforces the estate Markdown formatting baseline:

- `MD004` requires dash bullets, matching the documentation style guide.
- `MD010` permits hard tabs inside code blocks while still checking prose.
- `MD013` wraps prose at 80 columns, allows code blocks up to 120 columns, and
  ignores table and heading widths.
- `MD029` requires ordered lists to use increasing numeric markers.

`make fmt` runs `mdtablefix` over the Markdown that Git tracks before the lint
fix, and `make check-fmt` runs the matching `mdtablefix --check`, so
unformatted Markdown fails CI.

### 6.2. Spelling policy

Run `make spelling` to enforce en-GB-oxendict spelling in tracked Markdown
prose. The gate regenerates `typos.toml` on every run from the live shared
dictionary and the repository-specific `typos.local.toml` overlay, then checks
the tracked Markdown. Because the dictionary is live, `typos.toml` must never
be drift checked in continuous integration.

`TYPOS_CONFIG_BUILDER_VERSION` in the `Makefile` pins the
`typos-config-builder` release the gate runs (currently `v0.1.3`). Raise it
together with the regenerated `typos.toml`, never on its own.

The shared `typos-config-builder` CLI refreshes the estate dictionary into an
untracked local cache only when the authoritative copy is newer, so a valid
cache remains usable in a network-restricted checkout.

Keep repository-specific identifiers and deliberate quotations in
`typos.local.toml`. Never edit generated entries by hand.

The Stateright Explorer can help diagnose counterexamples:

```sh
cargo run -p skyjoust-stateright-validator --example serve_explorer
```

The trace validator can replay a fixture:

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace \
  < crates/skyjoust_stateright_validator/traces/tournament_reward_commit.json
```

Set `SKYJOUST_VALIDATOR_DEBUG=1` during debug builds to emit a
`tracing::debug!` event for each transition attempt during depth-first search.
`validate_trace` and the Explorer example both install a stderr-writing
`tracing_subscriber`, so the events are visible when running either; a caller
embedding the library elsewhere must install its own subscriber to observe them.

## 7. Lint baseline

Skyjoust follows the df12 estate's phase 2 Rust baseline for lint configuration.
`Cargo.toml` is the source of truth for the exact lint set; this section
explains where the tables live and how to work with them, not what every entry
does.

### 7.1. Table placement and inheritance

The canonical clippy, rust, and rustdoc lint tables live under
`[workspace.lints.clippy]`, `[workspace.lints.rust]`, and
`[workspace.lints.rustdoc]` in the root `Cargo.toml`. Every workspace member,
including the root `skyjoust` package, inherits them with:

```toml
[lints]
workspace = true
```

Any new crate added to the workspace must carry that stanza. A crate without it
silently opts out of the estate baseline instead of failing loudly, so review
new `Cargo.toml` files for it during code review.

### 7.2. What the tables enforce

The tables summarize the estate's phase 2 baseline; read `Cargo.toml` for the
authoritative, current list rather than relying on this summary. In brief:

- Clippy denies panic-prone operations (`unwrap_used`, `expect_used`,
  `indexing_slicing`, `unreachable`, and similar), debugging leftovers
  (`dbg_macro`, `print_stdout`, `print_stderr`), numerical foot-guns such as
  lossy casts, and direct environment access (`disallowed_methods`, see §7.4).
  `clippy::pedantic` runs at `warn`.
- The rust lint set forbids `unsafe_code` outright and denies `missing_docs`,
  so every public item needs a doc comment.
- The rustdoc set denies broken and private intra-doc links, bare URLs, and
  malformed code blocks, alongside `missing_crate_level_docs`.

### 7.3. Silencing a lint

Fix the violation. When a fix is not worthwhile — the site is deliberately
outside the mandate, or a rewrite would cost more clarity than it buys —
annotate it with:

```rust
#[expect(clippy::some_lint, reason = "why this site is a sanctioned exception")]
```

Never use `#[allow(...)]` for this. `expect` only suppresses the lint while the
violation still exists; once the site is fixed or refactored away, the
unfulfilled expectation itself becomes a warning, so a stale annotation
surfaces instead of rotting silently.

That rule governs handwritten code. Lints raised by *macro expansion* are a
separate case, because the expansion — not the visible source — decides whether
the lint fires, and often varies from one annotated item to the next. There the
argument for `expect` inverts: an expectation that goes unfulfilled on some
items warns on exactly the sites that were already clean. Such suppressions
belong in an attribute macro in `crates/skyjoust_test_macros`, not at the call
site. [ADR 006](adr/006-test-macro-crate-for-fixture-expansion-lints.md)
records the decision and the reasoning.

The case that prompted it is `rstest`'s `#[fixture]`, which re-wraps the
annotated body in a further block and so trips `unused_braces` on a
single-expression fixture. Splitting the body over several lines silences the
lint, but `.rustfmt.toml` sets `fn_single_line = true`, so `cargo fmt`
collapses it back: `make lint` and `make check-fmt` then demand mutually
exclusive spellings of the same fixture. Annotate the fixture instead:

```rust
use rstest::fixture;
use skyjoust_test_macros::allow_fixture_expansion_lints;

#[allow_fixture_expansion_lints]
#[fixture]
fn seed() -> u32 { 7 }
```

Add `skyjoust-test-macros` as a path development dependency of any crate whose
tests need it. Reach for a new attribute in that crate only when a lint comes
from macro expansion and no reformulation of the source satisfies every gate at
once; everything else takes `#[expect(...)]` at the site.

### 7.4. `clippy.toml` thresholds and the environment-access mandate

`clippy.toml` sets the code-health thresholds (cognitive complexity, argument
count, function length, nesting depth) and lists the `std::env` functions
clippy disallows: `var`, `var_os`, `vars`, `vars_os`, `set_var`, and
`remove_var`. Inject an environment reader (or, in tests, a stub environment)
instead of reading or mutating the process environment directly.
`allow-expect-in-tests` permits `.expect(...)` inside `#[test]` functions, but
not in shared, non-test helpers.

### 7.5. Toolchain

`rust-toolchain.toml` pins the dated nightly channel `nightly-2026-10-30` and
requires it to supply the `rustfmt`, `clippy`, and `rust-analyzer` components
(a repository may carry additional components beyond these three, such as an
opt-in Cranelift codegen backend for `tools/dev-fast/config.toml`). `cargo fmt`
and `cargo clippy` both run under the pinned nightly automatically; rustup
resolves the toolchain from `rust-toolchain.toml` without an explicit
`+nightly-...` argument.

The development profile follows the dev-build footprint standard: it sets
`incremental = false` and `debug = 0`, and `.cargo/config.toml` sets the native
Cargo key `[unstable] embed-metadata = false`, which `nightly-2026-07-31` and
later pins use to avoid duplicated metadata in `.rlib` files; Cargo still
produces the corresponding `.rmeta` metadata. Do not replace it with a raw
`rustc` metadata flag; those broke build scripts.
`tests/dev_build_footprint.rs` holds the three settings.

## 8. Fast development builds

`make dev-build` and `make dev-test` offer an opt-in, faster iteration loop for
local debug work: `dev-build` compiles debug binaries and `dev-test` runs the
test suite, both using the Cranelift codegen backend and the `mold` linker
configured in `tools/dev-fast/config.toml`.

The `DEV_FAST_CONFIG` variable names that fragment, defaulting to
`tools/dev-fast/config.toml`; both targets pass it to Cargo explicitly with
`--config "$(DEV_FAST_CONFIG)"`. Cargo never auto-discovers this fragment — it
takes effect only when a target invokes it directly.

Using the fragment requires a nightly toolchain, since the Cranelift codegen
backend is unstable. On Linux it also requires the `mold` linker on `PATH`; the
fragment gates the `-fuse-ld=mold` flag behind a `target_os = "linux"` `cfg`
table, so other platforms fall back to their default linker.

CI provisions the linker through the pinned `leynos/shared-actions`
`setup-rust` action rather than an `apt-get` step. The `Setup Rust` step in
`.github/workflows/ci.yml` passes `install-mold: 'true'`, which `make dev-test`
and the build standard need on Linux, and `install-clang-lld: 'true'`, which
installs `clang` and `lld`. The `lld` install is new for this repository and
unused by its configuration. The action fails the job unless each installed
tool resolves on `PATH`, skips with a notice on other platforms, and sets no
linker flag.

The repository also follows the estate's Rust build standard, which
`.cargo/config.toml` sets and Cargo auto-discovers, so a bare `cargo build`
gets it: Cranelift for the development profile (the whole suite passes under
it), the parallel `rustc` frontend with `-Zthreads=8` in both `rustflags`
sources, and `mold` in the `cfg(target_os = "linux")` source. The fragment
therefore repeats what the configuration already sets, and the Make targets
keep passing it so the dev-fast convention below still holds. `make release`
uses the release profile, which names no backend, and assigns the inherited
`RUSTFLAGS`, which is empty when the caller exports none. The assignment
displaces every `rustflags` source, so release adds neither the frontend flag
nor `mold` (a caller's own value can still contain either). A bare
`cargo build --release` still takes both, because Cargo does not select
`rustflags` by profile. The decision is recorded in
[ADR 007](adr/007-adopt-the-rust-build-standard.md). CI's coverage action
detects the Cranelift development profile and holds coverage on LLVM. Cargo
applies one `rustflags` source and an assigned `RUSTFLAGS` replaces them all,
so the Makefile restates both flags as `STANDARD_RUSTFLAGS` for the targets
that assign `RUSTFLAGS`, adding them to any `RUSTFLAGS` the recipe inherits
(setup-rust exports one in CI) rather than replacing it; every `lint` command
assigns it too. The Makefile adds `mold` only when both the host and the
compilation target (`CARGO_BUILD_TARGET`, when set) are Linux; an Android
triple contains `-linux-` but is not Linux to Cargo's `target_os`, so it does
not get `mold`. `tests/build_standard_contract.rs` (readers in
`tests/build_standard/support.rs`) holds the configuration, those recipes and
the Cranelift and Whitaker settings to this.

CI's coverage step is narrower than `make test`. The pinned coverage action
defaults `all-features`, `all-targets` and `doctests` to `false`, and the
workflow sets none of them, so coverage measures the default-feature library
and test targets only. Its optional doctests, when enabled, run uninstrumented
and contribute no coverage. Coverage therefore is not evidence that the whole
suite passes under Cranelift; the `make test` step that precedes it is. The
coverage step's effective `RUSTFLAGS` (setup-rust exports one) displace the
configured `mold` flag, so coverage does not depend on `mold` being installed;
`make lint` and `make test` do. `tests/build_standard_ci.rs` holds that order
(`mold`, then `make lint`, then `make test`, then coverage) with one validator
over the real `ci.yml` and over fixtures that echo, merely name, or
conditionally skip a gate, using the reader in
`tests/build_standard/workflow.rs`, which recognizes only the forms this
workflow uses and rejects any other (a folded `run: >` block, a comment after an
`install-mold` value) with a named error.

Skyjoust's own extra fact, beyond the general dev-fast contract above: per §7,
the standard `build`, `test`, `lint`, and `typecheck` targets already pass
`--config "$(DEV_FAST_CONFIG)"` to every `cargo` invocation they make. Dev-fast
is therefore skyjoust's standard development path, not only an opt-in one;
`make dev-build`/`make dev-test` remain useful for a build/test cycle that
skips the other standard targets' formatting and lint checks. The one exception
is `lint`'s Whitaker Dylint invocation: Whitaker runs its own dylint driver
under a separately pinned toolchain, outside rustup's toolchain-file
auto-install mechanism, so nothing guarantees that toolchain has the Cranelift
component the fragment selects — the fragment is deliberately not passed there.
Because `.cargo/config.toml` now selects Cranelift for every development build,
the Whitaker invocation also sets `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm`
(`WHITAKER_CODEGEN_BACKEND`), together with
`CARGO_UNSTABLE_CODEGEN_BACKEND=true`, because Dylint builds its driver in a
crate outside the repository that the `[unstable]` table does not reach. A
failing Whitaker run fails `make lint`; a missing `whitaker` binary skips the
check with a message. A Linux host needs `mold` installed before any `cargo` or
`make` build, build scripts included.

## 9. CodeScene coverage and the CV-005 contract

Main owns coverage publication (estate rule CV-005). A pull request measures
coverage for its own ratchet check only, in the `build-test` job of `ci.yml`
(`with-ratchet: 'true'`, `publish-artefact: 'false'`); it never contacts
CodeScene. `.github/workflows/coverage-main.yml` runs on every push to `main`
in the `codescene` environment, generates coverage again and uploads it with
`mode: upload`, guarded on `github.ref == 'refs/heads/main'` alone. The token
reaches the uploader only through its `access-token` input, never an `env`.
When the repository has no `CS_ACCESS_TOKEN` (no CodeScene project yet) the
uploader records a `CodeScene upload skipped` notice and a step-summary line
and the run still succeeds.

Both coverage callers (`ci.yml` and `coverage-main.yml`) set
`baseline-rust-file: .coverage-baseline-v2.rust`. The ratchet restores its
baseline from a cache keyed by that path, so a new path starts from the
committed figure instead of the cached one. The file was introduced when the
bump to `nightly-2026-10-30` lowered measured coverage from 87.36% to 86.17%
with every test unchanged, and holds the new measurement rounded down to one
decimal place (86.1). To reset the baseline after a similar toolchain change,
commit the new figure under a new suffix (`.coverage-baseline-v3.rust`) and
point both callers at it.

`make test-workflow-contracts` checks that shape. It runs the shared
`cv005-contracts check --repository .` from the commit named by
`CV005_CONTRACTS_REF` in the `Makefile` (a bump of that pin is a reviewed
change, never a silent upgrade), then the local wiring tests in
`tests/workflow_contracts`. It needs `uv`, Python 3.13 for the checker, and
`pytest` and `pyyaml`, which `uv run --with` supplies. Both runs go through
`UV` and `UV_ENV`, so a `uv` outside `PATH` can be injected. `make all` and CI
run the target; `.github/cv005.toml` holds the repository name and nothing
else. A Dependabot pull request merged by the automerge workflow fires no push
event, so that merge publishes nothing until the next push to `main`
(shared-actions #518 tracks the fix).

[^cargo-queue-budgets]: [Cuprum PR #631: Raise test budgets for Cargo cache waits](https://github.com/leynos/cuprum/pull/631).

[^build-resource-coordination]: [Cuprum issue #633: Control shared build resources before timed test workloads start](https://github.com/leynos/cuprum/issues/633).

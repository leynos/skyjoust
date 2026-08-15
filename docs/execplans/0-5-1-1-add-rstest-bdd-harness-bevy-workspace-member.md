# Add `rstest-bdd-harness-bevy` as a workspace member

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, and `Outcomes & retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

Approval gate: this plan must be approved before implementation begins. Do not
treat silence as approval.

Roadmap task: `0.5.1.1` in [the Skyjoust roadmap](../roadmap.md).

## Purpose / big picture

Skyjoust needs behavioural tests that drive a deterministic, headless Bevy
entity-component system (ECS) without dragging a renderer, a window, or a
graphics processing unit (GPU) into `cargo test`. The eventual vehicle is a
reusable harness crate, `rstest-bdd-harness-bevy`, that plugs a headless Bevy
application into the `rstest-bdd` harness contract. That harness must stay free
of Skyjoust and Lille code so it can later move to its own repository as a
directory move plus a dependency rewire.

This plan delivers the first step only: the crate exists as a workspace member,
compiles headlessly, carries the estate lint baseline, and proves — with
running tests — that the three load-bearing dependency choices actually work
together on this repository's pinned toolchain. It deliberately stops short of
the harness types themselves (`BevyScenario`, `BevyHarness`, `BevyProfile`,
`BareBevyProfile`, `MinimalBevyProfile`, `BevyAttributePolicy`); those are
roadmap task `0.5.1.2`.

After this change a developer can run one command and watch a headless Bevy
application advance a fixed tick from inside a Gherkin scenario:

```bash
cargo test -p rstest-bdd-harness-bevy
```

and see, among the passing tests, a behaviour-driven development (BDD) scenario
named `minimal_app_advances_one_tick` that drives a real `bevy::app::App`
through one `Update` schedule pass.

The observable outcome is: the workspace has a third crate; that crate has no
path to Skyjoust or Lille; `make check-fmt`, `make lint`, and `make test` all
pass; and the crate's own tests demonstrate a headless Bevy tick driven by
`rstest-bdd`.

## Constraints

These are hard invariants. If satisfying the objective would require violating
one, stop and escalate rather than working around it.

- The crate must not depend on `skyjoust`, `skyjoust-stateright-validator`, or
  any Lille crate, directly or transitively. This is the extraction contract
  from [the harness design](../rstest-bdd-harness-bevy-design.md) §11.
- The crate must not pull Bevy rendering, windowing, assets, audio, or GPU
  state. Bevy is declared with `default-features = false`.
- No Skyjoust gameplay profile, validator trace type, or runtime state resource
  may be checked into the harness crate. Downstream profiles live in the
  consuming crate (design §8).
- `crates/rstest-bdd-harness-bevy/Cargo.toml` must carry
  `[lints]\nworkspace = true`, per
  [the developer's guide](../developers-guide.md) §7.1.
- `unsafe_code` stays forbidden; `missing_docs` and `missing_crate_level_docs`
  stay denied. Every module opens with a `//!` comment and every public item
  carries a `///` comment with a worked example, per `AGENTS.md`.
- No source file exceeds 400 lines (`AGENTS.md`).
- Dependencies use caret requirements. No `*`, no `>=`, no wildcard
  (`AGENTS.md`).
- Do not modify `crates/skyjoust_stateright_validator/`, `src/`, or
  `tests/makefile_contract.rs`.
- Do not add or rename Makefile targets. `tests/makefile_contract.rs` asserts
  that the `build`, `test`, `lint`, and `typecheck` recipes each pass
  `--config tools/dev-fast/config.toml`; leaving the Makefile alone keeps that
  contract intact.
- Do not edit `typos.toml`; it is generated. Repository-specific spelling
  exceptions belong in `typos.local.toml`.
- Prose is British English with Oxford spelling (`-ize`, `-yse`, `-our`),
  wrapped at 80 columns; code fences wrap at 120 columns.

## Tolerances (exception triggers)

- Scope: if the change touches more than 18 files, stop and escalate.
- Public interface: this plan introduces exactly two public functions plus a
  re-export block (see `Interfaces and dependencies`). If a third public item
  appears necessary, stop and escalate — it probably belongs in `0.5.1.2`.
- Dependencies: the dependency set in `Interfaces and dependencies` is fixed.
  If any additional crate is required, stop and escalate.
- Bevy features: if `default-features = false` alone proves insufficient and a
  feature must be enabled, stop and escalate before adding it. A verified
  fallback is recorded in `Risks`.
- `rstest-bdd` version: if published `0.6.0-beta3` proves insufficient and a
  git dependency on `main` becomes necessary, stop and escalate.
- Iterations: if a gate still fails after three fix attempts, stop and
  escalate with the log path.
- Time: if any milestone exceeds two hours of work, stop and escalate.
- Ambiguity: if two readings of a roadmap or design statement would produce
  materially different crates, stop and present the options.

## Risks

- Risk: `bevy = { default-features = false }` disables Bevy's `std` feature,
  because `std` is part of Bevy's `default` feature set. A no-`std` Bevy might
  fail to provide a working `MinimalPlugins` schedule runner.
  Severity: high. Likelihood: low (retired — see `Surprises & discoveries`).
  Mitigation: retired by a compile-and-run probe. If a later milestone does hit
  a no-`std` wall, the verified fallback is
  `features = ["std"]`, which adds 28 crates to the graph; escalate before
  applying it.

- Risk: published `rstest-bdd` `0.6.0-beta3` might lack the harness-context
  application programming interface (API) the design targets, forcing a git
  dependency on `main`.
  Severity: high. Likelihood: low (retired — see `Surprises & discoveries`).
  Mitigation: retired by a probe that ran `#[scenario(harness = ...)]` with a
  non-unit `Context` against the published crates.

- Risk: `clippy::expect_used` is denied estate-wide, and
  `allow-expect-in-tests = true` in `clippy.toml` does **not** cover free
  functions in `tests/*.rs` that are not `#[test]`-annotated. `rstest-bdd` step
  functions are exactly such functions, so `.expect()` inside a step fails
  `make lint`.
  Severity: medium. Likelihood: high (observed).
  Mitigation: the step design in this plan carries no `Option`/`Result`
  unwrapping. Scenario state comes from an `rstest` fixture, not a
  thread-local slot.

- Risk: editing only a `.feature` file does not invalidate the build, so a
  changed scenario can appear to pass while the compiled step table is stale.
  Severity: medium. Likelihood: high.
  Mitigation: each scenario binding file carries
  `const _: &str = include_str!("features/<name>.feature");`, the idiom used by
  `rstest-bdd`'s own fixtures, which gives `rustc` a rebuild dependency on the
  feature file.

- Risk: the workspace root package pins `rstest = "0.18"` while `rstest-bdd`
  `0.6.0-beta3` requires `rstest = "0.26.1"`. Two major-incompatible `rstest`
  versions will coexist in `Cargo.lock`.
  Severity: low. Likelihood: certain.
  Mitigation: accept both. They are semver-incompatible, so Cargo keeps them
  side by side; no crate sees two versions at once. Do not bump the root
  package's `rstest` as part of this task.

- Risk: `googletest`'s `expect_that!` requires an active test context and
  panics with `No test context found` when the test is not annotated
  `#[gtest]`.
  Severity: low. Likelihood: high (observed).
  Mitigation: unit tests use `#[gtest]` above `#[rstest]`; step functions use
  `assert_that!`, which panics directly and needs no context.

- Risk: `typos` may reject Bevy or `rstest-bdd` vocabulary in the new prose.
  Severity: low. Likelihood: low.
  Mitigation: add narrow entries to `typos.local.toml` only if `make spelling`
  actually fails; never edit the generated `typos.toml`.

- Risk: `codecov.yml` sets an 80% patch-coverage target, and a scaffolding
  crate with few executable lines can swing the patch figure.
  Severity: low. Likelihood: low.
  Mitigation: the two public functions are covered by unit tests, doctests, a
  property test, and a behavioural scenario.

## Progress

- [ ] Milestone 0: orientation and evidence gathering (completed during
      planning; see `Artefacts and notes`).
- [ ] Milestone 1: record the decision — write ADR 006, amend the harness
      design document, and index both.
- [ ] Milestone 2 (red): add the crate manifest, an empty library, and the
      failing tests.
- [ ] Milestone 3 (green): implement `src/profile.rs` so the tests pass.
- [ ] Milestone 4: add behavioural, property, and extraction-boundary
      coverage.
- [ ] Milestone 5: documentation — repository layout, contents index,
      developer's guide, crate README, roadmap tick.
- [ ] Milestone 6: full gate run, branch push, draft pull request.

## Surprises & discoveries

- Observation: `rstest-bdd` `0.6.0-beta3` was published to crates.io on
  2026-07-07, along with `rstest-bdd-harness`, `rstest-bdd-macros`,
  `rstest-bdd-harness-tokio`, and `rstest-bdd-harness-gpui` at the same
  version. The roadmap's instruction to "use git `rstest-bdd` dependencies
  against `main` until v0.6.0-beta3 is published" is therefore already
  satisfied by published crates.
  Evidence: the crates.io versions endpoint lists `0.6.0-beta3` with
  `created_at` `2026-07-07T23:13:10Z`; `rstest-bdd-harness 0.6.0-beta3` is
  dated `2026-07-07T23:12:46Z`.
  Impact: this plan uses published caret requirements, not git dependencies.
  That also satisfies `AGENTS.md`'s "mandate caret requirements for all
  dependencies" rule, which a git dependency would sidestep. The roadmap
  sub-bullet and design §9 are amended accordingly.

- Observation: `rstest-bdd`'s `main` branch has moved to an unpublished
  `0.6.0-beta4` (workspace head commit `12b9357`, 2026-08-14), well past the
  commit `21b67a4` cited in the design document. `beta4` adds a
  `policy_conformance` module, a `testing` feature exposing `FailingHarness`,
  and guard-based fixture borrowing (`StepContext::borrow_mut` taking `&self`).
  None of those are present in `beta3`.
  Evidence: `main`'s `crates/rstest-bdd-harness/src/lib.rs` exports
  `policy_conformance` and a `#[cfg(feature = "testing")] FailingHarness`;
  the extracted `beta3` `.crate` archive exports neither and has no
  `[features]` table at all.
  Impact: pinning `beta3` means `rstest-bdd-harness = { features = ["testing"] }`
  will not resolve, and `assert_attribute_policy_conformance` is unavailable.
  Neither is needed by this milestone; `0.5.1.2` and `0.5.1.3` must plan around
  them. Recorded in the design document's dependency section.

- Observation: `bevy = { version = "0.17.3", default-features = false }`
  compiles and runs a headless `MinimalPlugins` application correctly, despite
  `std` being part of Bevy's `default` feature set. `MinimalPlugins`,
  `App`, `World`, `Resource`, and `Update` are all unconditionally re-exported
  from `bevy_internal::prelude`.
  Evidence: a probe crate at `~/.cache/bevy-probe` with exactly that dependency
  line compiles under `nightly-2026-03-26` and its test asserting that one
  `App::update()` bumps a toy resource passes. Dependency-graph sizes:
  98 crates with `default-features = false`, 126 with `features = ["std"]`
  added, 428 with Bevy defaults.
  Impact: the roadmap's literal instruction works as written, gives the
  smallest dependency graph, and — because the multi-threaded task pool is
  absent — is the most deterministic option for fixed-tick behavioural tests.
  No extra features are enabled.

- Observation: `clippy.toml`'s `allow-expect-in-tests = true` does not exempt
  non-`#[test]` free functions inside `tests/*.rs`. A step function calling
  `.expect()` fails `cargo clippy --all-targets -- -D warnings`.
  Evidence: a probe with a `thread_local!` `RefCell<Option<App>>` and
  `.expect("app is initialised")` in a `#[when]` step produced
  `error: used expect() on an Option value ... -D clippy::expect-used` at two
  sites.
  Impact: this plan's behavioural test uses an `rstest` `#[fixture]` returning
  `RefCell<App>`, which removes the `Option` and the `.expect()` entirely. The
  same trap will recur in `0.5.1.2`; note it in the developer's guide.

- Observation: `googletest 0.14.3`'s `expect_that!` requires the `#[gtest]`
  attribute; without it every assertion panics with `No test context found`.
  Evidence: four unit tests annotated only `#[rstest]` failed that way; adding
  `#[gtest]` above `#[rstest]` made all four pass.
  Impact: unit tests use `#[gtest]` plus `#[rstest]`; step functions use
  `assert_that!`.

- Observation: the design document's `docs/roadmap.md` companion list, and the
  earlier `1.1.1` execplan, both cite `docs/rstest-bdd-users-guide.md` and
  `docs/ortho-config-users-guide.md`. Neither file exists in this repository.
  Evidence: repository-wide glob for both names returns nothing.
  Impact: this plan cites the upstream `rstest-bdd` users' guide by uniform
  resource locator (URL) instead, and does not create either local file.

## Decision log

- Decision: depend on published `rstest-bdd` `0.6.0-beta3` crates with caret
  requirements rather than git dependencies against `main`.
  Rationale: the roadmap's git-dependency instruction was explicitly
  conditional on `beta3` not yet being published; it now is. Published crates
  satisfy `AGENTS.md`'s caret-requirement mandate, keep `Cargo.lock` stable,
  and avoid pulling `rstest-bdd`'s vendored `gpui` tree into this workspace's
  dependency resolution. The `beta4`-only APIs are not needed here.
  Date/Author: 2026-08-15, planning pass.

- Decision: declare Bevy exactly as
  `bevy = { version = "0.17.3", default-features = false }`, with no additional
  features.
  Rationale: it is what the roadmap and design say, it is empirically
  sufficient for headless `MinimalPlugins`, it gives the smallest dependency
  graph, and the absent multi-threaded task pool improves determinism. The
  `features = ["std"]` variant is a verified fallback held in reserve.
  Date/Author: 2026-08-15, planning pass.

- Decision: this milestone ships two public constructors, `bare_app()` and
  `minimal_app()`, in `src/profile.rs`, plus a re-export of the base harness
  API from `lib.rs`.
  Rationale: the ExecPlan bar is demonstrably working behaviour, not a crate
  that merely compiles. These two functions are the embryo of the design's
  `BareBevyProfile` and `MinimalBevyProfile` (design §4), so `0.5.1.2` folds
  them into `BevyProfile::configure` implementations rather than deleting them.
  The re-export follows the precedent set by `rstest-bdd-harness-gpui` and
  `rstest-bdd-harness-tokio`, both of which re-export the whole base harness
  API from their crate roots, and it makes the `rstest-bdd-harness` dependency
  load-bearing from day one rather than declared-but-unused.
  Alternative considered and rejected: an empty library with no public items.
  It would compile, but it would prove nothing and would leave `bevy` and
  `rstest-bdd-harness` as unused dependencies.
  Date/Author: 2026-08-15, planning pass.

- Decision: keep the crate directory hyphenated as
  `crates/rstest-bdd-harness-bevy/`, diverging from the existing
  `crates/skyjoust_stateright_validator/` underscore convention.
  Rationale: the roadmap and design both name the hyphenated path; the
  extraction target repository is `leynos/rstest-bdd-harness-bevy`; and
  upstream `rstest-bdd` uses hyphenated crate directories throughout. Matching
  the extraction target keeps the eventual move a pure directory copy.
  Date/Author: 2026-08-15, planning pass.

- Decision: write a new ADR (`docs/adr/006-...`) rather than amending ADR 002.
  Rationale: [the developer's guide](../developers-guide.md) §2 states "Record
  any such extraction in a follow-up ADR before changing `Cargo.toml` workspace
  members," and ADR 002 says the workspace stays at two crates "until a later
  ADR records a specific extraction." Adding a third crate is exactly that
  trigger. ADR 002 is `Accepted`; superseding text belongs in a new record that
  cross-references it, not in edits to a settled decision.
  Date/Author: 2026-08-15, planning pass.

- Decision: follow the repository's existing ADR file convention
  (`docs/adr/NNN-topic.md`, `# NNN: Title`, plain `Status:`/`Date:` lines) in
  preference to the literal template in
  [the documentation style guide](../documentation-style-guide.md).
  Rationale: all five existing ADRs use the repository convention, and
  [the repository layout](../repository-layout.md) explicitly acknowledges that
  `docs/adr/` "predates the style guide's canonical ADR filename convention."
  Consistency with the five siblings beats consistency with an unfollowed
  template. Reconciling the two conventions is out of scope for this task.
  Date/Author: 2026-08-15, planning pass.

- Decision: leave [the user's guide](../users-guide.md) unchanged.
  Rationale: that guide is scoped to operators and integrators running the
  Stateright validator tooling. A maintainer-facing test-harness crate changes
  none of those workflows and exposes no player- or operator-visible
  behaviour. The same reasoning was recorded for roadmap task `1.1.1`. The
  audience for this change is maintainers, so the internal conventions go in
  [the developer's guide](../developers-guide.md) instead.
  Date/Author: 2026-08-15, planning pass.

- Decision: include a `proptest` property over tick counts; do not use `kani`
  or `verus`.
  Rationale: `minimal_app().update()` repeated *n* times must leave
  `FrameCount` equal to *n* for every *n* in a bounded range. That is a genuine
  invariant over a range of inputs and it directly pre-figures the design's
  `update_times` (design §5), so the property carries forward. Bounded model
  checking and deductive proof are disproportionate: the property holds by
  Bevy's own frame counter, and neither `unsafe` code nor unbounded state is
  introduced. Record this judgement rather than silently omitting the tools.
  Date/Author: 2026-08-15, planning pass.

- Decision: do not add `insta` snapshot coverage in this milestone.
  Rationale: snapshots earn their keep when a multivariant output format must
  stay stable. This milestone emits no formatted output. Snapshot coverage
  becomes appropriate in `0.5.1.3`, where the panic-diagnostic message format
  (design §6) is the artefact worth pinning.
  Date/Author: 2026-08-15, planning pass.

## Context and orientation

Read this section if the repository is unfamiliar.

**What Skyjoust is.** A game project. The repository root is a Cargo workspace
whose root package, `skyjoust`, currently holds only a small binary in `src/`.
The one existing member crate,
`crates/skyjoust_stateright_validator/`, is a model checker for the game's
high-level interaction contract. The workspace is declared in the root
`Cargo.toml`:

```toml
[workspace]
members = [".", "crates/skyjoust_stateright_validator"]
resolver = "3"
```

**What Bevy is.** A Rust game engine built around an entity-component system.
An ECS stores game state as *components* on *entities*, and runs *systems*
(plain functions) over them in *schedules*. `bevy::app::App` is the top-level
object that owns the ECS world, the plugin list, and the schedules;
`App::update()` runs one pass of the main schedule, which includes the `Update`
schedule. `MinimalPlugins` is Bevy's smallest useful plugin group: task pools,
time, a frame counter, and a schedule runner — no window, no renderer.

**What `rstest-bdd` is.** A behaviour-driven development framework for Rust
that runs Gherkin scenarios through the ordinary `cargo test` harness. Gherkin
is the `Feature:` / `Scenario:` / `Given` / `When` / `Then` plain-text format.
Step functions are annotated `#[given("...")]`, `#[when("...")]`,
`#[then("...")]`; a `#[scenario(path = "...", index = N)]` function binds a
scenario in a feature file to a generated `#[rstest::rstest]` test. Feature
paths resolve relative to the crate root (`CARGO_MANIFEST_DIR`).

**What a harness adapter is.** `rstest-bdd` lets a third-party crate own the
framework setup around a scenario. The contract lives in the
`rstest-bdd-harness` crate:

```rust
pub trait HarnessAdapter {
    type Context: std::any::Any;
    fn run<T>(&self, request: ScenarioRunRequest<'_, Self::Context, T>) -> HarnessResult<T>;
}
```

The harness builds its `Context`, calls `request.run(context)`, and cleans up
afterwards. Step functions reach the context through the reserved fixture key
`rstest_bdd_harness_context`, written `#[from(rstest_bdd_harness_context)]`.
That is the machinery `0.5.1.2` will implement for Bevy. This milestone only
establishes the crate that will hold it.

**Where the design lives.**
[The `rstest-bdd-harness-bevy` design](../rstest-bdd-harness-bevy-design.md) is
the specification. §3 and §9 govern this task (prior art, constraints, crate
layout, dependency strategy). §§4–7 specify the API that `0.5.1.2` builds. §10
specifies verification. §11 is the extraction contract this milestone must not
compromise.

**Where the rules live.**

- `AGENTS.md` — engineering, documentation, Rust, and validation rules. Module
  `//!` comments, `///` docs with examples, 400-line file cap, caret
  dependency requirements, commit message format, quality gates.
- [The developer's guide](../developers-guide.md) — §2 covers the runtime crate
  boundary and the ADR-before-workspace-change rule; §7 covers the lint
  baseline and `clippy.toml` thresholds.
- [The documentation style guide](../documentation-style-guide.md) — sentence
  case headings, 80-column prose, en-GB Oxford spelling, table and figure
  captions, ADR structure.
- [The repository layout](../repository-layout.md) — the tree sketch and
  path-responsibility notes that must be updated when a crate is added.
- `clippy.toml` — cognitive-complexity threshold 9, at most 4 arguments, at
  most 70 lines per function, `allow-expect-in-tests = true`, and a
  `disallowed-methods` list that bans direct `std::env` access.
- `.rustfmt.toml` — nightly rustfmt, `imports_granularity = "Crate"`,
  `group_imports = "StdExternalCrate"`, `fn_single_line = true`.
- `rust-toolchain.toml` — pinned `nightly-2026-03-26`.

**Supporting references for the test work.**

- [Mastering test fixtures in Rust with `rstest`](../rust-testing-with-rstest-fixtures.md)
  — fixture and parameterization patterns; this plan's behavioural test uses a
  `#[fixture]` to hold scenario state.
- [Reliable testing in Rust via dependency injection](../reliable-testing-in-rust-via-dependency-injection.md)
  — why `clippy.toml` bans direct `std::env` access and what to do instead.
- [Effective, ergonomic, and dry doctests in Rust](../rust-doctest-dry-guide.md)
  — doctests compile as separate crates; they may use any dependency of the
  crate under test.
- [Navigating code complexity](../complexity-antipatterns-and-refactoring-strategies.md)
  — the complexity thresholds `clippy.toml` enforces.
- The upstream `rstest-bdd` users' guide, especially its third-party harness
  adapter cookbook: <https://github.com/leynos/rstest-bdd/blob/main/docs/users-guide.md>.

**Relevant agent skills.** Load `rust-router` to reach the Rust skills; then
`rust-unit-testing` for `rstest`, `googletest`, and `pretty_assertions`
assertion shape; `arch-crate-design` for crate boundary and feature-flag
questions; `arch-decision-records` for the ADR; `proptest` for the property
test; `hexagonal-architecture` for the adapter/domain split recorded below;
`commit-message` for the file-based commit workflow; `pr-creation` for the pull
request. Delegate full gate runs to the `scrutineer` sub-agent and mechanical
documentation edits to `scribe`.

**Hexagonal placement.** In ports-and-adapters terms the *port* is the
`rstest-bdd` harness contract (`HarnessAdapter`, `ScenarioRunRequest`,
`HarnessResult`). Bevy is a *driven framework*, and this crate is the *adapter*
that binds one to the other. It therefore holds no domain logic and no Skyjoust
rules — the constraint in §11 of the design is the hexagonal dependency rule
restated for a test-time adapter. Record this framing in the ADR.

## Plan of work

### Stage A: understand and propose (no code changes)

Completed during planning. The evidence is recorded in
`Surprises & discoveries` and `Artefacts and notes`. No further Stage A work is
required; go straight to Milestone 1 on approval.

### Stage B: record the decision (Milestone 1)

Documentation only. No Rust changes, so the code gates are not yet meaningful.

1. Create `docs/adr/006-in-tree-incubation-of-the-bevy-bdd-harness-crate.md`.
   Follow the shape of `docs/adr/002-crate-layout-and-public-api.md`: an
   `# 006: <title>` heading, then plain `Status: Proposed` and `Date:
   2026-08-15` lines, then `## Context`, `## Decision`, `## Consequences`.
   The record must state:
   - that the workspace grows to three crates, and why that does not reopen
     ADR 002's deferral of runtime crate splits (this is a tooling-facing test
     adapter, in the same category as the validator crate's stated exception);
   - the extraction contract: no `skyjoust` or `lille` dependency, no
     game-specific profile in-tree, stable module boundaries, one manifest;
   - the dependency decisions and their evidence: published `rstest-bdd`
     `0.6.0-beta3` over git `main`, Bevy `0.17.3` with
     `default-features = false` and no extra features;
   - the hexagonal placement (adapter, not domain);
   - a comparison table of the two dependency-sourcing options with a caption,
     per the style guide.
2. Amend [the harness design](../rstest-bdd-harness-bevy-design.md):
   - §3: replace the claim that the manifest "still labels those workspace
     crates as `0.6.0-beta2`" with the current position — `0.6.0-beta3` is
     published and carries the harness API this work targets; `main` has since
     moved to an unpublished `0.6.0-beta4`.
   - §9: replace the git-dependency instruction with the published caret
     requirements, and note the `beta4`-only APIs (`policy_conformance`, the
     `testing` feature and `FailingHarness`, guard-based fixture borrowing)
     that are unavailable under `beta3`.
   - §9: note that `default-features = false` is verified sufficient and that
     `features = ["std"]` is the held-in-reserve fallback.
   - §13: refresh the references, replacing the stale commit citation.
   - Add a pointer to ADR 006.
3. Add ADR 006 and this ExecPlan to [the contents index](../contents.md), in
   the `Architecture decision records` and `Execution plans` sections
   respectively, matching the existing bullet format.

Validation for Stage B: `make fmt`, `make markdownlint`, `make nixie`,
`git diff --check`. `make nixie` is required because the design document
contains a Mermaid diagram.

### Stage C: red tests (Milestone 2)

1. Add `"crates/rstest-bdd-harness-bevy"` to `members` in the root
   `Cargo.toml`.
2. Create `crates/rstest-bdd-harness-bevy/Cargo.toml` with the manifest given
   in `Interfaces and dependencies`.
3. Create `crates/rstest-bdd-harness-bevy/src/lib.rs` containing only the
   crate-level `//!` documentation and the harness re-export — deliberately
   *without* `mod profile;` or the `pub use profile::...` line.
4. Create `crates/rstest-bdd-harness-bevy/src/profile_tests.rs` with the unit
   tests, and `crates/rstest-bdd-harness-bevy/src/profile.rs` containing only
   its `//!` comment and the `#[cfg(test)] #[path = "profile_tests.rs"] mod
   tests;` declaration.

Run the focused test command. It must fail to compile because `bare_app` and
`minimal_app` do not exist, and because `lib.rs` does not declare `mod
profile;`. That is the red state, and the failure reason must be exactly
"cannot find function" / "unresolved import", not something incidental.

There is no expected-failure marker idiom in Rust equivalent to pytest's
`xfail(strict=True)`; a compile failure with the named missing symbols is the
strict red signal here. Record the observed error text.

### Stage D: implementation (Milestone 3)

1. Implement `bare_app()` and `minimal_app()` in
   `crates/rstest-bdd-harness-bevy/src/profile.rs`, each with a `///` doc
   comment carrying a worked example, and each marked `#[must_use]` (the
   workspace denies `clippy::must_use_candidate`).
2. Add `mod profile;` and `pub use profile::{bare_app, minimal_app};` to
   `lib.rs`.

Run the focused test command again. The four unit tests and two doctests must
pass. Make no other change in this step.

### Stage E: behavioural, property, and boundary coverage (Milestone 4)

1. Create `crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature`
   with the feature specification quoted in
   `Validation and acceptance`.
2. Create `crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs` binding
   that scenario, with an `rstest` `#[fixture]` supplying `RefCell<App>` and
   three step functions. Include the `include_str!` rebuild guard.
3. Create `crates/rstest-bdd-harness-bevy/tests/tick_properties.rs` with the
   `proptest` property over tick counts.
4. Create `crates/rstest-bdd-harness-bevy/tests/extraction_boundary.rs` with a
   pure predicate over manifest text, tested on both the real manifest (happy
   path) and a synthetic manifest naming a forbidden crate (unhappy path).

Each file is added red-first where practical: write the scenario binding before
the feature file exists to observe the macro's path diagnostic, then add the
feature file.

### Stage F: documentation and roadmap (Milestone 5)

1. Create `crates/rstest-bdd-harness-bevy/README.md`, following the shape of
   `crates/skyjoust_stateright_validator/README.md`: title, purpose,
   how-to-run commands, a `## Files` map, and an `## Extending` section that
   points at `0.5.1.2` and the design document.
2. Update [the repository layout](../repository-layout.md): add
   `crates/rstest-bdd-harness-bevy/` to the tree sketch, add path
   responsibility bullets for the crate, its `src/`, and its `tests/`, and
   update the `Cargo.toml` bullet that currently reads "The workspace currently
   includes `.` and `crates/skyjoust_stateright_validator`."
3. Update [the developer's guide](../developers-guide.md):
   - amend §2 so the "one runtime crate beside the validator crate" statement
     acknowledges the third, tooling-facing harness crate and cites ADR 006;
   - add a new section documenting the harness crate's boundary rules and the
     two testing traps this task uncovered — that `allow-expect-in-tests` does
     not cover `rstest-bdd` step functions in `tests/*.rs`, and that
     `.feature`-only edits do not invalidate the build without the
     `include_str!` guard.
4. Update [the roadmap](../roadmap.md): mark `0.5.1.1` as `- [x]`, and replace
   the now-satisfied sub-bullet "Use git `rstest-bdd` dependencies against
   `main` until v0.6.0-beta3 is published" with the decision actually taken.
   Leaving the stale instruction in place would misdirect `0.5.1.2`.
5. Add the new crate README to [the contents index](../contents.md) only if the
   index lists crate-level READMEs; it currently does not, so skip unless that
   changes.

### Stage G: gates and delivery (Milestone 6)

Full gate run, then push and open a draft pull request.

## Concrete steps

Run everything from the repository root:
`/home/leynos/.lody/repos/github---leynos---skyjoust/worktrees/df174b36-c975-4b56-ac05-70fc5938c151`.

Log every gate through `tee`, because long output is truncated by the
environment:

```bash
export LOGBASE="/tmp/\$ACTION-skyjoust-$(git branch --show-current).out"
```

Confirm the branch first:

```bash
git branch --show-current
```

Expected:

```plaintext
0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member
```

### Milestone 1 — record the decision

Write the ADR, amend the design document, update the contents index, then:

```bash
make fmt 2>&1 | tee /tmp/markdownfmt-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

Commit with a file-based message:

```bash
COMMIT_MSG_DIR=$(mktemp -d)
cat > "$COMMIT_MSG_DIR/msg" <<'EOF'
Record the Bevy BDD harness crate decision as ADR 006

Add ADR 006 covering in-tree incubation of
`rstest-bdd-harness-bevy`, the extraction contract, and the
dependency decisions taken for roadmap task 0.5.1.1.

Amend the harness design document: `rstest-bdd` 0.6.0-beta3 is
published, so the crate uses published caret requirements rather
than git dependencies against `main`. Note the beta4-only APIs
that are unavailable under beta3.
EOF
git add -A && git commit -F "$COMMIT_MSG_DIR/msg"
```

### Milestone 2 — red

Add the workspace member, manifest, and the three source and test files
described in Stage C, then:

```bash
cargo test -p rstest-bdd-harness-bevy 2>&1 | tee /tmp/red-skyjoust-$(git branch --show-current).out
```

Expected (abridged) — the run must fail at compilation with these symbols
named:

```plaintext
error[E0432]: unresolved import `super::bare_app`
 --> crates/rstest-bdd-harness-bevy/src/profile_tests.rs
error[E0432]: unresolved import `super::minimal_app`
 --> crates/rstest-bdd-harness-bevy/src/profile_tests.rs
error: could not compile `rstest-bdd-harness-bevy` (lib test) due to 2 previous errors
```

If the failure is anything else — a missing dependency, a manifest parse error,
a Bevy feature error — fix that first and re-run until the failure is exactly
the two unresolved imports. Do not proceed to Milestone 3 before then. Record
the observed text in `Artefacts and notes`.

Commit the red state so it is visible in history.

### Milestone 3 — green

Implement `profile.rs` and wire `lib.rs`, then:

```bash
cargo test -p rstest-bdd-harness-bevy 2>&1 | tee /tmp/green-skyjoust-$(git branch --show-current).out
```

Expected:

```plaintext
running 4 tests
test profile::tests::bare_app_adds_no_plugins ... ok
test profile::tests::minimal_app_adds_time_plugin ... ok
test profile::tests::minimal_app_counts_frames::case_1 ... ok
test profile::tests::minimal_app_counts_frames::case_2 ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests rstest_bdd_harness_bevy
running 2 tests
test crates/rstest-bdd-harness-bevy/src/profile.rs - profile::bare_app (line 10) ... ok
test crates/rstest-bdd-harness-bevy/src/profile.rs - profile::minimal_app (line 23) ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Then the refactor step: run the wider gates before moving on.

```bash
make check-fmt 2>&1 | tee /tmp/check-fmt-skyjoust-$(git branch --show-current).out
make lint 2>&1 | tee /tmp/lint-skyjoust-$(git branch --show-current).out
```

Commit.

### Milestone 4 — behavioural, property, and boundary coverage

Add the four files from Stage E, then:

```bash
cargo test -p rstest-bdd-harness-bevy 2>&1 | tee /tmp/behave-skyjoust-$(git branch --show-current).out
```

Expected additions to the run:

```plaintext
     Running tests/headless_scenario.rs
running 1 test
test minimal_app_advances_one_tick ... ok

     Running tests/extraction_boundary.rs
running 2 tests
test manifest_declares_no_game_dependencies ... ok
test guard_detects_a_forbidden_dependency ... ok

     Running tests/tick_properties.rs
running 1 test
test frame_count_tracks_update_calls ... ok
```

Prove the rebuild guard works: edit a word in the feature file's `Then` line so
it no longer matches the step pattern, re-run, and observe a failure rather
than a stale pass; then revert.

```bash
make lint 2>&1 | tee /tmp/lint-skyjoust-$(git branch --show-current).out
```

Commit.

### Milestone 5 — documentation

Write the crate README and update the repository layout, developer's guide, and
roadmap, then:

```bash
make fmt 2>&1 | tee /tmp/markdownfmt-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

Commit.

### Milestone 6 — full gates and delivery

Delegate the full gate run to the `scrutineer` sub-agent. It runs the gates
sequentially — sequential execution is required so the build cache is effective
— captures each gate's output under `/tmp`, and returns a bounded report. The
gates are, in order:

```bash
make check-fmt
make check-state-graphs
make markdownlint
make lint
make test
```

When `scrutineer` reports a failure, read the cited log rather than re-running
the gate; re-run only after applying a fix.

Then push and open the draft pull request:

```bash
git push -u origin 0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member
```

The pull request title must carry the roadmap number in parentheses — for
example `Add rstest-bdd-harness-bevy as a workspace member (0.5.1.1)` — and the
body must mention this ExecPlan by path and end with a `## References` section
linking the Lody session.

## Validation and acceptance

### Red-green-refactor evidence

- **Red.** `cargo test -p rstest-bdd-harness-bevy` fails to compile with
  `unresolved import super::bare_app` and `unresolved import
  super::minimal_app`, raised from
  `crates/rstest-bdd-harness-bevy/src/profile_tests.rs`. The failure must name
  those two symbols and nothing else.
- **Green.** After implementing `profile.rs` and wiring `lib.rs`, the same
  command reports `4 passed` for the library tests and `2 passed` for doctests,
  with zero failures.
- **Refactor.** `make check-fmt` prints nothing and exits zero; `make lint`
  completes `cargo doc`, `cargo clippy --workspace --all-targets --all-features
  -- -D warnings`, and the Whitaker Dylint suite with no diagnostics.

### The BDD feature specification

The behavioural work is driven by
`crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature`:

```gherkin
Feature: Headless Bevy scaffolding

  Scenario: A minimal application advances one tick
    Given a minimal headless Bevy application
    When the schedule advances once
    Then the frame count reads 1
```

The binding lives in
`crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs`. Before that file
compiles, `cargo test -p rstest-bdd-harness-bevy --test headless_scenario`
fails; afterwards it reports `test minimal_app_advances_one_tick ... ok`.

Keep this specification synchronized with the implementation. In `0.5.1.2` the
same feature file should be re-bound through `#[scenario(harness =
BevyHarness)]` with the steps taking `#[from(rstest_bdd_harness_context)]
scenario: &BevyScenario`, replacing the `RefCell<App>` fixture. The Gherkin
text should not need to change — which is itself a useful check that the
harness API is doing real work.

### Behavioural acceptance

A reader who has never seen this repository can verify the change like this.

1. Confirm the crate is a workspace member:

   ```bash
   cargo metadata --no-deps --format-version 1 | grep -c rstest-bdd-harness-bevy
   ```

   Expect a non-zero count.

2. Confirm the crate cannot reach Skyjoust or Lille:

   ```bash
   cargo tree -p rstest-bdd-harness-bevy -e normal,dev --prefix none \
     | sort -u | grep -Ei '^(skyjoust|lille)' || echo "clean"
   ```

   Expect `clean`.

3. Confirm the Bevy graph is headless — no renderer, window, or GPU crates:

   ```bash
   cargo tree -p rstest-bdd-harness-bevy -e normal --prefix none \
     | sort -u | grep -E 'wgpu|winit|bevy_render|bevy_window|bevy_asset' \
     || echo "headless"
   ```

   Expect `headless`.

4. Run the crate's tests and see a Gherkin scenario drive a Bevy tick:

   ```bash
   cargo test -p rstest-bdd-harness-bevy
   ```

   Expect `minimal_app_advances_one_tick ... ok` among the results, with zero
   failures across the library tests, the three integration targets, and the
   doctests.

### Quality criteria (what "done" means)

- Tests: `make test` passes with no failures. The new crate contributes four
  unit tests, one behavioural scenario, one property test, two
  extraction-boundary tests, and two doctests.
- Lint and typecheck: `make check-fmt` and `make lint` pass with no
  diagnostics. `make lint` includes `cargo doc` under
  `RUSTDOCFLAGS=-D warnings`, so rustdoc warnings are failures.
- Documentation: `make markdownlint` and `make nixie` pass. `make spelling`
  passes without editing the generated `typos.toml`.
- State graphs: `make check-state-graphs` passes (unchanged by this work, but
  part of the commit gate).
- Boundary: `cargo tree` shows no `skyjoust`, no `lille`, and no Bevy
  rendering, windowing, or asset crates.
- Coverage: the new crate's executable lines are exercised; the 80% patch
  target in `codecov.yml` is met.

### Quality method (how we check)

- `scrutineer` runs the full gate sequence and returns a bounded report with
  log paths.
- The three `cargo tree` and `cargo metadata` commands above are run by hand
  and their output pasted into `Artefacts and notes`.
- The feature-file rebuild guard is proved by deliberately breaking a step
  match and observing a failure, then reverting.
- The diff is reviewed against `Constraints` file by file before the pull
  request is opened.

## Idempotence and recovery

Every step is re-runnable. `cargo test`, `make lint`, and `make check-fmt` are
read-only with respect to tracked files, apart from `make fmt`, which rewrites
formatting deterministically.

Adding the crate is additive: nothing existing is deleted. If the work must be
abandoned, revert the commits and remove
`"crates/rstest-bdd-harness-bevy"` from the root `Cargo.toml` `members` array;
the deleted directory leaves no residue. `Cargo.lock` will shrink again on the
next resolve.

The first `cargo` command after adding Bevy downloads roughly 98 crates and
compiles them; this takes minutes, not hours, and is cached thereafter. If
another Cargo job holds the shared package-cache lock, wait for it rather than
creating a separate cache.

If a gate fails mid-milestone, fix forward and re-run that gate; do not stack
further changes on a red gate. If three attempts do not clear it, stop and
escalate per `Tolerances`.

Leave no scratch directories in the repository. Probe crates used during
planning live under `~/.cache/`, outwith the working tree.

## Artefacts and notes

### Evidence: the dependency stack works end to end

A probe crate reproducing this plan's exact manifest, lint tables,
`clippy.toml`, `.rustfmt.toml`, and pinned toolchain was built at
`~/.cache/shape-probe` during planning. Its result:

```plaintext
test profile::tests::bare_app_adds_no_plugins ... ok
test profile::tests::minimal_app_counts_frames::case_1 ... ok
test profile::tests::minimal_app_adds_time_plugin ... ok
test profile::tests::minimal_app_counts_frames::case_2 ... ok
test result: ok. 4 passed; 0 failed

test minimal_app_advances_one_tick ... ok
test result: ok. 1 passed; 0 failed

test crates/rstest-bdd-harness-bevy/src/profile.rs - profile::bare_app (line 10) ... ok
test crates/rstest-bdd-harness-bevy/src/profile.rs - profile::minimal_app (line 23) ... ok
test result: ok. 2 passed; 0 failed
```

### Evidence: the harness contract works under published beta3

A second probe implemented a minimal `HarnessAdapter` with
`type Context = BevyScenario` wrapping `Rc<RefCell<App>>`, selected it with
`#[scenario(..., harness = ProbeHarness)]`, and had steps take
`#[from(rstest_bdd_harness_context)] scenario: &BevyScenario`. It passed
against published `0.6.0-beta3`, confirming the whole of design §§4–5 is
implementable without a git dependency:

```plaintext
test harness_scenario ... ok
test result: ok. 1 passed; 0 failed
```

This is evidence for `0.5.1.2`, not work to be repeated here.

### Evidence: the `expect_used` trap in step functions

```plaintext
error: used `expect()` on an `Option` value
  --> crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs:22:21
   |
22 |     APP.with(|slot| slot.borrow_mut().as_mut().expect("app is initialised").update());
   = note: requested on the command line with `-D clippy::expect-used`
```

Resolved by replacing the `thread_local!` `Option` with an `rstest` `#[fixture]`
returning `RefCell<App>`.

### Evidence: Bevy dependency-graph sizes

| Bevy dependency declaration                                            | Crates in graph |
| ---------------------------------------------------------------------- | --------------- |
| `{ version = "0.17.3", default-features = false }`                     | 98              |
| `{ version = "0.17.3", default-features = false, features = ["std"] }` | 126             |
| `"0.17.3"` (Bevy defaults)                                             | 428             |

*Table 1: Resolved dependency-graph size for each Bevy declaration, measured
with `cargo tree -e normal --prefix none | sort -u | wc -l`.*

### Note on `beta3` message wording

Published `0.6.0-beta3` generated code panics with
`harness failed to initialise scenario: {err}` — spelled `initialise`, not
`initialize`, which `main` uses. Any future test that asserts on that string
must match the version in use. This milestone asserts on no such string.

## Interfaces and dependencies

### The manifest

Create `crates/rstest-bdd-harness-bevy/Cargo.toml` exactly as follows. The
package name is hyphenated to match the extraction target; the directory is
hyphenated for the same reason.

```toml
[package]
name = "rstest-bdd-harness-bevy"
version = "0.1.0"
edition = "2024"
description = "Headless Bevy harness adapter for rstest-bdd behavioural tests."
license = "MIT OR Apache-2.0"

[lints]
workspace = true

[dependencies]
bevy = { version = "0.17.3", default-features = false }
rstest-bdd-harness = "0.6.0-beta3"

[dev-dependencies]
googletest = "0.14.3"
pretty_assertions = "1"
proptest = "1"
rstest = "0.26.1"
rstest-bdd = "0.6.0-beta3"
rstest-bdd-macros = "0.6.0-beta3"
```

Notes on each entry:

- `bevy` is a normal dependency because the library builds `App` values.
  `default-features = false` is load-bearing; see `Risks` and `Decision log`.
- `rstest-bdd-harness` is a normal dependency because `lib.rs` re-exports its
  contract types, mirroring `rstest-bdd-harness-gpui` and
  `rstest-bdd-harness-tokio`. From `0.5.1.2` it also backs `BevyHarness`.
- `rstest-bdd` and `rstest-bdd-macros` are dev-dependencies: the crate's own
  behavioural tests use them, but the library does not.
- `rstest 0.26.1` matches what `rstest-bdd 0.6.0-beta3` expects, and is
  deliberately different from the root package's `rstest = "0.18"`.
- `tracing` is **not** yet declared. It becomes a normal dependency in
  `0.5.1.2`, where the panic-diagnostic path of design §6 needs it. Adding it
  now would leave it unused.

Add the member to the root `Cargo.toml`:

```toml
[workspace]
members = [
    ".",
    "crates/rstest-bdd-harness-bevy",
    "crates/skyjoust_stateright_validator",
]
resolver = "3"
```

### The public API

In `crates/rstest-bdd-harness-bevy/src/lib.rs`:

```rust
//! Reusable headless Bevy harness scaffolding for `rstest-bdd` behavioural
//! tests.
//!
//! This crate incubates in the Skyjoust workspace but depends on no Skyjoust or
//! Lille code, so it can move to its own repository as a directory move plus a
//! dependency rewire. Game-specific setup belongs in downstream profile types,
//! never here.

mod profile;

pub use profile::{bare_app, minimal_app};
pub use rstest_bdd_harness::{
    AttributePolicy, HarnessAdapter, HarnessError, HarnessResult, ScenarioMetadata,
    ScenarioRunRequest, ScenarioRunner, TestAttribute, tracing,
};
```

In `crates/rstest-bdd-harness-bevy/src/profile.rs`, these two functions must
exist at the end of Milestone 3:

```rust
/// Builds an empty Bevy application with no plugins added.
#[must_use]
pub fn bare_app() -> bevy::app::App;

/// Builds a headless Bevy application carrying only `MinimalPlugins`.
#[must_use]
pub fn minimal_app() -> bevy::app::App;
```

Both carry `///` documentation with a runnable example, per `AGENTS.md`.
`bare_app` is the seed of the design's `BareBevyProfile`; `minimal_app` is the
seed of `MinimalBevyProfile`. In `0.5.1.2` they become the bodies of the
corresponding `BevyProfile::configure` implementations.

### The file layout

The design's §9 layout is the destination. This milestone creates the subset it
needs; `0.5.1.2` adds `context.rs`, `harness.rs`, `panic.rs`, and `policy.rs`.

```plaintext
crates/rstest-bdd-harness-bevy/
|-- Cargo.toml
|-- README.md
|-- src/
|   |-- lib.rs
|   |-- profile.rs
|   `-- profile_tests.rs
`-- tests/
    |-- extraction_boundary.rs
    |-- headless_scenario.rs
    |-- tick_properties.rs
    `-- features/
        `-- headless_scenario.feature
```

`profile_tests.rs` sits beside `profile.rs` and is wired in with the
repository's existing idiom, as used throughout the validator crate:

```rust
#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
```

### Test shapes

Unit tests in `src/profile_tests.rs` use `#[gtest]` above `#[rstest]` so
`googletest` assertions have a test context:

```rust
//! Unit tests for the headless application constructors.

use bevy::diagnostic::FrameCount;
use googletest::prelude::*;
use rstest::rstest;

use super::{bare_app, minimal_app};

#[gtest]
#[rstest]
#[case(0)]
#[case(3)]
fn minimal_app_counts_frames(#[case] ticks: u32) {
    let mut app = minimal_app();
    for _ in 0..ticks {
        app.update();
    }
    expect_that!(app.world().resource::<FrameCount>().0, eq(ticks));
}
```

Cover both the happy path (`minimal_app` adds `TimePlugin`; ticking advances
`FrameCount`) and the negative case (`bare_app` adds no plugins, so `TimePlugin`
is absent).

The scenario binding in `tests/headless_scenario.rs` uses an `rstest` fixture
rather than a thread-local, so no step needs `.expect()`, and carries the
rebuild guard:

```rust
const _: &str = include_str!("features/headless_scenario.feature");

#[fixture]
fn app() -> RefCell<App> { RefCell::new(minimal_app()) }

#[when("the schedule advances once")]
fn when_schedule_advances_once(app: &RefCell<App>) { app.borrow_mut().update(); }

#[scenario(path = "tests/features/headless_scenario.feature", index = 0)]
fn minimal_app_advances_one_tick(app: RefCell<App>) {}
```

Step functions use `assert_that!`, which panics directly and needs no
`#[gtest]` context. The fixture parameter must appear on the `#[scenario]`
function as well as on the steps, so `rstest` injects it.

The property test in `tests/tick_properties.rs` states the invariant that
`0.5.1.2`'s `update_times` must preserve:

```rust
proptest! {
    #[test]
    fn frame_count_tracks_update_calls(ticks in 0_u32..=32) {
        let mut app = minimal_app();
        for _ in 0..ticks {
            app.update();
        }
        prop_assert_eq!(app.world().resource::<FrameCount>().0, ticks);
    }
}
```

Use `prop_assert_eq!`, which returns an error rather than panicking, keeping
the test free of denied panic-prone operations.

The boundary test in `tests/extraction_boundary.rs` encodes the roadmap's
success criterion as a pure, testable predicate:

```rust
/// Returns the forbidden game-crate names named anywhere in a manifest.
fn forbidden_dependencies(manifest: &str) -> Vec<&'static str>;
```

Test it twice: against
`include_str!("../Cargo.toml")`, which must yield an empty vector, and against
a synthetic manifest string that names `skyjoust`, which must yield exactly
that name. The second case is what proves the guard can fail — without it the
first assertion is vacuous. Use `pretty_assertions::assert_eq` for the vector
comparison so a regression prints a readable diff.

### Documents to change

| Path                                                               | Change                                                         |
| ------------------------------------------------------------------ | -------------------------------------------------------------- |
| `Cargo.toml`                                                       | Add the workspace member.                                      |
| `docs/adr/006-in-tree-incubation-of-the-bevy-bdd-harness-crate.md` | New ADR.                                                       |
| `docs/rstest-bdd-harness-bevy-design.md`                           | Amend §3, §9, §13; add the ADR pointer.                        |
| `docs/contents.md`                                                 | Index ADR 006 and this ExecPlan.                               |
| `docs/repository-layout.md`                                        | Tree sketch, path responsibilities, workspace membership note. |
| `docs/developers-guide.md`                                         | Amend §2; add a harness-crate conventions section.             |
| `docs/roadmap.md`                                                  | Tick `0.5.1.1`; correct the git-dependency sub-bullet.         |
| `crates/rstest-bdd-harness-bevy/README.md`                         | New crate README.                                              |
| `docs/users-guide.md`                                              | No change — see `Decision log`.                                |

*Table 2: Documents this plan changes, and what changes in each.*

## Revision note

Initial draft, 2026-08-15. Written after a reconnaissance pass over the
repository's gates, lint baseline, crate conventions, and documentation set,
and after four executable probes that verified the Bevy feature selection, the
published `rstest-bdd` version, the harness context contract, and the estate
lint interaction with `rstest-bdd` step functions. Those probes retired the two
highest-severity risks before implementation begins and produced three
non-obvious findings now recorded in `Surprises & discoveries`. The remaining
work is the six milestones above.

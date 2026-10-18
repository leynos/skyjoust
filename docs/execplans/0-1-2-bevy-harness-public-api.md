# Implement the reusable Bevy harness public API

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT

Approval gate: this plan must be explicitly approved by the maintainer before
any implementation milestone (EP-M1 onwards) begins. Silence is not approval.

Roadmap item: `0.1.2` in `docs/roadmap.md`. Branch:
`0-1-2-bevy-harness-public-api`.

## Purpose / big picture

After this change, a Skyjoust (or Lille) test author can write a Gherkin
scenario, bind it with
`#[scenario(path = "...", harness = BevyHarness, attributes = BevyAttributePolicy)]`,
and have every step receive the same headless Bevy application through the
reserved `rstest-bdd` fixture key:

```rust,no_run
#[when("the schedule advances {count:u32} times")]
fn schedule_advances(
    #[from(rstest_bdd_harness_context)] scenario: &BevyScenario,
    count: u32,
) {
    scenario.update_times(count);
}
```

The harness builds a fresh Bevy `App` per scenario, lets a profile type
configure it, hands the steps a cloneable `BevyScenario` wrapper, runs the
profile's cleanup hook when the scenario finishes (including when a step
panics), and re-raises any step panic with the feature path, scenario name, and
one-based feature-file line in the message. Game-specific setup lives in a
downstream profile type, never in the reusable crate.

Observable success: `make test` runs new behavioural scenarios whose generated
tests construct the harness through `<Harness as Default>::default()` and whose
steps receive `&BevyScenario` through `rstest_bdd_harness_context`; the existing
`tests/features/headless_scenario.feature` passes unchanged after being
re-bound through the harness; and a deliberately panicking scenario fails with
a message such as:

```plaintext
rstest-bdd-harness-bevy scenario panicked: harness=rstest_bdd_harness_bevy::harness::BevyHarness<...>,
feature=tests/features/bevy_scenario.feature:12, scenario="...": boom
```

## Signposts: documents and skills

Read these before starting. Line ranges were current at plan drafting
(2026-10-18) and may drift.

Repository documents:

- `docs/rstest-bdd-harness-bevy-design.md` §§4–7 (lines 117–257) is the
  governing design; §§9–12 cover layout, verification, extraction, and risks.
- `docs/adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md` —
  extension-seam rule, extraction contract, dependency strategy, headless
  guarantee.
- `docs/roadmap.md` lines 97–132 — tasks `0.1.2` to `0.1.5` and their split of
  responsibilities.
- `docs/rstest-bdd-v0.6.0-users-guide.md`: fixtures and `#[from]` (166–246),
  step return values (407–453), `#[scenario]` arguments (871–883), harness
  adapters and attribute policy (894–1005), the third-party harness cookbook
  (1007–1195, the closest upstream specification of this task), GPUI harness
  (1235–1286), fixture key versus parameter name (1661–1671), pedantic lint
  profile (1692–1721), harness core APIs (2331–2444). Cite this file, not the
  stale duplicate `docs/rtest-bdd-users-guide.md`.
- `docs/rstest-bdd-v0.6.0-migration-guide.md`: custom harness adapters
  (86–146), harness dependency matrix (304–358), harness context (407–471).
- `docs/rust-testing-with-rstest-fixtures.md` §V.B–V.C (524–623): `#[once]` is
  unsuitable for per-scenario apps; `#[from]` renaming semantics.
- `docs/rust-doctest-dry-guide.md` §§2.3–2.4, 3.1–3.2, 6.1–6.2: no `.unwrap()`
  or `.expect()` in doctests, hidden setup lines, panic paths belong in unit
  tests rather than doctests.
- `docs/reliable-testing-in-rust-via-dependency-injection.md` §6 (159–221):
  never mutate the process environment in tests; this plan needs no environment
  access.
- `docs/complexity-antipatterns-and-refactoring-strategies.md` §4.A–4.C
  (232–371): keep the harness lifecycle split into small functions; the
  `clippy.toml` cognitive-complexity threshold is 9.
- `docs/skyjoust-technical-design.md` §4 (102–127), §5 (129–144), §14
  (408–440): simulation ticks never read wall-clock time, so scenarios advance
  only through explicit `App::update` calls; gate list.
- `docs/skyjoust-state-graphs-README.md`: no harness guidance; relevant only to
  roadmap `0.1.4`, which this plan must not implement.
- `docs/developers-guide.md` §2.1 (49–150) — existing harness traps; §7.3 —
  fixture-expansion lint handling.
- `docs/users-guide.md` §4 (182–221) — the current headless helper guidance
  this plan extends.
- `docs/documentation-style-guide.md` — ADR template (355–490) and API doc
  comment rules (526–561).
- `docs/execplans/0-1-1-add-rstest-bdd-harness-bevy-workspace-member.md` — the
  prior plan; its `Surprises & discoveries` (693–936) records lint traps this
  plan inherits.
- `docs/ortho-config-users-guide.md` does not exist in this repository (a
  repository-wide search found no `*ortho*` file). This work reads no
  configuration, so it has no bearing on this plan; do not create it.

Skills to load during implementation: `rust-router` first, then
`rust-types-and-apis` (generic harness with a defaulted profile parameter,
`PhantomData`, manual trait impls), `rust-errors` (panic boundary, mapping onto
`HarnessError`), `rust-unit-testing` (rstest tables, `googletest`, `insta`),
`proptest` (tick properties), `rust-verification` (method selection and
mutation testing), `execplans` (keep this document current), `commit-message`,
`en-gb-oxendict-style` for documentation, and `codegraph-mcp` for navigation.
`verus` and `kani` are deliberately not loaded; see `Verification plan`.

## Constraints

- The harness crate `crates/rstest-bdd-harness-bevy` must not depend on
  `skyjoust`, `skyjoust-stateright-validator`, `skyjoust-test-macros`, or
  `lille`, in normal or development graphs (ADR 009 extraction contract).
  `tests/extraction_boundary.rs` must keep passing.
- No Skyjoust profile, resource, or gameplay module may be added to the
  harness crate. `SkyjoustBevyProfile` belongs to roadmap `0.1.4`.
- Bevy stays at `0.19.1` with `default-features = false` and
  `features = ["std"]`; no renderer, window, asset, audio, or `multi_threaded`
  feature may be enabled.
- No new crate may enter `Cargo.lock`. Adding `insta` to the harness crate's
  `[dev-dependencies]` is permitted because it already resolves in the
  workspace lockfile (via `crates/skyjoust_stateright_validator`) and the task
  brief authorizes it. Any other new dependency is an escalation.
- `rstest-bdd`, `rstest-bdd-macros`, and `rstest-bdd-harness` stay on the
  published stable `0.6.0` family with implicit caret requirements. No git
  dependencies.
- The public API must match design §§4–5 names and signatures:
  `BevyScenario`, `BevyProfile`, `BevyHarness<P = MinimalBevyProfile>`,
  `BareBevyProfile`, `MinimalBevyProfile`, and `BevyAttributePolicy`, with
  `BevyScenario::{metadata, with_app, with_world, update, update_times}`.
  `BevyScenario` wraps an `App`, not a raw `World`.
- `add_minimal_plugins` and `minimal_app` remain public; 0.1.1 promised they
  survive.
- No `unsafe` code (`unsafe_code = "forbid"`), no `Send`/`Sync` forwarding.
- No lint suppression except as a documented last resort with a tight
  `#[expect(..., reason = "...")]`; the nested
  `rstest-bdd-harness-bevy-test-macros` crate remains the only fixture-lint
  mechanism.
- Every source file stays at or under 400 lines; every module starts with a
  `//!` comment.
- Tests must not mutate process environment variables.
- Do not change `.config/nextest.toml`, `tests/nextest_policy.rs`, or CI
  workflow timeouts. A new `trybuild` runner is out of scope (roadmap `0.1.3`).
- Do not implement roadmap `0.1.3`'s remaining scope (tracing-capture
  assertions, the harness-alias `trybuild` compile-pass fixture), `0.1.4`, or
  `0.1.5`'s extraction documentation beyond what this API change makes
  inaccurate.

## Tolerances (exception triggers)

- Scope: if implementation needs more than 30 changed paths or more than 1,600
  net added lines (code, tests, and documentation together), stop and escalate.
- Interface: if any design §5 signature must change (for example `configure`
  needs `&mut self`, or `with_app` must return a `Result`), stop, record a
  proposed deviation, set status `BLOCKED`, and escalate.
- Dependencies: if anything beyond adding `insta` to the harness crate's
  development dependencies is needed (for example `tracing-subscriber` or
  `serial_test`), stop and escalate.
- Macro contract: if `#[scenario(harness = BevyHarness, ...)]` with the
  defaulted generic parameter, or with a type alias, fails to compile, stop and
  escalate with the diagnostic; do not work around it with wrapper newtypes.
- Iterations: if a focused test or gate still fails after three fix attempts
  for the same cause, stop and escalate.
- Time: if a milestone takes more than three hours of active work (excluding
  compile-admission queueing), stop and escalate.
- Gate cost: the 0.1.1 cost exception (19-minute warm `make all`, 6.7 GB
  `target/`) was approved for that task only. Record each gate's duration; if a
  warm `make test` exceeds 45 minutes without queueing being the evident cause,
  or `target/` grows by more than 1 GB, escalate.
- Ambiguity: if the design and this plan disagree, or a reviewer decision in
  `Decision log` turns out to be unimplementable, stop and present options.

## Risks

- Risk: `#[scenario(harness = BevyHarness)]` relies on the defaulted generic
  parameter being applied in `<BevyHarness as Default>::default()` and
  `<BevyHarness as HarnessAdapter>::Context`. Severity: medium. Likelihood: low
  (type-position defaults apply to bare paths). Mitigation: EP-M3 binds
  scenarios through both the bare name and a type alias
  (`type BareHarness = BevyHarness<BareBevyProfile>;`); the tolerance above
  escalates on failure.
- Risk: the `rstest-bdd` macro resolves `::rstest_bdd_harness` for any
  non-first-party harness, so consumers must declare `rstest-bdd-harness`
  directly. Severity: low. Likelihood: certain. Mitigation: the harness crate
  already declares it as a normal dependency (resolves for its own tests);
  document the requirement for downstream crates in the users' guide and README.
- Risk: `HarnessError` has a single `#[non_exhaustive]` variant,
  `RuntimeBuildFailed(std::io::Error)`, whose display text begins "failed to
  build runtime". Profile `configure` and `cleanup` failures must be mapped
  onto it, and the generated test reports them as "harness failed to initialize
  scenario", even for cleanup. Severity: low. Likelihood: certain. Mitigation:
  document the mapping idiom
  (`HarnessError::RuntimeBuildFailed(std::io::Error::other(message))`) in
  rustdoc and the users' guide; do not add a helper API (see `Decision log`).
- Risk: a Bevy system that panics inside `App::update` leaves the app without
  its `Main` schedule (Bevy removes schedules temporarily and has no reinsert
  guard), so any later `update()` panics with "The schedule with the label Main
  was not found". Profile cleanup on the panic path may hit this. Severity:
  medium. Likelihood: medium. Mitigation: the panic path runs cleanup inside
  its own `catch_unwind`, logs any cleanup panic, and resumes the original
  scenario panic; rustdoc warns that the app is unusable for further updates
  after a system panic.
- Risk: workspace lints (`panic_in_result_fn`, `missing_panics_doc`,
  `missing_const_for_fn`, `must_use_candidate`, `shadow_*`, `doc_markdown`,
  `cognitive_complexity` threshold 9) reject first-draft code. Severity: low.
  Likelihood: high. Mitigation: small functions; `# Panics` sections on every
  public accessor; keep `panic!` out of `Result`-returning functions (the
  harness re-raises with `std::panic::resume_unwind`, a function, not a macro);
  run `make lint` at every milestone.
- Risk: dropping the `App` while unwinding runs `World::drop`, which applies
  queued commands; a panicking queued command would abort the test process.
  Severity: low. Likelihood: low. Mitigation: record as a residual gap; no toy
  profile queues panicking commands.
- Risk: `cargo mutants` against the crate is slow under the shared compile
  queue. Severity: low. Likelihood: medium. Mitigation: scope the run to the
  three new modules and run it once, at EP-M5.
- Risk: compile-admission queueing makes gates slow and noisy. Severity: low.
  Likelihood: high. Mitigation: run gates sequentially through `scrutineer`,
  tee logs under `/tmp`, and never start a second build to "help".

## Progress

- [x] (2026-10-18) Branch renamed from `session/23ed6522` to
  `0-1-2-bevy-harness-public-api`.
- [x] (2026-10-18) Reconnaissance: `rstest-bdd` 0.6.0 harness API and macro
  codegen, Bevy 0.19.1 `App` semantics, repository conventions and gates,
  reference documents (four parallel Wyvern agents).
- [x] (2026-10-18) Prior-art research: Bevy's own `tests/how_to_test_apps.rs`
  (headless `MinimalPlugins` plus explicit `app.update()`), upstream GPUI and
  Tokio harnesses in the local registry.
- [x] (2026-10-18) First draft of this ExecPlan.
- [ ] Community-of-experts design review and revision.
- [ ] Maintainer approval of this plan.
- [ ] EP-M0: baseline and ADR 010 draft.
- [ ] EP-M1: attribute policy.
- [ ] EP-M2: scenario context, profiles, and harness normal path.
- [ ] EP-M3: panic and cleanup path.
- [ ] EP-M4: harness-backed behavioural scenarios and tick properties.
- [ ] EP-M5: documentation, mutation check, roadmap, and final gates.

## Surprises & discoveries

- Observation: Bevy's `App::update` never calls `Plugin::finish` or
  `Plugin::cleanup`; only runners do. Evidence:
  `bevy_app-0.19.1/src/app.rs:158-164` versus `run_once` at 1531–1542. Impact:
  harmless for `MinimalPlugins`, whose members override only `build`; the
  harness does not call `finish`/`cleanup`, keeping `add_plugins` legal in
  steps. A downstream profile needing them calls them in `configure`.
- Observation: `HarnessAdapter` imposes only `Context: Any`; no `Send`,
  `Sync`, or `Clone`. The macro moves the context into a step context cell and
  drops it when the runner closure ends, so the harness cannot recover it
  except through a handle it kept. Evidence:
  `rstest-bdd-harness-0.6.0/src/adapter.rs:35-49`,
  `rstest-bdd-macros-0.6.0/src/codegen/scenario/runtime/harness.rs:95-122`.
  Impact: confirms design §4's `Rc<RefCell<App>>` shared handle; the harness
  passes a clone and keeps one for cleanup.
- Observation: unknown third-party `attributes = ...` paths always expand to
  `#[rstest::rstest]` only; the policy contributes only a trait-bound
  assertion. Evidence:
  `rstest-bdd-macros-0.6.0/src/codegen/scenario/test_attrs.rs:125-147`. Impact:
  `BevyAttributePolicy` is a marker, as design §7 states.

## Decision log

- Decision: implement design §6's panic and cleanup path in `0.1.2`, with the
  focused tests that drive it, rather than deferring it wholesale to `0.1.3`.
  Rationale: the roadmap item cites design §§4–7, design §9 assigns
  `src/panic.rs` to `0.1.2`, and test-first delivery forbids landing untested
  behaviour. Roadmap `0.1.3` keeps the broader validation scope: tracing-record
  capture, the harness-alias `trybuild` fixture, and any further panic-path
  behavioural coverage. Date/Author: 2026-10-18, planning agent.
- Decision: pin the augmented panic message format with an `insta` snapshot in
  `0.1.2`. Rationale: the format is introduced here and has three payload
  classes (`&'static str`, `String`, opaque); 0.1.1 named it "the artefact
  worth pinning". `insta` already resolves in the lockfile. Date/Author:
  2026-10-18, planning agent.
- Decision: Verus and Kani are not used; obligations are discharged by
  exhaustive enumeration of a finite lifecycle model, property tests, and a
  scoped `cargo mutants` run. Rationale: see `Verification plan`. Date/Author:
  2026-10-18, planning agent.
- Decision: `BevyScenario::new` is `pub(crate)`. Rationale: the design lists
  no public constructor; scenarios are created only by `BevyHarness::run`, and
  integration tests reach a `BevyScenario` by calling `run` with a hand-built
  `ScenarioRunRequest`, which is public `rstest-bdd-harness` API. Widening the
  constructor later is additive; narrowing it would be breaking. Date/Author:
  2026-10-18, planning agent.
- Decision: `BevyScenario` derives `Clone` and `Debug`; clones share one app.
  Rationale: the harness must keep a handle for cleanup, and the macro's
  by-value step parameter form requires `Clone`. Metadata is held behind `Rc`
  so a clone is two reference-count increments. Date/Author: 2026-10-18,
  planning agent.
- Decision: `BevyHarness<P>` stores `PhantomData<fn() -> P>` and implements
  `Default`, `Clone`, `Copy`, and `Debug` by hand, plus `pub const fn new()`.
  Rationale: derives would demand `P: Default` and friends, which uninhabited
  profile enums cannot satisfy; `fn() -> P` keeps the harness `Send + Sync` and
  covariant regardless of `P`. Date/Author: 2026-10-18, planning agent.
- Decision: lifecycle semantics where design §6 is silent: a `configure`
  error returns `Err` without running the runner or cleanup; a `configure`
  panic propagates unchanged; on the normal path a cleanup error returns `Err`
  (the scenario value is dropped) and a cleanup panic propagates unchanged; on
  the panic path both a cleanup error and a cleanup panic are logged with
  `tracing::error!` and the original augmented panic resumes. Rationale:
  cleanup must never mask the step failure (design §§6 and 12), and cleanup is
  only owed for an app that `configure` successfully produced. Date/Author:
  2026-10-18, planning agent; subject to review.
- Decision: no `HarnessError` construction helper is added. Rationale: the
  design lists no such API, the abstraction policy discourages speculative
  helpers, and the one-line idiom is documented instead. Date/Author:
  2026-10-18, planning agent.
- Decision: public doctests stay runnable, following the crate's existing
  `profile.rs` doctests and `docs/rust-doctest-dry-guide.md`, rather than the
  style guide's generic `no_run` advice. Rationale: a headless app runs in
  milliseconds and runnable examples cannot rot silently. Date/Author:
  2026-10-18, planning agent.
- Decision: record the scenario-context representation and failure contract
  as ADR 010, referenced from the design document. Rationale: once extracted,
  the `Rc<RefCell<App>>` closure-accessor context and the panic/cleanup
  contract are public API that is hard to reverse; AGENTS.md requires an ADR
  for substantive decisions. Date/Author: 2026-10-18, planning agent; subject
  to review.

## Outcomes & retrospective

Not yet started. Record milestone outcomes, gate durations, and lessons here.

## Context and orientation

Skyjoust is a Rust workspace (root `Cargo.toml`) with four members: the root
`skyjoust` package, `crates/skyjoust_stateright_validator`,
`crates/skyjoust_test_macros`, and `crates/rstest-bdd-harness-bevy` with its
nested test-only proc-macro crate `crates/rstest-bdd-harness-bevy/test-macros`.
The toolchain is pinned to `nightly-2026-03-26` in `rust-toolchain.toml`.

`rstest-bdd` is a behaviour-driven development (BDD) framework: Gherkin feature
files (`Feature`/`Scenario`/`Given`/`When`/`Then`) bind to Rust step functions,
and the `#[scenario]` attribute generates an ordinary `rstest` test per
scenario. A *harness adapter* is a type implementing
`rstest_bdd_harness::HarnessAdapter`; the generated test creates it with
`<Harness as Default>::default()` and calls `run`, passing a
`ScenarioRunRequest` that holds `ScenarioMetadata` (feature path, scenario
name, one-based line, tags) and a `ScenarioRunner` (a boxed
`FnOnce(Context) -> T` that executes the steps). A harness with a non-unit
`Context` hands that value to the runner, and steps receive it through the
reserved fixture key `rstest_bdd_harness_context`. An *attribute policy*
implements `AttributePolicy` and names the test attributes to emit; for
third-party policies the macro emits only `#[rstest::rstest]`.

Bevy is an entity-component system (ECS) game engine. An `App` owns a `World`
(entities, components, resources) plus schedules and plugins. `App::update`
runs the `Main` schedule once, which runs `Startup` schedules on the first call
only and then `First`, `PreUpdate`, `RunFixedMainLoop`, `Update`, `PostUpdate`,
and `Last`. `MinimalPlugins` adds task pools, `FrameCount` (incremented once
per update, wrapping `u32`), `TimePlugin`, and `ScheduleRunnerPlugin` (whose
`run` loops forever, so tests never call `App::run`). `App` is neither `Send`
nor `Sync`.

The harness crate today (`crates/rstest-bdd-harness-bevy`):

- `src/lib.rs` declares `mod profile;` and re-exports `bevy`, `tracing`,
  `add_minimal_plugins`, `minimal_app`, and ten `rstest_bdd_harness` items.
- `src/profile.rs` holds `add_minimal_plugins(&mut App)` and `minimal_app()`;
  its unit tests are `src/profile_tests.rs`, included with
  `#[cfg(test)] #[path = "profile_tests.rs"] mod tests;`.
- `tests/headless_scenario.rs` binds `tests/features/headless_scenario.feature`
  with a plain `#[scenario]` and a `RefCell<App>` fixture (no harness yet).
- `tests/tick_properties.rs` is a `proptest` showing frame count equals the
  number of `App::update` calls for 0–32 updates.
- `tests/extraction_boundary.rs` is the manifest tripwire; `tests/macro_ui.rs`
  with `tests/ui/` holds `trybuild` fixtures (the `_ui` suffix places it in
  Nextest's serial `compile-ui` group).

Nothing in the repository yet defines any of the six API types (verified by a
repository-wide search; the only mentions are documentation and one comment in
`src/profile.rs`).

## Conformance basis

Upstream artefacts at plan drafting (base commit `2a58544`):

- Roadmap `docs/roadmap.md` task `0.1.2` (lines 97–106): RM-1 add the six
  types; RM-2 `BevyScenario` wraps an `App` and exposes `with_app`,
  `with_world`, `update`, `update_times`; RM-3 generated scenarios construct
  the harness via `<Harness as Default>::default()` and receive the wrapper via
  `rstest_bdd_harness_context`.
- Design `docs/rstest-bdd-harness-bevy-design.md` (status "Proposed companion
  design"): DS-4 architecture (§4), DS-5 public API contract (§5), DS-6 panic
  and cleanup behaviour (§6), DS-7 macro and attribute-policy constraints (§7),
  DS-9 crate layout (§9), DS-11 extraction plan (§11), DS-12 risks (§12).
- ADR 009 (Accepted 2026-08-17): ADR9-SEAM profile type is the single
  extension seam; ADR9-EXTRACT no Skyjoust or Lille dependency; ADR9-DEPS Bevy
  `0.19.1` headless, stable `rstest-bdd` 0.6.0.
- Technical design `docs/skyjoust-technical-design.md`: TD-5 simulation ticks
  never read wall-clock time, so advancement is explicit; TD-14 gates.
- Terms of Reference: none exists for this repository; none is invented.
- Governing standards: `AGENTS.md`, `docs/documentation-style-guide.md`,
  workspace lints in the root `Cargo.toml`, `clippy.toml`, `.rustfmt.toml`.

This plan's requirements and trace links:

- EP-REQ-1 (RM-1, DS-4, DS-5, DS-7): the six public types exist with the
  design's names and signatures.
- EP-REQ-2 (RM-2, DS-5, DS-12, TD-5): `BevyScenario` wraps
  `Rc<RefCell<App>>`, exposes closure accessors and explicit advancement, and
  panics with a crate-prefixed message on re-entrant access.
- EP-REQ-3 (DS-6, DS-12): profile cleanup runs on success and on step panic;
  step panics resume with harness type, feature path, scenario name, line, and
  payload text; cleanup failures never mask the step panic.
- EP-REQ-4 (RM-3, DS-7): generated scenarios construct the harness through
  `Default` and steps receive `&BevyScenario` through the reserved key, for the
  bare name and for a type alias.
- EP-REQ-5 (ADR9-SEAM, ADR9-EXTRACT, DS-11): no game code or dependency in
  the crate; profiles are the only extension seam.
- EP-REQ-6 (AGENTS.md): documentation, ADR, users' guide, developers' guide,
  and roadmap reflect the delivered API.

```plaintext
RM-1 -> DS-5 -> EP-REQ-1 -> EP-M1, EP-M2 -> policy_tests, profile_tests, context_tests, harness_tests
RM-2 -> DS-5/DS-12 -> EP-REQ-2 -> EP-M2 -> context_tests::*, tick_properties::scenario_*
DS-6 -> EP-REQ-3 -> EP-M3 -> harness_tests::lifecycle_*, panic_tests::*, snapshot
RM-3 -> DS-7 -> EP-REQ-4 -> EP-M4 -> bevy_scenario::*, headless_scenario::*
ADR9-EXTRACT -> EP-REQ-5 -> every milestone -> extraction_boundary::*, cargo tree
EP-REQ-6 -> EP-M0 (ADR 010), EP-M5 -> make markdownlint, make nixie
```

## Verification plan

### Abstract model (proof sketch first)

Model the harness run as a function of three abstract outcomes, independent of
Bevy and of `rstest-bdd` plumbing:

- configure outcome `c ∈ {Ok, Err(e), Panic(q)}`;
- runner outcome `r ∈ {Return(v), Panic(p)}`;
- cleanup outcome `k ∈ {Ok, Err(e), Panic(q)}`.

Observable result `run(c, r, k)` is a triple: the value returned or the panic
raised; the number of runner invocations; the number of cleanup invocations.
The specification is this finite table (`m(p)` is the augmented message built
from `p` and the scenario metadata; "log" means one `tracing::error!` record):

```plaintext
c        r          k          result          runner  cleanup  log
Err(e)   -          -          Err(e)          0       0        no
Panic(q) -          -          panic q         0       0        no
Ok       Return(v)  Ok         Ok(v)           1       1        no
Ok       Return(v)  Err(e)     Err(e)          1       1        no
Ok       Return(v)  Panic(q)   panic q         1       1        no
Ok       Panic(p)   Ok         panic m(p)      1       1        no
Ok       Panic(p)   Err(e)     panic m(p)      1       1        yes
Ok       Panic(p)   Panic(q)   panic m(p)      1       1        yes
```

Preconditions: the test profile uses unwinding panics (`panic = "unwind"`, the
default for tests); metadata is any `ScenarioMetadata`. Postconditions are the
table rows. The representation relation: every `BevyScenario` passed to the
runner and to cleanup refers to the single `App` that `configure` received.

### Invariants and lemmas

- EP-INV-1 configure precedence: the runner is invoked only after `configure`
  returned `Ok` on the same app.
- EP-INV-2 cleanup exactly once: if `configure` returned `Ok`, cleanup is
  invoked exactly once, after the runner terminates, on both paths; otherwise
  never.
- EP-INV-3 panic identity: if the runner panics with payload `p`, `run`
  panics with `m(p)` regardless of `k`, and `m(p)` contains the harness type
  name, feature path, one-based line, scenario name, and the payload text
  (`&str`, `String`) or the placeholder `Box<dyn Any>` for opaque payloads.
- EP-INV-4 value transparency: `r = Return(v) ∧ k = Ok ⇒ run = Ok(v)`.
- EP-INV-5 shared app identity: runner and cleanup observe the same `App`
  (state written by a step is visible to cleanup).
- EP-INV-6 tick additivity: for any sequence of `update()` and
  `update_times(n)` calls on a `MinimalBevyProfile` scenario, `FrameCount`
  equals the sum of advancements (modulo 2³²), and an `Update`-schedule counter
  system runs exactly that many times; `update_times(0)` is a no-op.
- EP-INV-7 borrow exclusivity and recovery: a re-entrant accessor call
  through any clone of the same scenario panics with the prefix
  `rstest-bdd-harness-bevy:`; after an accessor closure panics and the panic is
  caught, the scenario is borrowable again.
- EP-INV-8 profile semantics: `BareBevyProfile` adds no plugins (no
  `TimePlugin`, no `FrameCount`), while `Update` systems still run on
  `update()`; `MinimalBevyProfile` adds exactly `MinimalPlugins`.
- EP-INV-9 policy conformance: `BevyAttributePolicy::test_attributes()`
  renders exactly `["#[rstest::rstest]"]`.
- EP-INV-10 extraction: the crate's manifests declare no forbidden crate, and
  `cargo tree` shows no Skyjoust or Lille package.

Lemmas connecting these to EP-REQ-3 and EP-REQ-4 (hypotheses about proof
structure, to be refined by implementation):

- L1: the panic path's cleanup wrapper returns normally for every cleanup
  outcome (it catches cleanup panics). With EP-INV-2 this gives EP-INV-3:
  nothing between catching `p` and resuming `m(p)` can unwind.
- L2: `m` is a total function of `(type name, metadata, payload)`; it never
  panics (pure formatting).
- L3: the macro-generated test calls `run` exactly once per scenario on a
  `Default`-constructed harness (axiom A4), so EP-INV-1 to EP-INV-5 hold for
  generated scenarios, not only for hand-built requests.

### Axioms (external assumptions, not verified here)

- A1 Bevy: `App::update` runs `Main` once, including `Update` once and the
  `FrameCount` increment (`wrapping_add(1)`) once.
- A2 Rust: `RefCell` resets its borrow flag when a `RefMut` drops during
  unwinding; `catch_unwind` catches every unwinding panic.
- A3 Platform: tests build with `panic = "unwind"`. Under `panic = "abort"`
  no cleanup can run; documented, not handled.
- A4 `rstest-bdd` 0.6.0: generated tests construct the harness with
  `<H as Default>::default()`, call `run` once, inject the context by `TypeId`
  under `rstest_bdd_harness_context`, and turn `Err(HarnessError)` into a panic.
- A5 `HarnessError` exposes only `RuntimeBuildFailed(std::io::Error)`.
- A6 Bevy: a system panic during `update` loses the `Main` schedule; later
  updates panic. The harness does not repair this.

### Obligations, methods, and evidence

- Obligation: EP-INV-1, -2, -4, and the `log` column for the eight model rows.
  Method: an exhaustive parameterized `rstest` table, one case per row, driving
  `BevyHarness::<P>::run` with hand-built requests and one test profile type
  per configure/cleanup outcome. Rationale: the abstract domain is finite
  (eight rows); enumerating it is an exhaustive check of the model, stronger
  than sampling and cheaper than a prover. Payload and value classes are
  covered separately (EP-INV-3). Domain: all eight rows. Artefact:
  `crates/rstest-bdd-harness-bevy/src/harness_tests.rs`. Evidence: the focused
  `lifecycle` filter from `Concrete steps` (or `make test`) fails before
  EP-M2/EP-M3 code exists and passes after. Non-vacuity: each row asserts
  runner and cleanup counts through a `Cell`-based recorder captured by the
  runner closure and a stashed scenario clone, so a row cannot pass by never
  running the runner; the seeded faults "skip cleanup on the panic path", "run
  cleanup twice", and "drop the original payload in favour of the cleanup
  panic" must each fail at least one row (checked by hand once during EP-M3 and
  mechanically by `cargo mutants` at EP-M5).
- Obligation: EP-INV-3 and L2. Method: unit tests of the pure formatter over
  the three payload classes plus an `insta` snapshot of the full message for
  fixed metadata; one lifecycle row asserts the same text arrives through
  `run`. Rationale: the message is a stable diagnostic contract; a snapshot
  pins every field and its order. Artefact: `src/panic_tests.rs`,
  `src/snapshots/`. Evidence: `cargo insta test -p rstest-bdd-harness-bevy` is
  not required; `make test` with `INSTA_UPDATE=no` (the CI default) fails on
  drift. Non-vacuity: the snapshot uses distinct, non-default metadata values
  (feature path, name, line 12) so a formatter that drops or swaps a field
  changes the snapshot.
- Obligation: EP-INV-5. Method: lifecycle rows where a step writes a resource
  and cleanup reads it, failing the cleanup if absent. Artefact:
  `src/harness_tests.rs`. Non-vacuity: a mutant that builds a second app for
  cleanup fails the row.
- Obligation: EP-INV-6. Method: `proptest` over operation sequences (length
  0–16, each `Update` or `UpdateTimes(0..=8)`) applied to a scenario obtained
  by running `BevyHarness::<MinimalBevyProfile>` with a hand-built request,
  asserting `FrameCount` and a toy `Update` counter equal the running sum.
  Rationale: the invariant spans orderings; examples cover `0` and `1` as named
  unit cases. Domain: up to 128 updates per case, 64 cases. Artefact:
  `tests/tick_properties.rs`. Non-vacuity: the strategy guarantees
  `UpdateTimes(0)` and non-empty sequences appear (`prop_oneof!` weights,
  checked via `prop_assume!`-free generation); the seeded fault "`update_times`
  runs `count - 1` updates" must fail.
- Obligation: EP-INV-7. Method: named `#[gtest] #[rstest]` unit tests:
  nested `with_app` inside `with_app`, `update` inside `with_world` through a
  clone, and borrowability after a caught accessor panic. Artefact:
  `src/context_tests.rs`. Non-vacuity: assertions check the exact message
  prefix, so a generic `BorrowMutError` panic fails.
- Obligation: EP-INV-8 and EP-INV-9. Method: named unit tests; the upstream
  `rstest_bdd_harness::policy_conformance::assert_attribute_policy_conformance`
  for the policy. Artefacts: `src/profile_tests.rs`, `src/policy_tests.rs`.
- Obligation: L3 and EP-REQ-4. Method: behavioural scenarios through the real
  macro, bound with `harness = BevyHarness` (defaulted parameter) and
  `harness = BareHarness` (type alias). Artefacts: `tests/bevy_scenario.rs`,
  `tests/features/bevy_scenario.feature`, re-bound
  `tests/headless_scenario.rs`. Non-vacuity: a metadata step asserts the
  scenario name and line seen by the step, which only a harness-supplied
  context can provide; the bare-profile scenario asserts `FrameCount` is
  absent, which fails if the alias silently resolved to the minimal profile.
- Obligation: EP-INV-10. Method: existing `tests/extraction_boundary.rs` plus
  `cargo tree -p rstest-bdd-harness-bevy -e normal,dev` inspection at EP-M5.

Mutation audit (suite-level non-vacuity): at EP-M5 run `cargo mutants` scoped to
`src/context.rs`, `src/harness.rs`, and `src/panic.rs`. Every surviving mutant
is either killed by a new test or recorded in `Decision log` with a reason (for
example, an equivalent mutant in a `Debug` impl).

### Where formal verification is and is not warranted

Verus is not warranted. The introduced logic is integration plumbing: control
flow around `catch_unwind`, `Rc<RefCell<_>>` sharing, and calls into Bevy and
`rstest-bdd`. Verus does not model unwinding, and the interesting behaviour
lives in third-party code treated as axioms (A1–A6). There is no contractual
business logic or arithmetic kernel whose guarantee must hold over an unbounded
domain: the lifecycle model is finite and is enumerated exhaustively by the
table above, and tick additivity reduces to axiom A1 plus a loop whose property
tests and mutation audit give proportionate assurance. Kani is also unsuitable:
it treats panics as verification failures and does not support unwinding through
`catch_unwind`. No `assume`, `admit`, or trusted proof body is introduced. If
implementation exposes a pure kernel with an unbounded contract (none is
anticipated), return to this section before continuing.

## Plan of work

Stage A (EP-M0) establishes the baseline and records the architecture decision.
Stages B–D repeat per milestone: write the smallest failing test, confirm it
fails for the expected reason, implement, re-run, refactor, then run
`make check-fmt`, `make lint`, and `make test`.

Module layout at completion (design §9, adjusted):

```plaintext
crates/rstest-bdd-harness-bevy/
|-- src/
|   |-- lib.rs             re-exports; crate docs with a scenario example
|   |-- profile.rs         add_minimal_plugins, minimal_app, BevyProfile,
|   |                      BareBevyProfile, MinimalBevyProfile
|   |-- profile_tests.rs
|   |-- policy.rs          BevyAttributePolicy
|   |-- policy_tests.rs
|   |-- context.rs         BevyScenario
|   |-- context_tests.rs
|   |-- harness.rs         BevyHarness<P> and HarnessAdapter impl
|   |-- harness_tests.rs   lifecycle table and test profiles
|   |-- panic.rs           payload text, message formatting, panic-path cleanup
|   |-- panic_tests.rs
|   `-- snapshots/         insta snapshot for the augmented message
`-- tests/
    |-- bevy_scenario.rs   new harness-backed BDD scenarios
    |-- headless_scenario.rs  re-bound through the harness
    |-- tick_properties.rs    extended with BevyScenario properties
    `-- features/
        |-- bevy_scenario.feature
        `-- headless_scenario.feature   unchanged
```

If `harness_tests.rs` approaches 400 lines, move the test profile types into
`src/harness_tests/profiles.rs` with a `//!` comment.

## Milestones and plateaus

### EP-M0: baseline and ADR 010 (no production code)

- Outcome: a recorded baseline and a Proposed ADR. Run the focused crate tests
  (`cargo --config tools/dev-fast/config.toml nextest run -p rstest-bdd-harness-bevy`)
  and record the pass count. Confirm `multi_threaded` is off with
  `cargo tree -p rstest-bdd-harness-bevy -e features -i bevy_ecs`. Write
  `docs/adr/adr-010-bevy-harness-scenario-context-and-failure-contract.md`
  (status Proposed) recording: the `Rc<RefCell<App>>` closure-accessor context;
  why not a raw `World`, `&mut App`, or guard-returning accessors; the
  lifecycle table; the panic message format; the `HarnessError` mapping;
  options considered (Table 1). Reference it from design §§5–6 and add it to
  `docs/contents.md`.
- Requirements: EP-REQ-6 (partial).
- Acceptance: baseline recorded in `Progress`; `make markdownlint` and
  `make nixie` pass.
- Conformance check: design followed; no code change; no dependency change.
- Recovery: documentation-only; revert the commit.
- Remaining gaps: ADR acceptance by the maintainer (requested with the PR; the
  plan continues on Proposed status, and a rejection is an escalation).
- Compatibility decision: none.

### EP-M1: attribute policy

- Outcome: `BevyAttributePolicy` exists and is exported. It is the only API
  item with no dependency on the scenario context, so it forms a standalone
  plateau.
- Red: add `src/policy_tests.rs` calling
  `assert_attribute_policy_conformance::<BevyAttributePolicy>(&["#[rstest::rstest]"])`
  inside a `#[test]`. It fails to compile (missing item), which is the
  expected red.
- Green: implement `src/policy.rs` (signature in
  `Interfaces and dependencies`); export from `src/lib.rs`.
- Requirements: EP-REQ-1 (partial), EP-INV-9.
- Acceptance: the conformance test passes; `make check-fmt`, `make lint`,
  `make test` pass.
- Conformance check: marker semantics per design §7; no new dependency; no
  game code.
- Recovery: revert the milestone commit.
- Remaining gaps: the remaining five types (EP-M2).
- Compatibility decision: none (pre-1.0, unreleased API).

### EP-M2: scenario context, profiles, and the harness normal path

- Outcome: `BevyScenario` with all five accessors; `BevyProfile`,
  `BareBevyProfile`, and `MinimalBevyProfile`; `BevyHarness<P>` implementing
  `HarnessAdapter` with `Context = BevyScenario` for the configure-outcome and
  runner-returns rows of the lifecycle table. These land together because
  `BevyProfile::cleanup` takes `&BevyScenario` and `BevyScenario::new` is
  `pub(crate)` with `BevyHarness::run` as its only production caller; landing
  the context alone would leave dead code under `-D warnings`.
- Red: `src/context_tests.rs` — metadata retention; `with_app` and
  `with_world` return closure results and mutate the shared app; `update`
  advances `FrameCount` by one; `update_times(0)` and `update_times(3)`; clones
  share state; nested-borrow panics carry the `rstest-bdd-harness-bevy:` prefix
  (use `std::panic::catch_unwind(AssertUnwindSafe(...))` and inspect the
  payload); borrowability after a caught accessor panic.
  `src/profile_tests.rs` — `BareBevyProfile::configure` leaves `TimePlugin`
  unadded and inserts no `FrameCount` while an `Update` counter system still
  runs on `update()`; `MinimalBevyProfile::configure` adds `TimePlugin` and
  `FrameCount`; both return `Ok(())`; the default `cleanup` returns `Ok(())`.
  `src/harness_tests.rs` — the first five lifecycle rows (configure `Err`,
  configure panic, and the three `Return(v)` rows), `BevyHarness::default()` and
  `BevyHarness::new()` equivalence, and that `run` passes the request's
  metadata through to the scenario.
- Green: implement `src/context.rs`, the trait and profile enums in
  `src/profile.rs`, and `src/harness.rs`. In `run`, split the request with
  `into_parts`, build `App::new()`, call `P::configure`, wrap in
  `BevyScenario::new`, call the runner with a clone, then call
  `P::cleanup(&scenario, scenario.metadata())`. Emit
  `tracing::debug!(harness_type, feature_path, scenario_name, scenario_line,
  "starting Bevy scenario")`
  before running.
- Requirements: EP-REQ-1, EP-REQ-2, EP-INV-1, EP-INV-2 (normal path),
  EP-INV-4, EP-INV-5, EP-INV-7, EP-INV-8.
- Acceptance: new tests pass; gates pass.
- Conformance check: names match design §4; profiles are uninhabited enums;
  `BevyScenario` wraps `Rc<RefCell<App>>`; no raw `RefMut` escapes; no
  `Send`/`Sync` forwarding; `HarnessError` used only for configure and cleanup
  failures.
- Recovery: revert milestone commits; `profile.rs` returns to its EP-M1
  state; the EP-M1 plateau remains valid.
- Remaining gaps: a step panic propagates without cleanup or augmentation
  until EP-M3; recorded in `Progress` at the milestone boundary.
- Compatibility decision: none.

### EP-M3: panic and cleanup path

- Outcome: the three `Panic(p)` lifecycle rows hold; the message format is
  pinned.
- Red: `src/panic_tests.rs` — `payload_text` for `&'static str`, `String`,
  and an opaque payload (`rstest` cases); `scenario_panic_message` snapshot
  with metadata `("tests/features/demo.feature", "Demo scenario", 12)` and a
  harness type name string. `src/harness_tests.rs` — the three panic rows, each
  asserting the resumed payload is a `String` equal to `m(p)`, cleanup ran
  once, and (for rows 7–8) that the original message, not the cleanup failure,
  arrives.
- Green: implement `src/panic.rs` and wire it into `BevyHarness::run`: wrap
  the runner call in `catch_unwind(AssertUnwindSafe(...))`; on `Err(payload)`
  build `m(p)`, emit `tracing::error!` with the four metadata fields and the
  message, run `cleanup_after_panic::<P>(&scenario)` (which itself uses
  `catch_unwind` and logs `Err` or panic with `tracing::error!`), forget the
  original payload (as the GPUI harness does, so a panicking `Drop` cannot
  double-panic), and call `std::panic::resume_unwind(Box::new(message))`.
- Requirements: EP-REQ-3, EP-INV-2, EP-INV-3, L1, L2.
- Acceptance: new tests and the snapshot pass; gates pass; the three seeded
  faults listed in `Verification plan` each fail a row when applied by hand
  (record results, then revert the fault).
- Conformance check: message carries every field design §6 lists; cleanup
  errors on the panic path are logged, not returned; nothing in `run` uses the
  `panic!` macro (`panic_in_result_fn`).
- Recovery: revert milestone commits; EP-M2 plateau remains valid.
- Remaining gaps: tracing-record assertions (roadmap `0.1.3`).
- Compatibility decision: none.

### EP-M4: harness-backed behavioural scenarios and tick properties

- Outcome: RM-3 is demonstrated through the real macro.
- Red: add `tests/features/bevy_scenario.feature` (below) and
  `tests/bevy_scenario.rs` with step definitions but scenario bindings that
  name the harness; re-bind `tests/headless_scenario.rs` to
  `harness = BevyHarness, attributes = BevyAttributePolicy` with steps taking
  `#[from(rstest_bdd_harness_context)] scenario: &BevyScenario`. Before the
  steps are written, run the binary and observe `Step not found` failures; that
  is the red. Extend `tests/tick_properties.rs` with the sequence property.
- Green: implement the steps; keep `include_str!` bindings for both feature
  files (the 0.1.1 contract); remove the now-unused `RefCell<App>` fixture and
  the `allow_fixture_expansion_lints` import if nothing else uses them.
- Requirements: EP-REQ-4, L3, EP-INV-6, EP-INV-8 (through the macro).
- Acceptance: `make test` shows the four new scenario tests and the re-bound
  headless scenario passing; gates pass.
- Conformance check: the bare name and the alias both compile (RM-3); feature
  file `headless_scenario.feature` is byte-for-byte unchanged (design §9).
- Recovery: revert milestone commits; EP-M3 plateau remains valid.
- Remaining gaps: the `trybuild` compile-pass alias fixture (roadmap `0.1.3`).
- Compatibility decision: none.

Feature specification,
`crates/rstest-bdd-harness-bevy/tests/features/bevy_scenario.feature`:

```gherkin
Feature: Bevy scenario harness

  Scenario: Steps receive the scenario metadata from the harness
    Given a scenario driven by the minimal Bevy harness
    Then the scenario metadata names "Steps receive the scenario metadata from the harness" at line 3

  Scenario: Steps share one application across ticks
    Given a scenario driven by the minimal Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 3 times
    Then the toy counter reads 3
    And the frame count reads 3

  Scenario: Advancing zero times leaves the application untouched
    Given a scenario driven by the minimal Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 0 times
    Then the toy counter reads 0
    And the frame count reads 0

  Scenario: The bare profile adds no plugins
    Given a scenario driven by the bare Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 2 times
    Then the toy counter reads 2
    And the application has no frame counter
```

The first three scenarios bind with `harness = BevyHarness`; the fourth binds
with `harness = BareHarness`, where the test file declares
`type BareHarness = BevyHarness<BareBevyProfile>;`. "Given a scenario driven by
the minimal Bevy harness" asserts `TimePlugin` is added; "the bare Bevy
harness" asserts it is not. The unchanged `headless_scenario.feature` reads:

```gherkin
Feature: Headless Bevy scaffolding

  Scenario: A minimal application advances one tick
    Given a minimal headless Bevy application
    When the schedule advances once
    Then the frame count reads 1
```

### EP-M5: documentation, mutation audit, roadmap, and final gates

- Outcome: documentation matches the delivered API; mutation audit recorded;
  roadmap `0.1.2` marked done.
- Work: update design §§4–6 and §9 with the delivered decisions and layout,
  and change its status line to say §§4–7 are implemented; reference ADR 010;
  rewrite `crates/rstest-bdd-harness-bevy/README.md` (purpose, suite inventory,
  files table, extending section); extend `docs/users-guide.md` §4 with a
  harness-backed scenario example, the direct `rstest-bdd-harness`
  dev-dependency requirement, the `HarnessError` mapping idiom, and the
  advancement rule; update `docs/rstest-bdd-harness-bevy-adoption.md` lines
  55–56; update `docs/repository-layout.md` (src and tests entries); add to
  `docs/developers-guide.md` §2.1 the internal conventions (test profiles
  observe the app through a stashed scenario clone rather than globals;
  nested-borrow message; poisoned app after a system panic; `resume_unwind`
  rather than `panic!` in `Result` functions); add this plan to
  `docs/contents.md`; tick roadmap `0.1.2` (`- [x]`). Run the scoped
  `cargo mutants` audit and triage survivors.
- Requirements: EP-REQ-5, EP-REQ-6.
- Acceptance: `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  `make nixie`, and `make spelling` pass; `cargo tree` shows no forbidden
  package; mutation audit has no unexplained survivors.
- Conformance check: every Conformance-basis trace link has evidence; design
  and ADR agree with code; no unapproved public item (compare `cargo doc`
  output for the crate against `Interfaces and dependencies`).
- Recovery: documentation commits revert independently.
- Remaining gaps: roadmap `0.1.3` to `0.1.5`.
- Compatibility decision: none.

## Concrete steps

Run everything from the worktree root
`/home/leynos/.lody/repos/github---leynos---skyjoust/worktrees/23ed6522-c112-45c2-9697-efd9fefd0645`.
Log every gate with `tee` using
`/tmp/$ACTION-skyjoust-0-1-2-bevy-harness-public-api.out`.

Focused loop during a milestone:

```bash
cargo --config tools/dev-fast/config.toml nextest run -p rstest-bdd-harness-bevy 2>&1 \
  | tee /tmp/test-focused-skyjoust-0-1-2-bevy-harness-public-api.out
```

Expected red output (EP-M2 example):

```plaintext
error[E0432]: unresolved import `super::BevyScenario`
```

Expected green output tail:

```plaintext
     Summary [ ... ] NN tests run: NN passed, 0 skipped
```

Milestone gates, sequentially, never in parallel:

```bash
make check-fmt 2>&1 | tee /tmp/check-fmt-skyjoust-0-1-2-bevy-harness-public-api.out
make lint 2>&1 | tee /tmp/lint-skyjoust-0-1-2-bevy-harness-public-api.out
make test 2>&1 | tee /tmp/test-skyjoust-0-1-2-bevy-harness-public-api.out
```

Documentation gates (EP-M0, EP-M5):

```bash
make fmt 2>&1 | tee /tmp/fmt-skyjoust-0-1-2-bevy-harness-public-api.out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-0-1-2-bevy-harness-public-api.out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-0-1-2-bevy-harness-public-api.out
```

Extraction and feature checks (EP-M0, EP-M5):

```bash
cargo tree -p rstest-bdd-harness-bevy -e normal,dev | grep -E 'skyjoust|lille' || echo "no game crates"
cargo tree -p rstest-bdd-harness-bevy -e features -i bevy_ecs | grep multi_threaded || echo "single-threaded"
```

Mutation audit (EP-M5; confirm flags with `cargo mutants --help` first, and
pass the dev-fast configuration so the cache is shared):

```bash
cargo mutants -p rstest-bdd-harness-bevy \
  -f crates/rstest-bdd-harness-bevy/src/context.rs \
  -f crates/rstest-bdd-harness-bevy/src/harness.rs \
  -f crates/rstest-bdd-harness-bevy/src/panic.rs \
  --test-tool nextest --cargo-arg=--config=tools/dev-fast/config.toml 2>&1 \
  | tee /tmp/mutants-skyjoust-0-1-2-bevy-harness-public-api.out
```

Commit after each green-and-gated step using the `commit-message` skill; prefer
delegating full gate runs to the `scrutineer` agent.

## Validation and acceptance

Quality criteria:

- Tests: `make test` passes, including the lifecycle table (eight rows),
  context tests, panic formatter tests and snapshot, profile and policy tests,
  the extended property tests, four new behavioural scenarios, and the re-bound
  headless scenario. Each new test fails before its implementation and passes
  after.
- Verification: EP-INV-1 to EP-INV-10 have the evidence named in
  `Verification plan`; the mutation audit has no unexplained survivors. No
  Verus milestone exists, so no Verus command is part of acceptance.
- Lint and format: `make check-fmt` and `make lint` (rustdoc, Clippy,
  Whitaker) pass with no new suppression.
- Documentation: `make markdownlint`, `make nixie`, and `make spelling` pass.

Behavioural acceptance, observable by a human:

1. The focused run in `Concrete steps`, filtered to `bevy_scenario`, lists
   four passing scenario tests.
2. Temporarily add `Then the toy counter reads 4` to the second scenario: the
   test fails with a message beginning
   `rstest-bdd-harness-bevy scenario panicked:` that names
   `tests/features/bevy_scenario.feature`, the scenario name, and its line.
   Revert the edit.
3. `git diff main -- crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature`
   is empty.

## Idempotence and recovery

Every step is re-runnable. Tests create fresh apps and use no global or
environment state. Snapshot updates are explicit (`cargo insta review` or
`INSTA_UPDATE=always` on a focused run) and must be reviewed before commit.
Each milestone ends in a gated commit; `git revert` of a milestone's commits
returns to the previous plateau. Never use `git stash` without a unique tag
(the stash stack is shared with other worktrees).

## Artefacts and notes

The upstream GPUI harness message format, mirrored with a crate-specific prefix
and the harness type added:

```plaintext
rstest-bdd-harness-gpui scenario panicked: feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {message}
```

This crate's format (one line; wrapped here for width):

```plaintext
rstest-bdd-harness-bevy scenario panicked: harness={harness_type},
feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {payload_text}
```

## Interfaces and dependencies

No new crate enters the lockfile. `crates/rstest-bdd-harness-bevy/Cargo.toml`
gains `insta = "1"` under `[dev-dependencies]` (resolves to the locked
`1.49.0`).

In `src/profile.rs`:

```rust,no_run
pub trait BevyProfile: 'static {
    /// Configures the fresh application before any step runs.
    fn configure(app: &mut App, metadata: &ScenarioMetadata) -> HarnessResult<()>;

    /// Releases scenario resources after the runner finishes or panics.
    fn cleanup(_scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        Ok(())
    }
}

pub enum BareBevyProfile {}
pub enum MinimalBevyProfile {}
```

`BareBevyProfile::configure` returns `Ok(())` without touching the app;
`MinimalBevyProfile::configure` calls `add_minimal_plugins(app)` and returns
`Ok(())`.

In `src/context.rs`:

```rust,no_run
#[derive(Clone, Debug)]
pub struct BevyScenario {
    app: Rc<RefCell<App>>,
    metadata: Rc<ScenarioMetadata>,
}

impl BevyScenario {
    pub(crate) fn new(app: App, metadata: ScenarioMetadata) -> Self;
    pub fn metadata(&self) -> &ScenarioMetadata;
    #[track_caller]
    pub fn with_app<R>(&self, operation: impl FnOnce(&mut App) -> R) -> R;
    #[track_caller]
    pub fn with_world<R>(&self, operation: impl FnOnce(&mut World) -> R) -> R;
    #[track_caller]
    pub fn update(&self);
    #[track_caller]
    pub fn update_times(&self, count: u32);
}
```

`with_app` uses `RefCell::try_borrow_mut` and, on failure, panics with
`rstest-bdd-harness-bevy: BevyScenario is already borrowed; accessor closures
must not call with_app, with_world, update, or update_times`.
The other three delegate to `with_app`; `update_times` loops inside one
borrow. Each public accessor documents `# Panics`.

In `src/harness.rs`:

```rust,no_run
pub struct BevyHarness<P = MinimalBevyProfile> {
    profile: PhantomData<fn() -> P>,
}

impl<P> BevyHarness<P> {
    pub const fn new() -> Self;
}

impl<P> Default for BevyHarness<P> { /* Self::new() */ }
impl<P> Clone for BevyHarness<P> { /* *self */ }
impl<P> Copy for BevyHarness<P> {}
impl<P> fmt::Debug for BevyHarness<P> { /* type name */ }

impl<P: BevyProfile> HarnessAdapter for BevyHarness<P> {
    type Context = BevyScenario;
    fn run<T>(&self, request: ScenarioRunRequest<'_, BevyScenario, T>) -> HarnessResult<T>;
}
```

In `src/panic.rs` (crate-private):

```rust,no_run
pub(crate) fn payload_text(payload: &(dyn Any + Send)) -> &str;
pub(crate) fn scenario_panic_message(
    harness_type: &str,
    metadata: &ScenarioMetadata,
    payload: &(dyn Any + Send),
) -> String;
pub(crate) fn cleanup_after_panic<P: BevyProfile>(scenario: &BevyScenario, message: &str);
```

In `src/policy.rs`:

```rust,no_run
pub struct BevyAttributePolicy;

impl AttributePolicy for BevyAttributePolicy {
    fn test_attributes() -> &'static [TestAttribute] { &BEVY_TEST_ATTRIBUTES }
}

const BEVY_TEST_ATTRIBUTES: [TestAttribute; 1] = [TestAttribute::new("rstest::rstest")];
```

`src/lib.rs` adds `mod context; mod harness; mod panic; mod policy;` and
re-exports `BevyScenario`, `BevyHarness`, `BevyProfile`, `BareBevyProfile`,
`MinimalBevyProfile`, and `BevyAttributePolicy` alongside the existing items.
It does not re-export `StdHarness` or `DefaultAttributePolicy` (0.1.1 decision).

Downstream consumers need direct development dependencies on `rstest`,
`rstest-bdd`, `rstest-bdd-macros`, and `rstest-bdd-harness` (the macro emits
`::rstest_bdd_harness` paths for third-party harnesses), plus this crate.

## Revision note

- Revision 1, 2026-10-18: initial draft from four Wyvern reconnaissance
  reports and prior-art research. Awaiting community-of-experts review.

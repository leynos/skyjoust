# Implement the reusable Bevy harness public API

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT

Approval gate: this plan must be explicitly approved by the maintainer before
any implementation milestone (EP-M1 onwards) begins. Silence is not approval.
ADR 010 (drafted in EP-M0) must be accepted before EP-M3 starts.

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
panics), and re-raises any panic with the lifecycle phase, feature path,
scenario name, and one-based feature-file line in a message that `cargo test`
and `cargo nextest` print. Game-specific setup lives in a downstream profile
type, never in the reusable crate.

Observable success: `make test` runs behavioural scenarios whose generated
tests construct the harness through `<Harness as Default>::default()` and whose
steps receive the wrapper through `rstest_bdd_harness_context`; the existing
`tests/features/headless_scenario.feature` passes unchanged after being
re-bound through the harness; and an end-to-end scenario that fails on purpose
is asserted (with `#[should_panic(expected = ...)]`) to panic with a message
beginning:

```plaintext
rstest-bdd-harness-bevy scenario panicked: phase=steps, harness=
```

followed by the feature path and line, the quoted scenario name, and the
original failure text.

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
  access, and its test profiles inject behaviour through types rather than
  globals.
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

Upstream sources in the local Cargo registry
(`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`), read rather than
guessed:

- `rstest-bdd-harness-0.6.0/src/{adapter,runner,error,policy,policy_conformance}.rs`
  — the contract implemented here.
- `rstest-bdd-harness-gpui-0.6.0/src/gpui_harness/mod.rs` — the first-party
  panic path this crate mirrors.
- `rstest-bdd-macros-0.6.0/src/codegen/scenario/runtime/harness.rs` and
  `.../generators/step_loop.rs` — how generated tests build the harness, inject
  the context, and turn step failures into panics.
- `bevy_app-0.19.1/src/{app,sub_app,main_schedule}.rs`,
  `bevy_ecs-0.19.1/src/world/mod.rs`, `bevy_time-0.19.1/src/lib.rs`.

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
  brief authorizes it. Tests that observe `tracing` events use a handwritten
  subscriber against the `tracing` crate the harness already depends on, not
  `tracing-subscriber`. Any other new dependency is an escalation.
- `rstest-bdd`, `rstest-bdd-macros`, and `rstest-bdd-harness` stay on the
  published stable `0.6.0` family with implicit caret requirements. No git
  dependencies.
- The public API must match design §§4–5 names and signatures:
  `BevyScenario`, `BevyProfile` (static `configure` and default-bodied
  `cleanup` returning `HarnessResult<()>`),
  `BevyHarness<P = MinimalBevyProfile>`, `BareBevyProfile`,
  `MinimalBevyProfile`, and `BevyAttributePolicy`, with
  `BevyScenario::{metadata, with_app, with_world, update, update_times}`.
  `BevyScenario` wraps an `App`, not a raw `World`. The only additional public
  item is `BevyHarness::new`.
- `add_minimal_plugins` and `minimal_app` remain public; 0.1.1 promised they
  survive.
- No `unsafe` code (`unsafe_code = "forbid"`), no `Send`/`Sync` forwarding.
- No lint suppression except as a documented last resort with a tight
  `#[expect(..., reason = "...")]`; the nested
  `rstest-bdd-harness-bevy-test-macros` crate remains the only fixture-lint
  mechanism.
- Production code writes no output directly: `print_stderr` and
  `print_stdout` are denied, and diagnostics travel through the panic payload
  (printed by the panic hook) and `tracing`.
- Tests must not install a process-global panic hook (`std::panic::set_hook`)
  or a global `tracing` subscriber, and must not mutate environment variables;
  scoped `tracing::subscriber::with_default` is permitted.
- Every source file stays at or under 400 lines; every module starts with a
  `//!` comment.
- Do not change `.config/nextest.toml`, `tests/nextest_policy.rs`, or CI
  workflow timeouts. A new `trybuild` runner is out of scope (roadmap `0.1.3`).
- Do not implement roadmap `0.1.3`'s remaining scope (the harness-alias
  `trybuild` compile-pass fixture and process-level stderr snapshots), `0.1.4`,
  or `0.1.5`'s extraction documentation beyond what this API change makes
  inaccurate.

## Tolerances (exception triggers)

- Scope: if implementation needs more than 32 changed paths or more than
  2,000 net added lines (code, tests, and documentation together), stop and
  escalate.
- Interface: if any design §5 signature must change (for example `configure`
  needs `&mut self`, `with_app` must return a `Result`, or profiles must return
  a crate-owned error type), stop, record a proposed deviation, set status
  `BLOCKED`, and escalate.
- Dependencies: if anything beyond adding `insta` to the harness crate's
  development dependencies is needed (for example `tracing-subscriber` or
  `serial_test`), stop and escalate.
- Macro contract: if `#[scenario(harness = BevyHarness, ...)]` with the
  defaulted generic parameter, or with a type alias, fails to compile, stop and
  escalate with the diagnostic; do not work around it with wrapper newtypes.
- Unwinding: if the EP-M0 unwinding probe fails under the dev-fast
  (Cranelift) configuration, stop and escalate; the panic path depends on it.
- Lint: if `panic_in_result_fn` flags `std::panic::panic_any` inside
  `BevyHarness::run`, stop and escalate rather than suppressing it.
- Iterations: if a focused test or gate still fails after three fix attempts
  for the same cause, stop and escalate.
- Time: if a milestone takes more than three hours of active work (excluding
  compile-admission queueing), stop and escalate.
- Gate cost: the 0.1.1 cost exception (19-minute warm `make all`, 6.7 GB
  `target/`) was approved for that task only. Record each gate's duration; if a
  warm `make test` exceeds 45 minutes without queueing being the evident cause,
  or `target/` grows by more than 1 GB, escalate.
- Mutation audit: if `cargo mutants --list` reports more than 60 mutants for
  the scoped files, or the run exceeds two hours, stop it and escalate; the
  audit is advisory and never blocks the gates.
- Ambiguity: if the design and this plan disagree, or a reviewer decision in
  `Decision log` turns out to be unimplementable, stop and present options.

## Risks

- Risk: `#[scenario(harness = BevyHarness)]` relies on the defaulted generic
  parameter being applied in `<BevyHarness as Default>::default()` and
  `<BevyHarness as HarnessAdapter>::Context`. Severity: medium. Likelihood: low
  (every generated use is a type position, where defaults apply; the review
  confirmed this against the macro source). Mitigation: EP-M4 binds scenarios
  through both the bare name and a type alias; the tolerance above escalates on
  failure.
- Risk: the `rstest-bdd` macro resolves `::rstest_bdd_harness` for any
  non-first-party harness, so consumers must declare `rstest-bdd-harness`
  directly. Severity: low. Likelihood: certain. Mitigation: the harness crate
  declares it as a normal dependency, which `proc_macro_crate` resolves for its
  own tests; the users' guide states the requirement for downstream crates.
- Risk: `HarnessError` has a single `#[non_exhaustive]` variant,
  `RuntimeBuildFailed(std::io::Error)`, whose display begins "failed to build
  runtime", and the generated test reports any `Err` as "harness failed to
  initialize scenario", even for cleanup. Severity: low. Likelihood: certain.
  Mitigation: the harness wraps every profile error in a crate-private
  phase-tagged error (`Bevy profile cleanup hook failed: ...`) that keeps the
  original as its `source()`; rustdoc shows the profile-author idiom
  `HarnessError::RuntimeBuildFailed(std::io::Error::other(message))`.
- Risk: a Bevy system that panics inside `App::update` leaves the app without
  its `Main` schedule (Bevy removes schedules temporarily during a run and has
  no reinsert guard), so a later `update()` (for example in cleanup) would
  panic with the misleading "The schedule with the label Main was not found".
  Severity: medium. Likelihood: medium once real profiles exist. Mitigation:
  `update` and `update_times` check that the main sub-app's update schedule is
  still present and otherwise panic with a crate-prefixed message explaining
  that a system panicked in an earlier update; the panic path catches cleanup
  panics and re-raises the original; rustdoc and the users' guide say cleanup
  must not advance the app.
- Risk: the harness's last scenario handle drops after cleanup; a component,
  resource, or plugin `Drop` that panics while the harness is already
  re-raising would abort the test binary without diagnostics. (`World::drop`
  itself drops queued commands without applying them, so commands are not the
  danger.) Severity: low. Likelihood: low. Mitigation: on the panic path, the
  harness drops its handle inside `catch_unwind` before re-raising and logs a
  drop panic with `tracing::error!`.
- Risk: the development profile builds with Cranelift, and no workspace test
  has yet exercised `catch_unwind` or `#[should_panic]`; CI coverage builds
  with LLVM. Severity: medium. Likelihood: low. Mitigation: EP-M0 runs an
  unwinding probe under dev-fast before any panic-path code is written; the
  tolerance escalates on failure; `make test` (Cranelift) and coverage (LLVM)
  both exercise the panic tests afterwards.
- Risk: workspace lints reject first-draft code. Known traps:
  `panic_in_result_fn` (no `panic!`/`assert!` in `run`; re-raise with the
  `panic_any` function), `missing_panics_doc` (every public accessor documents
  `# Panics`), `print_stderr`, `let_underscore_must_use` (match on every
  `catch_unwind` result), `option_if_let_else` (write the payload downcast as an
  `or_else` chain), `must_use_candidate` (decide `#[must_use = "..."]` on
  `metadata`, `new`, `with_app`, and `with_world` up front),
  `missing_const_for_fn`, `shadow_*`, `doc_markdown`, `unwrap_used` and
  `expect_used` in step functions (which are not `#[test]` functions), and
  cognitive complexity 9. Severity: low. Likelihood: high. Mitigation: split
  `run` into phase helpers; run `make lint` at every milestone.
- Risk: every integration test binary links Bevy, and every source edit
  relinks all of them. Severity: low. Likelihood: certain. Mitigation: all
  harness-backed scenarios share the existing `tests/headless_scenario.rs`
  binary (steps in a submodule); `tests/tick_properties.rs` gains properties
  without a new binary.
- Risk: `cargo mutants` copies the tree to a cold scratch directory by default
  and runs every test binary, including the slow `macro_ui` trybuild runner.
  Severity: medium. Likelihood: certain if run naively. Mitigation: run it
  `--in-place` in this isolated worktree (sharing `target/`), exclude
  `macro_ui` and `extraction_boundary` with a Nextest filter, list first, cap
  by tolerance, and treat it as advisory.
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
- [x] (2026-10-18) First draft of this ExecPlan (Revision 1, commit
  `e84f4ac`).
- [x] (2026-10-18) Community-of-experts design review (three-agent Logisphere
  panel: contracts and structure; failure modes, cost, and pre-mortem;
  alternatives and verification). Verdict: proceed with conditions. All
  conditions folded into Revision 2.
- [ ] Maintainer approval of this plan.
- [ ] EP-M0: baseline, unwinding probe, and ADR 010 draft.
- [ ] ADR 010 accepted (required before EP-M3).
- [ ] EP-M1: attribute policy.
- [ ] EP-M2: scenario context, profiles, and harness normal path.
- [ ] EP-M3: panic and cleanup path.
- [ ] EP-M4: harness-backed behavioural scenarios and tick properties.
- [ ] EP-M5: documentation, mutation audit, roadmap, and final gates.

## Surprises & discoveries

- Observation: Bevy's `App::update` never calls `Plugin::finish` or
  `Plugin::cleanup`; only runners do. Evidence:
  `bevy_app-0.19.1/src/app.rs:158-164` versus `run_once` at 1531–1542. Impact:
  harmless for `MinimalPlugins`, whose members override only `build`. The
  harness does not call `finish`/`cleanup`, which keeps `add_plugins` legal
  inside steps; a downstream profile that needs plugin finalization calls them
  at the end of `configure`. This becomes part of the documented `BevyProfile`
  contract and is pinned by a probe-plugin test.
- Observation: `HarnessAdapter` imposes only `Context: Any`; no `Send`,
  `Sync`, or `Clone`. The macro moves the context into a step context cell and
  drops it when the runner closure ends, so the harness cannot recover it
  except through a handle it kept. Evidence:
  `rstest-bdd-harness-0.6.0/src/adapter.rs:35-49`,
  `rstest-bdd-macros-0.6.0/src/codegen/scenario/runtime/harness.rs:95-122`.
  Impact: confirms design §4's `Rc<RefCell<App>>` shared handle.
- Observation: unknown third-party `attributes = ...` paths always expand to
  `#[rstest::rstest]` only; the policy contributes only a trait-bound
  assertion. Evidence:
  `rstest-bdd-macros-0.6.0/src/codegen/scenario/test_attrs.rs:125-147`. Impact:
  `BevyAttributePolicy` is a marker, as design §7 states.
- Observation (design review): `std::panic::resume_unwind` does not invoke the
  panic hook, and libtest never prints a failed test's payload, so a harness
  that re-raises with `resume_unwind` produces an augmented message nobody
  sees. The GPUI harness compensates by writing to stderr. Evidence:
  `library/std/src/panic.rs` (`resume_unwind` docs),
  `library/test/src/test_result.rs`,
  `rstest-bdd-harness-gpui-0.6.0/src/gpui_harness/mod.rs:159-168`. Impact: this
  crate re-raises with `std::panic::panic_any(message)`, which runs the hook so
  the test runner captures and prints the message on failure, without writing
  to stderr directly (denied by `print_stderr`).
- Observation (design review): step failures do not unwind out of step
  wrappers immediately; the generated step loop records the error and calls
  `panic!` after the loop, inside the runner closure. Evidence:
  `rstest-bdd-macros-0.6.0/.../generators/step_loop.rs:110-118`. Impact: no
  step fixture borrow is live when the harness catches the panic, and the
  payload the harness augments is the step loop's message.
- Observation (design review): `World::drop` drops queued commands without
  applying them (`apply_or_drop_queued(None)`). Evidence:
  `bevy_ecs-0.19.1/src/world/mod.rs:144-155` and
  `world/command_queue.rs:227-228`. Impact: corrects the Revision 1 risk; the
  residual abort risk is a panicking user `Drop`.

## Decision log

- Decision: implement design §6's panic and cleanup path in `0.1.2`, with the
  focused unit, behavioural, and end-to-end tests that drive it, rather than
  deferring it to `0.1.3`. Rationale: the roadmap item cites design §§4–7,
  design §9 assigns `src/panic.rs` to `0.1.2`, and test-first delivery forbids
  landing untested behaviour. EP-M5 rewords roadmap `0.1.3` to its remaining
  scope (the harness-alias `trybuild` compile-pass fixture, process-level
  stderr and tracing-output snapshots, and broader panic-path scenarios such as
  outlines). Date/Author: 2026-10-18, planning agent; confirmed by review.
- Decision: re-raise with `std::panic::panic_any(message)` rather than
  `resume_unwind`. Rationale: `panic_any` invokes the panic hook, so libtest
  and Nextest capture the message per test and show it on failure, and
  `#[should_panic(expected = ...)]` matches it; `resume_unwind` is silent.
  Direct stderr writes are denied by `print_stderr` and would bypass libtest
  capture. The hook also prints a location inside `harness.rs`, which is
  acceptable. Date/Author: 2026-10-18, design review.
- Decision: one catch boundary covers all three lifecycle phases, and every
  augmented message carries `phase=configure|steps|cleanup`. Configure panics
  and normal-path cleanup panics are augmented too, not propagated unchanged.
  Rationale: plugin registration panics (for example duplicate plugins) will be
  the commonest downstream failure, and design §6 exists to say which scenario
  failed. Pinning the field before extraction avoids a later diagnostic change.
  Date/Author: 2026-10-18, design review.
- Decision: profile `Err` results are wrapped in a crate-private
  `ProfileHookError { phase, source }` (handwritten `Display` and
  `Error::source`, no `thiserror`) inside
  `HarnessError::RuntimeBuildFailed(std::io::Error::other(..))`. Profiles keep
  returning `HarnessResult<()>` per design §5. Rationale: names the failing
  hook in otherwise misleading upstream text without changing the §5 signature;
  a crate-owned profile error type would be an interface change and is recorded
  in ADR 010 as a rejected option for now. Date/Author: 2026-10-18, design
  review.
- Decision: `harness=` in the message comes from `std::any::type_name` and is
  informative, not a stable contract. Integration tests match the prefix and
  the other fields; only the pure-formatter snapshot, which takes a fixed
  harness string, pins the full text. Rationale: `type_name` output is unstable
  across toolchains and exposes private module paths. Date/Author: 2026-10-18,
  design review.
- Decision: the lifecycle table is verified with tripwire test profiles
  (hooks and runners that must not run panic with a sentinel) and a stashed
  scenario clone (state read after `run` returns), never with globals or
  thread-locals. Rationale: profile hooks are associated functions with no
  `self`, so "cleanup ran zero times" is otherwise unobservable; tripwires make
  a wrongly-run hook fail loudly. Date/Author: 2026-10-18, design review.
- Decision: the `log` column is verified in `0.1.2` with a small handwritten
  `tracing::Subscriber` installed with `tracing::subscriber::with_default`
  (thread-scoped). Rationale: logging on the panic path is behaviour this plan
  introduces, and `tracing-subscriber` would be a new dependency. Date/Author:
  2026-10-18, design review.
- Decision: Verus and Kani are not used. See `Verification plan` for the
  argument. Date/Author: 2026-10-18, planning agent; strengthened by review.
- Decision: the mutation audit is advisory, scoped to `src/harness.rs` and
  `src/panic.rs`, run `--in-place` with a Nextest filter excluding the trybuild
  and manifest-tripwire binaries. Rationale: proportionate suite-level
  non-vacuity evidence without a cold Bevy build or 90-minute UI runs per
  mutant. Date/Author: 2026-10-18, design review.
- Decision: pin the augmented message format with `insta` snapshots of the
  pure formatter (three phases and three payload classes). Rationale: the
  format is introduced here and is the crate's main diagnostic contract; the
  user's brief authorizes `insta`, which already resolves in the lockfile.
  Snapshots never auto-update in CI. Date/Author: 2026-10-18, planning agent.
- Decision: `BevyScenario::new` is `pub(crate)`. Rationale: the design lists
  no public constructor; integration tests and downstream profile authors reach
  a scenario by calling `run` with a hand-built `ScenarioRunRequest` (public
  `rstest-bdd-harness` API), which `BevyHarness`'s rustdoc example shows.
  Widening later is additive. Date/Author: 2026-10-18, planning agent;
  confirmed by review.
- Decision: `BevyScenario` derives `Clone` and `Debug`; clones share one app.
  Rationale: the harness keeps a handle for cleanup, and the macro's by-value
  step parameter form requires `Clone`. Rustdoc says clones must not outlive
  the scenario. A strong-count warning after cleanup is deferred to `0.1.3`
  with the other tracing-output work. Date/Author: 2026-10-18, planning agent.
- Decision: `BevyHarness<P>` stores `PhantomData<fn() -> P>` and implements
  `Default`, `Clone`, `Copy`, and `Debug` by hand, plus `pub const fn new()`.
  Rationale: derives would demand `P: Default` and friends, which uninhabited
  profile enums cannot satisfy; `fn() -> P` keeps the harness `Send + Sync` and
  covariant regardless of `P` (asserted at compile time in a test).
  Date/Author: 2026-10-18, planning agent.
- Decision: the harness never calls `App::run`, `App::finish`, or
  `App::cleanup`; plugin finalization is the profile's responsibility.
  Rationale: finishing would forbid `add_plugins` in steps, and `run` loops
  forever under `MinimalPlugins`. Date/Author: 2026-10-18, design review.
- Decision: `MinimalBevyProfile` stays exactly `MinimalPlugins`, with
  wall-clock `Time`. Deterministic time
  (`TimeUpdateStrategy::FixedTimesteps(n)`, which makes each `App::update` run
  the fixed loop exactly `n` times) is shown as a downstream-profile technique
  in rustdoc and one unit test, and recorded in ADR 010 as the strongest
  alternative. Rationale: changing the default profile would deviate from
  design §5; the seam is what roadmap `0.1.4` needs. Date/Author: 2026-10-18,
  design review.
- Decision: no `HarnessError` construction helper is added. Rationale: the
  design lists no such API; the one-line idiom is documented. Date/Author:
  2026-10-18, planning agent.
- Decision: public doctests stay runnable, following the crate's existing
  `profile.rs` doctests and `docs/rust-doctest-dry-guide.md`, rather than the
  style guide's generic `no_run` advice. Rationale: a headless app runs in
  milliseconds, and runnable examples cannot rot silently. Date/Author:
  2026-10-18, planning agent.
- Decision: ADR 010 records the failure contract (lifecycle table, phase
  tags, `panic_any` re-raise, error wrapping), the SemVer coupling (Bevy and
  `rstest-bdd-harness` types in public signatures make their minor bumps
  breaking for this crate; `BevyProfile` stays unsealed so default-bodied hooks
  can be added later), and the rejected alternatives. The context
  representation stays in the design document, which already decides it.
  Rationale: avoid two sources of truth; AGENTS.md requires an ADR for
  substantive, hard-to-reverse decisions. Date/Author: 2026-10-18, design
  review.
- Decision: the users' guide gains one short subsection now (harness-backed
  scenario example, direct `rstest-bdd-harness` dependency, error idiom,
  advancement and cleanup rules); fuller adoption and extraction guidance
  remains roadmap `0.1.5`. Rationale: AGENTS.md requires user-facing API
  changes to be documented when they land. Date/Author: 2026-10-18, design
  review.

## Outcomes & retrospective

Not yet started. Record milestone outcomes, gate durations, mutation-audit
counts (caught, missed, unviable, timeout), and lessons here.

## Context and orientation

Skyjoust is a Rust workspace (root `Cargo.toml`) with four members: the root
`skyjoust` package, `crates/skyjoust_stateright_validator`,
`crates/skyjoust_test_macros`, and `crates/rstest-bdd-harness-bevy` with its
nested test-only proc-macro crate `crates/rstest-bdd-harness-bevy/test-macros`.
The toolchain is pinned to `nightly-2026-03-26` in `rust-toolchain.toml`; the
development profile builds with Cranelift through `.cargo/config.toml` and
`tools/dev-fast/config.toml`.

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
per update, wrapping `u32`), `TimePlugin` (wall-clock by default), and
`ScheduleRunnerPlugin` (whose `run` loops forever, so tests never call
`App::run`). `App` is neither `Send` nor `Sync`.

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
  DS-9 crate layout (§9), DS-10 verification strategy (§10), DS-11 extraction
  plan (§11), DS-12 risks (§12).
- ADR 009 (Accepted 2026-08-17): ADR9-SEAM profile type is the single
  extension seam; ADR9-EXTRACT no Skyjoust or Lille dependency; ADR9-DEPS Bevy
  `0.19.1` headless, stable `rstest-bdd` 0.6.0.
- Technical design `docs/skyjoust-technical-design.md`: TD-5 simulation ticks
  never read wall-clock time, so advancement is explicit; TD-14 gates.
- Terms of Reference: none exists for this repository; none is invented.
- Governing standards: `AGENTS.md`, `docs/documentation-style-guide.md`,
  workspace lints in the root `Cargo.toml`, `clippy.toml`, `.rustfmt.toml`.

Deliberate refinements of design §6 (recorded in ADR 010, not deviations from
any §5 signature): the phase field, augmentation of configure and cleanup
panics, `panic_any` instead of an unspecified re-raise, and phase-tagged error
wrapping.

This plan's requirements and trace links:

- EP-REQ-1 (RM-1, DS-4, DS-5, DS-7): the six public types exist with the
  design's names and signatures.
- EP-REQ-2 (RM-2, DS-5, DS-12, TD-5): `BevyScenario` wraps
  `Rc<RefCell<App>>`, exposes closure accessors and explicit advancement, and
  panics with a crate-prefixed message on re-entrant access or on a broken
  `Main` schedule.
- EP-REQ-3 (DS-6, DS-12): profile cleanup runs on success and on step panic;
  panics re-raise with phase, harness type, feature path, scenario name, line,
  and payload text in hook-visible form; cleanup failures never mask the step
  panic.
- EP-REQ-4 (RM-3, DS-7, DS-10): generated scenarios construct the harness
  through `Default` and steps receive the wrapper through the reserved key, for
  the bare name and for a type alias; a generated failing scenario shows the
  augmented message.
- EP-REQ-5 (ADR9-SEAM, ADR9-EXTRACT, DS-11): no game code or dependency in
  the crate; profiles are the only extension seam.
- EP-REQ-6 (AGENTS.md): documentation, ADR, users' guide, developers' guide,
  and roadmap reflect the delivered API.

```plaintext
RM-1 -> DS-5 -> EP-REQ-1 -> EP-M1, EP-M2 -> policy_tests, profile_tests, context_tests, harness_tests
RM-2 -> DS-5/DS-12 -> EP-REQ-2 -> EP-M2 -> context_tests::*, tick_properties::scenario_*
DS-6 -> EP-REQ-3 -> EP-M3 -> harness_tests::lifecycle_*, panic_tests::*, insta snapshots
RM-3 -> DS-7/DS-10 -> EP-REQ-4 -> EP-M4 -> headless_scenario::* (all bound features)
ADR9-EXTRACT -> EP-REQ-5 -> every milestone -> extraction_boundary::*, cargo tree
EP-REQ-6 -> EP-M0 (ADR 010), EP-M5 -> make markdownlint, make nixie
```

## Verification plan

### Abstract model (proof sketch first)

Model the harness run as a function of three abstract outcome classes,
independent of Bevy and of `rstest-bdd` plumbing:

- configure outcome `c ∈ {Ok, Err(e), Panic(q)}`;
- runner outcome `r ∈ {Return(v), Panic(p)}`;
- cleanup outcome `k ∈ {Ok, Err(e), Panic(q)}`.

The observable result is: the value returned, the error returned, or the panic
raised; the number of runner invocations; the number of cleanup invocations;
and the `tracing::error!` records emitted. `m(φ, x)` is the augmented message
for phase `φ` and payload `x`; `w(φ, e)` is error `e` wrapped with phase `φ`.

```plaintext
c         r          k          result                runner  cleanup  error records
Err(e)    -          -          Err(w(configure, e))  0       0        0
Panic(q)  -          -          panic m(configure, q) 0       0        0
Ok        Return(v)  Ok         Ok(v)                 1       1        0
Ok        Return(v)  Err(e)     Err(w(cleanup, e))    1       1        0
Ok        Return(v)  Panic(q)   panic m(cleanup, q)   1       1        0
Ok        Panic(p)   Ok         panic m(steps, p)     1       1        1
Ok        Panic(p)   Err(e)     panic m(steps, p)     1       1        2
Ok        Panic(p)   Panic(q)   panic m(steps, p)     1       1        2
```

On the panic path one error record names the step panic; a second records the
cleanup failure. Preconditions: unwinding panics (`panic = "unwind"`, the test
default); any `ScenarioMetadata`. The representation relation: every
`BevyScenario` passed to the runner and to cleanup refers to the single `App`
that `configure` received.

The table enumerates every reachable cell of this outcome-class model; it is
exhaustive over classes, not over values. One representative per class suffices
because `run<T>` is parametric in `T` and cannot branch on `v`; `run` never
branches on the contents of an error, a configure or cleanup payload, or the
metadata (a review checklist item at EP-M3); and the only payload-dependent
code, `payload_text`, has its three classes tested separately. The table does
not prove that the code matches the model; sensitivity evidence comes from
seeded faults and the mutation audit. At the type level, `ScenarioRunner` is
`FnOnce`, so "runner runs at most once" is guaranteed by the compiler; the
tests establish "at least once" and the cleanup counts.

### Invariants and lemmas

- EP-INV-1 configure precedence: the runner is invoked only after `configure`
  returned `Ok` on the same app.
- EP-INV-2 cleanup exactly once: if `configure` returned `Ok`, cleanup is
  invoked exactly once, after the runner terminates, on both paths; otherwise
  never.
- EP-INV-3 panic identity: whenever `run` panics, the payload is a `String`
  `m(φ, x)` naming the correct phase and containing the harness type name,
  feature path, one-based line, quoted scenario name, and the payload text
  (`&str`, `String`) or the placeholder `Box<dyn Any>` for opaque payloads; on
  the panic path `x` is the step payload regardless of `k`.
- EP-INV-4 value transparency: `r = Return(v) ∧ k = Ok ⇒ run = Ok(v)`.
- EP-INV-5 shared app identity: runner and cleanup observe the same `App`
  (state written by a step is visible to cleanup).
- EP-INV-6 tick additivity: for any sequence of `update()` and
  `update_times(n)` calls on a `MinimalBevyProfile` scenario, `FrameCount`
  equals the sum of advancements, and an `Update`-schedule counter system runs
  exactly that many times; `update_times(0)` is a no-op. Wrapping at 2³² is
  axiom A1 and is not exercised.
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
- EP-INV-11 no leak: when no step keeps a clone, the `App` has been dropped by
  the time `run` returns or its panic reaches the caller, on both paths.
- EP-INV-12 no runner or finalization: the harness never calls `App::run`,
  `App::finish`, or `App::cleanup`.
- EP-INV-13 broken schedule diagnosis: after a system panic inside `update`,
  a later `update` or `update_times` panics with the crate-prefixed
  missing-`Main` message rather than Bevy's.
- EP-INV-14 metadata fidelity: the line a step sees equals the one-based line
  of its `Scenario:` declaration in the bound feature file, and the feature
  path ends with that file's name.

Lemmas connecting these to EP-REQ-3 and EP-REQ-4 (hypotheses about proof
structure, to be refined by implementation):

- L1: the panic-path cleanup wrapper and the guarded handle drop return
  normally for every cleanup outcome (they catch panics). With EP-INV-2 this
  gives EP-INV-3 on the panic path: nothing between catching `p` and raising
  `m(steps, p)` can unwind.
- L2: `m` is a total, panic-free function of
  `(phase, type name, metadata, payload)` (pure formatting).
- L3: the macro-generated test calls `run` exactly once per scenario on a
  `Default`-constructed harness (axiom A4), so EP-INV-1 to EP-INV-5 hold for
  generated scenarios, not only for hand-built requests.

### Axioms (external assumptions, not verified here)

- A1 Bevy: `App::update` runs `Main` once, including `Update` once and the
  `FrameCount` increment (`wrapping_add(1)`) once.
- A2 Rust: `RefCell` resets its borrow flag when a `RefMut` drops during
  unwinding; `catch_unwind` catches every unwinding panic; `panic_any` invokes
  the panic hook; libtest's `should_panic(expected)` matches a `String` payload
  by substring. (EP-M0 probes the first two under Cranelift.)
- A3 Platform: tests build with `panic = "unwind"`. Under `panic = "abort"`
  no cleanup can run; documented, not handled.
- A4 `rstest-bdd` 0.6.0: generated tests construct the harness with
  `<H as Default>::default()`, call `run` once, inject the context by `TypeId`
  under `rstest_bdd_harness_context`, report step failures by panicking inside
  the runner, and turn `Err(HarnessError)` into a panic.
- A5 `HarnessError` exposes only `RuntimeBuildFailed(std::io::Error)`.
- A6 Bevy: a system panic during `update` loses the `Main` schedule; the
  main sub-app's `update_schedule` label and `Schedules::contains` reveal it.

### Test profiles and probes

All lifecycle tests use profile types defined in the test module, one per
configure and cleanup behaviour. Tripwires make a wrongly-run hook fail loudly;
recorders make a correctly-run hook observable through a scenario clone that
the runner stashes in a local `RefCell<Option<BevyScenario>>` borrowed by the
runner closure (`ScenarioRunner::new` accepts non-`'static` closures).

```plaintext
profile          configure                                         cleanup
Recording        adds an Update counter system; Ok                 CleanupCalls += 1; Ok
ConfigureFails   Err(io::Error::other("configure"))                panics "tripwire: cleanup must not run"
ConfigurePanics  panics "configure boom"                           panics "tripwire: cleanup must not run"
CleanupFails     as Recording                                      CleanupCalls += 1; Err("cleanup")
CleanupPanics    as Recording                                      CleanupCalls += 1; panics "cleanup boom"
FinalizeProbe    adds a plugin whose finish and cleanup panic; Ok  Ok
FixedTimestep    MinimalPlugins, FixedTimesteps(1), FixedUpdate counter; Ok   Ok
NotSendProfile   (compile-time only: a struct holding Rc<()>)      -
```

Runners that must not run panic with "tripwire: runner must not run". The leak
rows use `Recording`: the runner inserts a `DropFlag(Arc<AtomicBool>)` resource
whose `Drop` sets the flag, and the test asserts the flag is clear inside the
runner and set after `run` returns or its panic is caught. Test-only `tracing`
capture uses a handwritten subscriber that counts `ERROR` events and records
their `phase` and `scenario_name` fields, installed with
`tracing::subscriber::with_default` around the `run` call.

EP-M0 unwinding probe (a scratch test, run once and not committed): under
`cargo --config tools/dev-fast/config.toml`, a test that panics inside
`catch_unwind(AssertUnwindSafe(..))` while holding a `RefCell` `borrow_mut`,
then asserts the payload was caught and the cell is borrowable again.

### Obligations, methods, and evidence

- Obligation: EP-INV-1, -2, -4, the result column, and the error-record
  column for the eight model rows. Method: a parameterized `rstest` table, one
  case per row, driving `BevyHarness::<P>::run` with hand-built requests and
  the test profiles above. Rationale: the outcome-class domain is finite;
  enumerating it covers every class, and tripwires make the zero counts
  observable. Artefact: `crates/rstest-bdd-harness-bevy/src/harness_tests.rs`
  (test profiles in `src/harness_tests/profiles.rs`, the recording subscriber in
  `src/harness_tests/recorder.rs`). Evidence: the focused run fails before
  EP-M2 and EP-M3 code exists (missing items, then wrong behaviour) and passes
  after. Non-vacuity: each row asserts runner and cleanup counts through the
  stashed clone or a tripwire; hand-seeded faults "skip cleanup on the panic
  path", "run cleanup twice", "raise the cleanup panic instead of the step
  panic", and "run cleanup after a configure error" must each fail at least one
  row (applied once during EP-M3, results recorded, then reverted).
- Obligation: EP-INV-3 and L2. Method: `rstest` cases for `payload_text` over
  the three payload classes, and `insta` snapshots of `scenario_panic_message`
  for each phase, with fixed metadata
  `("tests/features/demo.feature", "Demo scenario", 12)` and a fixed harness
  string. Lifecycle rows assert the raised payload is a `String` that starts
  with `rstest-bdd-harness-bevy scenario panicked: phase=<φ>, harness=` and
  contains `BevyHarness`, the feature path and line, and the quoted name.
  Artefact: `src/panic_tests.rs`, `src/snapshots/`. Evidence: `make test` (CI
  never accepts new snapshots; drift fails). Non-vacuity: non-default metadata
  values mean a formatter that drops or swaps a field changes a snapshot.
- Obligation: EP-INV-5. Method: the `Recording` profile's cleanup increments
  a resource the runner created; the row fails if the count is absent.
  Non-vacuity: building a second app for cleanup fails the row.
- Obligation: EP-INV-6. Method: `proptest` with strategy
  `vec(prop_oneof![Just(Op::Update), (0_u32..=8).prop_map(Op::UpdateTimes)], 0..=16)`
  (generated by construction, no filtering), applying the operations inside
  the runner closure of a `BevyHarness::<MinimalBevyProfile>` run and asserting
  `FrameCount` and a toy `Update` counter equal the running sum after each
  operation; 64 cases. Named `rstest` cases pin the empty sequence, a lone
  `UpdateTimes(0)`, and `UpdateTimes(0)` first. Artefact:
  `tests/tick_properties.rs`. Non-vacuity: hand-seeded faults "`update_times`
  runs `count - 1`", "`count + 1`", and "`update` is a no-op" must each fail.
- Obligation: EP-INV-7 and EP-INV-13. Method: named `#[gtest] #[rstest]` unit
  tests: nested `with_app` inside `with_app`; `update` inside `with_world`
  through a clone; borrowability after a caught accessor panic; and a toy
  `Update` system that panics once, after which `update()` panics with the
  crate's missing-`Main` message. Artefact: `src/context_tests.rs`.
  Non-vacuity: assertions match the crate prefix, so a raw `BorrowMutError` or
  Bevy's own message fails.
- Obligation: EP-INV-8, EP-INV-9, EP-INV-12, and the `FixedTimestep` seam.
  Method: named unit tests; the upstream
  `rstest_bdd_harness::policy_conformance::assert_attribute_policy_conformance`
  for the policy; `FinalizeProbe` runs a scenario with three updates and must
  not trip; `FixedTimestep` shows exactly one `FixedUpdate` per `update()`.
  Artefacts: `src/profile_tests.rs`, `src/policy_tests.rs`,
  `src/harness_tests.rs`.
- Obligation: EP-INV-11. Method: two lifecycle rows (return and panic) with
  the `DropFlag` resource. Non-vacuity: the flag is asserted clear inside the
  runner first; a mutant that leaks the handle (for example `mem::forget` of
  the scenario) fails.
- Obligation: the `Send + Sync` claim. Method: a compile-time assertion
  `fn assert_send_sync<T: Send + Sync>() {}` called with
  `BevyHarness<NotSendProfile>`. Artefact: `src/harness_tests.rs`.
- Obligation: L3, EP-REQ-4, and EP-INV-14 (end to end). Method: behavioural
  scenarios through the real macro, bound with `harness = BevyHarness`
  (defaulted parameter) and `harness = BareHarness` (a type alias), including
  one step that takes `BevyScenario` by value; a metadata step whose oracle is
  independent of the harness (it scans the `include_str!` text of the bound
  feature file for the `Scenario:` line); and a deliberately failing scenario
  annotated
  `#[should_panic(expected = "rstest-bdd-harness-bevy scenario panicked: phase=steps")]`.
  Artefacts: `tests/headless_scenario.rs` (all bindings, one binary) with
  steps in `tests/headless_scenario/steps.rs`, and the feature files below.
  Non-vacuity: the bare-profile scenario asserts `FrameCount` is absent, which
  fails if the alias resolved to the minimal profile; the `should_panic` test
  fails if the harness re-raises the step loop's message unaugmented.
- Obligation: EP-INV-10. Method: existing `tests/extraction_boundary.rs` plus
  `cargo tree -p rstest-bdd-harness-bevy -e normal,dev` at EP-M5.

Mutation audit (advisory, suite-level non-vacuity): at EP-M5 run
`cargo mutants` on `src/harness.rs` and `src/panic.rs` (commands below). It
replaces function bodies and operators; it does not generate the structural
faults above, which stay hand-seeded. Every surviving mutant is killed by a new
test or recorded in `Decision log` with a reason (for example, an equivalent
mutant in a `Debug` impl). Record the caught, missed, unviable, and timeout
counts in `Outcomes & retrospective`.

### Where formal verification is and is not warranted

Verus is not warranted, for a decisive reason: the only pure kernel this work
could expose is a decision function `decide(c, r, k)` that maps outcome classes
to results. Proving it would restate the table above, which AGENTS.md rules out
as a non-substantive proof ("not merely a restatement of the assumed
property"). The real risk lives in the glue: `catch_unwind` around a boxed
`FnOnce`, `Rc<RefCell<App>>` sharing, the panic hook, and calls into Bevy and
`rstest-bdd`. Verus does not model unwinding and cannot reach through those
third-party crates, which this plan treats as axioms A1–A6. There is no
contractual business logic or arithmetic kernel whose guarantee must hold over
an unbounded domain; tick additivity reduces to axiom A1 plus a loop, covered
by properties and seeded faults. Kani is also unsuitable: besides the same
restatement problem, it treats panics as verification failures and does not
support unwinding through `catch_unwind`. No `assume`, `admit`, or trusted
proof body is introduced. If implementation exposes a pure kernel with an
unbounded contract (none is anticipated), return to this section before
continuing.

## Plan of work

Stage A (EP-M0) establishes the baseline, probes unwinding, and drafts the ADR.
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
|   |-- harness.rs         BevyHarness<P>, HarnessAdapter impl, phase helpers,
|   |                      ProfileHookError
|   |-- harness_tests.rs   lifecycle table, leak, finalize, Send/Sync checks
|   |-- harness_tests/
|   |   |-- profiles.rs    tripwire and recording test profiles
|   |   `-- recorder.rs    handwritten tracing subscriber for tests
|   |-- panic.rs           pure: Phase, payload_text, scenario_panic_message
|   |-- panic_tests.rs
|   `-- snapshots/         insta snapshots of the augmented message
`-- tests/
    |-- headless_scenario.rs   every harness-backed #[scenario] binding
    |-- headless_scenario/
    |   `-- steps.rs           shared step definitions
    |-- tick_properties.rs     extended with BevyScenario properties
    `-- features/
        |-- headless_scenario.feature   unchanged
        |-- bevy_scenario.feature       new
        `-- bevy_panic.feature          new, one deliberately failing step
```

`harness.rs` holds lifecycle orchestration; `panic.rs` stays pure formatting
with no Bevy or profile dependency. If `harness.rs` approaches 400 lines, move
`ProfileHookError` into `src/harness/error.rs`.

## Milestones and plateaus

### EP-M0: baseline, unwinding probe, and ADR 010 (no production code)

- Outcome: a recorded baseline, a passed unwinding probe, and a Proposed ADR.
  Run the focused crate tests and record the pass count. Confirm
  `multi_threaded` is off with
  `cargo tree -p rstest-bdd-harness-bevy -e features -i bevy_ecs`. Run the
  unwinding probe from `Test profiles and probes` in a scratch test file under
  dev-fast, record the result, and delete the file. Write
  `docs/adr/adr-010-bevy-harness-failure-contract.md` (status Proposed) with
  the lifecycle table, phase tags, `panic_any` re-raise, phase-tagged error
  wrapping, the never-`finish` rule, the SemVer coupling, and options
  considered (Table 1): `resume_unwind` plus direct stderr (GPUI's approach), a
  `Drop`-guard cleanup (a cleanup panic during unwinding aborts), a `&mut App`
  context (cleanup cannot reach the app after the macro drops the context), a
  profile carried as a value (the harness is only built through `Default`), a
  crate-owned profile error type (interface change), and deterministic time by
  default (`FixedTimesteps`; deviates from design §5). Reference the ADR from
  design §6 and add it to `docs/contents.md`.
- Requirements: EP-REQ-6 (partial); axiom A2 probe.
- Acceptance: baseline and probe result recorded in `Progress`;
  `make markdownlint` and `make nixie` pass.
- Conformance check: design followed; no code committed; no dependency
  change.
- Recovery: documentation-only; revert the commit. If the probe fails,
  escalate (tolerance).
- Remaining gaps: ADR acceptance by the maintainer, required before EP-M3.
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

- Outcome: `BevyScenario` with all five accessors and the schedule-presence
  check; `BevyProfile`, `BareBevyProfile`, and `MinimalBevyProfile`;
  `BevyHarness<P>` implementing `HarnessAdapter` with `Context = BevyScenario`
  for the configure-`Err` row and the runner-returns rows with cleanup `Ok` and
  `Err` (rows 1, 3, 4). These land together because `BevyProfile::cleanup` takes
  `&BevyScenario` and `BevyScenario::new` is `pub(crate)` with
  `BevyHarness::run` as its only production caller; landing the context alone
  would leave dead code under `-D warnings`.
- Red: `src/context_tests.rs` — metadata retention; `with_app` and
  `with_world` return closure results and mutate the shared app; `update`
  advances `FrameCount` by one; `update_times(0)` and `update_times(3)`; clones
  share state; nested-borrow panics carry the `rstest-bdd-harness-bevy:` prefix
  (caught with `catch_unwind(AssertUnwindSafe(..))`, payload inspected);
  borrowability after a caught accessor panic; the broken-`Main` diagnosis.
  `src/profile_tests.rs` — `BareBevyProfile::configure` leaves `TimePlugin`
  unadded and inserts no `FrameCount` while an `Update` counter system still
  runs on `update()`; `MinimalBevyProfile::configure` adds `TimePlugin` and
  `FrameCount`; both return `Ok(())`. `src/harness_tests.rs` — rows 1, 3, and
  4; the leak row (return path); `FinalizeProbe`; `FixedTimestep`; the
  `Send + Sync` assertion; `BevyHarness::default()` and `BevyHarness::new()`
  equivalence; the request's metadata reaches the scenario.
- Green: implement `src/context.rs`, the trait and profile enums in
  `src/profile.rs`, and `src/harness.rs` with `ProfileHookError`. Split `run`
  into small helpers (`configure_app`, `run_steps`, `finish_scenario`) to stay
  under the complexity threshold. Emit a `tracing::debug!` event ("starting
  Bevy scenario") carrying `harness_type`, `feature_path`, `scenario_name`, and
  `scenario_line` before running. A runner panic still propagates unaugmented
  at this plateau.
- Requirements: EP-REQ-1, EP-REQ-2, EP-INV-1, EP-INV-2 (normal path),
  EP-INV-4, EP-INV-5, EP-INV-7, EP-INV-8, EP-INV-11 (return path), EP-INV-12,
  EP-INV-13.
- Acceptance: new tests pass; gates pass.
- Conformance check: names match design §4; profiles are uninhabited enums;
  `BevyScenario` wraps `Rc<RefCell<App>>`; no raw `RefMut` escapes; no
  `Send`/`Sync` forwarding; `HarnessError` used only for profile hook failures;
  only `BevyHarness::new` added beyond the design's items.
- Recovery: revert milestone commits; `profile.rs` returns to its EP-M1
  state; the EP-M1 plateau remains valid.
- Remaining gaps: rows 2 and 5–8 and the panic-path leak row until EP-M3;
  recorded in `Progress` at the milestone boundary.
- Compatibility decision: none.

### EP-M3: panic and cleanup path (requires accepted ADR 010)

- Outcome: all eight lifecycle rows hold; panics carry phase-tagged,
  hook-visible messages; the format is pinned.
- Red: `src/panic_tests.rs` — `payload_text` for `&'static str`, `String`,
  and an opaque payload (`rstest` cases); `scenario_panic_message` snapshots
  for the three phases. `src/harness_tests.rs` — rows 2, 5, 6, 7, and 8, the
  panic-path leak row, and error-record assertions for every row through the
  recording subscriber; each panic row asserts the payload is a `String`
  beginning with the phase-tagged prefix, cleanup counts per the table, and
  (rows 7–8) that the step message, not the cleanup failure, is raised.
- Green: implement `src/panic.rs` (pure) and extend `BevyHarness::run`: wrap
  configure and the runner each in `catch_unwind(AssertUnwindSafe(..))`; on a
  step panic build `m(steps, p)`, emit `tracing::error!` with `phase`,
  `harness_type`, `feature_path`, `scenario_name`, `scenario_line`, and the
  message, run cleanup inside `catch_unwind` (logging an `Err` or a panic),
  drop the harness's scenario handle inside `catch_unwind` (logging a drop
  panic), forget the original payload (as the GPUI harness does, so a panicking
  payload `Drop` cannot double-panic), and raise
  `std::panic::panic_any(message)`. Configure panics and normal-path cleanup
  panics are raised as `m(configure, q)` and `m(cleanup, q)`. Match every
  `catch_unwind` result explicitly.
- Requirements: EP-REQ-3, EP-INV-2, EP-INV-3, EP-INV-11 (panic path), L1,
  L2.
- Acceptance: new tests and snapshots pass; gates pass; the four hand-seeded
  lifecycle faults each fail a row (results recorded, faults reverted); the
  review checklist confirms `run` does not branch on payload, error, or
  metadata contents.
- Conformance check: the message carries every field design §6 lists plus
  the phase; cleanup failures on the panic path are logged, not returned; `run`
  contains no `panic!` or `assert!` macro (`panic_in_result_fn`).
- Recovery: revert milestone commits; the EP-M2 plateau remains valid.
- Remaining gaps: process-level stderr and tracing-output snapshots of a
  generated scenario (roadmap `0.1.3`).
- Compatibility decision: none.

### EP-M4: harness-backed behavioural scenarios and tick properties

- Outcome: RM-3 and the end-to-end diagnostic are demonstrated through the
  real macro, in one integration binary.
- Red: add `tests/features/bevy_scenario.feature` and
  `tests/features/bevy_panic.feature` (below); move the step definitions into
  `tests/headless_scenario/steps.rs`; bind all scenarios in
  `tests/headless_scenario.rs` with `attributes = BevyAttributePolicy` and
  `harness = BevyHarness` or `harness = BareHarness`
  (`type BareHarness = BevyHarness<BareBevyProfile>;`); re-bind the existing
  headless scenario the same way, with steps taking
  `#[from(rstest_bdd_harness_context)] scenario: &BevyScenario`. Before the new
  steps exist, the generated tests fail with `Step not found`; that is the red.
  Extend `tests/tick_properties.rs` with the sequence property and its named
  cases.
- Green: implement the steps (steps that can fail return `StepResult` and use
  no `.expect()` or `.unwrap()`); keep `include_str!` bindings for all three
  feature files (the 0.1.1 contract, and the metadata oracle's input); remove
  the `RefCell<App>` fixture and the `allow_fixture_expansion_lints` import if
  nothing else uses them.
- Requirements: EP-REQ-4, L3, EP-INV-6, EP-INV-8 (through the macro),
  EP-INV-14.
- Acceptance: `make test` shows the five harness-backed scenario tests plus
  the `should_panic` scenario passing; gates pass.
- Conformance check: the bare name and the alias both compile (RM-3);
  `headless_scenario.feature` is byte-for-byte unchanged (design §9).
- Recovery: revert milestone commits; the EP-M3 plateau remains valid.
- Remaining gaps: the `trybuild` compile-pass alias fixture (roadmap
  `0.1.3`).
- Compatibility decision: none.

Feature specification,
`crates/rstest-bdd-harness-bevy/tests/features/bevy_scenario.feature`:

```gherkin
Feature: Bevy scenario harness

  Scenario: Steps receive the scenario metadata from the harness
    Given a scenario driven by the minimal Bevy harness
    Then the scenario metadata matches its declaration in the feature file

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
with `harness = BareHarness`. "Given a scenario driven by the minimal Bevy
harness" asserts `TimePlugin` is added; "the bare Bevy harness" asserts it is
not. "The toy counter reads {n}" takes the scenario by value
(`scenario: BevyScenario`), exercising the `Clone` form. The metadata step
locates the line of `Scenario: {name}` in the `include_str!` text, adds one for
one-based numbering, and compares it and the path suffix with
`scenario.metadata()`.

`crates/rstest-bdd-harness-bevy/tests/features/bevy_panic.feature`:

```gherkin
Feature: Bevy scenario failure diagnostics

  Scenario: A failing step reports its scenario
    Given a scenario driven by the minimal Bevy harness
    When a step fails with "deliberate failure"
```

It binds with `harness = BevyHarness` and
`#[should_panic(expected = "rstest-bdd-harness-bevy scenario panicked: phase=steps")]`.
The failing step returns `Err("deliberate failure")` as a `StepResult`, so the
step loop's panic (axiom A4) is what the harness augments.

The unchanged `headless_scenario.feature` reads:

```gherkin
Feature: Headless Bevy scaffolding

  Scenario: A minimal application advances one tick
    Given a minimal headless Bevy application
    When the schedule advances once
    Then the frame count reads 1
```

### EP-M5: documentation, mutation audit, roadmap, and final gates

- Outcome: documentation matches the delivered API; mutation audit recorded;
  roadmap `0.1.2` marked done and `0.1.3` reworded to its remaining scope.
- Work: update design §§4–6 and §9 with the delivered decisions and layout,
  change its status line to say §§4–7 are implemented, and reference ADR 010;
  rewrite `crates/rstest-bdd-harness-bevy/README.md` (purpose, suite inventory,
  files table, extending section); add a short subsection to
  `docs/users-guide.md` §4 (harness-backed scenario example, the direct
  `rstest-bdd-harness` development dependency, the error idiom, explicit
  advancement, cleanup must not advance the app, clones must not outlive the
  scenario, the never-`finish` rule and `FixedTimesteps` technique); update
  `docs/rstest-bdd-harness-bevy-adoption.md` lines 55–56; update
  `docs/repository-layout.md` (src and tests entries); add to
  `docs/developers-guide.md` §2.1 the internal conventions (tripwire and
  recording test profiles instead of globals; the handwritten recording
  subscriber; `panic_any` rather than `resume_unwind` or `panic!` in `Result`
  functions; the nested-borrow and broken-`Main` messages; the scoped
  mutation-audit command); add this plan to `docs/contents.md`; tick roadmap
  `0.1.2` (`- [x]`) and reword `0.1.3`. Run the scoped `cargo mutants` audit
  and triage survivors.
- Requirements: EP-REQ-5, EP-REQ-6.
- Acceptance: `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  `make nixie`, and `make spelling` pass; `cargo tree` shows no forbidden
  package; the mutation audit has no unexplained survivors (or was stopped by
  tolerance and escalated).
- Conformance check: every Conformance-basis trace link has evidence; design
  and ADR agree with code; no unapproved public item (compare the crate's
  `cargo doc` output against `Interfaces and dependencies`).
- Recovery: documentation commits revert independently.
- Remaining gaps: roadmap `0.1.3` (reworded), `0.1.4`, and `0.1.5`.
- Compatibility decision: none.

## Concrete steps

Run everything from the worktree root
`/home/leynos/.lody/repos/github---leynos---skyjoust/worktrees/23ed6522-c112-45c2-9697-efd9fefd0645`.
Log every gate with `tee` using
`/tmp/$ACTION-skyjoust-0-1-2-bevy-harness-public-api.out`.

Focused loop during a milestone (append a Nextest filter such as `lifecycle`,
`context`, or `headless_scenario` to narrow it):

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

Mutation audit (EP-M5, advisory). List first and check the count against the
tolerance; then run in place so the existing `target/` is reused; finally
confirm the worktree is clean:

```bash
cargo mutants --list -p rstest-bdd-harness-bevy \
  -f crates/rstest-bdd-harness-bevy/src/harness.rs \
  -f crates/rstest-bdd-harness-bevy/src/panic.rs | wc -l
cargo mutants --in-place -p rstest-bdd-harness-bevy \
  -f crates/rstest-bdd-harness-bevy/src/harness.rs \
  -f crates/rstest-bdd-harness-bevy/src/panic.rs \
  --test-tool nextest --cargo-arg=--config=tools/dev-fast/config.toml \
  -- -E 'not binary(macro_ui) & not binary(extraction_boundary)' 2>&1 \
  | tee /tmp/mutants-skyjoust-0-1-2-bevy-harness-public-api.out
git status --short   # must be empty: --in-place restores each mutated file
```

Confirm the exact flag spelling with `cargo mutants --help` (version 27.1.0 was
installed at drafting) before running.

Commit after each green-and-gated step using the `commit-message` skill; prefer
delegating full gate runs to the `scrutineer` agent.

## Validation and acceptance

Quality criteria:

- Tests: `make test` passes, including the eight lifecycle rows and the
  leak, finalize, fixed-timestep, and `Send + Sync` checks; context tests;
  panic formatter tests and snapshots; profile and policy tests; the extended
  property tests; four new behavioural scenarios, the re-bound headless
  scenario, and the `should_panic` end-to-end scenario. Each new test fails
  before its implementation and passes after.
- Verification: EP-INV-1 to EP-INV-14 have the evidence named in
  `Verification plan`; the hand-seeded faults are recorded as rejected; the
  advisory mutation audit has no unexplained survivors. No Verus milestone
  exists, so no Verus command is part of acceptance.
- Lint and format: `make check-fmt` and `make lint` (rustdoc, Clippy,
  Whitaker) pass with no new suppression.
- Documentation: `make markdownlint`, `make nixie`, and `make spelling` pass.

Behavioural acceptance, observable by a human:

1. The focused run filtered to `headless_scenario` lists six passing
   scenario tests, one of them the `should_panic` diagnostic scenario.
2. Optional colour, not acceptance: temporarily change
   `Then the toy counter reads 3` to `4`. The failure output printed by Nextest
   contains a line beginning
   `rstest-bdd-harness-bevy scenario panicked: phase=steps` that names
   `tests/features/bevy_scenario.feature`, the scenario's line, and its quoted
   name. Revert the edit.
3. `git diff main -- crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature`
   is empty.

## Idempotence and recovery

Every step is re-runnable. Tests create fresh apps and use no global or
environment state. Snapshot updates are explicit (`cargo insta review` or
`INSTA_UPDATE=always` on a focused run) and must be reviewed before commit;
delete stray `.snap.new` files. Each milestone ends in a gated commit;
`git revert` of a milestone's commits returns to the previous plateau. After an
interrupted `cargo mutants --in-place` run, restore any mutated file with
`git checkout -- crates/rstest-bdd-harness-bevy/src`. Never use `git stash`
without a unique tag (the stash stack is shared with other worktrees).

## Artefacts and notes

The upstream GPUI harness message format, which this crate extends with the
phase and harness type:

```plaintext
rstest-bdd-harness-gpui scenario panicked: feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {message}
```

This crate's format (one line; wrapped here for width):

```plaintext
rstest-bdd-harness-bevy scenario panicked: phase={phase}, harness={harness_type},
feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {payload_text}
```

## Interfaces and dependencies

No new crate enters the lockfile. `crates/rstest-bdd-harness-bevy/Cargo.toml`
gains `insta = "1"` under `[dev-dependencies]` (resolves to the locked
`1.49.0`).

Upstream items every lifecycle test builds by hand (from `rstest-bdd-harness`
0.6.0, already re-exported by this crate):

```rust,no_run
impl ScenarioMetadata {
    pub fn new(
        feature_path: impl Into<String>,
        scenario_name: impl Into<String>,
        scenario_line: u32,
        tags: Vec<String>,
    ) -> Self;
}
impl<'a, C, T> ScenarioRunner<'a, C, T> {
    pub fn new(inner: impl FnOnce(C) -> T + 'a) -> Self;
}
impl<'a, C, T> ScenarioRunRequest<'a, C, T> {
    pub fn new(metadata: ScenarioMetadata, runner: ScenarioRunner<'a, C, T>) -> Self;
    pub fn into_parts(self) -> (ScenarioMetadata, ScenarioRunner<'a, C, T>);
}
pub trait HarnessAdapter {
    type Context: std::any::Any;
    fn run<T>(&self, request: ScenarioRunRequest<'_, Self::Context, T>) -> HarnessResult<T>;
}
```

In `src/profile.rs`:

```rust,no_run
pub trait BevyProfile: 'static {
    /// Configures the fresh application before any step runs. Profiles that
    /// need plugin finalization call `app.finish()` and `app.cleanup()` here;
    /// the harness never does.
    fn configure(app: &mut App, metadata: &ScenarioMetadata) -> HarnessResult<()>;

    /// Releases scenario resources after the runner finishes or panics. Must
    /// not advance the application.
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

`with_app` uses `RefCell::try_borrow_mut` and, on failure, panics with the
first message below. `with_world` delegates to `with_app`. `update` and
`update_times` borrow once, check that `app.main().update_schedule` names a
schedule still present in the world's `Schedules`, and otherwise panic with the
second message below; `update_times` loops inside the one borrow. Each public
accessor documents `# Panics`.

```plaintext
rstest-bdd-harness-bevy: BevyScenario is already borrowed; accessor closures must not call
with_app, with_world, update, or update_times
rstest-bdd-harness-bevy: the Main schedule is missing; a system panicked during an earlier
update, so this application cannot advance
```

Each message is one line in code; they are wrapped here for width.

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

// Crate-private: names the failing hook and keeps the profile error as source.
struct ProfileHookError { phase: Phase, source: HarnessError }
```

`BevyHarness`'s rustdoc example shows a profile author running a hand-built
`ScenarioRunRequest` against their profile, which is how downstream crates
unit-test `configure` and `cleanup`.

In `src/panic.rs` (crate-private, pure):

```rust,no_run
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase { Configure, Steps, Cleanup }

pub(crate) fn payload_text(payload: &(dyn Any + Send)) -> &str;
pub(crate) fn scenario_panic_message(
    phase: Phase,
    harness_type: &str,
    metadata: &ScenarioMetadata,
    payload: &(dyn Any + Send),
) -> String;
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
  reports and prior-art research (commit `e84f4ac`).
- Revision 2, 2026-10-18: folded in the community-of-experts review
  (verdict: proceed with conditions). The re-raise now uses `panic_any`, so the
  message reaches test output. Augmentation is phase-tagged across configure,
  steps, and cleanup, and profile errors are wrapped with their phase. Tripwire
  profiles and a handwritten tracing recorder make the lifecycle table's zero
  counts and error records observable without globals or new dependencies. The
  exhaustiveness claim was reworded to classes, and the Verus and Kani argument
  was strengthened. Invariants EP-INV-11 to EP-INV-14 were added (no leak, never
  `finish`, broken-`Main` diagnosis, metadata fidelity). The brittle
  line-number step became an independent oracle. A `should_panic` end-to-end
  scenario was added, and all bindings were consolidated into one test binary.
  The proptest strategy is now fully specified. The mutation audit is advisory,
  scoped, in-place, and capped. An EP-M0 Cranelift unwinding probe and ADR 010
  acceptance before EP-M3 were added, the `World::drop` risk was corrected, and
  EP-M5 now rewords roadmap `0.1.3` to its remaining scope. The plan keeps its
  six-milestone shape and awaits maintainer approval.

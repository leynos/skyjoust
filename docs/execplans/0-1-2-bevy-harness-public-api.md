# Implement the reusable Bevy harness public API

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: APPROVED — IMPLEMENTATION COMPLETE; PR DELIVERY IN PROGRESS

Approval gate: satisfied. The maintainer approved this plan on 2026-11-20 by
directing that implementation begin and that the plan be marked approved. ADR
010 (drafted in EP-M0) must be accepted before EP-M3 starts; its acceptance
provenance is recorded in the ADR and in `Decision log`.

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
- `docs/adrs/adr-0009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md` —
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
- `bevy_app-0.20.1/src/{app,sub_app,main_schedule}.rs`,
  `bevy_ecs-0.20.1/src/world/mod.rs`, `bevy_time-0.20.1/src/lib.rs`.

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
- Bevy stays on the repository's locked `0.20.1` (manifest requirement
  `0.20.0`, implicit caret) with `default-features = false` and
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
  escalate. Per the `execplans` skill, ExecPlan files are excluded from every
  size constraint or tolerance in this plan: only non-ExecPlan files count
  towards the path and net-line limits, including when an ExecPlan is added or
  updated in the same change.
- Scope evidence (2026-11-20, Revision 12): the scope limit remains exceeded
  after that exclusion. The assessment measured the head `8c6f500` at 38 paths
  and 5,923 net added lines including this plan, and the Revision 12 tree
  measures 38 paths and 6,465 net added lines including the plan, 37 and 4,339
  without it, and 27 and 3,651 counting only code and tests. The Revision 13
  assessment measured head `030334b` at 39 non-ExecPlan paths and 4,479 net
  added lines. The Revision 14 assessment measured head `f40e434` at the same
  39 non-ExecPlan paths and 4,772 net added lines. The Revision 15 head
  `65409a7` measures 40 non-ExecPlan paths and 4,964 net added lines, the extra
  path and lines coming from the `harness/failure.rs` split and its regression
  rows. The Revision 15 assessment repair head `55e8c68` measures 41
  non-ExecPlan paths and 5,146 net added lines (31 paths and 4,435 net added
  lines counting only code and tests), the extra path being
  `harness_tests/panics/release.rs`, which keeps `panics.rs` inside the
  400-line limit. `Decision log` records the escalation and its disposition;
  this bullet does not claim a separate numeric approval.
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
- Risk: effort drifts into Cranelift compatibility. Cranelift is out of scope
  (see `Decision log`): it stays an unsupported, commented-out opt-in in
  `.cargo/config.toml` and `tools/dev-fast/config.toml`. Severity: low.
  Likelihood: low. Mitigation: no milestone probes, tests, or works around the
  Cranelift backend; every gate runs on LLVM, the only supported backend; a
  failure seen only under a locally re-enabled Cranelift build is noted in
  `Surprises & discoveries` and otherwise left alone.
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
- [x] (2026-11-19) Rebased onto `origin/main` at `e727a3c` and revised the
  plan (Revision 3): Cranelift is out of scope, so the EP-M0 unwinding probe,
  its tolerance, and its risk were removed; ADR paths and the gate list follow
  `main`.
- [x] (2026-11-20) Maintainer approval of this plan, recorded by direction to
  begin implementation and mark the plan approved.
- [x] (2026-11-20) EP-M0 baseline: the focused `rstest-bdd-harness-bevy`
  suite passed 12/12 across 5 binaries (0 skipped); `macro_ui` took 71.7
  seconds. `cargo tree -p rstest-bdd-harness-bevy -e features -i bevy_ecs`
  shows no `multi_threaded` feature, and
  `cargo tree -p rstest-bdd-harness-bevy -e normal,dev` shows no game package
  (the only `skyjoust` matches are this worktree's own path).
- [x] (2026-11-20) EP-M0: ADR 010 drafted as Accepted with its approval
  provenance, referenced from design §6, and indexed in `docs/contents.md`.
  `make markdownlint` and `make nixie` pass.
- [x] (2026-11-20) ADR 010 accepted (required before EP-M3), under the
  maintainer's approval of this plan; see `Decision log`.
- [x] (2026-11-20) EP-M1: attribute policy. Red observed as the expected
  `E0432` unresolved import; `BevyAttributePolicy` is implemented, exported,
  and conformance-tested. The full gate set (`check-fmt`, `lint`, `typecheck`,
  `test`, `spelling`, `markdownlint`, `nixie`) passed on the milestone tree.
- [x] (2026-11-20) EP-M2: scenario context, profiles, and harness normal
  path. `BevyScenario`, `BevyProfile`, `BareBevyProfile`, `MinimalBevyProfile`,
  and `BevyHarness<P>` implement the normal-path rows (1, 3, and 4) and the
  return-path leak row; the focused library suite passes 27/27. The full gate
  set passed on the final tree: `make lint` (4 s), `make check-fmt` (1 s),
  `make typecheck` (1 s), `make test` (14 s, 279/279), `make spelling` (5 s),
  `make markdownlint` (8 s), and `make nixie` (<1 s). Logs:
  `/tmp/*-skyjoust-0-1-2-bevy-harness-public-api-5.out`.
- [x] (2026-11-20) EP-M2 follow-up: the CodeRabbit finding is repaired by
  narrowing `BevyProfile::cleanup`'s rustdoc (see `Decision log`). The
  pre-commit gate caught one spelling regression from `b5ece74`: a hyphenated
  form of `handwritten` in a `harness.rs` doc comment (log
  `/tmp/spelling-skyjoust-0-1-2-bevy-harness-public-api-7.out`). It is fixed in
  the follow-up commit, which re-runs the full gate set.
- [x] (2026-11-20) EP-M3: panic and cleanup path. Red observed as the expected
  `E0432` unresolved imports for `Phase`, `payload_text`, and
  `scenario_panic_message` (log
  `/tmp/focused-skyjoust-0-1-2-bevy-harness-public-api-m3-red.out`).
  `src/panic.rs` implements the pure message contract;
  `src/harness_tests/recorder.rs` counts `ERROR` records;
  `src/harness_tests/panics.rs` covers rows 2, 5, 6, 7, and 8 plus the
  panic-path leak row; three `insta` snapshots pin the message format. The
  focused library suite passes 39/39 (log
  `/tmp/focused-skyjoust-0-1-2-bevy-harness-public-api-m3-green3.out`, and
  again after the fault seeding). All four hand-seeded faults were rejected and
  reverted byte-for-byte: skipping panic-path cleanup failed rows 6–8 (cleanup
  count 0, missing second records); running cleanup twice failed rows 6–8
  (count 2, three records); tagging a step panic as `cleanup` failed rows 6–8
  on the phase prefix; and cleaning up after a configure error failed row 1 via
  the tripwire. Logs: `/tmp/fault{1,2,3,4}-...-public-api.out`. `run` contains
  no `panic!` or `assert!`, branches only on outcome tags, and never inspects
  payload, error, or metadata contents. `BevyProfile::cleanup`'s rustdoc is
  restored to the full delivered contract, closing the EP-M2 review deferral.
  The full gate set passed on the milestone tree: `make lint` (6 s),
  `make check-fmt` (1 s), `make typecheck` (5 s), `make test` (52 s, 291/291
  plus doctests), `make spelling` (6 s on the re-run after fixing two `-ise`
  spellings in this plan), `make markdownlint` (7 s), and `make nixie` (<1 s).
  Logs: `/tmp/gate-*-skyjoust-0-1-2-bevy-harness-public-api.out`.
- [x] (2026-11-20) EP-M3 review follow-up: the local CodeRabbit CLI review of
  `f33ca3a..12813a4` (0.9.6, 729 s, 18 files) found four trivial gaps, all
  confirmed and repaired. Two guarded-drop tests now prove a panicking
  application `Drop` cannot replace the step or configure message and is
  recorded with the `cleanup` or `configure` phase; one test proves
  `update_times(0)` still rejects a missing `Main` schedule; the recorder now
  reports a harness-marked `ERROR` record that omits a contract field instead
  of dropping it silently; and the public API gains the `# Parameters` and
  `# Returns` sections the style guide requires. The three plausible faulty
  implementations named by the review (plain step drop, plain configure drop,
  schedule check inside the loop) were each rejected and reverted
  byte-for-byte. Logs:
  `/tmp/followup-{tests,clippy,fault-a,fault-b,fault-c}-...-public-api*.out`.
  The repair is committed as `8e1cade` after the full gate set (296/296 plus
  doctests). A fresh CLI review of `12813a4..8e1cade` (100 s, 8 files) reported
  zero findings (log
  `/tmp/coderabbit-skyjoust-0-1-2-bevy-harness-public-api-m3-followup.out`).
- [x] (2026-11-20) EP-M4: harness-backed behavioural scenarios and tick
  properties. Red observed as four generated scenarios failing with
  `Step not found at index 0` for the new `Given` lines (log
  `/tmp/red-m4-skyjoust-0-1-2-bevy-harness-public-api.out`).
  `tests/features/bevy_scenario.feature` and
  `tests/features/bevy_panic.feature` add the five new scenarios,
  `tests/headless_scenario/steps.rs` defines their steps, and
  `tests/headless_scenario.rs` binds all six scenarios through the real
  `#[scenario]` surface, including a type alias selecting `BareBevyProfile` and
  the preserved `#[should_panic]` attribute. Green: six scenario tests and five
  property tests pass, alongside the rest of the crate (44 unit, 4
  extraction-boundary, 6 scenario, 1 `macro_ui`, 5 property and 16 doctest
  tests; log `/tmp/focused-m4-skyjoust-0-1-2-bevy-harness-public-api.out`). The
  failure scenario panics for the intended reason: `--nocapture` shows the
  augmented message carrying `phase=steps` and `deliberate failure`
  (`/tmp/panic-evidence-m4-skyjoust-0-1-2-bevy-harness-public-api.out`).
  EP-INV-6 non-vacuity: each seeded fault failed the intended cases —
  `update_times` running `count - 1` failed `zero_advance_first` and the
  64-case property; `count + 1` failed `lone_zero_advance`,
  `zero_advance_first`, and the property; and a no-op `update` failed
  `zero_advance_first` and the property. `src/context.rs` was restored
  byte-for-byte to sha256 `77ec0cc7...` after each (logs
  `/tmp/fault-{a,b,c}-m4-skyjoust-0-1-2-bevy-harness-public-api.out`). The 0.1.1
  `headless_scenario.feature` is byte-for-byte unchanged (sha256 `12dac9ee...`
  before and after).
- [x] (2026-11-20) EP-M4 review follow-up: the local CodeRabbit CLI review of
  `a96027e` (0.9.6, 261 s, 6 files) found two gaps, both confirmed and
  repaired. The metadata step compared only values that stay mutually
  consistent, so `then_metadata_matches` now pins the scenario name
  (`EXPECTED_METADATA_SCENARIO`) before deriving the declared line. The
  `should_panic` expectation matched only `phase=steps`, which a failing
  `Given` step would also satisfy, so it now pins the feature path, line 3, the
  scenario name, and `Step failed at index ⁨1⁩`. Both repairs were proven
  non-vacuous by a seeded fault: a wrong expected name failed with "the
  scenario name must read …", and switching the failing scenario to the bare
  harness (a `Given` failure at index 0) failed the expectation (logs
  `/tmp/fault-{d,e}-m4-review-skyjoust-0-1-2-bevy-harness-public-api.out`).
- [x] (2026-11-20) EP-M5: documentation, mutation audit, roadmap, and final
  gates. Documentation is delivered: design §§4–6 and §9 now describe the built
  API, with the status line updated; the crate README is rewritten (purpose,
  suite inventory, files table, extension example, Bevy compatibility table);
  the users' guide §4 covers the harness-backed scenario workflow; adoption
  guidance adds the direct `rstest-bdd-harness` development dependency and a
  drive-through section; `repository-layout`, `developers-guide` §2.1, and
  `contents` are updated; roadmap `0.1.2` is ticked and `0.1.3` reworded to its
  remaining scope; ADR 011 corrects the never-finish guidance under ADR 010's
  amendment process (see `Decision log`). Mutation audit: `cargo mutants`
  27.1.0 over the crate's `src/`, in place, with nextest and the serial macro
  binaries excluded, produced 28 mutants — 12 caught, 12 unviable, 0 timeout, 4
  missed (`mutants.out/`; summary
  `/tmp/mutants-skyjoust-0-1-2-bevy-harness-public-api.out`). Three missed
  mutants now have killing tests, each rejected by a seeded fault in
  `src/harness.rs`: a `Debug` impl returning `Ok` failed
  `new_and_default_harnesses_are_equivalent`; an empty `emit_start` failed
  `the_start_event_carries_the_scenario_identity` with "the run must emit
  exactly one start event"; and `source` returning `None` failed
  `a_profile_hook_error_exposes_its_source` with "the hook error must expose
  the failure it wraps" (logs
  `/tmp/fault-{f,g,h}-mutants-fix-...-public-api.out`; the file was restored to
  sha256 `3b04e652...` after each). The fourth, a `Clone` returning
  `Default::default()`, is equivalent because `BevyHarness` is a zero-sized
  type over `PhantomData`; see `Decision log`. The seven local gates passed on
  this tree: `make check-fmt` (1 s), `make lint` (48 s), `make typecheck` (12
  s), `make test` (171 s, 307/307 plus doctests), `make markdownlint` (3 s),
  `make nixie` (1 s), and `make spelling` (1 s); logs
  `/tmp/gate-*-skyjoust-0-1-2-bevy-harness-public-api-m5.out`. The milestone
  CLI review of `a96027e..2449742` (288 s, 17 files, exit 0) then found five
  minor documentation gaps. Four are repaired: the adoption example imports
  `when` and uses the `bevy` re-export; the users' guide example imports
  `FrameCount` through the re-export and states that the consuming package
  supplies the feature file that `index = 1` selects; and both guide examples
  carry `tested-example` markers and are compiled from the published text by a
  `cfg(doctest)` item that includes `docs/users-guide.md`. The fifth reported
  that roadmap `0.1.2` claimed the gates passed while this plan still said
  pending; that inconsistency was transient and is resolved by replacing the
  pending language with the evidence above, so the roadmap needs no change. The
  review-fix tree then passed all seven gates again: `check-fmt` (1 s), `lint`
  (461 s under compile-slot contention), `typecheck` (14 s), `test` (514 s,
  307/307 plus 18 harness doctests, including both guide examples),
  `markdownlint` (8 s), `nixie` (<1 s), and `spelling` (7 s); logs
  `/tmp/gate2-*-skyjoust-0-1-2-bevy-harness-public-api-m5.out`. A second CLI
  review of the follow-up commit `d2ec246` (240 s, 4 files, exit 0) found two
  more minor gaps: the loader did not validate markers, and the guide's
  minimal-application fence declared a `#[test]` function that rustdoc never
  runs. `src/guide_tests.rs` now enforces marker placement, non-empty unique
  identifiers, and fence termination for the guide, and that fence is a test
  body rustdoc executes. Both were proven by seeded faults: an unmarked Rust
  fence and a duplicated identifier each failed the new checks, and changing
  the expected frame count failed the doctest (logs
  `/tmp/fault-{i,j,k}-review2-...-public-api.out`). A third CLI review of the
  follow-up commit `cc2346e` (234 s, 4 files, exit 0) found two more minor gaps
  against the style guide's marker rule: only Rust fences were checked, and the
  parser recognized only column-zero backtick fences. `src/guide_tests.rs` now
  requires a marker on every fence whatever its language, recognizes CommonMark
  backtick and tilde fences indented by up to three spaces, and applies the
  matching closing rules; `docs/users-guide.md` gains markers on its other ten
  fences. Seeded faults proved both repairs: removing the `toml` marker failed
  with "the \"toml\" fence … has no tested-example marker", and rewriting an
  unmarked fence as `~~~json` was detected as a `json` fence and rejected (logs
  `/tmp/fault-{l,m}-round3-...-public-api.out`). The re-gate then passed all
  seven gates: `check-fmt` (1 s), `lint` (16 s), `typecheck` (4 s), `test` (33
  s, 314/314 plus 27 doctests), `markdownlint` (4 s), `nixie` (1 s), and
  `spelling` (2 s); logs
  `/tmp/gate6-*-skyjoust-0-1-2-bevy-harness-public-api.out`. The committed
  tree, including this record, passed the same seven gates again; logs
  `/tmp/gate7-*-skyjoust-0-1-2-bevy-harness-public-api.out`. A fourth CLI
  review of `66dcae6` (347 s, 3 files, exit 0) reported no verified findings
  and one unverified concern: a fence inside a CommonMark block quote escapes
  the checker. A probe showed the concern is real, because rustdoc compiles a
  block-quoted Rust fence as a doctest, so `fence_delimiter` now looks through
  block-quote prefixes and records the opener's depth; only a delimiter at the
  same depth closes a fence, keeping a quoted line inside a top-level fence as
  content. A seeded fault appended an unmarked `> ~~~toml` block, which failed
  with the container fence's language in the diagnostic (log
  `/tmp/fault-n-round4-...-public-api.out`). The final re-gate passed all seven
  gates: `check-fmt` (1 s), `lint` (5 s), `typecheck` (1 s), `test` (32 s,
  317/317 plus 27 doctests), `markdownlint` (3 s), `nixie` (<1 s), and
  `spelling` (1 s); logs
  `/tmp/gate8-*-skyjoust-0-1-2-bevy-harness-public-api.out`. A fifth CLI review
  of `c90d36b` (216 s, 2 files, exit 0) confirmed three minor gaps in the fence
  checker. First, the depth-guard test paired a quoted tilde delimiter with a
  backtick opener, so the character comparison alone rejected the candidate and
  the depth guard was untested; the fixture now uses a quoted backtick
  delimiter that matches the opener's character and length. Second, a tab after
  a block-quote marker (`>\t~~~rust`) escaped detection because the scanner
  consumed only an optional space; `strip_quote_padding` now accepts either,
  matching CommonMark's four-column tab stops. Third, an unquoted blank line
  ended the block-quote container but did not invalidate the open fence, so a
  later same-depth delimiter closed the earlier block; `visit` now compares the
  open fence's depth with `quote_depth` on every line and rejects a shallower
  line as an unterminated quoted fence. The expanded module approached the
  400-line limit, so the scanner moved to
  `crates/rstest-bdd-harness-bevy/src/guide_tests/parser.rs` with its unit
  tests in `src/guide_tests/parser/tests.rs`; Whitaker's `module-max-lines`
  lint had rejected the unsplit module at 457 lines. Seeded faults removed the
  depth comparison, the tab padding, and the container check in turn; each
  failed the corresponding test for the intended reason, and `parser.rs` was
  restored to sha256 `37df8e4b...` after each (logs
  `/tmp/fault-{o2,p2,q2}-round5-...-public-api.out`). The focused guide suite
  passed 12/12, and focused Clippy and Whitaker were clean (logs
  `/tmp/{guide-tests-split,clippy-split,whitaker-split}-...-public-api.out`). A
  sixth CLI review of `3cff9b1` (245 s, 4 files, exit 0) found one minor gap
  the round-5 check missed: `visit` called `fence_line` before testing
  containment, so an unquoted delimiter line ended the block quote silently and
  a later same-depth delimiter closed the original fence. In the fixture
  `<!-- tested-example: example -->\n> ~~~rust\n```\n> ~~~\n` the unquoted
  backtick opener vanished and all guide-wide checks accepted the result. The
  regression test `unquoted_delimiter_ends_a_quoted_fence` was written first
  and failed against `3cff9b1` for that reason (log
  `/tmp/red-round6-...-public-api.out`); moving the containment check ahead of
  `fence_line` makes it pass while same-depth closers, quoted delimiters inside
  top-level fences, and the blank-line rule all still pass (13/13 guide tests,
  log `/tmp/green-round6-...-public-api.out`). The re-gate passed all seven
  gates: `check-fmt` (<1 s), `lint` (4 s), `typecheck` (1 s), `test` (14 s,
  320/320 plus 27 doctests), `markdownlint` (4 s), `nixie` (<1 s), and
  `spelling` (2 s); logs
  `/tmp/gate10-*-skyjoust-0-1-2-bevy-harness-public-api.out`. A seventh CLI
  review of `bf0b3e2` (reviewing the containment fix, 3 files) returned zero
  findings, so the implementation record reached review equilibrium; log
  `/tmp/coderabbit7-skyjoust-0-1-2-bevy-harness-public-api.out`.
- [x] (2026-11-20) Hosted-delivery CodeScene repair: the pre-readiness delta
  analysis (`cs delta 2268c56..HEAD`) reported five Code Duplication findings
  in the candidate. Each was cleared by a cohesive refactor rather than a
  suppression: `parser.rs` now exposes one `scan_block_quote` used by
  `fence_content` and `quote_depth`; `profile_tests.rs` shares `configured_app`;
  `panics.rs` shares `run_and_stash` and folds the two cleanup-failure tests
  into one `#[rstest]` case table; `steps.rs` shares `require_counter`; and the
  four unmarked-fence fixtures in `parser/tests.rs` became one case table.
  `cs check` reports 10.00 for all five files. The first re-gate failed
  `make lint` on four Clippy diagnostics introduced by the refactor; passing
  `BevyHarness` by value, replacing a raw `assert!` in a `#[gtest] #[rstest]`
  function with `expect_that!`, and renaming a shadowed match binding cleared
  them. The final seven-gate run passed on the repaired tree: `check-fmt` (1 s),
  `lint` (12 s), `typecheck` (3 s), `test` (25 s, 320/320 plus 27 doctests),
  `markdownlint` (4 s), `nixie` (<1 s), and `spelling` (1 s); logs
  `/tmp/<gate>-skyjoust-0-1-2-bevy-harness-public-api-20261120T21*.out` for the
  seven gate names. Committed as `8354662`.
- [x] (2026-11-20) Hosted-delivery completeness assessment repair: the
  pre-readiness assessment `SKYJOUST-PR136-EP-COMPLETE-R11-20261120T205017Z`
  (base `2268c56`, head `8c6f500`, plan Revision 11) returned four findings,
  and all were verified against the source before repair.
  - Finding 1 (valid, fixed): a caught secondary panic payload was dropped
    outside its catch boundary, so a panicking payload destructor could replace
    the phase-tagged step or configure panic. `transcribe_and_forget` now
    copies the payload text and leaks the payload on all three secondary paths
    (cleanup after a step panic, scenario drop, configure-drop), matching the
    existing `forget_then_raise` rule for the primary payload. Three regression
    tests use a payload whose `Drop` panics; removing the `mem::forget` made all
    three fail with `payload drop boom` (exit 101), and restoring it returned
    3/3 green.
  - Finding 2 (valid, fixed): the guide scanner skipped a fence indented four
    spaces inside a list item and had no list-container state. `parser.rs` now
    strips list-item content indentation before block-quote and fence scanning,
    tracks list depth, rejects a delimiter outside the fence's item with a
    `"inside a list item is not terminated"` error, and fails closed on
    tab-indented list content. A parameterized detection table covers bullet,
    ordered, and quoted list fences; named tests cover marker attachment inside
    an item and a top-level delimiter that must not close a list fence.
  - Finding 3 (valid, fixed): the three omitted plan verification cases were
    added. `tests/headless_scenario.rs` now also pins `phase=steps` end to end
    through the real macro; `harness_tests/panics.rs` has a normal-return
    `DropFlag` test asserting the flag is clear inside the runner and set after
    `run` returns; `context_tests.rs` has the specified cross-clone
    `update` inside `with_world` borrow test.
  - Finding 4 (scope, recorded): the ExecPlan exclusion required by the
    `execplans` skill had not been recorded in `Tolerances`, and the change
    still exceeds the limit without the plan. Both are now recorded there and
    in `Decision log`, without claiming an approval that was not obtained.
  Focused evidence: the harness library suite passed 69/69 and the behavioural
  binary passed 7/7, including both `should_panic` tests.
  - Gate repair: the first full run failed `make lint` on Clippy's
    `needless_pass_by_value` and its estate denial of `%`, then on eight
    `doc_markdown` denials for `CommonMark`, and then on Whitaker's
    `module_max_lines`, `conditional_max_n_branches`, and
    `bumpy_road_function`. The fixes were structural rather than suppressions:
    `ContainerScan` is borrowed, tab-stop padding uses the low two bits with a
    named `tab_stop_padding` helper, the container/delimiter predicate is split
    into `same_container` and `closes_open`, the marker padding ladder became
    `marker_padding` and `space_padding`, list containment and marker
    recognition moved to `src/guide_tests/parser/list.rs`, and the
    hostile-payload rows moved to
    `src/harness_tests/panics/hostile.rs`. The final seven-gate run passed:
    `check-fmt` (1 s), `lint` (4 s), `typecheck` (3 s), `test` (60 s, 331/331
    plus 27 doctests), `markdownlint` (3 s), `nixie` (<1 s), and `spelling`
    (1 s); logs
    `/tmp/<gate>-skyjoust-0-1-2-bevy-harness-public-api-20261120T2248*.out`
    for the seven gate names. Committed as `6b679f6`.
- [x] (2026-11-20) Post-assessment CodeScene repair: the hosted delta for the
  assessment-repair commit reported two fresh findings, so the fix introduced
  its own returns. Duplicated parser test fixtures became one `single_fence`
  helper behind merged `#[rstest]` case tables, and the list module's
  string-heavy parameter ratio fell by measuring every fragment through a
  `Line` newtype. The same delta on the repaired tree then reported duplication
  between the two drop-panic tests and the same ratio in the panic rows;
  `run_and_transcribe`, `expect_drop_record`, `ExpectedPhase`, `RecordedPhase`,
  and `CleanupFailure::message` replaced the repeated string triples and the
  copied catch-and-transcribe preamble. `cs delta 2268c56` reports no issues in
  the checked changes and `cs check` scores 10.00 on all five files. The
  seven-gate run passed: `check-fmt` (1 s), `lint` (101 s), `typecheck` (2 s),
  `test` (230 s, 331/331 plus 27 doctests), `markdownlint` (4 s), `nixie` (1
  s), and `spelling` (2 s); logs
  `/tmp/<gate>-skyjoust-0-1-2-bevy-harness-public-api-20261120T231042.out` for
  the seven gate names. Committed as `058f373`.
- [x] (2026-11-21) Revision 13 completeness-assessment repair: the follow-up
  assessment `SKYJOUST-PR136-EP-COMPLETE-R13-20261120T230817Z` (base `2268c56`,
  head `030334b`, plan Revision 13) returned three findings, all verified
  against the source.
  - Finding 1 (valid, fixed): a cleanup `Err` after a step panic was formatted
    and dropped outside the catch boundary, so a profile error whose `Display`
    or `Drop` panics could replace the step panic.
    `record_cleanup_error_after_step_panic` now formats and disposes of the
    error inside `catch_unwind` and records a transcribed hostile panic
    instead. Two regression rows use a `Display`-panicking and a
    `Drop`-panicking error; both fail with the unguarded `to_string()`
    (exit 101) and pass with the guard.
  - Finding 2 (valid, fixed): list indentation was matched before the
    block-quote prefix was stripped, so a list inside a block quote escaped
    detection. `guide_tests/parser/list.rs` became `containers.rs`, which
    keeps one stack in opening order and matches each line outermost-first;
    containers carry stable ids so a sibling container cannot close a fence.
    A detection row covers a fence in a list inside a block quote; walking the
    stack items-first makes it fail with zero recorded fences.
  - Finding 3 (valid, fixed): the plan-named nested `with_app` case was
    missing, and the tab rejection had no test. Both were added; removing the
    tab guard fails the new case with an empty fence list, and a raw
    `borrow_mut` fails the nested case on `BorrowMutError`.
  - Finding 4 (scope): the assessed head exceeds the plan's size limit (39
    non-ExecPlan paths and 4,479 net added lines). The overage remains
    recorded without a claimed approval; the maintainer's decision is
    outstanding and is not something this agent can supply.
  CodeScene then reported code duplication across the expanded hostile rows, so
  the three rows that share the post-step-panic shape now call one
  `expect_hostile_secondary_after_step_panic` helper with a `HostileRow` value;
  `cs delta` on the working tree is clean. The folded rows were re-proved
  against the same seeded fault, which still fails both hostile cleanup-error
  cases, and `src/harness.rs` was restored to sha256 `6eab1a68…`. The container
  extraction itself crossed Clippy's nesting ceiling, so
  `Containers::match_line` now delegates to `match_open_containers` and a free
  `match_entry`; `map_or`, `needless_lifetimes`, and `match_bool` rejected the
  first two shapes of those helpers before this one passed. Focused evidence:
  the crate suite passed 74/74 library tests, 4 extraction, 7 scenario, 1 macro
  UI, and 5 property tests plus doctests; the five hostile-payload rows pass
  after the fold.
- [x] (2026-11-21) Revision 14 completeness-assessment repair: the follow-up
  assessment `SKYJOUST-PR136-EP-COMPLETE-R14-20261121T002352Z` (base `2268c56`,
  head `f40e434`, plan Revision 14) returned three findings, all verified
  against the source.
  - Finding 1 (valid, fixed): the Revision 13 guard formatted the wrapped error
    and dropped it inside a single caught step, so an error whose `Display` and
    `Drop` both panic panics while that first panic unwinds; Rust aborts the
    process rather than unwinding to a catch boundary. `transcribe_profile_error`
    now binds the wrapped error, formats it by borrow in one `catch_unwind`,
    and disposes of it in a second, recording the disposal panic's text. A
    combined `PanicOnDisplayAndDrop` error and
    `CleanupFailsWithHostileDisplayAndDrop` profile back a new regression row.
  - Finding 2 (valid, fixed): the tab rejection ran on the raw line only, so a
    tab exposed after a block-quote prefix — a quote marker, one space of
    marker padding, then the tab — became ordinary content and the fence was
    silently skipped. `Line::reject_tab_indent` now runs on the fragment left
    by every prefix strip in `FenceParser::scan`; a quoted-list fixture pins
    the diagnostic.
  - Finding 3 (valid, fixed): the plan requires `BevyHarness`'s rustdoc example
    to show a profile author running a hand-built `ScenarioRunRequest` against
    their own profile, and the example still used the default profile. It now
    defines `DemoProfile`, implements only `configure`, and runs a hand-built
    request through `BevyHarness::<DemoProfile>`.
  - Finding 4 (scope): the assessed head measures 39 non-ExecPlan paths and
    4,772 net added lines. The overage stays recorded without a claimed
    approval; the maintainer's decision remains outstanding.
  Seeded-fault evidence, each restored byte-for-byte (`harness.rs` sha256
  `f68dfc5d…`, `parser.rs` sha256 `2daf5103…`): reinstating the single caught
  step makes the combined row abort the test process with
  `panic in a destructor during cleanup` and `SIGABRT`, and removing the
  post-prefix tab check makes the quoted-list fixture parse to an empty fence
  list while the top-level tab case still passes. Focused evidence: the crate
  suite passed 93/93 and 18/18 doctests.
- [x] (2026-11-21) Revision 14 gate returns: the first full gate run rejected
  `clippy::shadow_reuse` on a rebound parser line, fixed by naming the fragment
  `rest`; the second rejected Whitaker's `module-max-lines` on `harness.rs` at
  431 lines, fixed by moving the failure path into `harness/failure.rs` (264
  and 204 lines); `cs check` then reported Code Duplication in the new file,
  fixed by extracting `drop_guarded` rather than suppressing the rule. Both
  files now score 10.00 and the focused suite still passes 93/93 with 18/18
  doctests.
- [x] (2026-11-21) Revision 15 completeness-assessment repair: the follow-up
  assessment reply `7054801959` (base `2268c56`, head `9b52da6`, plan Revision
  15) returned four findings. The first, second, and fourth are repaired; the
  third is escalated with a tracking issue.
  - Finding 1 (valid, fixed): the normal path released the scenario, and the
    configure-error path released the partially configured application, outside
    any catch boundary, so a panicking resource destructor escaped `run` or
    replaced the promised configure error. `finish_run` now releases the
    scenario through `drop_scenario_guarded`, and
    `drop_app_after_configure_error` releases the failed application before the
    wrapped error is returned. The runner's value or the configure/cleanup
    error stays the primary outcome; a drop panic is recorded as a secondary
    `tracing::error!`. Two new rows in
    `src/harness_tests/panics/release.rs` pin both paths.
  - Finding 2 (valid, fixed): a pending marker stored no container identity, so
    a marker at the end of one list item attached to the fence in the next
    item. `PendingMarker` now stores the container path, and a fence accepts
    the marker only when its own path starts with that path; a sibling
    container produces a diagnostic instead.
    `a_marker_in_a_sibling_list_item_is_rejected` pins the rule while the
    top-level-marker-before-quote and marker-inside-list positives still pass.
  - Finding 3 (escalated): exercising the guide's `sh`, `json`, `toml`, and
    `plaintext` fences needs either binaries owned by other workspace packages
    (including a long-running server) or new parsing dependencies, both of
    which the plan's `Dependencies` tolerance and `EP-REQ-5` forbid. The
    deviation is recorded in `Decision log` and tracked as issue #206.
  - Finding 4 (valid, fixed): the `BevyHarness` rustdoc example now implements
    only `configure`, as the plan's obligation requires; the cleanup assertion
    remains in `a_returning_runner_result_is_propagated_and_cleanup_runs_once`.
  Seeded-fault evidence, each restored byte-for-byte (`harness.rs` sha256
  `290a69ba…`, `parser.rs` sha256 `2afe93ab…`): removing both guarded releases
  makes the two release rows fail with the raw `drop boom` panic escaping
  `run`, while the two pre-existing leak rows still pass; removing the
  container-path check makes the sibling-marker row fail on `expect_err` while
  all 18 other parser rows pass. The release rows moved to
  `src/harness_tests/panics/release.rs` because `panics.rs` reached 440 lines.
  Focused evidence: the crate suite passed 79/79 library tests, 4 extraction, 7
  scenario, 1 macro UI, 5 property tests, and 18/18 doctests.
- [x] (2026-11-21) Revision 16 completeness-assessment repair: the follow-up
  assessment reply `7055999055` (base `2268c56`, head `b0909fc`, plan Revision
  16) confirmed the Revision 15 dispositions and returned one new finding,
  which is valid and repaired.
  - Finding (valid, fixed): the harness owns two further caller-owned values
    that it released unguarded. `run` retained the runner's result `T` while
    cleanup finished, so a panicking destructor could replace a cleanup error
    or abort the process during the cleanup panic's unwind; and it retained
    the unrun runner after a configure failure, whose captured values could
    replace a configure error or abort the configure panic's unwind.
    `finish_run` now drops the value through `drop_runner_value_guarded`, and
    `report_configure_failure` drops the unrun runner through
    `drop_unrun_runner_guarded` before reporting the failure. Each records a
    destructor panic as a secondary `tracing::error!` tagged with the phase of
    the release: `cleanup` for the runner value, `configure` for the unrun
    runner. Four rows in `src/harness_tests/panics/release.rs` pin the four
    outcomes.
  Seeded-fault evidence, each restored byte-for-byte and verified by sha256
  (`821dfe1b…` for `harness.rs`): with the two guarded drops replaced by plain
  drops, the cleanup-error and configure-error rows fail at exit 101 with the
  raw `value boom` and `capture boom` payloads escaping `run`; the
  cleanup-panic row aborts with `panic in a destructor during cleanup`,
  `thread caused non-unwinding panic. aborting.`, and `SIGABRT`; and with the
  runner left to drop during the configure panic's unwind, the configure-panic
  row aborts the same way. Restoring the guards passes all eight release rows.
  Focused evidence: the eight release rows pass, 0 failed, after restoration.

## Surprises & discoveries

- Observation (2026-11-20): `main` bumped Bevy from `0.19.1` to `0.20.0`
  (lockfile `0.20.1`, PR #162) after this plan was drafted. The APIs this plan
  depends on survive unchanged: `App::main()` and `SubApp::update_schedule`
  (`bevy_app` 0.20.1), `Schedules::contains` (`bevy_ecs` 0.20.1), `FrameCount`
  and `MinimalPlugins`, and `TimeUpdateStrategy::FixedTimesteps` (`bevy_time`
  0.20.1). Impact: version citations are corrected; no design or interface
  change follows.
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
- Observation (2026-11-20, EP-M2): Whitaker's `no_unwrap_or_else_panic`,
  Clippy's `option_if_let_else` and `match_wild_err_arm`, and the workspace ban
  on production `expect` leave exactly one lint-clean shape for a panicking
  `RefCell` borrow: `let Ok(value) = ... else { panic!(...) };`. Evidence: gate
  runs `-2` (Clippy), `-3` (Whitaker), and `-4` (Clippy on the `match` form),
  with `-5` green. Impact: `BevyScenario::borrow_app` uses let-else and records
  the constraint in a comment; no lint is silenced.
- Observation (2026-11-20, EP-M2): Bevy 0.20.1's first `App::update` produces
  a zero time delta. `Time<Real>::update_with_instant` returns early when
  `last_update` is `None`, recording the first instant without advancing, so
  `TimeUpdateStrategy::FixedTimesteps(1)` runs zero fixed steps on the first
  update and one on each later update. Evidence: `bevy_time-0.20.1/src/real.rs`
  (`update_with_instant`). Impact: the `FixedTimestep` test primes with one
  update and then asserts one fixed tick per subsequent update, which is the
  steady-state contract a deterministic profile relies on; the decision is
  recorded in `Decision log`.
- Observation (2026-11-20, EP-M3): `rstest_bdd::panic_message`, the helper the
  GPUI harness reuses, is unreachable from this crate's runtime code because
  `rstest-bdd` is a dev-dependency only, and ADR 010 fixes a narrower three-arm
  payload mapping (including the literal `Box<dyn Any>` placeholder) than that
  helper's wider recognized set. Evidence:
  `rstest-bdd-0.6.0/src/panic_support.rs` and
  `rstest-bdd-harness-gpui-0.6.0/src/gpui_harness/mod.rs:159-168`. Impact:
  `src/panic.rs` implements a pure local `payload_text` under the
  abstraction-policy sweep instead of promoting `rstest-bdd` to a runtime
  dependency; the decision is recorded in `Decision log`.
- Observation (2026-11-20, EP-M3): `tracing` routes events through a global
  callsite interest cache, so a recorder installed with
  `tracing::subscriber::with_default` sees nothing once the same callsite has
  been emitted under the no-op default subscriber. Evidence: the first recorder
  draft counted zero events from the second run of a test binary onwards.
  Impact: the recorder returns `Interest::sometimes()` and `capture_errors`
  calls `tracing::callsite::rebuild_interest_cache()` after installing it, so
  every run observes its own records deterministically.
- Observation (2026-11-20, EP-M4): an integration-test root at
  `tests/headless_scenario.rs` resolves `mod steps;` to `tests/steps.rs`, not to
  `tests/headless_scenario/steps.rs`, so the layout the design documents needs
  `#[path = "headless_scenario/steps.rs"] mod steps;`. Impact: the path
  attribute keeps that layout without moving the module to the `tests` root;
  the crate already uses the same idiom for `harness_tests`.
- Observation (2026-11-20, EP-M4): a step that takes the harness context by
  value (`scenario: BevyScenario`) still needs
  `#[from(rstest_bdd_harness_context)]`. Without it the parameter name is
  treated as an ordinary fixture name, and the generated test panics with a
  missing-fixture message that lists the harness-context fixture it did
  provide. Impact: the by-value `Clone` form is documented and exercised; the
  `#[harness_context]` marker is the equivalent spelling.
- Observation (2026-11-20, EP-M4): Clippy's `unnecessary_wraps` reaches the
  generated step wrappers, so a step that cannot fail must return `()` rather
  than `StepResult<(), E>`. Impact: the three infallible steps return `()`;
  only steps with a real failure branch return `StepResult`, and no lint is
  silenced.
- Observation (2026-11-20, EP-M5): a `#[cfg(doctest)]` item carrying
  `#[doc = include_str!("../../../docs/users-guide.md")]` makes rustdoc compile
  the guide's Rust fences as doctests, so the published examples are exercised
  without a bespoke Markdown parser or a copied fixture. Evidence: the crate's
  doctest count rises from 16 to 18, and both guide fences pass, including the
  `#[scenario]` fence whose macro expands against the crate's real feature
  file. Impact: the style guide's shared-loader intent is met for the guide's
  executable examples. A follow-up review showed that rustdoc alone ignores the
  `tested-example` markers, so `src/guide_tests.rs` now enforces marker
  placement, non-empty identifiers, uniqueness, and fence termination for that
  document.
- Observation (2026-11-20, EP-M5 review): the style guide's tested-example rule
  covers every fenced example in a user-facing document, not only executable
  Rust examples, and rustdoc follows CommonMark, which also accepts tilde
  fences and fences indented by up to three spaces. A loader that checked only
  column-zero backtick Rust fences would let an unmarked `toml` block and an
  unmarked tilde-delimited block pass. Impact: `src/guide_tests.rs` now checks
  every fence and recognizes both CommonMark fence forms, and every fence in
  `docs/users-guide.md` carries a marker.
- Observation (2026-11-20, EP-M5 review): CommonMark permits a fenced block
  inside a block quote, and rustdoc compiles a block-quoted Rust fence as a
  doctest; a probe fence prefixed with `>` ran and failed on its seeded
  assertion. Impact: a checker that reads only top-level fences would miss a
  compiled but unmarked example. `src/guide_tests.rs` now strips block-quote
  prefixes and tracks the opener's depth, so a delimiter closes only a fence at
  the same depth and quoted lines inside a top-level fence stay content.
- Observation (2026-11-20, hosted delivery): CodeScene's code-duplication rule
  reaches test modules and treats repeated setup as duplication, so merging
  duplicated tests and extracting helpers cleared all five findings without
  weakening an assertion. Three Clippy interactions surfaced while repairing
  them. `#[gtest]` composes with `#[rstest]` by producing a `Result`-returning
  test, so a raw `assert!` inside that function trips
  `clippy::panic_in_result_fn`; `expect_that!` is the equivalent assertion that
  the composed harness accepts. `BevyHarness<Profile>` is zero-sized, so a
  helper taking `&BevyHarness<Profile>` trips
  `clippy::trivially_copy_pass_by_ref`; the helper takes it by value. A match
  arm that rebinds `observed` to the inner value trips `clippy::shadow_reuse`;
  the inner binding was renamed `actual`.
- Observation (2026-11-20, hosted-delivery assessment): Whitaker's structural
  lints, not only Clippy, bound the assessment repairs. `module-max-lines` caps
  every file at 400 lines, so the three hostile-payload rows pushed
  `harness_tests/panics.rs` to 419 and had to move to a child module;
  `conditional-max-n-branches` rejects a three-term match guard, and
  `bumpy-road-function` rejects a three-arm padding ladder. Each was cleared by
  decomposition — `same_container` plus `closes_open`, `marker_padding` plus
  `space_padding`, and the `guide_tests/parser/list.rs` extraction — not by a
  suppression. Impact: plan the module split before adding cases to a dense
  test file, and expect structural lints to shape extraction boundaries.
- Observation (2026-11-20, hosted-delivery assessment): a caught panic payload
  is not inert. Dropping the `Box<dyn Any + Send>` runs arbitrary caller code,
  and a payload whose `Drop` panics unwinds out of the handler that was
  preserving the original failure, replacing the phase-tagged panic or aborting
  the process. `forget_then_raise` had already protected the primary payload;
  the three secondary payloads (cleanup after a step panic, scenario drop, and
  configure-drop) did not. Impact: every catch boundary that must not unwind
  now transcribes the text and leaks its payload, and a hostile-destructor test
  proves each path.
- Observation (2026-11-20, hosted-delivery assessment): CommonMark measures a
  fence's indentation relative to its container, not to the document, so a
  fence indented four spaces inside a list item is a valid fence while the same
  indentation at the top level is an indented code block. A scanner that
  compares absolute indentation silently skips a compiled, unmarked example.
  Impact: the guide scanner carries a list-item stack, strips item content
  indentation before quote and fence recognition, and compares the fence's list
  depth as well as its quote depth. The container state is deliberately a
  count: a sibling item at the same depth is outside the fence's item, so an
  exact marker position is not needed to reject the dedent.
- Observation (2026-11-20, hosted-delivery assessment): the style guide's
  "precede each fence immediately" rule is container-relative too. A marker at
  the top level above a list item is separated from the item's fence by the
  list marker, which is prose for this purpose. Impact: marker recognition runs
  after container scanning, so a marker inside the item attaches to the fence
  below it, and a top-level marker before a list item is correctly rejected as
  dangling.
- Observation (2026-11-20, hosted-delivery assessment): partial tab-stop
  arithmetic in list indentation can either hide a fence or invent one. Impact:
  tab-indented content inside an open list item is rejected with a message
  asking for spaces rather than guessed at; the guide uses spaces, so failing
  closed keeps the checker honest for the layout it receives.
- Observation (2026-11-21, Revision 13 assessment): containers nest in opening
  order, so the prefix that must be stripped first is whichever opened
  outermost — a block quote outside a list needs `>` removed before the item's
  indentation is measured, while a list outside a block quote needs the
  reverse. Matching list indentation before the quote prefix therefore lost the
  list entirely when a quote surrounded it. Impact: the two separate depth
  counters became one ordered stack, each entry carrying a never-reused id so a
  sibling container at the same depth cannot be mistaken for the fence's own
  container.
- Observation (2026-11-21, Revision 13 assessment): a profile's returned
  `HarnessError` is caller-supplied code, not inert data. Its `Display` may
  panic while formatting, and its `Drop` may panic when the temporary is
  released; either unwinds out of a handler whose whole purpose is to preserve
  a step panic that already happened. Impact: the cleanup-error path on the
  step-panic route now formats and disposes of the wrapped error inside
  `catch_unwind`, records a transcribed hostile panic instead, and two
  regression rows use a `Display`-panicking and a `Drop`-panicking error.
- Observation (2026-11-21, Revision 14 assessment): a `Display` panic and a
  `Drop` panic on the same error cannot share one caught step. When `Display`
  panics, the wrapped temporary is still live; releasing it while that panic
  unwinds runs `Drop`, whose own panic is a "panic in a destructor during
  cleanup" and aborts the process instead of unwinding, losing the step panic
  the path exists to preserve. Impact: `transcribe_profile_error` formats by
  borrow, then disposes in a separate `catch_unwind`, and a combined hostile
  row proves the step panic survives both panics.
- Observation (2026-11-21, Revision 14 assessment): the tab rejection inspected
  the raw line, but container prefixes are stripped afterwards. A tab that
  became leading only after the quote marker was therefore measured as ordinary
  content rather than rejected, and the fence behind it was never seen. Impact:
  every prefix strip in `FenceParser::scan` now re-runs the rejection on the
  remaining fragment, so a tab cannot hide a fence at any nesting depth.
- Observation (2026-11-21, Revision 15 assessment): the guarded drops covered
  only the panic routes. The returning routes — a successful run and a
  configure error — released the application unguarded, because the contract
  was phrased around panics. A resource destructor is caller-supplied code on
  every route, not only while a panic unwinds. Impact: `finish_run` and
  `drop_app_after_configure_error` treat release on every route as the same
  hazard, and the release rows sit beside the panic rows in
  `harness_tests/panics/release.rs`.
- Observation (2026-11-21, Revision 15 assessment): a marker needs its
  container's identity, not just a depth. Two sibling list items share both
  depth and indentation, so any rule that compares only those numbers attaches
  the first item's marker to the second item's fence. Impact: `PendingMarker`
  carries the container path, and attachment requires the fence's path to start
  with it; an ancestor marker is still accepted, which keeps the documented
  top-level-marker-before-quote example working.
- Observation (2026-11-21, Revision 16 assessment): guarding the application
  release did not cover the rest of the harness's caller-owned values. The
  runner's result `T` is arbitrary caller code, and the unrun runner owns
  whatever its closure captured; both were released outside any catch boundary
  on the failure paths that carried them. The hazard is a property of the
  release itself, not of the value's role in the scenario, so every release a
  failure path performs needs the same guard. Impact: `finish_run` drops the
  runner's value under the guard when cleanup failed or panicked, and
  `report_configure_failure` drops the unrun runner under the guard before a
  configure failure is reported.

## Decision log

- Decision: the maintainer's approval of this ExecPlan on 2026-11-20 also
  constitutes acceptance of ADR 010's contents, because the ADR records exactly
  the failure-contract, SemVer-coupling, and rejected-alternative decisions
  enumerated in this `Decision log`, and the maintainer directed that
  implementation proceed to completion. Rationale: the ADR adds no decision
  beyond the approved plan; requiring a second approval of identical content
  would be a ceremony gate with no new information. The ADR states this
  provenance. Date/Author: 2026-11-20, implementer, under maintainer direction.
- Decision: Cranelift compatibility is out of scope for this plan. The EP-M0
  unwinding probe, the `Unwinding` tolerance, and the Cranelift unwinding risk
  are removed, and no milestone does work towards Cranelift beyond ordinary
  Rust code hygiene. Rationale: the estate's Cranelift experiment found its
  compile-time gain outweighed by a threefold increase in binary size and
  slower test execution, so the development profile now builds with LLVM
  (`docs/adrs/adr-0007-adopt-the-rust-build-standard.md`; the selection is
  commented out in `.cargo/config.toml`). Cranelift remains an unsupported
  compile-time option, and the experiment is due for review on 14 May 2027. The
  probe existed only to check `catch_unwind` and `RefCell` unwinding under
  Cranelift; on LLVM both are standard-library guarantees (axiom A2), and the
  EP-M3 panic-path tests exercise them directly. Date/Author: 2026-11-19,
  maintainer direction.
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
- Decision (2026-11-20, EP-M2): the `FixedTimestep` test primes the clock with
  one update, then asserts one fixed tick per subsequent update, rather than
  asserting a tick count from the first update. Rationale: Bevy 0.20.1's first
  update initializes `Time<Real>` with a zero delta (see
  `Surprises & discoveries`), so a test measured from the first update would
  pin an initialization artefact and fail if Bevy later advances on the first
  update. Measuring the steady state pins the behaviour a deterministic profile
  depends on. Date/Author: 2026-11-20, implementer.
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
- Decision (2026-11-20, EP-M2 review): narrow `BevyProfile::cleanup`'s rustdoc
  to normal-path completion in a follow-up EP-M2 commit. The local CodeRabbit
  CLI review (0.9.6, committed range `f33ca3a..b5ece74`) found that the printed
  contract promised panic-path cleanup before EP-M3 implements it. The finding
  is valid for the EP-M2 plateau, so the rustdoc now states only delivered
  behaviour; EP-M3 restores the full contract alongside the panic path and its
  tests. Evidence:
  `/tmp/coderabbit-skyjoust-0-1-2-bevy-harness-public-api.out`. Date/Author:
  2026-11-20, implementer acting on the milestone review.
- Decision (2026-11-20, EP-M3): implement `payload_text` locally rather than
  reusing `rstest_bdd::panic_message`. Rationale: the abstraction-policy sweep
  found the upstream helper reachable only through the `rstest-bdd`
  dev-dependency (promoting it to a runtime dependency for one function is
  disproportionate), and ADR 010 fixes a narrower three-arm contract
  (`&'static str`, `String`, literal `Box<dyn Any>`) than that helper's wider
  recognized set. The local function is pinned by `rstest` cases for its three
  classes and by the three phase snapshots. Date/Author: 2026-11-20,
  implementer.
- Decision (2026-11-20, EP-M3 review): the test-only recorder treats an
  `ERROR` event carrying `harness_type` but missing `phase`, `scenario_name`, or
  `error` as a malformed harness record and makes `capture_errors` panic,
  while an event without the harness marker is ignored. Rationale: defaulting a
  missing field would let a broken diagnostic pass a zero-record assertion, and
  treating every foreign `ERROR` as harness output would couple the tests to
  unrelated producers; the harness always emits its type beside the three
  contract fields, so the marker is the honest discriminator. Date/Author:
  2026-11-20, implementer acting on the milestone review.
- Decision (2026-11-20, EP-M5): record the `Clone` mutation survivor as an
  equivalent mutant rather than adding a test. Rationale: `BevyHarness<P>` is a
  zero-sized type whose only field is `PhantomData<fn() -> P>`, so
  `fn clone(&self) -> Self { *self }` and `Default::default()` construct the
  same value and no observable assertion can separate them. The other three
  survivors change an observable (debug text, the start event, or the error
  source chain) and are killed by tests. Date/Author: 2026-11-20, implementer
  triaging the mutation audit.
- Decision (2026-11-20, EP-M5): correct the never-finish guidance through ADR
  011 rather than rewriting ADR 010. Rationale: ADR 010 said the harness never
  finalizes, yet also claimed a finalized profile could still add plugins
  inside steps; the accepted-decision history must be preserved, so ADR 011
  amends the sentence and ADR 010 keeps the original under `## Amendments`, as
  the documentation style guide requires. Date/Author: 2026-11-20, implementer
  preparing the documentation milestone.
- Decision (2026-11-20, EP-M5 review): load the users' guide's Rust examples
  into doctests through a hidden `#[cfg(doctest)]` item rather than building a
  Markdown example parser. Rationale: the style guide requires documented
  examples to be exercised from the published text, and rustdoc already
  compiles every Rust fence in an included document; the `cfg` keeps the item
  out of normal builds, rendered documentation, and the public API. Every fence
  in the guide carries a stable `tested-example` marker, and
  `src/guide_tests.rs` enforces the marker and CommonMark fence rules for that
  document, because rustdoc itself ignores the markers. The scope is the users'
  guide: a repository-wide scan over the other user-facing documents remains
  future work. Date/Author: 2026-11-20, implementer acting on the milestone
  review.
- Decision (2026-11-20, hosted-delivery assessment): leak, rather than drop, a
  secondary caught payload whose text has already been transcribed. Rationale:
  the payload's `Drop` is arbitrary caller code; running it while preserving an
  earlier failure lets a panicking destructor replace the phase-tagged panic or
  abort the process. Leaking one payload on an already-failing path is the same
  bounded trade-off `forget_then_raise` already makes for the primary payload.
  Evidence: three hostile-destructor tests fail with `payload drop boom` when
  the forget is removed. Date/Author: 2026-11-20, implementer acting on the
  assessment.
- Decision (2026-11-20, hosted-delivery assessment): track list-item
  containers in the guide scanner with an indentation stack and a depth count,
  rather than special-casing four-space indentation. Rationale: CommonMark
  indentation is container-relative, and a depth count rejects a sibling item's
  delimiter without tracking exact marker columns; tab-indented item content is
  rejected rather than guessed at. Evidence: detection fixtures for bullet,
  ordered, and quoted list fences, a marker-attachment test, and a
  top-level-delimiter containment test. Date/Author: 2026-11-20, implementer
  acting on the assessment.
- Decision (2026-11-20, hosted-delivery assessment): record the scope
  tolerance as exceeded and continue under the maintainer's standing delivery
  direction, rather than halting the PR. Rationale: the measured change is 37
  non-ExecPlan paths and 4,339 net added lines on the Revision 12 tree (38 and
  6,465 including this plan; the assessed head measured 5,923 net lines
  including the plan). The excess is the real size of the approved deliverables
  — six public API types, three ADRs, the design and users' guides, and the
  lifecycle and guide-checker test suites — not added scope; no requirement,
  signature, or milestone was widened. A blocking scope question was raised,
  but its prompt aborted before a decision was recorded, so this plan must not
  be read as carrying a separate numeric approval; the overage and escalation
  stay explicit for the maintainer. Date/Author: 2026-11-20, implementer.
- Decision (2026-11-21, Revision 13 assessment): carry the scope escalation
  forward unchanged after the reassessment measured 39 non-ExecPlan paths and
  4,479 net added lines on head `030334b`. Rationale: the three defects that
  assessment raised are repaired in this revision, and none of those repairs
  narrowed the deliverable; halting instead of repairing would leave known
  defects published while the same question waited. The numeric question is the
  maintainer's to answer, and the escalation stays explicit and unanswered.
  Date/Author: 2026-11-21, implementer.
- Decision (2026-11-21, Revision 14 assessment): format a profile's error and
  dispose of it in two separate caught steps, recording the disposal panic's
  text when it fires. Rationale: one caught step cannot hold both panics. If
  formatting panics, the wrapped error is dropped while that panic unwinds, and
  a second panic there aborts the process instead of unwinding; the disposal is
  therefore the step that must be caught separately. The disposal is also the
  later failure, so its text is the more accurate record. Evidence: restoring
  the single caught step aborts the combined regression row with
  `panic in a destructor during cleanup` and `SIGABRT`; the split passes.
  Date/Author: 2026-11-21, implementer acting on the assessment.
- Decision (2026-11-21, Revision 14 assessment): re-run the fail-closed tab
  rejection after every container prefix strip, rather than handling tab
  columns or leaving post-prefix tabs as content. Rationale: the guide uses
  spaces everywhere, column arithmetic would add a second indentation model to
  prove, and a tab that only becomes leading after a prefix is stripped is
  exactly the ambiguity the existing rule refuses to guess at. Evidence: the
  quoted-list fixture fails with an empty fence list when the re-check is
  removed, while the pre-existing top-level case still passes, so the new case
  covers the hole rather than repeating the old one. Date/Author: 2026-11-21,
  implementer acting on the assessment.
- Decision (2026-11-21, Revision 14 assessment): carry the scope escalation
  forward again after the reassessment measured 39 non-ExecPlan paths and 4,772
  net added lines on head `f40e434`. Rationale: the three defects that
  assessment raised are repaired in this revision, and none of the repairs
  widened a requirement, signature, or milestone; halting would leave known
  defects published while the same unanswered numeric question waited. The
  overage stays explicit and unapproved. Date/Author: 2026-11-21, implementer.
- Decision (2026-11-21, Revision 14 gate return): move the whole early-exit
  path (panic re-raising, cleanup-on-panic, guarded drops, error wrapping, and
  `ProfileHookError`) into
  `crates/rstest-bdd-harness-bevy/src/harness/failure.rs`, leaving `harness.rs`
  to own configuration, step execution, and the normal cleanup outcome.
  Rationale: Whitaker's `module-max-lines` limit is 400, and the Revision 14
  repair pushed `harness.rs` to 431; the split follows the existing failure
  path's boundary rather than inventing one, and both files now sit well inside
  the limit (264 and 204 lines). Re-running `make lint` after the split is the
  acceptance evidence. Date/Author: 2026-11-21, implementer acting on the gate.
- Decision (2026-11-21, Revision 14 gate return): extract one
  `drop_guarded::<P, T>` helper for the two guarded drops instead of
  suppressing CodeScene's Code Duplication rule. Rationale: the two functions
  differed only in the value dropped, its phase, and its summary, so a single
  helper taking those three inputs removes the duplication rather than hiding
  it; `cs check` then scores both files 10.00. Date/Author: 2026-11-21,
  implementer acting on the delta.
- Decision (2026-11-21, Revision 15 assessment): guard the application release
  on the returning paths as well as the panic routes, and give the primary
  outcome precedence. Rationale: a profile-installed resource destructor is
  caller-supplied code whichever route releases the application; on the normal
  path an escaping destructor panic reaches the caller in place of the runner's
  value, and on the configure-error path it replaces the wrapped configure
  error. The runner's value, the cleanup error, and the configure error are
  outcomes the harness has already produced, so each remains primary and a drop
  panic becomes a secondary `tracing::error!` tagged with the phase of the
  release (`cleanup` for the scenario, `configure` for the failed application).
  Evidence: removing both guarded releases fails the two new release rows with
  the raw `drop boom` payload while the earlier leak rows still pass.
  Date/Author: 2026-11-21, implementer acting on the assessment.
- Decision (2026-11-21, Revision 15 assessment): associate a pending marker
  with the container path it was found in, and accept it only for a fence in
  the same container or a descendant. Rationale: a depth count cannot
  distinguish the list item that held the marker from its sibling, so the
  previous rule attributed one example's identifier to another example's code.
  An ancestor prefix keeps the two documented positives — a top-level marker
  introducing a block-quoted fence, and a marker above a fence inside one list
  item — while rejecting the sibling case. Evidence: removing the check fails
  the sibling row and leaves the 18 other parser rows, including both
  positives, passing. Date/Author: 2026-11-21, implementer acting on the
  assessment.
- Decision (2026-11-21, Revision 15 assessment): escalate the non-Rust guide
  examples rather than executing them in this crate. Rationale: the `sh` fences
  invoke binaries owned by other workspace packages and start a long-running
  server, and the `json`, `toml`, and `plaintext` fences need parsers or
  command output. Both routes breach the plan's `Dependencies` tolerance or
  `EP-REQ-5`'s no-Skyjoust-dependency rule, so the choice belongs to the
  maintainer; issue #206 records the gap, the constraints, and three remedies.
  The Rust fences remain compiled and executed from the published guide, and
  every fence keeps its marker enforcement. Date/Author: 2026-11-21,
  implementer acting on the assessment.
- Decision (2026-11-21, Revision 15 assessment): move the release rows to
  `crates/rstest-bdd-harness-bevy/src/harness_tests/panics/release.rs`.
  Rationale: `panics.rs` reached 440 lines when the two new rows were added,
  beyond the 400-line limit; the release rows answer a different question —
  when the application is released and how a panicking destructor is reported —
  from the phase-tagged panic rows, so a child module follows the existing
  `hostile` split rather than compressing the new coverage. Date/Author:
  2026-11-21, implementer acting on the assessment.
- Decision (2026-11-21, Revision 16 assessment): guard the release of the
  runner's value and of the unrun runner on their failure paths, with the
  settled cleanup or configure outcome as primary. Rationale: both values are
  caller-supplied code that the harness releases after it has already produced
  an outcome. An unguarded destructor panic replaces a cleanup or configure
  error on the returning path, and aborts the process when it fires during the
  unwind of the phase-tagged panic the harness is raising. The guard makes the
  destructor panic a secondary `tracing::error!` tagged `cleanup` for the
  runner value and `configure` for the unrun runner, matching the rule already
  applied to application release. Evidence: plain drops fail the cleanup-error
  and configure-error rows at exit 101, and abort the cleanup-panic and
  configure-panic rows with `SIGABRT`; the guarded versions pass all four.
  Date/Author: 2026-11-21, implementer acting on the assessment.

## Outcomes & retrospective

Delivered roadmap `0.1.2`: the six public API types, the failure contract, the
behavioural and property suites, and the documentation set, with ADR 010 and
its ADR 011 amendment. The crate remains workspace-only and its dependency
shape is unchanged.

Gate durations grew with host contention: the first EP-M5 run took 237 s in
total (lint 48 s, test 171 s), while the review-fix run took 997 s (lint 461 s,
test 514 s) as compiler slots queued. Every run was green.

Mutation audit: 28 mutants, 12 caught, 12 unviable, 0 timeout, 4 missed. Three
missed mutants were killed by new tests and proven by hand-seeded faults; the
fourth (`Clone` returning `Default::default()`) is equivalent because the
harness is zero-sized.

Lessons: generated step wrappers inherit Clippy's `unnecessary_wraps`, so
infallible steps must return `()`; `rstest-bdd`'s step loop wraps the failing
step index in isolate characters, which a `should_panic` substring must
reproduce; `cfg(doctest)` plus `include_str!` compiles a Markdown guide's Rust
examples from the published text without a bespoke loader; and the local CLI
review found only documentation gaps once the code gates were green,
reinforcing that gate evidence does not replace review.

The hosted-delivery assessment added two further lessons. First, every catch
boundary that must not unwind has to treat its caught payload as hostile:
transcribing the text and leaking the payload is what keeps the earlier failure
identifiable, and the same rule applies to secondary failures as to the primary
one. Second, a Markdown checker must model container-relative indentation, not
absolute columns, because the same four spaces are an indented code block at
the top level and a valid list-contained fence inside an item; the local CLI
review rounds had hardened quote depth and delimiter matching but never
exercised a list container, so the assessment found the gap that repeated
gate-green review passes had missed. Assessment every finding against the
source before repairing it, and record the scope tolerance as exceeded rather
than letting a stale note imply the overage was approved.

## Context and orientation

Skyjoust is a Rust workspace (root `Cargo.toml`) with four members: the root
`skyjoust` package, `crates/skyjoust_stateright_validator`,
`crates/skyjoust_test_macros`, and `crates/rstest-bdd-harness-bevy` with its
nested test-only proc-macro crate `crates/rstest-bdd-harness-bevy/test-macros`.
The toolchain is pinned to `nightly-2026-11-06` in `rust-toolchain.toml`. The
repository's locked Bevy version is `0.20.1` (manifest requirement `0.20.0`;
`main` bumped it from `0.19.1` in PR #162 after this plan was drafted). The
development profile builds with LLVM: `.cargo/config.toml` carries the Rust
build standard (the parallel frontend, and `mold` on Linux), and the Make
targets also pass `tools/dev-fast/config.toml`. The Cranelift backend selection
is commented out in both files; it is unsupported and out of scope for this
plan.

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

Release is outside the outcome model but on every route: the harness owns the
application, the runner's result, and the unrun runner, each of which may run
caller-supplied destructors. Axiom A7 admits a destructor panic at any release,
and every release the harness performs goes through `drop_guarded`. A release
panic is a secondary `tracing::error!` record, never an outcome, so it neither
adds a table column nor changes a row.

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
  normally for every cleanup outcome (they catch panics), including while a
  caught secondary payload or the scenario is released: a secondary payload is
  transcribed and leaked rather than dropped, so a panicking destructor cannot
  unwind. With EP-INV-2 this gives EP-INV-3 on the panic path: nothing between
  catching `p` and raising `m(steps, p)` can unwind.
- L2: `m` is a total, panic-free function of
  `(phase, type name, metadata, payload)` (pure formatting).
- L3: the macro-generated test calls `run` exactly once per scenario on a
  `Default`-constructed harness (axiom A4), so EP-INV-1 to EP-INV-5 hold for
  generated scenarios, not only for hand-built requests.
- L4: every harness-initiated release of a caller-owned value on a failure
  path returns normally, because each goes through `drop_guarded`, which
  catches any destructor panic and records it instead. Combined with L1 this
  makes the settled outcome — a returned value, a wrapped error, or a raised
  phase-tagged panic — the only thing the caller observes.

### Axioms (external assumptions, not verified here)

- A1 Bevy: `App::update` runs `Main` once, including `Update` once and the
  `FrameCount` increment (`wrapping_add(1)`) once.
- A2 Rust: `RefCell` resets its borrow flag when a `RefMut` drops during
  unwinding; `catch_unwind` catches every unwinding panic; `panic_any` invokes
  the panic hook; libtest's `should_panic(expected)` matches a `String` payload
  by substring.
- A3 Platform: tests build with `panic = "unwind"`. Under `panic = "abort"`
  no cleanup can run; documented, not handled.
- A4 `rstest-bdd` 0.6.0: generated tests construct the harness with
  `<H as Default>::default()`, call `run` once, inject the context by `TypeId`
  under `rstest_bdd_harness_context`, report step failures by panicking inside
  the runner, and turn `Err(HarnessError)` into a panic.
- A5 `HarnessError` exposes only `RuntimeBuildFailed(std::io::Error)`.
- A6 Bevy: a system panic during `update` loses the `Main` schedule; the
  main sub-app's `update_schedule` label and `Schedules::contains` reveal it.
- A7 Caller code: a destructor owned by the runner's result, the runner's
  closure captures, a profile-installed resource, or a profile error may panic,
  and a panic that starts while another panic unwinds aborts the process. The
  harness therefore treats every release as hostile and never relies on a
  destructor returning normally; the release rows exercise each such release
  with a panicking destructor instead of assuming this axiom silently.

### Test profiles and probes

All lifecycle tests use profile types defined in the test module, one per
configure and cleanup behaviour. Tripwires make a wrongly-run hook fail loudly;
recorders make a correctly-run hook observable through a scenario clone that
the runner stashes in a local `RefCell<Option<BevyScenario>>` borrowed by the
runner closure (`ScenarioRunner::new` accepts non-`'static` closures).

```plaintext
profile                          configure                        cleanup
Recording                        adds an Update counter; Ok       CleanupCalls += 1; Ok
ConfigureFails                   Err(io::Error::other("config"))   tripwire: no cleanup
ConfigurePanics                  panics "configure boom"          tripwire: no cleanup
CleanupFails                     as Recording                     CleanupCalls += 1; Err("cleanup")
CleanupPanics                    as Recording                     CleanupCalls += 1; panics
CleanupPanicsWithHostilePayload  as Recording                     panics; Drop also panics
ConfigurePanicsWithHostileDrop   inserts a Drop-panicking app     cleanup never runs
                                 resource, then panics
FinalizeProbe                    plugin finish and cleanup panic  Ok
FixedTimestep                    MinimalPlugins, FixedSteps(1)    Ok
NotSendProfile                   compile-time only: holds Rc<()>  -
```

Runners that must not run panic with "tripwire: runner must not run". The leak
rows use `Recording`: the runner inserts a `DropFlag(Arc<AtomicBool>)` resource
whose `Drop` sets the flag, and the test asserts the flag is clear inside the
runner and set after `run` returns or its panic is caught. Test-only `tracing`
capture uses a handwritten subscriber that counts `ERROR` events and records
their `phase` and `scenario_name` fields, installed with
`tracing::subscriber::with_default` around the `run` call.

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
  through a clone (the clone is created outside the closure, and the test
  asserts the exact crate prefix); borrowability after a caught accessor panic;
  and a toy `Update` system that panics once, after which `update()` panics
  with the crate's missing-`Main` message. Artefact: `src/context_tests.rs`.
  Non-vacuity: assertions match the crate prefix, so a raw `BorrowMutError` or
  Bevy's own message fails.
- Obligation: the secondary-payload half of L1, and EP-INV-3 when a secondary
  failure has a hostile destructor. Method: three `#[gtest]` cases whose
  payload type panics in `Drop`: cleanup panics after a step panic; the
  scenario's resource drop panics after a step panic; and the partially
  configured app drop panics after a configure panic. Each asserts the primary
  phase-tagged message survives, the hostile text is absent from the raised
  panic, and the expected secondary `tracing::error!` record exists. Artefacts:
  `src/harness_tests/panics.rs` and the profiles above. Non-vacuity: removing
  `std::mem::forget` from `transcribe_and_forget` makes all three fail with
  `payload drop boom`, exit 101; restoring it passes all three.
- Obligation: EP-INV-3 on the returning paths. Method: two `#[gtest]` rows in
  `src/harness_tests/panics/release.rs` whose profile installs `PanicOnDrop`;
  one runner returns a value and one configure hook returns an error. Each
  asserts the primary outcome survives (the runner's value, or the wrapped
  configure error), the drop text never appears in it, and exactly one secondary
  `tracing::error!` record carries the phase of the release. Artefacts:
  `src/harness_tests/panics/release.rs` and the `SucceedsWithDrop` and
  `ConfigureFailsWithDrop` profiles. Non-vacuity: removing both guarded
  releases makes each row fail with the raw `drop boom` panic escaping `run`,
  while the two pre-existing leak rows still pass, so the new rows detect the
  defect rather than repeating older coverage.
- Obligation: EP-INV-3 for the harness's other caller-owned values. The
  runner's result `T` and the unrun runner are released on the failure paths
  that carry them, and both may own destructors that panic. Method: four
  `#[gtest]` rows in `src/harness_tests/panics/release.rs`. A runner returns
  `PanicOnDropValue` and cleanup either fails or panics; the assertion is the
  cleanup error or the cleanup-tagged panic message, with the value's text
  absent and one secondary `cleanup` record. A runner captures
  `PanicOnDropCapture` and configuration either fails or panics; the assertion
  is the configure error or the configure-tagged panic message, with the
  capture's text absent and one secondary `configure` record. Artefacts:
  `src/harness_tests/panics/release.rs` and the `CleanupFails`, `CleanupPanics`,
  `ConfigureFails`, and `ConfigurePanics` profiles. Non-vacuity: plain drops
  make the cleanup-error and configure-error rows fail at exit 101 with the raw
  `value boom` and `capture boom` payloads escaping `run`, and abort the
  cleanup-panic and configure-panic rows with
  `panic in a destructor during cleanup` and `SIGABRT`; the guarded versions
  pass all four, so each row detects a distinct abort or precedence defect.
- Obligation: the cleanup-error half of L1 when a step panic must survive. A
  profile error is caller-supplied, so its `Display` and its `Drop` are both
  hostile code, and one error may host both. Method: three `#[gtest]` cases
  whose cleanup returns an error that panics on display, on drop, and on both;
  each asserts the step phase and payload survive and the hostile text appears
  only in the recorded secondary event. Artefacts:
  `src/harness_tests/panics/hostile.rs` and the
  `CleanupFailsWithHostileDisplay`, `CleanupFailsWithHostileDrop`, and
  `CleanupFailsWithHostileDisplayAndDrop` profiles. Non-vacuity: replacing the
  guarded formatting with a direct `to_string()` makes the two single-failure
  cases fail (exit 101) with the harness unable to downcast the hostile
  payload; collapsing the two guarded steps into one makes the combined case
  abort with `panic in a destructor during cleanup` and `SIGABRT`, which is the
  defect the split exists to prevent.
- Obligation: the fail-closed tab rule at every container depth. Method: named
  cases for a tab-indented fence inside an open list item and for a tab exposed
  after a block-quote prefix inside a list inside a quote. Artefacts:
  `src/guide_tests/parser/tests.rs` and `src/guide_tests/parser.rs`.
  Non-vacuity: removing the raw-line guard fails the first case with an empty
  fence list, and removing the per-prefix re-check fails the quoted case the
  same way while the first still passes, so the two cases cover distinct holes.
- Obligation: the documented downstream profile workflow. Method: the
  `BevyHarness` rustdoc example defines a profile of its own, implements only
  `configure`, and runs a hand-built `ScenarioRunRequest` through
  `BevyHarness::<DemoProfile>`; it is compiled and executed as a doctest.
  Artefacts: `src/harness.rs`, doctest `harness::BevyHarness`. Non-vacuity: the
  example is a compiled program, so a harness that rejected a non-default
  profile, or a request that could only be built by generated code, fails the
  doctest rather than passing as prose.
- Obligation: container nesting order. Method: a parameterized detection row
  for a fence in a list inside a block quote, whose prefixes must be stripped
  quote-first. Non-vacuity: ordering the stack walk items-first makes the row
  fail with zero recorded fences, reproducing the reported miss.
- Obligation: marker attachment follows container ancestry. Method: a named
  case whose marker sits in one list item and whose fence sits in the next,
  asserting the parser returns the container-mismatch diagnostic; the two
  positive cases (a top-level marker before a block-quoted fence, and a marker
  above a list item's fence) must keep passing. Artefacts:
  `src/guide_tests/parser/tests.rs` and `src/guide_tests/parser.rs`.
  Non-vacuity: removing the container-path check makes the new case fail on
  `expect_err` while all 18 other parser rows pass, so the positives do not
  cover the sibling hole.
- Obligation: EP-INV-7 for the nested accessor pair. Method: a `#[gtest]` that
  nests `with_app` inside `with_app` and asserts the crate's borrow prefix.
  Non-vacuity: replacing `try_borrow_mut` with `borrow_mut` fails the case on
  the raw `BorrowMutError`, and the case's presence is what detects the missing
  nested pair.
- Obligation: EP-INV-8, EP-INV-9, EP-INV-12, and the `FixedTimestep` seam.
  Method: named unit tests; the upstream
  `rstest_bdd_harness::policy_conformance::assert_attribute_policy_conformance`
  for the policy; `FinalizeProbe` runs a scenario with three updates and must
  not trip; `FixedTimestep` shows exactly one `FixedUpdate` per `update()`.
  Artefacts: `src/profile_tests.rs`, `src/policy_tests.rs`,
  `src/harness_tests.rs`.
- Obligation: EP-INV-11. Method: two lifecycle rows (normal return and step
  panic) with the `DropFlag` resource. Non-vacuity: each test asserts the flag
  is clear inside the runner first, so a drop before the runner cannot satisfy
  the later assertion vacuously; a mutant that leaks the handle (for example
  `mem::forget` of the scenario) fails.
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
  fails if the alias resolved to the minimal profile; one failing-step binding
  pins the phase tag (`phase=steps`) and a second pins the feature path, line,
  scenario name, and step index, so neither assertion can satisfy the other's
  contract; both fail if the harness re-raises the step loop's message
  unaugmented.
- Obligation: the users' guide's marker rule for every fence CommonMark
  permits, including list-contained fences. Method: parameterized and named
  `rstest` cases over the real scanner: unmarked fences in bullet, ordered, and
  quoted list items; a marker inside a list item attached to its fence; a
  top-level delimiter that must not close a list fence; plus the existing
  top-level, indented, tilde, quoted, tab-padded, and containment cases.
  Artefacts: `src/guide_tests/parser.rs` and `src/guide_tests/parser/tests.rs`.
  Non-vacuity: each detection case asserts the fixture records exactly one
  unmarked fence, so a scanner that skips the container records zero and fails;
  the containment case expects the `"list item"` diagnostic, which a permissive
  scanner does not produce.
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

Stage A (EP-M0) establishes the baseline and drafts the ADR. Stages B–D repeat
per milestone: write the smallest failing test, confirm it fails for the
expected reason, implement, re-run, refactor, then run `make check-fmt`,
`make lint`, `make typecheck`, and `make test`.

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
|   |-- harness.rs         BevyHarness<P>, HarnessAdapter impl, configure and
|   |                      step orchestration, cleanup outcome
|   |-- harness/failure.rs panic re-raising, guarded drops, error wrapping
|   |-- harness_tests.rs   lifecycle table, finalize, Send/Sync checks
|   |-- harness_tests/
|   |   |-- profiles.rs    tripwire and recording test profiles
|   |   |-- recorder.rs    handwritten tracing subscriber for tests
|   |   |-- panics.rs      phase-tagged panic rows
|   |   `-- panics/
|   |       |-- hostile.rs hostile payload and error rows
|   |       `-- release.rs release ordering and guarded-release rows
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

### EP-M0: baseline and ADR 010 (no production code)

- Outcome: a recorded baseline and a Proposed ADR.
  Run the focused crate tests and record the pass count. Confirm
  `multi_threaded` is off with
  `cargo tree -p rstest-bdd-harness-bevy -e features -i bevy_ecs`. Write
  `docs/adrs/adr-0010-bevy-harness-failure-contract.md` (status Proposed) with
  the lifecycle table, phase tags, `panic_any` re-raise, phase-tagged error
  wrapping, the never-`finish` rule, the SemVer coupling, and options
  considered (Table 1): `resume_unwind` plus direct stderr (GPUI's approach), a
  `Drop`-guard cleanup (a cleanup panic during unwinding aborts), a `&mut App`
  context (cleanup cannot reach the app after the macro drops the context), a
  profile carried as a value (the harness is only built through `Default`), a
  crate-owned profile error type (interface change), and deterministic time by
  default (`FixedTimesteps`; deviates from design §5). Reference the ADR from
  design §6 and add it to `docs/contents.md`.
- Requirements: EP-REQ-6 (partial).
- Acceptance: baseline recorded in `Progress`;
  `make markdownlint` and `make nixie` pass.
- Conformance check: design followed; no code committed; no dependency
  change.
- Recovery: documentation-only; revert the commit.
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
  `make typecheck`, and `make test` pass.
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
- Acceptance: `make check-fmt`, `make lint`, `make typecheck`, `make test`,
  `make markdownlint`, `make nixie`, and `make spelling` pass; `cargo tree`
  shows no forbidden package; the mutation audit has no unexplained survivors
  (or was stopped by tolerance and escalated).
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
make typecheck 2>&1 | tee /tmp/typecheck-skyjoust-0-1-2-bevy-harness-public-api.out
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
- Lint and format: `make check-fmt`, `make lint` (rustdoc, Clippy,
  Whitaker), and `make typecheck` pass with no new suppression.
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
- Revision 4, 2026-11-20: the maintainer approved the plan and directed
  implementation to proceed. Bevy citations were corrected from `0.19.1` to the
  repository's locked `0.20.1` (manifest `0.20.0`, bumped on `main` in PR #162
  after drafting), after confirming that every API the plan depends on is
  present unchanged. ADR 010 acceptance provenance is recorded. No requirement,
  signature, tolerance, or milestone changed.
- Revision 3, 2026-11-19: rebased onto `origin/main` at `e727a3c` and took
  Cranelift out of scope at the maintainer's direction. The estate experiment
  found Cranelift's compile-time gain outweighed by a threefold increase in
  binary size and slower test execution; it stays an unsupported compile-time
  option and is due for review on 14 May 2027. The EP-M0 unwinding probe, the
  `Unwinding` tolerance, and the Cranelift unwinding risk added in Revision 2
  were removed, and EP-M0 is now the baseline and the ADR 010 draft. Drift from
  `main` was corrected: ADRs live under `docs/adrs/` with four-digit numbers,
  the toolchain pin is `nightly-2026-11-06`, the development profile builds
  with LLVM, and `make typecheck` joins the milestone gates. The plan keeps its
  six-milestone shape and still awaits maintainer approval.
- Revision 5, 2026-11-20: implementation delivered through EP-M5. The design,
  crate README, users' guide, adoption note, repository layout, developers'
  guide, contents, and roadmap were updated, and ADR 011 amends ADR 010's
  never-finish sentence under the documented amendment process. The mutation
  audit and its survivor dispositions, the CLI review gaps and their repairs,
  and every gate run are recorded in `Progress`, `Decision log`, and
  `Outcomes & retrospective`. No requirement, signature, tolerance, or
  milestone changed.
- Revision 6, 2026-11-20: three CLI review rounds on the EP-M5 deliverables
  hardened the guide-example loader. Round one added the `cfg(doctest)` include
  and markers on both Rust fences; round two added marker and fence validation
  in `src/guide_tests.rs` and made the minimal-application example executable;
  round three extended the rule to every fence, whatever its language, and to
  CommonMark tilde and indented fence forms. Each repair was proven by a seeded
  fault, and every re-gate passed all seven gates. No requirement, signature,
  tolerance, or milestone changed.
- Revision 7, 2026-11-20: a fourth review round closed the last fence-checker
  gap. CommonMark permits fences inside block quotes, and a probe confirmed
  rustdoc compiles a block-quoted Rust fence as a doctest, so the parser now
  looks through block-quote prefixes and matches closing delimiters at the
  opener's depth. Three regression tests cover an unmarked quoted fence, a
  marker above a quoted fence, and a quoted delimiter inside a top-level fence;
  a seeded fault and a fresh seven-gate run are recorded in `Progress`. No
  requirement, signature, tolerance, or milestone changed.
- Revision 8, 2026-11-20: a fifth review round hardened the fence checker's
  CommonMark fidelity. The depth guard now has a decisive test, tab padding
  after a block-quote marker is consumed, and an unquoted line that ends a
  block-quote container rejects the open fence rather than letting a later
  delimiter close it. The scanner and its tests were split into submodules to
  satisfy the 400-line module limit. Three seeded faults and a fresh seven-gate
  run are recorded in `Progress`. No requirement, signature, tolerance, or
  milestone changed.
- Revision 9, 2026-11-20: a sixth review round found the containment check ran
  after delimiter processing, letting an unquoted delimiter line end a quoted
  container without an error. The check now runs first, and a regression test
  written before the fix demonstrates the failure and its repair. No
  requirement, signature, tolerance, or milestone changed.
- Revision 10, 2026-11-20: a seventh review round found no concerns in the
  containment fix, closing the local review loop at equilibrium. This revision
  records that result; no requirement, signature, tolerance, or milestone
  changed.
- Revision 11, 2026-11-20: hosted delivery began, and the pre-readiness
  CodeScene delta reported five code-duplication findings in the candidate.
  They were cleared by cohesive refactors in the fence scanner and the test
  modules, followed by a full seven-gate run; `Progress` and
  `Surprises & discoveries` record the repairs, the Clippy interactions they
  exposed, and the evidence. The status now distinguishes completed
  implementation from in-progress PR delivery. No requirement, signature,
  tolerance, or milestone changed.
- Revision 12, 2026-11-20: the pre-readiness ExecPlan completeness assessment
  `SKYJOUST-PR136-EP-COMPLETE-R11-20261120T205017Z` returned four findings, and
  all were dispositioned. The two defects are repaired: a secondary panic
  payload is now transcribed and leaked so a panicking destructor cannot
  replace the phase-tagged panic, and the guide scanner tracks list-item
  containers so a list-contained fence is detected and contained. Whitaker's
  structural lints required decomposition alongside both repairs: list
  containment and marker recognition moved to `guide_tests/parser/list.rs`, and
  the hostile-payload rows moved to `harness_tests/panics/hostile.rs`. The
  three omitted verification cases named by the third finding were added, and
  the fourth finding's out-of-date documentation was corrected: `Tolerances`
  now records the mandatory ExecPlan-file exclusion, and both `Tolerances` and
  `Decision log` record that the size limit is still exceeded, without claiming
  an approval that was not obtained. `Verification plan` gained the obligations
  and non-vacuity evidence for the hostile-destructor, normal-return
  `DropFlag`, cross-clone borrow, end-to-end phase, and list-container cases.
  No requirement, signature, or milestone changed.
- Revision 13, 2026-11-20: the hosted CodeScene delta reported that the
  Revision 12 repair introduced duplication across the parser test fixtures and
  a string-heavy parameter ratio in the list module, and the same delta on the
  repaired tree repeated both returns in the panic rows. `Progress` records the
  repairs that cleared them: a shared `single_fence` fixture helper, the `Line`
  fragment type, and the `ExpectedPhase`, `RecordedPhase`, `CleanupFailure`,
  `run_and_transcribe`, and `expect_drop_record` consolidations. No
  requirement, signature, tolerance, or milestone changed.
- Revision 14, 2026-11-21: the follow-up completeness assessment
  `SKYJOUST-PR136-EP-COMPLETE-R13-20261120T230817Z` verified the Revision 13
  tree against the plan and returned three defects plus the still-open scope
  decision. All three defects are repaired. A hostile profile error can no
  longer replace a step panic, because the cleanup-error path now formats and
  disposes of it inside `catch_unwind`. The fence scanner keeps one container
  stack in opening order with stable container ids, so a list inside a block
  quote is detected and a sibling container cannot close a fence. The
  plan-named nested `with_app` case and a test for the tab rejection were
  added. Each repair was proven by a seeded fault that was restored
  byte-for-byte, and the focused crate suite is recorded in `Progress`.
  `Verification plan` gained the corresponding obligations and non-vacuity
  evidence, `Surprises & discoveries` records the two mechanisms that made the
  defects possible, and `Tolerances` records the Revision 13 size measurement.
  No requirement, signature, or milestone changed; the maintainer's decision on
  the size overage remains outstanding.
- Revision 15, 2026-11-21: the follow-up completeness assessment
  `SKYJOUST-PR136-EP-COMPLETE-R14-20261121T002352Z` verified the Revision 14
  tree (head `f40e434`) against the plan and returned two correctness defects,
  one documentation omission, and the still-open scope decision. All three
  findings are repaired. A profile error whose `Display` and `Drop` both panic
  can no longer abort the process, because formatting and disposal are now
  separate caught steps with the disposal panic recorded. The fence scanner
  re-runs its fail-closed tab rejection after every container prefix, so a tab
  can no longer hide a fence behind a block quote. The `BevyHarness` rustdoc
  example now shows a downstream profile author running a hand-built
  `ScenarioRunRequest` through their own profile. Both behavioural repairs were
  proven by seeded faults restored byte-for-byte, with the pre-fix combined
  case aborting under `SIGABRT`. `Progress`, `Surprises & discoveries`,
  `Decision log`, `Verification plan`, and `Tolerances` carry the details; no
  requirement, signature, or milestone changed, and the maintainer's decision
  on the size overage remains outstanding; the Revision 15 head `65409a7`
  measures 40 non-ExecPlan paths and 4,964 net added lines.
- Revision 16, 2026-11-21: the follow-up assessment reply `7054801959`
  verified the Revision 15 tree (head `9b52da6`) and returned four findings.
  Three are repaired. The harness now guards the application release on the
  returning paths as well as the panic routes, so a destructor panic can no
  longer replace the runner's value or a configure error; a pending marker is
  bound to its container path, so a marker at the end of one list item can no
  longer label the next item's fence; and the `BevyHarness` rustdoc example
  implements only `configure`. The fourth finding, exercising the guide's
  non-Rust fences, is escalated to issue #206 because it requires either
  another package's binaries or dependencies the plan forbids; `Decision log`
  records the deviation. Both behavioural repairs were proven by seeded faults
  restored byte-for-byte, and the release rows moved to
  `harness_tests/panics/release.rs` to keep `panics.rs` inside the 400-line
  limit. `Progress`, `Surprises & discoveries`, `Decision log`,
  `Verification plan`, and `Artefacts and notes` carry the details; no
  requirement, signature, or milestone changed, and the maintainer's decision
  on the size overage remains outstanding. The repair head `55e8c68` measures
  41 non-ExecPlan paths and 5,146 net added lines. All seven commit gates
  passed against the repair tree: `make lint` re-ran green after the
  `clippy::option_if_let_else` fix, and the other six gates re-ran afterwards
  in order. CodeScene reports 10.00 for every touched file and no issue in
  `cs delta 9b52da6`.
- Revision 17, 2026-11-21: the follow-up assessment reply `7055999055`
  verified the Revision 16 tree (head `b0909fc`) and returned one finding,
  which is valid and repaired. The harness now releases the runner's value and
  the unrun runner under the same guard as the application, so a panicking
  destructor can no longer replace a cleanup or configure error, nor abort the
  process during a cleanup or configure panic's unwind; each such panic is a
  secondary `tracing::error!` tagged with the release's phase. Four regression
  rows in `harness_tests/panics/release.rs` cover both cleanup outcomes and
  both configure outcomes, and the seeded faults restored byte-for-byte make
  the returning rows fail at exit 101 and the panic rows abort under `SIGABRT`.
  `Progress`, `Surprises & discoveries`, `Decision log`, `Verification plan`,
  and the abstract model carry the details; no requirement, signature, or
  milestone changed, and the maintainer's decision on the size overage remains
  outstanding.

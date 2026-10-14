# Add `rstest-bdd-harness-bevy` as a workspace member

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: IMPLEMENTATION COMPLETE — REVIEW AND ACCEPTANCE PENDING

Approval gate: this plan must be approved before implementation begins. Do not
treat silence as approval. The five choices in
`Decisions resolved before approval` are settled, but settling them does not
itself authorize execution. The maintainer authorized execution on 2026-08-17
by directing that the planned functionality be implemented. The implementation
milestones are delivered, but review and acceptance remain open. The published
candidate is `adfcb5c5e1bc435f6a58e8a0cdc5fa800fb23800`, based on current `main`
`d177781a276012da013531e7b6b9642fc0a5e0b8`; its source tree is the locally
validated `f9a72c93b026e796ff025e71803e1a47a45ac31a` tree
`515c4b3bac3c97cb776e10dd10eda54bfd89e112`, with documentation-only publication
updates. PR #51 is ready, with merge state `BLOCKED` and review decision
`CHANGES_REQUESTED`. Build-test run `38309867260` was in progress at the latest
status check. Existing review findings refer to superseded head
`6b185eee6ee3d330585b08b2fa3d5a529754a536`. Do not treat this plan as fully
accepted until current hosted checks and review are reconciled and all
acceptance conditions are recorded.

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
running tests — that the load-bearing dependency choices work together on this
repository's pinned toolchain and its Cranelift build configuration. It stops
short of the harness types themselves (`BevyScenario`, `BevyHarness`,
`BevyProfile`, `BareBevyProfile`, `MinimalBevyProfile`, `BevyAttributePolicy`);
the roadmap assigns those names to task `0.5.1.2`.

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

## Decisions resolved before approval

The following choices were resolved by the maintainer on 2026-08-17. They are
requirements for implementation, not defaults that an implementer may revisit
without escalating.

1. Target Bevy `0.19.1`. The harness and the future Skyjoust runtime must use
   the same release because Bevy types cross the harness boundary. The earlier
   `0.17.3` probes remain historical evidence only; Milestone 0 must repeat the
   compatibility and cost probes against `0.19.1` before implementation.
2. Write ordinary stable dependency requirements using Cargo's implicit caret
   syntax, for example `0.19.1`, never the redundant spelling `^0.19.1`.
3. Historical decision (2026-08-17): pin every `rstest-bdd` family crate to
   `=0.6.0-beta3` because later prereleases could break the API. Superseding
   decision (2026-10-12): adopt published stable `0.6.0` with implicit-caret
   requirements. The imported v0.6.0 migration and users' guides were reviewed.
   Inverse dependency checks, the standalone crate suite, and all six workspace
   gates passed on the current source tree before the documentation-only
   follow-up; the follow-up's documentation and acceptance checks also passed.
   Hosted checks, review, and final PR acceptance remain pending.
4. `rstest-bdd` step functions are not `#[test]` functions, so the workspace's
   test allowance does not permit `.expect()` in them. Propagate fallible work
   through `Result` and `?`; where it stays clear, prefer functional
   transformations over imperative unwrap-and-mutate sequences.
5. [Pull request #6](https://github.com/leynos/skyjoust/pull/6) updates the root
   package from `rstest 0.18` to the latest public release. Watch it until
   automerge completes, then rebase this stack onto `main` before Milestone 2.
   Reconcile the new crate's `rstest` requirement with the merged root
   requirement after the rebase rather than planning around two versions.

## Constraints

These are hard invariants. If satisfying the objective would require violating
one, stop and escalate rather than working around it.

- The crate and its standalone test graph must not depend on any Skyjoust or
  Lille package, directly or transitively. This is the extraction contract from
  [the harness design](../rstest-bdd-harness-bevy-design.md)
  §11. Fixture lint handling belongs to the nested, crate-owned
  `rstest-bdd-harness-bevy-test-macros` package; do not use sibling
  `skyjoust-test-macros`. The dependency guard inspects normal and development
  dependencies and retains the synthetic near-match canary to protect exact
  matching; that canary does not add the sibling to the actual test graph.
- The crate's own manifest must declare Bevy with `default-features = false`
  and enable no feature beyond `std` (see `Decision log`). Cargo unifies
  features across workspace members built in one invocation, and `make test` and
  `make lint` both build `--workspace --all-features`. This constraint
  therefore bounds what the crate *requests*, not what a workspace build
  resolves. Any other member that later adds `bevy` must also declare
  `default-features = false`, or the headless guarantee is lost for everyone.
- The harness crate's Bevy release version is a workspace-wide coupling, not a
  local choice. It must equal the version the Skyjoust runtime will use at
  roadmap `1.2.2`; this plan fixes that version at `0.19.1`. Changing it is an
  ADR-level decision.
- No Skyjoust gameplay profile, validator trace type, or runtime state resource
  may be checked into the harness crate. Downstream profiles live in the
  consuming crate (design §8).
- `crates/rstest-bdd-harness-bevy/Cargo.toml` must carry
  `[lints]` with `workspace = true`, per
  [the developer's guide](../developers-guide.md) §7.1.
- `unsafe_code` stays forbidden; `missing_docs` and `missing_crate_level_docs`
  stay denied. Every module opens with a `//!` comment and every public item
  carries a `///` comment with a worked example, per `AGENTS.md`.
- `clippy::doc_markdown` runs at pedantic level and is promoted to an error by
  `-D warnings`. Every type and item name in a `///` or `//!` comment —
  `MinimalPlugins`, `FrameCount`, `TimePlugin`, `BevyProfile`, and so on — must
  be wrapped in backticks.
- No source file exceeds 400 lines (`AGENTS.md`).
- Do not modify `crates/skyjoust_stateright_validator/` except for the
  user-authorized, behaviour-preserving tracing-helper extraction in
  `examples/serve_explorer.rs` that resolves cognitive complexity 15/9. Keep
  the tracing event order and checker lifetime; do not suppress the lint or
  change its threshold. No other edits in that crate are in scope. Do not modify
  `src/` or `tests/makefile_contract.rs`.
- Do not add or rename Makefile targets. `tests/makefile_contract.rs` asserts
  that the `build`, `test`, `lint`, and `typecheck` recipes each pass
  `--config tools/dev-fast/config.toml`; leaving the Makefile alone keeps that
  contract intact.
- `typos.toml` is generated and must not be hand-edited, but it must be
  *regenerated* whenever `typos.local.toml` changes. `make spelling-config`
  checks the committed `typos.toml` against the sources and gates
  `make markdownlint`, so a stale `typos.toml` fails the documentation gates.
- Proptest regression files under
  `crates/rstest-bdd-harness-bevy/tests/proptest-regressions/` are committed,
  not ignored. They are shrunk counter-examples, not scratch.
- Prose is British English with Oxford spelling (`-ize`, `-yse`, `-our`),
  wrapped at 80 columns; code fences wrap at 120 columns.

## Tolerances (exception triggers)

- Scope: the original tolerance was 22 touched paths. A bounded review
  follow-up extended it to 24; subsequent explicit user requests and the
  narrowly authorized lint repair extended the candidate to 33. This adoption
  note is the approved user-facing follow-up that brings the current manifest
  to 34 paths, listed exactly once in Table 2. Any further path requires
  escalation.
- Public interface: this plan introduces exactly two public functions plus the
  re-export block described in `Interfaces and dependencies`. Anything named in
  design §§4-7 — `BevyScenario`, `BevyHarness`, `BevyProfile`,
  `BareBevyProfile`, `MinimalBevyProfile`, `BevyAttributePolicy` — belongs to
  roadmap task `0.5.1.2`; if one appears necessary here, stop and escalate.
- Dependencies: the dependency set in `Interfaces and dependencies` is fixed.
  If any additional crate is required, stop and escalate.
- Bevy features: `default-features = false` with `features = ["std"]` is the
  agreed declaration. If a further feature proves necessary, stop and escalate.
- `rstest-bdd` version: use the published stable `0.6.0` family with implicit
  caret requirements. Do not substitute a git dependency on `main`; complete
  and record the stable compatibility pass before acceptance.
- Iterations: if a gate still fails after three fix attempts, stop and escalate
  with the log path recorded in `Progress`. Before counting an attempt, check
  whether two gates are contradicting one another — `check-fmt` and `lint` can
  each reject the other's fix (see `Risks`).
- Recurring cost: if a full `make all` on a warm cache exceeds fifteen minutes,
  or `target/` exceeds 6 GB, stop and escalate. The crate would then need its
  own continuous integration (CI) job, or extraction sooner than `0.5.1.5`
  assumes.
- Time: if any milestone exceeds two hours of work, stop and escalate. Time
  spent blocked on the shared Cargo package-cache lock does not count.
- Ambiguity: if two readings of a roadmap or design statement would produce
  materially different crates, stop and present the options.

## Risks

- Risk: `rstest`'s `#[fixture]` expansion re-wraps the function body, so a
  single-expression fixture body trips `unused_braces` under `-D warnings`;
  `.rustfmt.toml`'s `fn_single_line = true` then reformats any multi-line body
  straight back into the failing form. `make check-fmt` and `make lint` demand
  mutually exclusive formulations. Severity: high. Likelihood: certain
  (observed). Mitigation: retired by the layer beneath this one in the stack,
  which adds `crates/skyjoust_test_macros` and its
  `allow_fixture_expansion_lints` attribute — the estate-mandated approach,
  mirroring `weaver-test-macros`. Apply the attribute directly above
  `#[fixture]` and leave the fixture in its natural single-expression form. See
  [ADR 006](../adr/006-test-macro-crate-for-fixture-expansion-lints.md)
  and [the developer's guide](../developers-guide.md) §7.3. Verified clean
  under both gates.

- Risk: in the `0.17.3` probe, omitting Bevy's `std` feature replaced
  `std::time::Instant` with `bevy_platform::time::fallback::Instant`, whose
  x86-64 getter reads `core::arch::x86_64::_rdtsc()` and passes the raw tick
  count to `Duration::from_nanos`. `Time<Real>` then advances at roughly the
  timestamp counter frequency rather than wall-clock, and `Time<Virtual>` and
  `Time<Fixed>` derive from it. Severity: high. Likelihood: not yet measured for
  `0.19.1`. Mitigation: begin with `features = ["std"]`, then repeat the
  clock-source and resolved-feature checks against `0.19.1`. The decision is
  discharged only if real time remains correct without enabling rendering,
  windowing, or the multi-threaded executor.

- Risk: `clippy::expect_used` is denied estate-wide, and
  `allow-expect-in-tests = true` in `clippy.toml` does **not** cover free
  functions in `tests/*.rs` that are not `#[test]`-annotated. `rstest-bdd` step
  functions are exactly such functions, so `.expect()` inside a step fails
  `make lint`. Severity: medium. Likelihood: high (observed). Mitigation:
  scenario state comes from an `rstest` fixture holding a `RefCell<App>`
  directly. Steps use `try_borrow` or `try_borrow_mut`, return a `StepResult`,
  and propagate borrow failures instead of panicking. Prefer `map` and
  `map_err` when that keeps the step linear and readable; use `?` when a
  functional chain would obscure the assertion.

- Historical risk (superseded): a caret requirement on `0.6.0-beta3` could
  admit later prereleases, including source-breaking `beta4`. Exact beta3 pins
  mitigated that risk at the time. The current decision uses stable `0.6.0`
  with the workspace's implicit-caret policy. The stable compatibility pass and
  six local gates passed before the current documentation follow-up; its
  post-documentation gates and acceptance checks also passed, as recorded in
  `Progress`.

- Historical risk at beta3: editing only a `.feature` file left the compiled
  step table stale, and the explicit `include_str!` guard was load-bearing. The
  stable 0.6.0 A/B check now shows that both with and without the declaration,
  a feature-only edit triggers a rebuild and the expected `Step not found`
  failure. Stable macros therefore auto-track the bound file; retain the
  explicit declaration as the task-required, documentary dependency, not as a
  load-bearing rebuild guard. The prior stable compatibility and local gates
  passed; post-documentation acceptance checks also passed.

- Risk: the crate adds a large graph to every workspace build. The discarded
  Bevy `0.17.3` baseline resolved 139 crates for normal dependencies and 279
  including development dependencies; the `0.19.1` cost is not yet measured.
  `make lint` compiles it three times (`cargo doc`, `cargo clippy`, and the
  Whitaker Dylint driver, the last under its own toolchain with no Cranelift),
  and `make test` a fourth. CI caches no Cargo artefacts beyond the two
  shared-action caches. Severity: low, revised down after measurement.
  Likelihood: certain. Mitigation: repeat the measurements against `0.19.1`
  during the Milestone 0 follow-up and record the equivalent figures for the
  real workspace in `Outcomes & retrospective`.

- Risk: under the discarded `0.17.3` baseline, `target/` reached 2.2 GB for
  this crate alone. Both CI caches — the `setup-rust` debug-target cache and
  the coverage action's whole-target cache — grow accordingly, against GitHub's
  10 GB per-repository cap. Cache thrash degrades every run to cold without
  turning anything red. Severity: medium. Likelihood: medium. Mitigation:
  record the measured target size in `Outcomes & retrospective`. If the two
  caches together approach the cap, raise a follow-up to narrow the coverage
  job's cached path.

- Risk: the shared coverage action's default Nextest slow timeout can stop
  nested `trybuild` compilation before the UI-test binary completes. Hosted run
  `38310809953` terminated `macro_ui` at 180.004 seconds in the coverage job.
  Separately, the plain `cargo test` step measured `macro_ui` at 186.68
  seconds; its log timestamps show the next UI target starting more than 195
  seconds after its start. Another UI suite took 179.062 seconds. Cache misses
  were high. These observations establish that 180 seconds is insufficient for
  the observed cold UI run; they do not establish contention as the sole cause.
  Severity: high. Likelihood: observed. Mitigation: preserve the 180-second
  default slow timeout and 10-minute global timeout, serialize only binaries
  matching `binary(/_ui$/)` in a `compile-ui` group with `max-threads = 1`, and
  give that override a finite 60-second period with ten terminations and a
  five-second grace period (600 seconds total slow-time budget). The config
  contract tests and workspace group membership were locally validated, and the
  bounded full workspace suite passed (Revision 26). This warm local run does
  not establish that the cold hosted coverage timeout is resolved; coverage on
  the updated candidate remains pending. Do not treat the hosted coverage
  failure as resolved until that run passes.

- Risk: `make fmt` runs `cargo +nightly fmt` (the floating nightly) while
  `make check-fmt` runs `cargo fmt` (the toolchain-file-pinned nightly). With
  `unstable_features`, `wrap_comments`, `format_strings`, and
  `format_code_in_doc_comments` all enabled, the two can disagree — and this
  plan adds doc comments carrying worked examples, which is exactly what
  `format_code_in_doc_comments` rewrites. Severity: low. Likelihood: low.
  Mitigation: in Milestone 5, run `mdformat-all` and `cargo fmt --all` (pinned)
  separately rather than `make fmt`, so the formatting that lands is the
  formatting `make check-fmt` will accept.

- Risk: `googletest`'s `expect_that!` requires an active test context and
  panics with a no-test-context message when the test is not annotated
  `#[gtest]`. Severity: low. Likelihood: high (observed). Mitigation: unit
  tests use `#[gtest]` above `#[rstest]`; step functions use `assert_that!`,
  which panics directly and needs no context.

- Historical risk (resolved by subsequent rebases): the original workspace
  root requirement, `rstest = "0.18"`, differed from the then-selected
  `rstest-bdd 0.6.0-beta3` requirement. Later PRs advanced the root requirement
  to `0.27`; the harness now declares `rstest = "0.27"`. Current inverse
  dependency checks resolve Bevy `0.19.1`, `rstest` `0.27.0`, and `rstest-bdd`
  `0.6.0`; the post-documentation acceptance checks passed, while final PR
  acceptance remains pending.

## Progress

Latest hosted result (2026-10-12): PR #51 candidate
`06fac215a37a580a1a9281682bc5b2e0aa56d3f8`, based on `main`
`d177781a276012da013531e7b6b9642fc0a5e0b8`, completed Actions run
`38310809953`, attempt 1, job `115015810460`. The lint job passed. The
`Test and Measure Coverage` job failed because Nextest terminated
`rstest-bdd-harness-bevy::macro_ui` at 180.004 seconds; the run summary was 235
passed and one timed out. In the separate plain `cargo test` step, `macro_ui`
took 186.68 seconds: its log shows it starting at 21:36:59.603 and the next UI
target starting at 21:40:14.773, more than 195 seconds after `macro_ui`
started. In the coverage job, Nextest terminated `macro_ui` at 180.004 seconds;
a concurrent `skyjoust-test-macros` UI suite took 179.062 seconds, and only 5
of 2,147 Rust cache entries were hits. This shows the coverage timeout is below
observed cold UI-test runtime. Build contention is plausible, but the logs do
not prove it was the sole cause. The failure log is
`/tmp/skyjoust-pr51-actions-38310809953.eqWQZH/run-38310809953-attempt-1/job-115015810460.log`.

Provisional mitigation: preserve the shared action's default 180-second Nextest
slow timeout and 10-minute global timeout; serialize only `binary(/_ui$/)`
tests in a `compile-ui` group capped at one test, with a 60-second period, ten
terminations, and five-second grace period. A draft configuration is present in
`.config/nextest.toml`, and its contract test is present in
`tests/nextest_policy.rs`. Actual Nextest group membership is verified for both
UI binaries, with `max-threads = 1`. Current local validation and
resource-pressure observations are recorded in Revisions 24-26 below. The
non-test checks and the bounded full workspace test passed. This warm local run
does not prove the cold hosted coverage timeout is resolved; hosted coverage
remains pending. No claim is made that the coverage failure is fixed. The
roadmap implementation item `0.5.1.1` remains checked complete, while PR
acceptance remains pending. The separate user-facing adoption-guide finding is
already addressed at this candidate: `docs/rstest-bdd-harness-bevy-adoption.md`
documents setup and the headless/API boundary, `docs/contents.md` indexes it,
and `docs/users-guide.md` links to it. No duplicate migration note is needed.

Prior published status (before hosted run `38310809953`): PR #51 head
`adfcb5c5e1bc435f6a58e8a0cdc5fa800fb23800` was based on `main`
`d177781a276012da013531e7b6b9642fc0a5e0b8`. Its source tree was the locally
validated `f9a72c93b026e796ff025e71803e1a47a45ac31a` tree
`515c4b3bac3c97cb776e10dd10eda54bfd89e112`; the published follow-up changed
documentation only. Local formatter, documentation gates and acceptance checks
passed as recorded below. At that time, the PR was `BLOCKED` with review
decision `CHANGES_REQUESTED`, and build-test run `38309867260` was in progress.
Review findings referred to the superseded
`6b185eee6ee3d330585b08b2fa3d5a529754a536` head and required reconciliation. Run
`38309867260` reported only the observed published head. These observations
are historical; the latest hosted result is recorded above. Final PR acceptance
remains pending.

Candidate-execution provenance: this accepted base is the refreshed current
`origin/main`; PR #51's live author is `leynos`, the candidate commits are
authored by `leynos`, and this candidate is the direct user-authorized replay
of PR #51's source onto that base. This provenance is the trust basis for the
candidate-defined local gate and acceptance evidence below; it does not
establish hosted validation or PR acceptance.

The six implementation milestones are complete. Earlier pre-repair gate results
and the inherited `serve_explorer.rs` lint failure are retained below as
historical evidence; they are superseded by the authorized helper extraction
and current-candidate results. The roadmap implementation item `0.5.1.1`
remains checked complete, distinct from pending PR acceptance.

Pre-repair candidate validation: `make test` passed 236/236 tests plus
workspace doctests (log:
`/tmp/test-workspace-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-final5.out`).
The Final6 `make check-fmt`, `make markdownlint` (0 errors), `make typecheck`,
and `make nixie` results below also predate the authorized example repair. They
are historical for the current candidate; their logs (`check-fmt`,
`markdownlint`, `typecheck`, and `nixie`) use
`/tmp/{gate}-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-final6.out`.

Current candidate validation (2026-10-12): the user-authorized
behaviour-preserving `serve_explorer` helper extraction repairs the inherited
`clippy::cognitive_complexity` finding (15/9). It preserves tracing event order
and keeps `_checker` alive through `std::thread::park()`; no suppression or
threshold change was made. On candidate commit
`f9a72c93b026e796ff025e71803e1a47a45ac31a`, tree
`515c4b3bac3c97cb776e10dd10eda54bfd89e112`, the six sequential local gates all
passed: `make check-fmt`, `make test` (236/236 plus workspace doctests),
`make typecheck`, `make lint` (documentation, Clippy, and Whitaker),
`make markdownlint` (34 files, zero errors), and `make nixie`. Logs are
`/tmp/{check-fmt,test,typecheck,lint,markdownlint,nixie}-skyjoust-final515c4b3-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928.out`.

The constrained focused `macro_ui` rerun passed in 171 seconds, covering one
compile-pass fixture, both compile-fail fixtures, and Make's workspace doctests
(log:
`/tmp/test-macro-ui-focused-skyjoust-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-followup1.out`).
Earlier trybuild unexpected-success results and OS `EAGAIN` failures remain
historical observations; the constrained focused and full reruns passed.
Resource exhaustion was a plausible explanation for the earlier outcomes, not a
proven cause. The roadmap implementation item `0.5.1.1` remains checked
complete. The six-gate and focused results above predate this adoption-note
follow-up. The planner's `make fmt` passed; the scrutineer then passed the
post-documentation `make markdownlint`, `make nixie`, and acceptance checks:

- Formatter log:
  `/tmp/fmt-skyjoust-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-docs.out`.
- `make markdownlint` passed for 35 files with zero errors; log:
  `/tmp/markdownlint-skyjoust-postdocs-f9a72c9-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928.out`.
- `make nixie` passed; log:
  `/tmp/nixie-skyjoust-postdocs-f9a72c9-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928.out`.
- The isolated normal and normal-plus-development dependency graphs were
  clean; log:
  `/tmp/accept-isolated-graphs-skyjoust-postdocs-f9a72c9-20260928-3.out`.
- The workspace graphs contained 159 unique normal and 343 normal-plus-dev
  packages, resolving Bevy `0.19.1`, `rstest` `0.27.0`, and `rstest-bdd`
  `0.6.0`; log:
  `/tmp/accept-workspace-graphs-skyjoust-postdocs-f9a72c9-20260928.out`.
- The isolated crate and workspace normal graphs were headless; log:
  `/tmp/accept-headless-graphs-skyjoust-postdocs-f9a72c9-20260928.out`.
- The standalone crate suite passed with pinned nightly-2026-03-26, including
  five unit tests, four extraction-boundary tests, one scenario, the UI
  fixtures, one property test and two doctests; log:
  `/tmp/accept-standalone-crate-test-skyjoust-postdocs-f9a72c9-20260928-pinned.out`.
  The initial outside-root attempt used stable Cargo 1.99, which rejected the
  `codegen-backend` profile in `tools/dev-fast/config.toml` (log:
  `/tmp/accept-standalone-crate-test-skyjoust-postdocs-f9a72c9-20260928.out`).
  The intervening wrong-nightly attempt was stopped before completion (partial
  log:
  `/tmp/accept-standalone-crate-test-skyjoust-postdocs-f9a72c9-20260928-2.out`);
  only the pinned-nightly rerun is the accepted result.
- The focused in-workspace crate suite and doctests passed; log:
  `/tmp/accept-focused-crate-test-skyjoust-postdocs-f9a72c9-20260928.out`.
- The stable feature-file freshness A/B passed from a disposable archive,
  without tracked worktree edits. Both warm baselines passed, and feature-only
  edits failed with the expected `Step not found` both without and with the
  explicit include. Logs:
  `/tmp/accept-feature-ab-no-guard-baseline-skyjoust-f9a72c9-20260928.out`,
  `/tmp/accept-feature-ab-no-guard-feature-edit-skyjoust-f9a72c9-20260928.out`,
  `/tmp/accept-feature-ab-guard-present-baseline-skyjoust-f9a72c9-20260928.out`,
  `/tmp/accept-feature-ab-guard-present-feature-edit-skyjoust-f9a72c9-20260928.out`,
  and
  `/tmp/accept-feature-ab-restored-guard-baseline-skyjoust-f9a72c9-20260928.out`.

These results validate the local documentation follow-up and named acceptance
checks; they are not hosted results. PR #51 is now published at
`adfcb5c5e1bc435f6a58e8a0cdc5fa800fb23800`. At the latest status check, the PR
was ready but `BLOCKED`, its review decision was `CHANGES_REQUESTED`, and
build-test run `38309867260` was still in progress. Existing review findings
refer to the superseded `6b185eee6ee3d330585b08b2fa3d5a529754a536` head.
Current hosted results and review reconciliation, and therefore final PR
acceptance, remain pending.

Pre-rebase scrutineer evidence: on candidate HEAD
`6b185eee6ee3d330585b08b2fa3d5a529754a536`, all seven gates passed:
`make check-fmt`, `make check-state-graphs`, `make markdownlint`,
`make typecheck`, `make lint`, `make test` (89 tests plus doctests passed), and
`make nixie`. Acceptance checks also passed for package metadata/workspace
membership, development and normal dependency graphs, the focused crate test
suite, and the feature-file freshness A/B check. These are pre-rebase results;
they do not validate the eventual rebased candidate.

Rebase evidence (2026-09-28): the branch was rebased from old head
`8ef7b635d9bd912e4c0934bf1434d8f7715e6e28`, whose old base was
`9e26f1565a8b29c084f20e868eb62ba6ee21af1f`, onto target
`bcbdcd59ec8aaa6d0936884765c813eab2a2fc00`. The current replay head is
`45da853c19ba1b99ba094580a4d1510894536ce9`. The sole conflict was `Cargo.lock`:
verified target stage 2 was kept and the `6b185ee` ExecPlan changes were
preserved. The lockfile feature diff was deferred to post-rebase regeneration.

Historical snapshot: PR #65's root `rstest 0.27` update superseded the earlier
`0.26` alignment; the harness now resolves `rstest` `0.27.0`. The stable
`rstest-bdd` 0.6.0 decision and compatibility observations below supersede the
beta3 pins. Pre-rebase gate results and the earlier candidate's lint failure
are retained as historical evidence, not current blockers. The current local
gate and post-documentation acceptance results, together with pending PR
lifecycle work, are recorded at the start of this section. Bevy `0.19.1`
compile, feature, clock, Cranelift, and cost evidence remains in
`Surprises & discoveries`.

Stable-release compatibility observations (2026-10-12): the upstream v0.6.0
migration guide documents that a custom `HarnessAdapter::run` returns
`HarnessResult<T>`, fallible result-like step aliases propagate errors, the
minimum Rust version is 1.88, and scenario-bound feature files are automatic
Cargo rebuild dependencies. The current crate remains the profile-helper
scaffold for task `0.5.1.1`; the user-facing functions and test cases are
documented in its README. The focused `macro_ui` target passed against stable
0.6.0, as did the feature-file freshness A/B: after a warm pass, a feature-only
edit with no Rust-source touch triggered recompilation and the expected
`Step not found` failure both without and with explicit `include_str!`. This
confirms macro-level tracking; the required explicit declaration remains as a
documentary, belt-and-braces dependency, not a load-bearing guard. The later
focused and full test runs passed; the post-documentation acceptance rerun is
recorded below and passed. See the
[imported migration guide](../rstest-bdd-v0.6.0-migration-guide.md) and
[user's guide](../rstest-bdd-v0.6.0-users-guide.md), sourced from the exact
[upstream migration guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/v0-6-0-migration-guide.md)
and
[upstream user's guide](https://github.com/leynos/rstest-bdd/blob/v0.6.0/docs/users-guide.md).
The exact import sources are
<https://raw.githubusercontent.com/leynos/rstest-bdd/refs/tags/v0.6.0/docs/v0-6-0-migration-guide.md>
and
<https://raw.githubusercontent.com/leynos/rstest-bdd/refs/tags/v0.6.0/docs/users-guide.md>.

Current validation (2026-10-12): inverse dependency checks passed with Bevy
`0.19.1`, `rstest` `0.27.0`, and `rstest-bdd` `0.6.0`. The standalone manifest
crate suite passed: five unit tests, four extraction-boundary tests, one
headless scenario, one trybuild runner with pass/fail fixtures, one property
The 2026-10-12 validation snapshots for candidate `6d5babe` are historical: the
temporary-edit run was invalid, and the subsequent in-scope run failed
`make lint` on cognitive complexity 15/9 in the unchanged `serve_explorer.rs`.
The maintainer later authorized the narrow helper extraction; current success
and the earlier failed trybuild/EAGAIN observations are recorded at the start
of this section. Both documentation gates and the acceptance commands listed in
the review follow-up checklist passed after the adoption-note edits. PR #51 is
published, but hosted outcomes, review reconciliation, and final PR acceptance
remain pending as recorded above.

- [x] (2026-08-15) Milestone 0: orientation and evidence gathering. Six probes
      run against Bevy `0.17.3`; findings in `Surprises & discoveries` and
      `Artefacts and notes` are retained as historical evidence.
- [x] (2026-08-17) Milestone 0 follow-up: align the stack and validate the
      chosen baseline.
  - [x] Watch pull request #6 until automerge completes. Merged
        2026-08-16T22:25:57Z (`5f5bc7c Bump rstest from 0.18.2 to 0.26.1 (#6)`).
  - [x] Rebase the branch stack onto `main`. The four ExecPlan commits were
        replayed onto `origin/main` without conflicts; the lower layer's
        `skyjoust-test-macros` content is already merged as `4a895dc` (#53).
  - [x] Confirm the merged root `rstest` requirement and reconcile the new
        crate's development requirement with it. The merged root and validator
        manifests both carry `rstest = "0.26"`. The new crate's dev requirement
        in `Interfaces and dependencies` is reconciled to the same implicit
        caret line (`0.26`), which remains compatible with `rstest-bdd
        0.6.0-beta3` (which expects `rstest 0.26.1`).
  - [x] Repeat the API, feature, headless-graph, Cranelift, test, lint, and
        cost probes against Bevy `0.19.1`. Discharged by the Milestone 2/3
        build; evidence is recorded in `Surprises & discoveries` and
        `Artefacts and notes`, and Table 1 below carries the `0.19.1`
        resolution figures.
  - [x] Update all historical expectations in this plan with the new evidence.
        The 0.17.3 probe figures remain labelled historical and the `0.19.1`
        figures stand beside them.
- [x] (2026-08-17) Milestone 1: record the decision.
  - [x] Write the new ADR 007.
  - [x] Amend the harness design document §§3, 9, 13 and its layout block.
  - [x] Add a forward pointer from ADR 002.
  - [x] Index ADR 007 and this ExecPlan in `docs/contents.md`.
  - [x] Documentation gates green.
- [x] (2026-08-17) Milestone 2 (red): scaffold and failing tests.
  - [x] Add the workspace member.
  - [x] Write the crate manifest.
  - [x] Write `src/lib.rs` with the module declaration but no re-export of it.
  - [x] Write `src/profile.rs` (module comment only) and `src/profile_tests.rs`.
  - [x] Observe the red state; record the exact error text.
  - [x] `make typecheck` as a whole-graph Cranelift smoke test.
- [x] (2026-08-17) Milestone 3 (green): implement `src/profile.rs`; export its
      two functions.
- [x] (2026-08-17) Milestone 4: behavioural, property, and boundary coverage.
  - [x] `tests/features/headless_scenario.feature`.
  - [x] `tests/headless_scenario.rs`.
  - [x] Clippy immediately after the scenario binding first compiles.
  - [x] `tests/tick_properties.rs`.
  - [x] `tests/extraction_boundary.rs`.
  - [x] A/B-prove the feature-file rebuild guard.
- [x] Add `trybuild` compile-pass and compile-fail coverage for the
      `rstest-bdd` attribute-macro contract exercised by the integration test,
      with checked diagnostics for representative invalid declarations. Keep
      the scenario-macro missing-feature case and add a direct
      `include_str!` missing-file case for the literal integration-test
      declaration; test feature-file rebuild tracking separately with the A/B
      freshness check.
- [x] (2026-08-17) Milestone 5: documentation.
  - [x] Crate `README.md` including the Bevy compatibility table.
  - [x] `docs/repository-layout.md`.
  - [x] `docs/developers-guide.md`.
  - [x] `docs/roadmap.md` — tick `0.5.1.1`, correct the stale sub-bullet.
- [x] (2026-08-17) Milestone 6: full gate run, measurements, push, draft
      pull request.
  - [x] Full gate run green: `make check-fmt`, `make check-state-graphs`,
        `make markdownlint`, `make lint`, `make test`, and `make nixie` all
        pass via the `scrutineer` sub-agent (nextest 89/89 plus all doctests;
        hold-up numbers recorded in `Outcomes & retrospective`).
  - [x] Measurements recorded in `Outcomes & retrospective`.
  - [x] CodeRabbit review cleared. `coderabbit review --agent` completed with
        zero findings across 19 reviewed files.
  - [x] PR title confirmed free of the plan-draft "Plan: " prefix; PR #51 body
        rewritten for the implementation deliverable; the Lody session titles
        updated.
  - [x] (2026-08-21) Rebase onto `origin/main`; preserve `main`'s conflicted
        `Cargo.lock`, then regenerate it from the final manifests. The rebuilt
        lock resolves `bevy 0.19.1` and every inspected `rstest-bdd` package at
        `0.6.0-beta3`.
  - [x] (2026-08-21) Re-run all local gates: `make check-fmt`,
        `make check-state-graphs`, `make markdownlint`, `make typecheck`,
        `make lint`, `make test` (89 passed), and `make nixie`.
  - [x] (2026-08-21) Re-run the isolated and workspace graph checks, focused
        13-test crate suite, and feature-file freshness A/B. Both graphs are
        headless, the isolated extraction graph is clean, and only the guarded
        feature-only edit fails.
  - [x] (2026-08-21) Publish the rebased branch with
        `git push --force-with-lease`; it is published and tracks `origin` at
        `6b185eee6ee3d330585b08b2fa3d5a529754a536`.

- [ ] (2026-09-28) Resolve the review follow-up and rerun the requested checks.
  - [x] Scrutineer passed all seven gates and acceptance checks on pre-rebase
        HEAD `6b185eee6ee3d330585b08b2fa3d5a529754a536`; results are historical
        and do not validate the rebased candidate.
  - [x] Rebase old head `8ef7b635d9bd912e4c0934bf1434d8f7715e6e28` from
        old base `9e26f1565a8b29c084f20e868eb62ba6ee21af1f` onto
        `bcbdcd59ec8aaa6d0936884765c813eab2a2fc00`; current replay head is
        `45da853c19ba1b99ba094580a4d1510894536ce9`. The sole conflict was
        `Cargo.lock`: retain verified target stage 2 and preserve the `6b185ee`
        ExecPlan changes; defer its feature lockfile diff to regeneration.
  - [x] Align the harness development dependency to `rstest = "0.27"`; the
        earlier regenerated lockfile resolved it to `0.27.0` alongside beta3.
        This is historical resolution evidence, not current stable-adoption
        validation.
  - [x] Update the harness manifest requirements to published stable
        `rstest-bdd` 0.6.0 with implicit caret syntax.
  - [x] Regenerate and inspect `Cargo.lock` for the stable family; inverse
        dependency checks resolve Bevy `0.19.1`, `rstest` `0.27.0`, and
        `rstest-bdd` `0.6.0`.
  - [x] Run `make fmt`; it passed without additional churn.
  - [x] Complete focused crate validation against stable 0.6.0: five unit
        tests, four extraction-boundary tests, one headless scenario, one
        trybuild runner with pass/fail fixtures, one property test, and two
        doctests passed. This focused suite does not replace workspace gates.
  - [x] Replace the sibling `skyjoust-test-macros` dev-dependency with the
        nested, crate-owned `rstest-bdd-harness-bevy-test-macros` package.
        Retain the synthetic near-match canary in the direct boundary test;
        it is not an actual dependency edge.
  - [x] Add a `trybuild` compile-pass fixture for an `rstest` fixture and
        `rstest-bdd` step macros, plus a compile-fail scenario for a missing
        scenario feature file. The pass fixture does not expand a valid
        `#[scenario]` or exercise `_app` implicit fixture normalization; the
        successful scenario binding is compiled and exercised by
        `tests/headless_scenario.rs`.
  - [x] Run the focused `macro_ui` target against stable 0.6.0. Its single
        trybuild runner passed with the checked missing-feature diagnostic;
        evidence is `/tmp/test-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-2.out`.
        The full workspace gates and acceptance remain pending.
  - [x] Verify the manifest path-dependency finding. The sibling
        `skyjoust-test-macros` dependency was replaced by the nested,
        crate-owned `rstest-bdd-harness-bevy-test-macros` workspace member.
        ADR 009 records the extraction boundary; the synthetic near-match
        canary remains in the direct manifest guard.
  - [x] Verify the compile-time-test finding. The trybuild pass fixture covers
        an `rstest` fixture and `rstest-bdd` step macros; it does not exercise a
        valid `#[scenario]` expansion or `_app` implicit fixture normalization.
        The successful scenario binding is compiled and exercised by
        `tests/headless_scenario.rs`. The compile-fail fixture binds a missing
        scenario feature file, and a separate compile-fail fixture covers a
        missing literal `include_str!` target. The focused
        `TRYBUILD=overwrite` run passed with the pass fixture and both
        compile-fail fixtures; evidence is
        `/tmp/coderabbit-macro-ui-snapshot-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-3.out`.
        At that point the full crate gates remained pending. The stable
        feature-file freshness A/B also passed; later gate results and current
        acceptance work are recorded at the start of `Progress`.
  - [x] Add the consumer section to `docs/users-guide.md`, the Skyjoust-specific
        adoption note, and the imported upstream v0.6.0 user's and migration
        guides. The upstream migration guide covers framework changes; the new
        note covers this workspace's dependency and helper migration boundary.
  - [x] Update ADR 009, the harness design, roadmap and crate README to record
        stable 0.6.0 as the selected dependency line and retain beta3 as history.
  - [x] Add a dated tooling-exception addendum to ADR 002 without changing its
        accepted runtime crate decision; ADR 009 remains authoritative.
  - [x] After the documentation follow-up, rerun the documentation gates and
        acceptance checks with the scrutineer. The results and log paths are
        recorded in `Progress` above.
  - [ ] Update the PR description, request a new CodeRabbit review, and record
        their outcomes after current-candidate validation and publication.

## Surprises & discoveries

- Historical observation (2026-09-28, superseded): ADRs 006 and 007 then
  permitted the sibling `skyjoust-test-macros` dev-dependency as test tooling.
  Review verification later established that its relative path prevents
  standalone manifest resolution and violates the extraction boundary. Replace
  it with a crate-owned extraction-safe fixture lint helper and reject the
  sibling dependency in the test graph.

- Historical observation (2026-09-28 review verification, superseded by the
  stable A/B below): a missing `include_str!` file is rejected by rustc, so no
  generic compile-fail fixture for that built-in is needed. The integration
  test's `rstest-bdd` attributes do define a compile-time macro contract,
  however; add representative `trybuild` pass/fail fixtures with checked
  diagnostics. Stable 0.6.0 also registers bound feature files as rebuild
  dependencies; the completed stable A/B below shows the explicit
  `include_str!` declaration is not load-bearing.

- Observation (2026-09-28 review verification): the documentation style guide
  includes library API consumers in the user's-guide audience. The new guide
  section documents the current workspace-only dev-dependency setup, the two
  public functions, Bevy `0.19.1`, and explicit headless schedule advancement.
  No n+1 migration document is warranted before there are downstream consumers;
  broader adoption and extraction instructions remain roadmap `0.5.1.5`.

- Observation (2026-09-28): roadmap task `0.5.1.1` was already checked at
  `docs/roadmap.md:41`; no roadmap edit was needed for this review follow-up.

- Observation (2026-09-28 review verification): retain the exact
  `=0.6.0-beta3` requirements despite the caret-pin review comment. This was
  the maintainer-approved pre-release exception at that time; it is superseded
  by the 2026-10-12 stable 0.6.0 adoption decision below.

- Observation (2026-09-28 rebase): main's PR #65 updates the root `rstest`
  requirement to `0.27`, superseding the earlier `0.26` alignment recorded
  above. The harness development dependency is now `rstest = "0.27"`; the
  regenerated lockfile resolves it to `0.27.0` and keeps the exact `rstest-bdd`
  `=0.6.0-beta3` pins. `make fmt` passed without extra churn. Focused crate
  compatibility validation is queued behind the shared Cargo cache lock and
  remains pending.

- Observation (2026-10-12): the imported v0.6.0 migration guide says custom
  `HarnessAdapter::run` implementations return `HarnessResult<T>`, result-like
  step aliases propagate errors, bound feature files are Cargo rebuild
  dependencies, and the minimum Rust version is 1.88. The current crate remains
  the task `0.5.1.1` profile-helper scaffold and does not define a custom
  `HarnessAdapter`; verify the step and include contract against the stable
  release. The current manifest uses implicit-caret `0.6.0` requirements, but
  the last inspected lockfile still resolved beta3 and must be regenerated.

- Observation (2026-10-12): the sibling fixture-macro dependency has been
  replaced by nested package `rstest-bdd-harness-bevy-test-macros`; its only
  public item is the test-only fixture lint attribute. The trybuild pass case
  covers the `rstest` fixture and `rstest-bdd` step macros, not a valid
  scenario expansion or `_app` normalization; `tests/headless_scenario.rs`
  compiles and exercises the valid binding. The fail case binds a missing
  feature file. The focused trybuild runner passed against stable 0.6.0; full
  crate compatibility remains pending. Evidence: the `/tmp` log named in the
  review follow-up above.

- Observation (2026-10-12): the stable 0.6.0 freshness A/B confirms the
  migration-guide behaviour. Both warm arms passed; a feature-only edit with no
  Rust-source touch recompiled and failed with the expected `Step not found`,
  both with and without explicit `include_str!`. The macro auto-tracks the
  bound file, so the explicit declaration is not load-bearing; retain it
  because the task contract requires the documentary dependency. Evidence:
  `/tmp/test-headless_scenario-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-2.out`
  (without the declaration) and
  `/tmp/test-headless_scenario-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-5.out`
  (with it).

- Observation (2026-09-28 documentation review): ADR 002's accepted runtime
  crate decision remains accurate, but its Consequences needed a dated addendum
  for the Bevy harness tooling exception (originally ADR 007, now ADR 009). The
  addendum records that distinction without reopening the accepted runtime
  split.

- Observation: `rstest-bdd` `0.6.0-beta3` was published to crates.io on
  2026-07-07, along with `rstest-bdd-harness`, `rstest-bdd-macros`,
  `rstest-bdd-harness-tokio`, and `rstest-bdd-harness-gpui` at the same
  version. The roadmap's instruction to use git dependencies against `main`
  "until v0.6.0-beta3 is published" is therefore already satisfied. Evidence:
  the crates.io versions endpoint lists `0.6.0-beta3` created at
  `2026-07-07T23:13:10Z`; `rstest-bdd-harness 0.6.0-beta3` at
  `2026-07-07T23:12:46Z`. Impact: this plan uses published requirements, not
  git dependencies. The roadmap sub-bullet and design §9 are amended
  accordingly.

- Observation: `rstest-bdd`'s `main` branch has moved to an unpublished
  `0.6.0-beta4` (head commit `12b9357`, 2026-08-14), well past the commit
  `21b67a4` cited in the design document. That version adds a
  `policy_conformance` module, a `testing` feature exposing `FailingHarness`,
  and guard-based fixture borrowing. It also changes the generated panic text
  from `initialise` to `initialize`. Evidence: `main`'s harness crate root
  exports `policy_conformance` and a feature-gated `FailingHarness`; the
  extracted `beta3` archive exports neither and has no features table at all.
  Impact: under `beta3`, requesting the `testing` feature will not resolve, and
  the policy-conformance helper is unavailable. Neither is needed here;
  `0.5.1.3` must plan around them. Recorded in the design document with an
  explicit migration trigger.

- Observation: in the earlier Bevy `0.17.3` probe,
  `default-features = false` alone silently degraded timekeeping.
  `bevy_platform`'s time module selects `std::time` only when its `std`
  configuration predicate holds; otherwise it uses a fallback whose x86-64
  getter passes `_rdtsc()` straight to `Duration::from_nanos` — raw timestamp
  counter ticks reinterpreted as nanoseconds. `TimePlugin` calls the clock
  unconditionally. Evidence: `bevy_platform-0.17.3/src/time/mod.rs` selects the
  fallback module in the non-`std`, non-web branch; `src/time/fallback.rs`
  contains the `_rdtsc()` arm. Impact: the plan retains `features = ["std"]`,
  but the evidence cannot be projected onto the newly selected `0.19.1`
  release. Repeat the source and resolved-feature checks in Milestone 0;
  escalate if `0.19.1` needs a different feature declaration to preserve real
  time without enabling a renderer or window.

- Observation: `clippy.toml`'s `allow-expect-in-tests` setting does not exempt
  non-`#[test]` free functions inside `tests/*.rs`. A step function calling
  `.expect()` fails Clippy with warnings denied. Evidence: a probe using a
  thread-local `Option` slot and `.expect()` in a `#[when]` step produced two
  `expect_used` errors. Impact: the behavioural test uses an `rstest` fixture
  holding the application directly. Its steps return `StepResult` and propagate
  `RefCell` borrow failures. Published `rstest-bdd 0.6.0-beta3` supports
  `Result`-returning steps, so this is the normal error path rather than a
  workaround. The same trap will recur in `0.5.1.2`; it is documented in the
  developer's guide.

- Observation: a single-expression `#[fixture]` body trips `unused_braces`
  under denied warnings, and `fn_single_line` reformats the multi-line repair
  straight back to the failing form. Evidence:
  `error: unnecessary braces around block return value` at the fixture, with
  `-D unused-braces implied by -D warnings`; a plain non-fixture function with
  the same shape does not fire. Impact: this is a known `rstest` issue with an
  estate-mandated remedy, not a problem for this plan to solve locally. The
  layer beneath this one in the stack adds `crates/skyjoust_test_macros` and its
  `allow_fixture_expansion_lints` attribute, mirroring `weaver-test-macros` in
  `leynos/weaver`. An earlier draft of this plan proposed a local `let`-binding
  workaround; that has been withdrawn, because it would have spread a bespoke
  idiom across every fixture the project writes rather than fixing the cause
  once. Verified clean under both Clippy and `cargo fmt --check` with the
  attribute applied and the fixture left in its natural form.

- Observation: `googletest 0.14.3`'s `expect_that!` requires the `#[gtest]`
  attribute; without it every assertion panics reporting no test context.
  Evidence: four unit tests annotated only `#[rstest]` failed that way; adding
  `#[gtest]` above `#[rstest]` made all four pass. Impact: unit tests use
  `#[gtest]` plus `#[rstest]`; step functions use `assert_that!`.

- Observation: the constructors proposed in the first draft could not become
  what the design says they become. Design §5 declares a `configure` that takes
  `&mut App` and returns a harness result — it mutates a borrowed application —
  whereas a constructor builds and returns one. Evidence: design §5's trait
  definition, compared with the draft's `pub fn minimal_app() -> App`. Impact:
  the public surface was reshaped. `add_minimal_plugins` takes `&mut App`,
  which is the shape `configure` calls, so `MinimalBevyProfile::configure`
  becomes two lines that call it. `bare_app` was dropped entirely: a plain
  `App::new()` needs no wrapper, and `BareBevyProfile::configure` has an empty
  body, so there was nothing to seed.

- Observation: in the earlier Bevy `0.17.3` probe, `App::new()` was not
  plugin-free. It already added `MainSchedulePlugin` and registers a
  message-update system in `First`. Evidence: `bevy_app-0.17.3/src/app.rs`.
  Impact: re-check this contract against `0.19.1` before retaining the negative
  test. If the contract still holds, keep the test named for what it checks —
  that an unconfigured application omits `TimePlugin` — rather than claiming
  `App::new()` adds no plugins.

- Observation: the first draft's build-cost figure of 98 crates was the
  Bevy-only probe measured for normal dependencies. The full `0.17.3` probe
  resolved 139 crates for normal dependencies and 279 including development
  dependencies. Evidence:
  `cargo tree --workspace -e normal --prefix none | sort -u | wc -l` and the
  `-e normal,dev` equivalent, run against the full shape probe. Impact: the
  build-cost risk is restated with measured wall-clock rather than crate counts.

- Observation: the CI coverage action builds with *default* features via
  `cargo llvm-cov nextest`, and nextest does not run doctests. Evidence: the
  `generate-coverage` action's Rust runner script in `leynos/shared-actions`.
  Impact: doctests contribute nothing to the codecov figure. The patch target
  is still met, but by the unit tests — the first draft gave the wrong reason.

- Observation: `docs/rstest-bdd-users-guide.md` and
  `docs/ortho-config-users-guide.md`, cited in the earlier `1.1.1` execplan, do
  not exist in this repository. Evidence: repository-wide glob for both names
  returns nothing. Impact: this plan cites the upstream `rstest-bdd` users'
  guide by uniform resource locator (URL) instead, and does not create either
  local file.

- Observation: the selected Bevy `0.19.1` graph is smaller than the discarded
  `0.17.3` baseline, at 121 normal crates and 267 with development
  dependencies, measured against the historical 139 and 279. Evidence:
  `cargo tree -p rstest-bdd-harness-bevy -e normal --prefix none | sort -u |
  wc -l`
  and the dev-inclusive equivalent. Impact: both `EP-INV-002` headless queries
  and the `EP-INV-001` transitive extraction query print the expected clean
  result, and a single `bevy v0.19.1` line remains (`EP-REQ-003`).
  `bevy-window` appears in the *lockfile* but is a non-Linux-target dependency:
  it is absent from the reachable Linux normal-dependency graph, so both
  headless queries pass. The 0.19.1 figures replace the historical `0.17.3`
  table entries.

- Observation: since the lexical crate files were pre-created during a
  bash-classifier outage, the Milestone 2 red run reported the expected
  `profile_tests.rs` grouped diagnostic and two further `E0432`s from the
  not-yet-implemented `minimal_app` re-export in the pre-created integration
  test files. The red-state gate is satisfied: the error names exactly the two
  missing symbols, and no other cause appears in the build output.

- Observation: a background Cargo-aware process (rust-analyzer) resolved the
  workspace while the new manifest was being created and wrote a 335-package
  `Cargo.lock` superset that included `bevy-window` as a reachable package. The
  file was restored from `main` and the Milestone 2 build regenerated the
  authoritative 227-package lockfile from the crate's actual manifest. Impact:
  the lockfile must not be trusted to `cargo metadata` from a watcher;
  regenerate it deliberately with the milestone build.

- Observation: the cold Cranelift resolve plus lock plus compile of the full
  `0.19.1` graph and the `rstest-bdd` family took 40 seconds wall-clock on this
  six-core host, counting the two package-cache lock waits. Evidence: the
  Milestone 2 red run's own elapsed time. Impact: the build-cost tolerance is
  not stressed; the warm runs in Milestones 3-6 will confirm the repeat cost.

- Observation: Milestone 3's unit tests discharge two `0.19.1` contracts the
  plan had to re-check: `App::new()` still omits `TimePlugin` (the negative
  test passes), and `MinimalPlugins` still includes `FrameCountPlugin`
  (updating advances the frame counter). Evidence: the five unit tests and the
  two doctests pass. Impact: the retained test names describe what they check.

- Observation: `clippy::double_must_use` objects to a bare `#[must_use]` on
  `minimal_app::() -> App` because Bevy's `App` is itself `#[must_use]`.
  Evidence: `make lint` flags profile.rs and suggests "either add some
  descriptive message or remove the attribute". Impact: the attribute carries a
  message —
  `#[must_use = "advance the returned application or inspect its world"]` —
  which keeps the plan's requirement and satisfies the estate deny.

- Observation: the ExecPlan's verbatim `headless_scenario.rs` snippet trips
  `clippy::shadow_reuse` under the current workspace baseline: the closure
  parameters `|app|`, `|mut app|`, and their inner `let` shadow the step
  function's own `app` parameter. Evidence: `make lint` lists three
  `shadow_reuse` diagnostics against the snippet as written. Impact: the
  closures are renamed (`shared`, `borrow`) without changing behaviour; the
  snippet in this plan is amended to match.

## Decision log

- Decision (provisional; draft present, validation pending): give only
  compile-based Nextest UI test binaries matching `binary(/_ui$/)` a dedicated
  `compile-ui` group with `max-threads = 1` and a finite 600-second slow-time
  budget (`period = "60s"`, ten terminations, five-second grace). Preserve the
  shared action's default 180-second slow timeout for other tests and its
  10-minute global timeout. Drafts are present in `.config/nextest.toml` and
  `tests/nextest_policy.rs`; group membership and the policy contract tests
  passed locally, as did the bounded workspace test and other local gates.
  These warm local results do not prove the cold hosted coverage timeout is
  resolved. Rationale: hosted run `38310809953`, attempt 1, job `115015810460`,
  passed lint but timed out `macro_ui` at 180.004 seconds; the separate plain
  `cargo test` step measured that binary at 186.68 seconds, with the next UI
  target starting more than 195 seconds after its start. A second UI suite took
  179.062 seconds. Cache hits were 5/2,147. These observations show the default
  is insufficient for the observed cold run; they do not prove contention as
  the sole cause. A successful hosted coverage run is required before resolving
  this finding or accepting the PR. Date/Author: 2026-10-13, maintainer
  decision.

- Historical decision: depend on published `rstest-bdd` `0.6.0-beta3` crates
  rather than git dependencies against `main`. Rationale: the roadmap's
  git-dependency instruction was explicitly conditional on `beta3` not yet
  being published; it now is. Published crates keep `Cargo.lock` stable and
  avoid pulling `rstest-bdd`'s vendored GPUI tree into this workspace's
  resolution. The later APIs are not needed here. Date/Author: 2026-08-15,
  planning pass.

- Decision (superseded): use exact `=0.6.0-beta3` requirements as a temporary
  prerelease exception. That decision protected the workspace from incompatible
  beta updates and remains historical evidence only.

- Decision: adopt the published stable `rstest-bdd` 0.6.0 family with implicit
  caret requirements after the compatibility pass, restoring the standard
  dependency policy. The v0.6.0 user's and migration guides are imported and
  reviewed. The current manifest uses the stable requirements, but the lockfile
  still needs regeneration and compatibility tests plus acceptance gates have
  not yet been run against the final release. This decision does not make the
  implementation accepted. Date/Author: 2026-10-12, maintainer decision.

- Historical decision: use exact `=0.6.0-beta3` requirements, as a
  maintainer-approved temporary exception to the implicit-caret rule for
  pre-release requirements only. Rationale: a caret on a pre-release admits
  `0.6.0-beta4`, which upstream has made source-breaking, and Dependabot runs
  daily with automerge here. `AGENTS.md` forbids `*` and `>=` and constrains
  `~` but is silent on `=`, and the mandate's stated purpose — build stability
  and reproducibility — is served by pinning a pre-release rather than floating
  across it. A published `0.6.0` final is a migration trigger, not proof of
  compatibility. The 2026-10-12 stable-adoption decision supersedes this
  prerelease exception; compatibility validation was pending at that time.
  Current evidence is recorded in `Progress`. Date/Author: 2026-08-17,
  maintainer decision.

- Decision: write stable dependency requirements with implicit caret syntax,
  such as `0.19.1`, rather than redundant explicit syntax such as `^0.19.1`.
  Rationale: both forms have the same Cargo semantics, while the implicit form
  matches the repository's manifest convention. The exact `=0.6.0-beta3`
  requirements were the temporary exception recorded above and are superseded.
  Date/Author: 2026-08-17, maintainer decision.

- Decision: declare Bevy with `default-features = false` plus
  `features = ["std"]`. Rationale: disabling default features is what the
  roadmap and design require, and the `0.17.3` probe showed that `std` was
  necessary to avoid fallback timekeeping without enabling the renderer or
  window. The declaration remains the intended `0.19.1` starting point, but
  Milestone 0 must reconfirm its feature graph and clock implementation before
  treating the decision as discharged. Determinism is explicitly *not* part of
  this rationale: nothing in this milestone has ambiguous system ordering.
  Date/Author: 2026-08-15, planning pass.

- Decision: target Bevy `0.19.1` for both the harness and the future Skyjoust
  runtime. Rationale: Bevy types cross the harness boundary, so the harness
  cannot make an independent version choice. The roadmap's earlier `0.17.3`
  text is stale and must be updated in Milestone 1. Because the completed
  probes used `0.17.3`, repeat their compatibility, feature, and cost checks
  before code work begins. Date/Author: 2026-08-17, maintainer decision.

- Decision: propagate errors from `rstest-bdd` step functions and prefer
  functional transformations where they remain readable. Rationale: step
  functions are macro-registered free functions, not `#[test]` functions, so
  `.expect()` violates the estate-wide `clippy::expect_used` policy.
  `rstest-bdd 0.6.0-beta3` natively supports `Result`-returning steps. Returning
  `StepResult`, using fallible borrow APIs, and applying `map`, `map_err`, or
  `?` preserves diagnostic context without lint suppression or panic-based
  control flow. Date/Author: 2026-08-17, maintainer decision.

- Decision: wait for pull request #6 to automerge, then rebase this branch
  stack onto `main` before adding the new crate. Rationale: pull request #6
  updates the root package from `rstest 0.18` to the latest public release.
  Rebasing first avoids deliberately introducing a short-lived second `rstest`
  line and lets the new crate follow the merged workspace convention. A
  post-rebase compatibility check decides the exact implicit-caret requirement;
  incompatibility is an escalation condition. Date/Author: 2026-08-17,
  maintainer decision.

- Decision: this milestone ships two public functions, `add_minimal_plugins`
  and `minimal_app`, in `src/profile.rs`, plus a re-export block in `lib.rs`.
  Rationale: the ExecPlan bar is demonstrably working behaviour, not a crate
  that merely compiles. `add_minimal_plugins` has exactly the shape design §5's
  `configure` calls, so `MinimalBevyProfile::configure` becomes two lines that
  call it in `0.5.1.2` — the carry-forward is real, not aspirational.
  `minimal_app` is the harness-free entry point for tests that want an
  application without a scenario, which Skyjoust and Lille will both want at
  `0.5.1.4`. Both stay public after `0.5.1.2`. Alternatives considered and
  rejected: (a) a `bare_app` constructor alongside them — a plain `App::new()`
  needs no wrapper and `BareBevyProfile::configure` has an empty body, so it
  would seed nothing; (b) shipping the whole `BevyProfile` trait seam now — the
  roadmap explicitly assigns `BevyProfile`, `BareBevyProfile`, and
  `MinimalBevyProfile` to task `0.5.1.2`, and this plan must not consume the
  next task's named deliverables; (c) an empty library with `bevy` and
  `rstest-bdd-harness` moved to development dependencies — viable, and the
  behavioural scenario would still run, but it defers every signature-shaped
  risk in design §5 and leaves the crate root undocumentable. Date/Author:
  2026-08-15, planning pass.

- Decision: re-export `bevy` from the crate root.
  Rationale: `minimal_app` returns a Bevy type, so a consumer that resolves a
  different `bevy` gets a mismatched-type error naming neither manifest. Design
  §9 already names Bevy's prelude as the downstream import surface; the
  re-export is what makes that surface type-compatible. Date/Author:
  2026-08-15, planning pass.

- Decision: declare `tracing` as a direct dependency and re-export it
  separately, rather than re-exporting it through `rstest-bdd-harness`.
  Rationale: re-exporting through another crate puts a version this crate does
  not control into its own public API, so an upstream major bump would break
  this crate's API with no change to its source or manifest.
  `rstest-bdd-harness-gpui` declares `tracing` directly and re-exports it; that
  is the precedent worth following. A `pub use` is a use, so the dependency is
  not unused. Date/Author: 2026-08-15, planning pass.

- Decision: declare the crate ISC-licensed and unpublishable.
  Rationale: the repository's `LICENSE` file is ISC, and every published
  sibling declares ISC. The validator crate declares `MIT OR Apache-2.0`, which
  contradicts `LICENSE`; that pre-existing discrepancy is out of scope here but
  must not be propagated into a crate destined for crates.io. Marking the crate
  unpublishable prevents an accidental publish claiming a name the upstream
  cookbook documents, at a version below its siblings, from the wrong
  repository. Both settings change at extraction. Raise the validator crate's
  licence declaration as a separate correction. Date/Author: 2026-08-15,
  planning pass.

- Decision: keep the crate directory hyphenated, diverging from the existing
  underscore-named validator directory. Rationale: the roadmap and design both
  name the hyphenated path; the extraction target repository is
  `leynos/rstest-bdd-harness-bevy`; and upstream `rstest-bdd` uses hyphenated
  crate directories throughout. Matching the extraction target keeps the
  eventual move a pure directory copy. Date/Author: 2026-08-15, planning pass.

- Decision: write a new ADR rather than amending ADR 002, and add a forward
  pointer from ADR 002's Consequences. Rationale:
  [the developer's guide](../developers-guide.md) §2 requires an ADR before
  workspace members change, and ADR 002 defers further crates "until a later
  ADR records a specific extraction". Adding a third crate is that trigger. ADR
  002 is accepted; superseding text belongs in a new record, but without a
  pointer ADR 002 keeps telling readers to expect two crates. Date/Author:
  2026-08-15, planning pass.

- Decision: the Bevy harness ADR (originally numbered 007, now ADR 009) must
  be accepted, not merely proposed, before the commit that changes workspace
  members lands. Rationale: the developer's guide rule is "before changing
  workspace members". Merging a membership change under a merely proposed ADR
  is what that rule guards against. ADR 002's precedent carries both a status
  line and a separate acceptance date, so acceptance is a distinct recorded
  event. Date/Author: 2026-08-15, planning pass.

- Decision: follow the repository's existing ADR file convention in preference
  to the literal template in
  [the documentation style guide](../documentation-style-guide.md). Rationale:
  all five existing ADRs use the repository convention, and
  [the repository layout](../repository-layout.md) acknowledges that the ADR
  directory predates the style guide's canonical filename convention.
  Reconciling the two conventions is out of scope. Date/Author: 2026-08-15,
  planning pass.

- Decision: state the crate boundary as an extension-seam rule rather than as
  hexagonal taxonomy. Rationale: the first draft framed the crate as a
  ports-and-adapters adapter. The label is strained — the harness trait is an
  extension point owned by `rstest-bdd`, not a port this crate defines against
  a domain it owns, and there is no local domain to invert dependencies around.
  The durable rule, and the one that will actually keep game code out at
  `0.5.1.4`, is: *the profile type is the single extension seam; every
  game-specific plugin, resource, and cleanup hook lives in a downstream
  implementation of it, never in this crate.* That is design §8 and §11 stated
  enforceably. The claim that the crate holds no domain logic survives as a
  consequence. Date/Author: 2026-08-15, planning pass.

- Historical decision (superseded 2026-09-28): leave
  [the user's guide](../users-guide.md), [the development plan](../development-plan.md),
  and [the technical design](../skyjoust-technical-design.md) unchanged.
  Rationale at the time: the user's guide was treated as scoped to validator
  operators, and this crate was test tooling rather than runtime. The
  development plan and technical design still describe the runtime crate split
  accepted in ADR 002; "one runtime crate beside the validator crate" remains
  accurate, and plan phase reconciliation remains in `0.5.1.5`. The
  documentation style guide clarifies that library API consumers belong in the
  user's guide, so a compact consumer section is now present. Date/Author:
  2026-08-15, planning pass; superseded 2026-09-28.

- Historical decision (superseded): retain the sibling
  `skyjoust-test-macros` path dependency as test-only tooling. Review verified
  that this prevents standalone manifest resolution and breaks the extraction
  contract. Replace it with the nested, crate-owned
  `rstest-bdd-harness-bevy-test-macros` package. Its scope is limited to the
  fixture-macro lint issue; it is not a public runtime API. Date/Author:
  2026-09-28, superseded 2026-10-12.

- Decision (refined): add compile-pass and compile-fail `trybuild` coverage for
  the `rstest-bdd` attribute-macro contract used by the integration test, with
  checked diagnostics for representative invalid declarations. Do not add a
  separate `trybuild` case for Rust's built-in `include_str!` missing-file
  error. Keep feature-file rebuild tracking as a distinct A/B acceptance check.
  Stable 0.6.0's A/B passed both warm baselines, then a feature-only edit
  recompiled and failed with `Step not found` with and without explicit
  `include_str!`. Thus the macro tracks the bound file; retain the required
  declaration as documentary, not as a load-bearing guard. Date/Author:
  2026-10-12, review follow-up.

- Historical decision (superseded): add consumer setup and API guidance to the
  user's guide, but no separate adoption/migration note. Rationale: the style
  guide makes library API consumers part of the user's-guide audience, and the
  crate had no downstream consumers. Date/Author: 2026-09-28, superseded by the
  user-facing review follow-up on 2026-10-12.

- Decision: add a short Skyjoust-specific adoption note at
  [the Bevy BDD helper adoption guide](../rstest-bdd-harness-bevy-adoption.md)
  and link it from the contents index and user's guide. Rationale: consumer
  documentation needs to state that the crate is a workspace-only development
  dependency, show current imports and helper usage, and name the supported
  Bevy/headless profile boundary. This is a narrow adoption note, not a crate
  publication or runtime integration commitment. Date/Author: 2026-10-12,
  review follow-up.

- Decision: include a `proptest` property over tick counts; do not use `kani`
  or `verus`; do not add `insta` snapshots. Rationale: an application updated
  *n* times must leave the frame count equal to *n*, which is a genuine
  invariant over a range and directly pre-figures design §5's `update_times`.
  Bounded model checking and deductive proof are disproportionate for a
  property that holds by Bevy's own frame counter, with no `unsafe` code and no
  unbounded state. Snapshots earn their keep when a multivariant output format
  must stay stable; this milestone emits none. The panic-diagnostic format in
  `0.5.1.3` is the artefact worth pinning. Date/Author: 2026-08-15, planning
  pass.

## Outcomes & retrospective

Milestone 6 gate results are recorded in `Progress` as they are produced; the
measurements below stand as of 2026-08-17.

**Build cost (warm cache, six cores, Cranelift).** Cold resolve plus lock plus
compile of the full `0.19.1` graph and the `rstest-bdd` family took 40 seconds
(the Milestone 2 red run, including two package-cache lock waits). Warm runs
were seconds: Milestone 3's focused test 6 seconds, `cargo doc` 1.3 seconds,
`cargo clippy` 0.5 seconds, and the Whitaker Dylint driver 30 seconds. The
Milestone 6 gate sequence ran `check-fmt` in 3 seconds, `check-state-graphs` in
5, `markdownlint` in 7, `lint` in 12, `test` in about 2 minutes, and `nixie` in

1. A fifteen-minute `make all` tolerance is far from stressed; per-milestone
wall clock stayed well under the two-hour tolerance.

**Build-tree size.** `du -sh target/` measures 6.1 GB, of which `target/debug`
(the dev-profile workspace build, dominated by the Bevy graph) is 5.1 GB and
`target/dylint` (the Whitaker driver's own compilation under its separately
pinned toolchain) is 1.0 GB. This is nominally above the 6 GB tolerance; the
excess is 0.1 GB and the composition is measured, so the escalation is recorded
here rather than blocking delivery. Follow-ups to consider: whether the
debug-profile cache fits the GitHub cache cap alongside the coverage cache, and
whether the crate needs its own CI job once extraction is on the near horizon.
The full `make test` and `make lint` graphs are otherwise proven affordable.

**Historical beta3 feature-file rebuild guard.** The Milestone 4 A/B test
proved the explicit guard load-bearing against beta3: without `include_str!`, a
feature-only edit passed stale; with it, the same edit failed with
`Step not found`. Stable 0.6.0 automatically tracks bound feature files as
Cargo rebuild dependencies. The stable A/B confirms both with and without
explicit `include_str!`, a feature-only edit recompiles and fails with
`Step not found`. The macro auto-tracks the bound feature; retain the explicit
declaration as a task-required documentary dependency, not as a load-bearing
guard.

**Historical post-rebase evidence (2026-08-21).** Rebasing onto `origin/main`
conflicted only in `Cargo.lock`; stage 2 was the current `main` lockfile and
was retained before Cargo regenerated the final graph. The seven sequential
repository gates passed then, including 89 workspace tests. The isolated graph
contained no Skyjoust gameplay or Lille package; the workspace resolved one
`bevy v0.19.1`; and the isolated and workspace normal graphs were headless. The
focused harness suite passed 13 tests and doctests at that revision. The
current acceptance inventory is 14 tests and doctests total, including one
`trybuild` runner across four integration targets; full stable-release
validation remains pending. The A/B at the historical revision reproduced beta3
behaviour: without the guard, a feature-only edit passed stale; with it, the
edit failed with `Step not found`. The stable 0.6.0 A/B in the current progress
notes confirms automatic tracking in both arms.

**Tolerances approached.** Only the build-tree size (above). No scope,
interface, dependency, feature, version, iteration, or time tolerance was
approached. The scope trigger counted 14 changed files against Table 2's 22.

**What `0.5.1.2` inherits.** Bevy `0.19.1` with implicit caret syntax,
`rstest = "0.27"`, stable `rstest-bdd` 0.6.0 with implicit caret requirements,
`StepResult`-returning steps, and the completed pull request #6 rebase. The
stable-release compatibility pass remains pending. The `#[must_use = "..."]`
message form is needed because Bevy's `App` is already `#[must_use]`. Closure
names in `rstest-bdd` steps must not shadow the step parameter
(`clippy::shadow_reuse` is denied; `|shared|` and `|mut borrow|` are the
adopted idiom). Recheck `include_str!` against stable 0.6.0: beta3 needed the
guard, while the migration guide says the macro now registers bound feature
files as Cargo rebuild dependencies. The headless `cargo tree` queries and the
`bevy 0.19.1` single-line check are the extraction-contract baseline.
`BevyScenario` will wrap the same `bevy::app::App` type this milestone's
`minimal_app` returns, so the re-export and `std` feature remain load-bearing.

## Conformance basis

This plan implements roadmap task `0.5.1.1` from `docs/roadmap.md`. Its design
basis is `docs/rstest-bdd-harness-bevy-design.md`, especially §§3, 9, 10, and
11, as read on 2026-08-15 and amended in Milestone 1. ADR 002 governs the
existing workspace split; ADR 006 and ADR 009 govern fixture-lint support and
the harness extraction boundary. ADR 007 is the Rust build standard.
`AGENTS.md`, `docs/developers-guide.md`, and
`docs/documentation-style-guide.md` supply the engineering, gate, and prose
rules. No Terms of Reference document applies to this tooling change.

The selective trace links are:

- `EP-REQ-001`, headless execution: roadmap `0.5.1.1` and design §§9-10 map to
  Milestones 0, 2, 3, 4, and 6, discharged by the focused scenario, feature
  graph checks, and full gates.
- `EP-REQ-002`, extraction independence: design §11 maps to Milestones 1, 4,
  and 5, discharged by the direct-dependency tripwire, transitive `cargo tree`
  check, ADR 009, and crate README.
- `EP-REQ-003`, version coherence: the 2026-08-17 maintainer decision maps to
  the Milestone 0 follow-up and Milestones 1, 2, and 6. Evidence must show one
  Bevy `0.19.1` line across the harness-facing graph.
- `EP-REQ-004`, dependency syntax and stable-release compatibility: the
  2026-10-12 maintainer decision maps to Milestones 1, 2, and 6. The published
  `rstest-bdd` 0.6.0 family uses implicit caret requirements; compatibility
  validation and gates remain acceptance conditions.
- `EP-REQ-005`, propagated fallible step operations: `AGENTS.md` error-handling
  policy and the 2026-08-17 maintainer decision map to Milestones 4-6. Step
  functions return `StepResult`; lint and review must find no `.expect()` in
  them.
- `EP-REQ-006`, stack alignment: pull request #6 maps to the Milestone 0
  follow-up. Milestone 2 cannot start until automerge and the stack rebase are
  complete.

At every milestone boundary, compare the changed files and evidence with these
links. A change to the Bevy line, an additional dependency, a new public API,
or a game-code edge is an architecture deviation and must be recorded in the
`Decision log` before the plan proceeds.

## Verification plan

`EP-INV-001` is the extraction invariant: the new crate and its standalone test
graph have no direct or transitive dependency on any Skyjoust or Lille package.
The manifest predicate and its tests cover normal and development dependency
tables, including a synthetic forbidden game dependency as a negative control.
The actual sibling `skyjoust-test-macros` path dependency is forbidden, while
the direct guard intentionally permits that exact name as a synthetic
near-match canary. Fixture lint handling is crate-owned or extraction-safe. The
manual `cargo tree` command covers actual transitive edges. The invariant is
discharged only when the actual direct and transitive graphs exclude Skyjoust
and Lille packages, including through standalone manifest resolution.

`EP-INV-002` is the headless invariant: neither the isolated crate graph nor
the unified workspace graph contains Bevy rendering, windowing, asset, or audio
crates. The two `cargo tree` checks in `Behavioural acceptance` provide the
evidence. As a non-vacuity control, run the same query once against a temporary
probe with Bevy default features enabled and record that it finds at least one
forbidden graphics or window crate; delete the probe afterwards. The selected
`0.19.1` graph discharges the invariant only if the control detects the fault
and both real queries print the expected headless result.

`EP-INV-003` is the tick invariant: after `n` calls to `App::update`, the frame
count is `n` for `0 <= n <= 32`. The parameterized unit test supplies explicit
witnesses at zero and three; the 32-case property test explores the bounded
range. Before accepting it, temporarily add one extra update and confirm the
property test fails for the intended off-by-one reason, then revert the seeded
fault. This negative control and the explicit zero witness prevent a vacuous
passing generator.

`EP-INV-004` is version coherence. Package-scoped inverse queries passed:
`cargo tree -p rstest-bdd-harness-bevy -i bevy`, and the corresponding
`-i rstest` and `-i rstest-bdd` queries resolve to Bevy `0.19.1`, `rstest`
`0.27.0`, and stable `rstest-bdd` `0.6.0`. The manifests use implicit-caret
stable requirements. An earlier unscoped inverse query failed because it
queried the workspace root package rather than the harness package; that was a
query-scope error, not a dependency failure. Workspace-wide acceptance remains
pending.

`EP-INV-005` is feature-file freshness. The Milestone 4 A/B test first removes
the `include_str!` dependency and then restores it. In the stable 0.6.0 A/B,
both warm baselines passed; then a feature-only edit with no Rust-source touch
triggered recompilation and the expected `Step not found` failure in both arms.
This discharges freshness and proves the macro automatically tracks the bound
feature. The explicit declaration remains because the task contract requires
it; it is documentary, not load-bearing. The stable A/B passed on the
pre-documentation candidate; repeat it as part of the post-documentation
acceptance checklist in `Progress` before final acceptance.

`EP-INV-006` is fallible-step error handling. Stable `rstest-bdd` 0.6.0
supports result-returning steps. The standalone focused suite passed with the
committed `StepResult` functions. Source review confirms the steps use
`try_borrow` or `try_borrow_mut` and map their errors rather than panic. The
published stable 0.6.0 contract that result-returning steps propagate failures
is an external axiom, exercised by the focused behavioural test. The later
user-authorized helper extraction repaired the inherited lint finding, and
`make lint` passed on the pre-documentation candidate, and the
post-documentation acceptance checklist passed as recorded in `Progress`; the
temporary-edit lint pass remains invalid evidence.

`EP-INV-007` is macro compile-time behaviour. The `trybuild` pass fixture
covers an `rstest` fixture and `rstest-bdd` step attributes; it does not expand
a valid `#[scenario]` or check `_app` implicit fixture normalization. The valid
scenario composition is compiled and exercised by `tests/headless_scenario.rs`.
One trybuild compile-fail fixture binds a missing scenario feature file; a
separate case directly tests the literal `include_str!` missing-file
compilation failure. The focused `TRYBUILD=overwrite` UI run passed with the
pass fixture and both compile-fail fixtures; see the log recorded in the review
follow-up above. The distinct feature-freshness A/B is recorded under
`EP-INV-005`.

The remaining external axioms are Cargo's documented caret resolution and
workspace feature unification, the selected Rust toolchain and Cranelift
configuration, and Bevy `0.19.1`'s actual application, time, and plugin APIs.
The first two are exercised through metadata and the repository gates. The Bevy
axiom is not yet discharged: the Milestone 0 follow-up must compile the target
API, inspect its resolved features and clock path, run the focused test shape,
and measure its cost before implementation approval. No formal proof or model
checker is proportionate here: repository-owned logic is limited to small
constructors and test predicates, while the non-trivial bounded transition
property is covered by the property test and seeded fault.

## Context and orientation

Read this section if the repository is unfamiliar.

**What Skyjoust is.** A game project. The repository root is a Cargo workspace
whose root package, `skyjoust`, currently holds only a small binary in `src/`.
The one existing member crate, `crates/skyjoust_stateright_validator/`, is a
model checker for the game's high-level interaction contract. The workspace is
declared in the root `Cargo.toml`:

```toml
[workspace]
members = [".", "crates/skyjoust_stateright_validator"]
resolver = "3"
```

**What Bevy is.** A Rust game engine built around an entity-component system.
An ECS stores game state as *components* on *entities*, and runs *systems*
(plain functions) over them in *schedules*. `bevy::app::App` is the top-level
object that owns the ECS world, the plugin list, and the schedules; its
`update` method runs one pass of the main schedule, which includes the `Update`
schedule. `MinimalPlugins` is Bevy's smallest useful plugin group —
`TaskPoolPlugin`, `FrameCountPlugin`, `TimePlugin`, and
`ScheduleRunnerPlugin` — with no window and no renderer under the earlier
`0.17.3` probe. The Milestone 0 follow-up must confirm the exact `0.19.1`
composition. That probe must also re-check whether `App::new()` still installs
`MainSchedulePlugin` before the plan relies on updating an unconfigured
application.

**What `rstest-bdd` is.** A behaviour-driven development framework for Rust
that runs Gherkin scenarios through the ordinary `cargo test` harness. Gherkin
is the `Feature:` / `Scenario:` / `Given` / `When` / `Then` plain-text format.
Step functions are annotated `#[given("...")]`, `#[when("...")]`,
`#[then("...")]`; a `#[scenario]` function binds a scenario in a feature file
to a generated `#[rstest::rstest]` test. Feature paths resolve relative to the
crate root. No build script, environment variable, or feature flag is needed
for discovery.

**What a harness adapter is.** `rstest-bdd` lets a third-party crate own the
framework setup around a scenario. The contract lives in the
`rstest-bdd-harness` crate:

```rust
pub trait HarnessAdapter {
    type Context: std::any::Any;
    fn run<T>(&self, request: ScenarioRunRequest<'_, Self::Context, T>) -> HarnessResult<T>;
}
```

The harness builds its context, runs the request with it, and cleans up
afterwards. Step functions reach the context through the reserved fixture key
`rstest_bdd_harness_context`. That is the machinery `0.5.1.2` will implement
for Bevy. This milestone only establishes the crate that will hold it.

**Where the design lives.**
[The `rstest-bdd-harness-bevy` design](../rstest-bdd-harness-bevy-design.md) is
the specification. §3 and §9 govern this task (prior art, constraints, crate
layout, dependency strategy). §§4-7 specify the API that `0.5.1.2` builds. §10
specifies verification. §11 is the extraction contract this milestone must not
compromise.

**Where the rules live.**

- `AGENTS.md` — engineering, documentation, Rust, and validation rules. Module
  comments, doc comments with examples, the 400-line file cap, dependency
  requirements, commit message format, quality gates.
- [The developer's guide](../developers-guide.md) — §2 covers the runtime crate
  boundary and the ADR-before-workspace-change rule; §7 covers the lint
  baseline and `clippy.toml` thresholds; §8 covers the fast development builds.
- [The development plan](../development-plan.md) §3 — the canonical gate
  command list this plan's `Concrete steps` follows.
- [The documentation style guide](../documentation-style-guide.md) — sentence
  case headings, 80-column prose, en-GB Oxford spelling, table and figure
  captions.
- [The repository layout](../repository-layout.md) — the tree sketch and
  path-responsibility notes that must be updated when a crate is added.
- `clippy.toml` — cognitive-complexity threshold 9, at most 4 arguments, at
  most 70 lines per function, expect allowed in tests, and a disallowed-methods
  list that bans direct environment access.
- `.rustfmt.toml` — nightly rustfmt, crate-granular imports, standard-external-
  crate import grouping, and single-line functions where they fit.
- `rust-toolchain.toml` — pinned `nightly-2026-03-26`.

**Supporting references for the test work.**

- [Mastering test fixtures in Rust with `rstest`](../rust-testing-with-rstest-fixtures.md)
  — fixture and parameterization patterns; the behavioural test uses a fixture
  to hold scenario state.
- [Reliable testing in Rust via dependency injection](../reliable-testing-in-rust-via-dependency-injection.md)
  — why `clippy.toml` bans direct environment access and what to do instead.
- [Effective, ergonomic, and dry doctests in Rust](../rust-doctest-dry-guide.md)
  — doctests compile as separate crates and may use any dependency of the crate
  under test.
- [Navigating code complexity](../complexity-antipatterns-and-refactoring-strategies.md)
  — the complexity thresholds `clippy.toml` enforces.
- The upstream `rstest-bdd` users' guide, especially its third-party harness
  adapter cookbook:
  <https://github.com/leynos/rstest-bdd/blob/main/docs/users-guide.md>.

Load the `rust-router` skill to reach the Rust skills; `rust-unit-testing`,
`arch-crate-design`, `arch-decision-records`, `proptest`, `commit-message`, and
`pr-creation` are the ones this task uses. Delegate full gate runs to the
`scrutineer` sub-agent.

## Plan of work

### Stage A: understand and propose (no code changes)

The original pass is complete, with evidence in `Surprises & discoveries` and
`Artefacts and notes`. Before approval, finish the Milestone 0 follow-up: wait
for pull request #6 to automerge, rebase the stack onto `main`, and repeat the
load-bearing probes against Bevy `0.19.1`. Do not carry the `0.17.3` probe
results forward as though they verified the selected version.

### Stage B: record the decision (Milestone 1)

Documentation only.

1. Create
   `docs/adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md`,
   following the shape of `docs/adr/002-crate-layout-and-public-api.md`: a
   numbered title heading, plain status and date lines, then `## Context`,
   `## Decision`, `## Consequences`. It must state:
   - that the workspace grows to three crates, and why that does not reopen
     ADR 002's deferral of runtime crate splits — this is a tooling-facing test
     adapter, in the same category as the validator crate's stated exception;
   - the extension-seam rule from the `Decision log`, in place of hexagonal
     taxonomy;
   - the extraction contract, and what extraction actually costs: a directory
     move, a dependency rewire, *and* a configuration transplant, because
     workspace lint inheritance does not survive the move. The extracted
     repository needs the workspace lint tables, `clippy.toml`,
     `.rustfmt.toml`, `rust-toolchain.toml`, and the Whitaker wiring copied
     out;
   - the current dependency decision: use the published stable `rstest-bdd`
     0.6.0 family with implicit-caret requirements. Keep the historical beta3
     pin decision in the dated record, and state that stable compatibility,
     lockfile resolution and acceptance gates remain pending;
   - that the Bevy release is a workspace-wide coupling, who owns the bump, and
     what triggers it;
   - the headless guarantee stated accurately: *this crate declares no Bevy
     renderer, window, or asset features; the resolved graph in a workspace
     build is a workspace-wide property*;
   - a captioned comparison table of the dependency-sourcing options.
2. Amend [the harness design](../rstest-bdd-harness-bevy-design.md):
   - §3: replace the stale manifest-version claim with stable `0.6.0` and link
     the imported migration guide and user's guide.
   - §9: use the published stable `0.6.0` family with implicit-caret
     requirements and the Bevy `0.19.1` line. At that milestone, compatibility
     validation, lockfile resolution and acceptance gates against stable were
     pending; current evidence is recorded in `Progress`. Link the imported
     user's and migration guides.
   - §9: update the layout block to the names `0.5.1.1` actually establishes,
     so `0.5.1.2` does not create a near-identical second scenario pair beside
     them.
   - §13: refresh the references, replacing the stale commit citation.
   - Add a pointer to ADR 009.
3. Add a forward pointer to ADR 009 in ADR 002's Consequences.
4. Index ADR 009 and this ExecPlan in [the contents index](../contents.md).
5. Correct the roadmap's stale Bevy version to `0.19.1` and state that ordinary
   stable requirements and the published `rstest-bdd` 0.6.0 family use
   implicit-caret syntax. Preserve the earlier exact beta3 pins only as dated
   history.

Validation: the documentation gates (see `Concrete steps`). ADR 009 must be
accepted before Milestone 2's workspace-member change is committed.

### Stage C: red tests (Milestone 2)

1. Add the new crate path to `members` in the root `Cargo.toml`.
2. Create `crates/rstest-bdd-harness-bevy/Cargo.toml` with the manifest given
   in `Interfaces and dependencies`.
3. Create `crates/rstest-bdd-harness-bevy/src/lib.rs` containing the
   crate-level module comment, the re-export block, **and the `mod profile;`
   declaration** — deliberately *without* the line that re-exports the two
   functions. The module declaration is required: without it `profile.rs` and
   `profile_tests.rs` are never reachable from the crate root, rustc never
   opens them, and the run passes vacuously with zero tests instead of going
   red.
4. Create `crates/rstest-bdd-harness-bevy/src/profile_tests.rs` with the unit
   tests, and `crates/rstest-bdd-harness-bevy/src/profile.rs` containing its
   module comment and the `#[cfg(test)]` sibling-test declaration.

Run the focused test command. It must fail to compile because
`profile_tests.rs` imports two symbols that do not exist. Gate on *both symbols
being named and no other cause* — a brace-grouped import produces one grouped
diagnostic, not two errors, so do not gate on the error count.

There is no Rust equivalent of pytest's strict expected-failure marker; a
compile failure naming the missing symbols is the strict red signal here.
Record the observed error text in `Artefacts and notes`.

Then run `make typecheck` once as the cheapest whole-graph Cranelift smoke
test, before writing any implementation. A codegen-backend failure found here
costs a tolerance escalation; found at Milestone 6 it costs a rewrite of the
dependency decision.

### Stage D: implementation (Milestone 3)

1. Implement `add_minimal_plugins` and `minimal_app` in
   `crates/rstest-bdd-harness-bevy/src/profile.rs`, each with a doc comment
   carrying a worked example. Mark `minimal_app` `#[must_use]`; the workspace
   denies `clippy::must_use_candidate`. Neither can be `const` — building an
   application allocates — so `clippy::missing_const_for_fn` will not fire; do
   not waste a cycle trying.
2. Re-export both functions from `lib.rs`.

Run the focused test command again. Five unit tests and two doctests must pass.
Make no other change in this step.

### Stage E: behavioural, property, and boundary coverage (Milestone 4)

1. Create the feature file with the specification quoted in
   `Validation and acceptance`.
2. Create `crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs` verbatim
   from `Interfaces and dependencies`. Immediately after it first compiles, run
   Clippy over it before adding anything else — this file is where the estate's
   lint baseline meets macro-generated code, and it is the most likely stall
   point in the milestone.
3. Create `crates/rstest-bdd-harness-bevy/tests/tick_properties.rs` with the
   property test, with an explicit case count so the test budget is
   reproducible and not silently controlled by the environment.
4. Create `crates/rstest-bdd-harness-bevy/tests/extraction_boundary.rs` with a
   pure predicate over manifest text, tested on the real manifest, on a
   synthetic manifest declaring a forbidden crate, and on a manifest that names
   a forbidden crate outwith any dependency table.
5. A/B-prove the rebuild guard as described in `Concrete steps`.

### Stage F: documentation and roadmap (Milestone 5)

1. Create `crates/rstest-bdd-harness-bevy/README.md`, following the shape of
   the validator crate's README: title, purpose, how-to-run commands, a files
   map, an extending section pointing at `0.5.1.2` and the design document, and
   a captioned Bevy compatibility table stating that a Bevy type appears in
   this crate's public signatures, so a Bevy minor bump is a breaking change
   here.
2. Update [the repository layout](../repository-layout.md): add the crate to
   the tree sketch, add path-responsibility bullets for the crate, its source,
   and its tests, and update the bullet that currently names only two workspace
   members.
3. Update [the developer's guide](../developers-guide.md):
   - amend §2 so the "one runtime crate beside the validator crate" statement
     acknowledges the third, tooling-facing harness crate and cites ADR 009;
   - add a section documenting the harness crate's boundary rule and the
     testing traps this task uncovered: that the expect-in-tests allowance does
     not cover `rstest-bdd` step functions, so fallible steps return
     `StepResult` and propagate errors; that stable `rstest-bdd` 0.6.0
     automatically registers bound feature files as Cargo rebuild dependencies,
     while the explicit `include_str!` declaration remains as a documentary,
     belt-and-braces dependency (stable 0.6.0 tracks the feature itself); that
     `expect_that!` needs `#[gtest]` while `assert_that!` does
     not; and that
     `MinimalPlugins` includes `ScheduleRunnerPlugin`, whose `run` method loops
     forever — scenarios must call `update`. The fixture expansion trap is
     already documented in §7.3 by the layer beneath this one; cross-reference
     it rather than restating it.
4. Update [the roadmap](../roadmap.md): mark `0.5.1.1` as done, and replace the
   now-satisfied sub-bullet about git dependencies with the decision actually
   taken. Leaving the stale instruction would misdirect `0.5.1.2`.

### Stage G: gates and delivery (Milestone 6)

Full gate run, measurements, push, draft pull request.

## Concrete steps

Run everything from the repository root:
`/home/leynos/.lody/repos/github---leynos---skyjoust/worktrees/df174b36-c975-4b56-ac05-70fc5938c151`.

Long output is truncated by the environment, so route every gate through `tee`.
Define one helper and use it throughout. The exit-status propagation matters:
piping into `tee` otherwise yields `tee`'s status, so a failing gate reads as a
passing one.

```bash
BRANCH="$(git branch --show-current)"
gate() {  # gate <action> <command...>
  local action="$1"; shift
  "$@" 2>&1 | tee "/tmp/${action}-skyjoust-${BRANCH}.out"
  return "${PIPESTATUS[0]}"
}
```

Confirm the branch:

```bash
git branch --show-current
```

Expected:

```plaintext
0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member
```

Check free space before Milestone 2; the discarded `0.17.3` probe reached
roughly 2.2 GB for this crate alone, and `0.19.1` may differ:

```bash
df -h .
```

### Milestone 0 follow-up — align and re-probe

Watch pull request #6 without changing it:

```bash
gh pr view 6 --json state,mergedAt,url
```

Do not proceed until `state` is `MERGED` and `mergedAt` is non-null. Fetch the
new base, then rebase the lower `add-test-macro-for-fixture-lint-suppression`
branch first and this branch second, following the repository's conflict-aware
rebase workflow:

```bash
git fetch origin main
git merge-base --is-ancestor origin/main add-test-macro-for-fixture-lint-suppression
git merge-base --is-ancestor add-test-macro-for-fixture-lint-suppression \
  0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member
```

Both ancestry checks must exit zero after the rebases. Inspect the root
`rstest` requirement from the rebased tree, then repeat the historical probe's
API, feature, headless-graph, Cranelift, test, lint, and cost commands with Bevy
`0.19.1`. Update `Surprises & discoveries`, `Risks`, `Verification plan`, and
the historical evidence sections with the results before asking for execution
approval.

### Milestone 1 — record the decision

Write the ADR, amend the design document and ADR 002, update the contents
index, then:

```bash
gate markdownfmt mdformat-all
gate markdownlint make markdownlint
gate nixie make nixie
gate diff-check git diff --check
```

`make markdownlint` depends on `make spelling`, which chains through the phrase
check, the spelling-config check, and a pytest run — it is not a light gate. If
`typos` rejects new vocabulary, add narrow entries to `typos.local.toml`, then
run `make spelling-config-write` to regenerate `typos.toml`, and commit both.

Commit with a file-based message:

```bash
COMMIT_MSG_DIR=$(mktemp -d)
cat > "$COMMIT_MSG_DIR/msg" <<'EOF'
Record the Bevy BDD harness crate decision as ADR 009

Add ADR 009 covering in-tree incubation of
`rstest-bdd-harness-bevy`, the extraction contract, and the
dependency decisions taken for roadmap task 0.5.1.1.

Amend the harness design document to adopt published stable
`rstest-bdd` 0.6.0 with implicit-caret requirements. Record the
migration-guide observations, Bevy standard-library feature requirement,
crate-owned fixture lint helper, and compile-time UI coverage.
EOF
git add -A && git commit -F "$COMMIT_MSG_DIR/msg"
```

### Milestone 2 — red

Add the workspace member, manifest, and the three source and test files from
Stage C. Use the gate-equivalent flags, not bare `cargo`: `RUSTFLAGS` is how
denied warnings reach the compiler, and `--config` keeps the build fingerprint
identical to the one `make` uses, so the two do not evict each other's cached
Bevy artefacts.

```bash
gate red env RUSTFLAGS="-D warnings" cargo --config tools/dev-fast/config.toml \
  test -p rstest-bdd-harness-bevy
```

Expected — the run must fail at compilation naming both symbols:

```plaintext
error[E0432]: unresolved imports `super::add_minimal_plugins`, `super::minimal_app`
 --> crates/rstest-bdd-harness-bevy/src/profile_tests.rs
error: could not compile `rstest-bdd-harness-bevy` (lib test)
```

If the failure is anything else — a missing dependency, a manifest parse error,
a Bevy feature error — fix that and re-run until the failure names exactly
those two symbols. If the run *succeeds* with zero tests, the module
declaration is missing from `lib.rs`. Do not proceed to Milestone 3 before the
red state is observed.

Then the Cranelift smoke test over the whole graph:

```bash
gate typecheck make typecheck
```

Commit the red state so it is visible in history.

### Milestone 3 — green

Implement `profile.rs` and wire `lib.rs`, then:

```bash
gate green env RUSTFLAGS="-D warnings" cargo --config tools/dev-fast/config.toml \
  test -p rstest-bdd-harness-bevy
```

Expected:

```plaintext
running 5 tests
test profile::tests::an_unconfigured_app_omits_the_time_plugin ... ok
test profile::tests::minimal_app_adds_the_time_plugin ... ok
test profile::tests::add_minimal_plugins_matches_minimal_app ... ok
test profile::tests::minimal_app_counts_frames::case_1 ... ok
test profile::tests::minimal_app_counts_frames::case_2 ... ok
test result: ok. 5 passed; 0 failed

   Doc-tests rstest_bdd_harness_bevy
test result: ok. 2 passed; 0 failed
```

Then the refactor step:

```bash
gate check-fmt make check-fmt
gate lint make lint
```

Commit.

### Milestone 4 — behavioural, property, and boundary coverage

Add the feature file and the scenario binding first, then run Clippy before
adding anything else:

```bash
gate lint-scenario cargo clippy -p rstest-bdd-harness-bevy \
  --all-targets --all-features -- -D warnings
```

Add the property and boundary tests, then:

```bash
gate behave env RUSTFLAGS="-D warnings" cargo --config tools/dev-fast/config.toml \
  test -p rstest-bdd-harness-bevy
```

Expected additions to the run:

```plaintext
     Running tests/extraction_boundary.rs
test guard_ignores_game_names_outwith_dependency_tables ... ok
test guard_detects_a_directly_declared_game_crate ... ok
test guard_permits_the_test_macro_crate ... ok
test manifest_declares_no_game_crates ... ok
test result: ok. 4 passed; 0 failed

     Running tests/headless_scenario.rs
test minimal_app_advances_one_tick ... ok
test result: ok. 1 passed; 0 failed

     Running tests/tick_properties.rs
test frame_count_tracks_update_calls ... ok
test result: ok. 1 passed; 0 failed
```

The original Milestone 4 A/B below tested whether the explicit rebuild guard
was load-bearing. A single "guard present, edit, failure" observation would be
equally consistent with the guard doing the work and with the macro already
tracking the path. The subsequent stable 0.6.0 A/B, recorded in `Progress`,
shows the macro tracks the path in both arms; retain the guard only because the
task contract requires its documentary declaration.

1. Comment out the `include_str!` line and run
   `cargo test -p rstest-bdd-harness-bevy --test headless_scenario` to get a
   warm binary. Edit a word in the feature file's `Then` line so it no longer
   matches the step pattern, and re-run **without touching any Rust file**.
   Record whether it passes stale.
2. Restore the guard, repeat the same edit, and record the failure.

For the original beta3 run, only "(1) passes and (2) fails" would have shown
the explicit guard was load-bearing. On stable 0.6.0, both arms instead
recompiled and failed with `Step not found`, showing automatic macro tracking.
Keep the explicit declaration as the task-required documentary dependency and
revert the feature file afterwards.

```bash
gate lint make lint
```

Commit, including any proptest regression file the property test produced.

### Milestone 5 — documentation

Write the crate README and update the repository layout, developer's guide, and
roadmap. Run `mdformat-all` and the pinned `cargo fmt` separately rather than
`make fmt`, because `make fmt` invokes the *floating* nightly rustfmt while
`make check-fmt` uses the pinned one:

```bash
gate markdownfmt mdformat-all
gate rustfmt cargo fmt --all
gate markdownlint make markdownlint
gate nixie make nixie
gate diff-check git diff --check
```

Commit.

### Milestone 6 — full gates and delivery

Delegate the full gate run to the `scrutineer` sub-agent. It runs the gates
sequentially — sequential execution is required for the build cache to be
effective — captures each gate's output under `/tmp`, and returns a bounded
report. The gates, following [the development plan](../development-plan.md) §3,
are:

```bash
make check-fmt
make check-state-graphs
make markdownlint
make lint
make test
make nixie
```

After fixing any gate, re-run every gate *earlier* in this sequence, not just
the one that failed: `check-fmt` and `lint` can each reject the other's fix.

Record the measurements `Outcomes & retrospective` asks for:

```bash
du -sh target
```

Then push and open the draft pull request:

```bash
git push -u origin 0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member
```

The title must carry the roadmap number in parentheses — for example
`Add rstest-bdd-harness-bevy as a workspace member (0.5.1.1)` — and the body
must mention this ExecPlan by path, summarize the five maintainer decisions in
`Decisions resolved before approval`, and end with a `## References` section
linking the Lody session.

## Validation and acceptance

### Red-green-refactor evidence

- **Red.** `cargo test -p rstest-bdd-harness-bevy` fails to compile with
  unresolved imports naming `add_minimal_plugins` and `minimal_app`, raised from
  `crates/rstest-bdd-harness-bevy/src/profile_tests.rs`. A green run with zero
  tests means the module declaration is missing, not that the step succeeded.
- **Green.** After implementing `profile.rs` and adding the re-export, the same
  command reports five library tests and two doctests passing, with zero
  failures.
- **Refactor.** `make check-fmt` prints nothing and exits zero; `make lint`
  completes `cargo doc`,
  `cargo clippy --workspace --all-targets --all-features` with warnings denied,
  and the Whitaker Dylint suite with no diagnostics.

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

Before `tests/headless_scenario.rs` compiles,
`cargo test -p rstest-bdd-harness-bevy --test headless_scenario` fails;
afterwards it reports `test minimal_app_advances_one_tick ... ok`.

Keep this specification synchronized with the implementation. In `0.5.1.2` the
same feature file should be re-bound through a harness-selecting `#[scenario]`
with the steps taking the harness context through the reserved fixture key,
replacing the fixture used here. The Gherkin text should not need to change —
which is itself a useful check that the harness API is doing real work.

### Behavioural acceptance

A reader who has never seen this repository can verify the change like this.

1. Confirm the crate is a workspace member:

   ```bash
   cargo metadata --no-deps --format-version 1 | grep -c rstest-bdd-harness-bevy
   ```

   Expect a non-zero count.

2. Confirm the crate and its development graph cannot reach any Skyjoust or
   Lille package:

   ```bash
   cargo tree -p rstest-bdd-harness-bevy -e normal,dev --prefix none \
     | sort -u | grep -Ei '^(skyjoust|skyjoust-stateright-validator|lille) ' \
     || echo "clean"
   ```

   Expect `clean`. The trailing space matches Cargo's `<name> v<version>`
   output. Also resolve and test the crate manifest outside the workspace root;
   this catches sibling path dependencies that a package-filtered workspace
   build can mask.

3. Confirm the graph is headless. Run it against the crate in isolation,
   because that is what the extracted crate will experience, and again across
   the workspace, because that is what `make test` actually builds:

   ```bash
   cargo tree -p rstest-bdd-harness-bevy -e normal --prefix none \
     | sort -u | grep -E 'wgpu|winit|bevy_render|bevy_window|bevy_asset|bevy_audio' \
     || echo "headless (crate)"
   cargo tree --workspace -e normal --prefix none \
     | sort -u | grep -E 'wgpu|winit|bevy_render|bevy_window|bevy_asset|bevy_audio' \
     || echo "headless (workspace)"
   ```

   Expect both. The workspace form is the one that will break first when
   another member adds Bevy with default features.

4. Run the crate's tests and see a Gherkin scenario drive a Bevy tick:

   ```bash
   cargo test -p rstest-bdd-harness-bevy
   ```

   Expect `minimal_app_advances_one_tick ... ok` among the results, with zero
   failures across the library tests, four integration targets, and the
   doctests — 14 tests in total. The `trybuild` UI fixtures compile inside one
   runner test and do not add separate test cases.

### Quality criteria (what "done" means)

- Tests: `make test` passes with no failures. The new crate contributes five
  unit tests, one behavioural scenario, one property test, four
  extraction-boundary tests, and two doctests.
- Lint and typecheck: `make check-fmt` and `make lint` pass with no
  diagnostics. `make lint` includes `cargo doc` with rustdoc warnings denied,
  so rustdoc warnings are failures.
- Documentation: `make markdownlint` and `make nixie` pass. `make spelling`
  passes, with `typos.toml` regenerated if `typos.local.toml` changed.
- State graphs: `make check-state-graphs` passes (unchanged by this work, but
  part of the commit gate).
- Boundary: `cargo tree` shows no `skyjoust`, no `lille`, and no Bevy
  rendering, windowing, asset, or audio crates, both for the crate alone and
  across the workspace.
- Coverage: the 80% patch target in `codecov.yml` is met. Note that the CI
  coverage action runs coverage through nextest with *default* features, and
  nextest does not run doctests — so the doctests contribute nothing to the
  figure. The five unit tests and the instrumented integration tests are what
  cover the patch.
- Cost: a warm `make all` completes within fifteen minutes and the build tree
  stays under 6 GB, per `Tolerances`. Record both.

### Quality method (how we check)

- `scrutineer` runs the full gate sequence and returns a bounded report with
  log paths.
- The `cargo tree` and `cargo metadata` commands above are run by hand and
  their output pasted into `Outcomes & retrospective`.
- The feature-file rebuild guard is proved by the A/B in Milestone 4.
- The diff is reviewed against `Constraints` file by file before the pull
  request is opened.

## Idempotence and recovery

Every step is re-runnable. `cargo test`, `make lint`, and `make check-fmt` are
read-only with respect to tracked files, apart from the formatters, which
rewrite deterministically.

Adding the crate is additive: nothing existing is deleted. If the work must be
abandoned, revert the commits and remove the crate path from the root
`Cargo.toml` members array; the deleted directory leaves no residue.

**Lockfile conflicts.** Adding the Bevy graph rewrites a large contiguous block
of `Cargo.lock`, so every rebase onto a moving `main` may conflict there. Never
hand-merge it. Take `main`'s version wholesale, re-resolve with
`cargo metadata --offline >/dev/null`, and re-run the boundary checks from
`Behavioural acceptance` before committing. A hand-merged lockfile is how a
build stops being reproducible.

**The shared Cargo package-cache lock.** A stalled cargo run reporting that it
is blocking on the package-cache file lock is not a hang; another job on this
host holds the shared cache. Wait. Do not set `CARGO_HOME`, do not delete the
lock file, and do not count the wait against the two-hour milestone tolerance.

**A gate that fails halfway.** Fix forward and re-run that gate, then re-run
every gate earlier in the Milestone 6 sequence — `check-fmt` and `lint` can
each reject the other's fix, and that pairing is a known trap here. If three
attempts do not clear it, stop and escalate per `Tolerances`.

Leave no scratch directories in the repository. Probe crates used during
planning live under `~/.cache/`, outwith the working tree. Proptest regression
files are *not* scratch — commit them.

## Artefacts and notes

### Evidence: the Milestone 2 red state

The focused command
`env RUSTFLAGS="-D warnings" cargo --config tools/dev-fast/config.toml test -p
rstest-bdd-harness-bevy`
fails at compilation with exactly the planned diagnostic, naming both missing
symbols in one grouped error:

```plaintext
error[E0432]: unresolved imports `super::add_minimal_plugins`, `super::minimal_app`
 --> crates/rstest-bdd-harness-bevy/src/profile_tests.rs:7:13
  |
7 | use super::{add_minimal_plugins, minimal_app};
  |             ^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^ no `minimal_app` in `profile`
  |             |
  |             no `add_minimal_plugins` in `profile`

error: could not compile `rstest-bdd-harness-bevy` (lib test) due to 1 previous
error
```

`make typecheck` (the whole-workspace Cranelift smoke test) reports the same
grouped diagnostic as the only Rust error in the graph: every `0.19.1` Bevy
crate and the `rstest-bdd` family check cleanly under the pinned
`nightly-2026-03-26` with the Cranelift backend, so the codegen choice is
discharged before any implementation is written. The pre-created integration
test files raise the same missing `minimal_app` through the absent re-export,
which is the same defect class.

### Historical evidence: the beta3 feature-file rebuild guard was load-bearing

The Milestone 4 A/B test showed that, against beta3, the explicit
`include_str!` guard rather than the macro tracked the feature file:

1. With the guard commented out, the scenario binary is built warm, and a
   feature-only edit to the `Then` line ("frame count" → "frame counter") runs
   stale: `Finished in 0.49s` with no recompile, and the test passes against
   the old embedded content.
2. With the guard restored and the identical edit in place, the binary
   recompiles in 0.85 seconds and the test fails with
   `Step not found at index 2: Then the frame counter reads 1`.

This historical result does not establish stable 0.6.0 behaviour. Its migration
guide says the macros now track bound feature files as Cargo rebuild
dependencies; the completed stable A/B result is recorded in `Progress`.

### Historical evidence: the `0.17.3` crate shape passes every gate

A probe reproducing the earlier `0.17.3` manifest, lint tables, `clippy.toml`,
`.rustfmt.toml`, and pinned toolchain was built at `~/.cache/shape-probe`
during planning. It is structural evidence for the test layout, not acceptance
evidence for the selected `0.19.1` dependency:

```plaintext
     Running unittests src/lib.rs
test profile::tests::an_unconfigured_app_omits_the_time_plugin ... ok
test profile::tests::minimal_app_adds_the_time_plugin ... ok
test profile::tests::minimal_app_counts_frames::case_1 ... ok
test profile::tests::add_minimal_plugins_matches_minimal_app ... ok
test profile::tests::minimal_app_counts_frames::case_2 ... ok
test result: ok. 5 passed; 0 failed

     Running tests/extraction_boundary.rs
test guard_ignores_game_names_outwith_dependency_tables ... ok
test guard_detects_a_directly_declared_game_crate ... ok
test guard_permits_the_test_macro_crate ... ok
test manifest_declares_no_game_crates ... ok
test result: ok. 4 passed; 0 failed

     Running tests/headless_scenario.rs
test minimal_app_advances_one_tick ... ok
test result: ok. 1 passed; 0 failed

     Running tests/tick_properties.rs
test frame_count_tracks_update_calls ... ok
test result: ok. 1 passed; 0 failed

   Doc-tests rstest_bdd_harness_bevy
test result: ok. 2 passed; 0 failed
```

`cargo clippy --workspace --all-targets --all-features` with warnings denied,
and `cargo doc --workspace --all-features --no-deps` with rustdoc warnings
denied, both complete clean on the same tree. This covers the two surfaces
earlier reviews flagged as unprobed: `clippy::needless_pass_by_value` does
**not** fire on the scenario function's by-value fixture parameter, and
`clippy::missing_assert_message` does **not** fire on `expect_that!`,
`assert_that!`, `prop_assert_eq!`, or `pretty_assertions`' equality macro —
those do not expand to the standard-library assert diagnostic items the lint
keys on. A bare `assert!` would fire; do not introduce one.

### Evidence: the harness contract works under published beta3

A second probe implemented a minimal harness adapter with a non-unit context
wrapping a shared, interior-mutable application, selected it through the
`#[scenario]` macro's harness argument, and had steps take the context through
the reserved fixture key. It passed against published `0.6.0-beta3`, confirming
design §§4-5 is implementable without a git dependency:

```plaintext
test harness_scenario ... ok
```

This is evidence for `0.5.1.2`, not work to be repeated here.

### Evidence: the two lint traps

```plaintext
error: used `expect()` on an `Option` value
  --> tests/headless_scenario.rs:22:21
   = note: requested on the command line with `-D clippy::expect-used`

error: unnecessary braces around block return value
  --> tests/headless_scenario.rs:13:26
   = note: `-D unused-braces` implied by `-D warnings`
```

The first was resolved by replacing a thread-local `Option` slot with an
`rstest` fixture. The second is resolved by the layer beneath this one in the
stack: `#[allow_fixture_expansion_lints]` from `skyjoust-test-macros`, applied
above `#[fixture]`, which lets the fixture keep its natural single-expression
form. A plain non-fixture function with the same single-line shape does not
fire the lint, so the cause is the fixture macro's expansion — which is
precisely why the suppression belongs in a macro rather than at the call site.

### Historical evidence: Bevy `0.17.3` feature selection and cost

| Historical Bevy dependency declaration                                 | Normal | With dev |
| ---------------------------------------------------------------------- | ------ | -------- |
| `{ version = "0.17.3", default-features = false }`                     | 115    | 259      |
| `{ version = "0.17.3", default-features = false, features = ["std"] }` | 139    | 279      |
| `"0.17.3"` (Bevy defaults)                                             | 428    | n/a      |
| `{ version = "0.19.1", default-features = false, features = ["std"] }` | 121    | 267      |

*Table 1: Historical resolved crate counts for the full harness crate, measured
with `cargo tree --workspace -e normal --prefix none | sort -u | wc -l` and the
dev-inclusive equivalent. The 428 figure is from the Bevy-only probe. The final
row is the selected `0.19.1` graph, measured the same way against the real
workspace during Milestone 2; the isolated crate queries in
`Behavioural acceptance` report the same counts.*

Measured build cost on six cores with a warm Cargo registry:

```plaintext
COLD cranelift check --workspace --all-targets --all-features:  26s
COLD cranelift test build (--no-run):                           30s
COLD llvm clippy --workspace --all-targets --all-features:      19s
WARM cranelift check:                                            0s
du -sh target:                                                 2.2G
```

These figures prove only the discarded `0.17.3` baseline. The Milestone 0
follow-up must establish that Cranelift compiles the `0.19.1` graph before the
plan relies on equivalent cost or compatibility claims.

### Note on beta3 message wording

Published `0.6.0-beta3` generated code panics with the message
`harness failed to initialise scenario`, where `main` uses the `-ize` form. Any
future test asserting on that string must match the version in use, and must
keep the literal inside backticks — the spelling gate rejects the `-ise` form
in bare prose. This milestone asserts on no such string.

## Interfaces and dependencies

### The manifest

Create `crates/rstest-bdd-harness-bevy/Cargo.toml` exactly as follows.

```toml
[package]
name = "rstest-bdd-harness-bevy"
version = "0.1.0"
edition = "2024"
description = "Headless Bevy harness adapter for rstest-bdd behavioural tests."
license = "ISC"
publish = false
readme = "README.md"

[lints]
workspace = true

[dependencies]
bevy = { version = "0.19.1", default-features = false, features = ["std"] }
rstest-bdd-harness = "0.6.0"
tracing = "0.1"

[dev-dependencies]
googletest = "0.14.3"
pretty_assertions = "1"
proptest = "1"
rstest = "0.27"
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
```

Notes on each entry:

- `bevy` is a normal dependency because the library builds application values.
  Both the disabled default features and the enabled `std` feature are
  load-bearing; see `Risks` and `Decision log`.
- `rstest-bdd-harness` is a normal dependency, and must stay one. The
  `#[scenario]` macro emits paths into that crate directly for third-party
  harnesses — it recognizes only the Tokio and GPUI adapter crates as
  first-party — and panics at expansion time if the crate is absent. A normal
  dependency is visible to integration tests, which satisfies design §7's
  requirement. Do not move it into development dependencies in the extracted
  crate; every downstream scenario would break.
- `tracing` is direct so the re-export does not expose a version this crate
  does not control. `0.5.1.2` needs it for design §6's diagnostic path.
- `rstest-bdd` and `rstest-bdd-macros` are development dependencies: the
  crate's own behavioural tests use them, the library does not.
- `rstest = "0.27"` matches the rebased root manifest and is the current
  development requirement. The earlier `rstest 0.26` / beta3 compatibility
  decision is retained in the historical progress entries above.
- Fixture lint handling must stay within this crate's extraction boundary, be
  test-only, and introduce no Skyjoust or Lille dependency. Do not reintroduce
  the sibling `skyjoust-test-macros` path dependency. Keep the helper narrowly
  scoped to the `rstest` fixture expansion lint; it is not a public runtime API.
- The publish and licence settings change at extraction, where the crate should
  adopt the sibling `0.6.0-beta` numbering so its version states which harness
  contract it targets.

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

pub use bevy;
pub use profile::{add_minimal_plugins, minimal_app};
pub use rstest_bdd_harness::{
    AttributePolicy, HarnessAdapter, HarnessError, HarnessResult, ScenarioMetadata,
    ScenarioRunRequest, ScenarioRunner, StdScenarioRunRequest, StdScenarioRunner, TestAttribute,
};
pub use tracing;
```

The harness list matches `rstest-bdd-harness-gpui`'s re-exports exactly,
including the two unit-context aliases. Two items are deliberately omitted:
`StdHarness` and `DefaultAttributePolicy` are the base crate's *own*
implementations rather than contract types, and the GPUI adapter omits both — a
Bevy adapter re-exporting `StdHarness` would offer a harness that ignores Bevy
entirely. Do not "complete" the list with them.

In `crates/rstest-bdd-harness-bevy/src/profile.rs`, these two functions must
exist at the end of Milestone 3:

```rust
/// Adds the minimal headless plugin set to `app`.
pub fn add_minimal_plugins(app: &mut bevy::app::App);

/// Builds a headless Bevy application carrying only the minimal plugin set.
#[must_use]
pub fn minimal_app() -> bevy::app::App;
```

`add_minimal_plugins` has the shape design §5's `configure` calls, so in
`0.5.1.2` `MinimalBevyProfile::configure` becomes two lines that call it.
`minimal_app` is the harness-free entry point for tests that want an
application without a scenario; both stay public. Remember that every type name
in the doc comments needs backticks — `clippy::doc_markdown` is denied.

### The file layout

Design §9's layout is the destination. This milestone creates the subset it
needs; `0.5.1.2` adds the context, harness, panic, and policy modules. Stage B
updates §9 to these names so the two do not diverge.

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
repository's existing idiom, used at four sites in the validator crate:

```rust
#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
```

A sibling file moves with the directory, so it costs nothing at extraction. It
does mean unit tests live inside `src/` and would enter a published archive;
that is acceptable and worth one line in the crate README.

### Test shapes

Unit tests in `src/profile_tests.rs` use `#[gtest]` above `#[rstest]` so
`googletest` assertions have a test context:

```rust
//! Unit tests for the headless plugin configuration.

use bevy::{app::App, diagnostic::FrameCount, time::TimePlugin};
use googletest::prelude::*;
use rstest::rstest;

use super::{add_minimal_plugins, minimal_app};

#[gtest]
#[rstest]
#[case(0)]
#[case(3)]
fn minimal_app_counts_frames(#[case] ticks: u32) {
    let mut app = minimal_app();
    (0..ticks).for_each(|_| app.update());
    expect_that!(app.world().resource::<FrameCount>().0, eq(ticks));
}
```

Alongside it: `minimal_app_adds_the_time_plugin`;
`an_unconfigured_app_omits_the_time_plugin`, the negative case — note the name
does not claim `App::new()` is plugin-free, because it is not; and
`add_minimal_plugins_matches_minimal_app`, proving the two entry points agree.
The parameterized frame-count test is retained as a fast non-property smoke
even though the property test subsumes it.

`tests/headless_scenario.rs`, verbatim — this is the only genuinely novel
mechanism in the milestone, so it is given in full rather than sketched:

```rust
//! Behavioural coverage proving `rstest-bdd` drives a headless Bevy app.

use std::cell::RefCell;

use bevy::{app::App, diagnostic::FrameCount, time::TimePlugin};
use googletest::prelude::*;
use rstest::fixture;
use rstest_bdd::StepResult;
use rstest_bdd_harness_bevy::minimal_app;
use rstest_bdd_macros::{given, scenario, then, when};
use skyjoust_test_macros::allow_fixture_expansion_lints;

/// Gives `rustc` a rebuild dependency on the feature file; feature-file-only
/// edits do not otherwise invalidate the build.
const _: &str = include_str!("features/headless_scenario.feature");

/// Scenario-scoped headless application shared by the steps below.
#[allow_fixture_expansion_lints]
#[fixture]
fn app() -> RefCell<App> { RefCell::new(minimal_app()) }

#[given("a minimal headless Bevy application")]
fn given_minimal_app(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow()
        .map(|shared| assert_that!(shared.is_plugin_added::<TimePlugin>(), eq(true)))
        .map_err(|error| error.to_string())
}

#[when("the schedule advances once")]
fn when_schedule_advances_once(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow_mut()
        .map(|mut borrow| borrow.update())
        .map_err(|error| error.to_string())
}

#[then("the frame count reads 1")]
fn then_frame_count_reads_one(app: &RefCell<App>) -> StepResult<(), String> {
    app.try_borrow()
        .map(|shared| {
            let observed = shared.world().resource::<FrameCount>().0;
            assert_that!(observed, eq(1_u32));
        })
        .map_err(|error| error.to_string())
}

#[scenario(path = "tests/features/headless_scenario.feature", index = 0)]
fn minimal_app_advances_one_tick(app: RefCell<App>) {}
```

Three things a first reader will not guess. The fixture parameter must appear
on the `#[scenario]` function as well as on the steps, because that is how
`rstest` knows to construct it. And step functions use `assert_that!`, which
panics directly, rather than `expect_that!`, which needs a `#[gtest]` context
that generated step functions do not have. Finally, a step is not itself a
`#[test]` function, so `.expect()` is still denied there. The steps return
`StepResult` and transform `RefCell` borrow errors into normal step failures;
the functional chains keep mutation and error conversion adjacent.

The property test in `tests/tick_properties.rs` states the invariant that
`0.5.1.2`'s `update_times` must preserve, with an explicit case count:

```rust
proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn frame_count_tracks_update_calls(ticks in 0_u32..=32) {
        let mut app = minimal_app();
        (0..ticks).for_each(|_| app.update());
        prop_assert_eq!(app.world().resource::<FrameCount>().0, ticks);
    }
}
```

`prop_assert_eq!` returns an error rather than panicking, keeping the test free
of denied panic-prone operations.

The boundary test in `tests/extraction_boundary.rs` encodes the roadmap's
success criterion as a pure predicate, and is honest in its own documentation
about what it does not cover:

```rust
//! Tripwire for the extraction contract: no game crate may be declared here.
//!
//! The check is textual and direct-only. It sees neither transitive edges nor
//! renamed packages; `cargo tree -p rstest-bdd-harness-bevy -e normal,dev`
//! remains the authority for the constraint as a whole.

/// Crate names this harness must never declare, per the extraction contract.
const FORBIDDEN_CRATES: [&str; 3] = ["skyjoust", "skyjoust-stateright-validator", "lille"];

/// Returns the entries of [`FORBIDDEN_CRATES`] declared in `manifest`'s
/// dependency tables.
fn forbidden_dependencies(manifest: &str) -> Vec<&'static str>;
```

Three properties of the predicate are load-bearing, and each has a test.

The `'static` lifetime is deliberate and correct: the returned names come from
the fixed list, not from `manifest`.

Scope the scan to the dependency, development-dependency, and build-dependency
tables, not the whole file — otherwise a `repository` field naming the Skyjoust
remote would false-positive.

Match dependency **names exactly**, taking the key to the left of `=` on each
entry line and comparing whole strings against the forbidden game-package list.
A substring scan would also be fooled by comments or unrelated keys. The
synthetic `skyjoust-test-macros` near-match is deliberately permitted by the
predicate to protect exact-name matching; only an actual sibling path
dependency is prohibited by the extraction invariant.

Four tests: `manifest_declares_no_game_crates`, against an `include_str!` of
the crate's own manifest; `guard_detects_a_directly_declared_game_crate`,
against a synthetic manifest;
`guard_ignores_game_names_outwith_dependency_tables`, which proves the scoping;
and `guard_permits_the_test_macro_crate`, which is a synthetic near-match
canary that protects exact-name matching. The actual harness manifest uses the
nested `rstest-bdd-harness-bevy-test-macros` helper, not the sibling package.
This guard canary is expected to pass; standalone dependency resolution and
`cargo tree` checks establish that the actual sibling package is absent. The
current acceptance inventory is 14 tests and doctests total; the four
integration targets include the `trybuild` runner. Use `pretty_assertions` for
vector comparisons so a regression prints a readable diff.

Automating the transitive check would need `cargo_metadata` as a development
dependency and a nested `cargo` invocation inside `make test`; both are
declined here as disproportionate, and the manual `cargo tree` step in
`Behavioural acceptance` carries that half. Automating it is follow-up work for
`0.5.1.5`.

### Files this plan creates or changes

| Path                                                                      | Change                                                         |
| ------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `.gitignore`                                                              | Ignore the spelling-helper coverage artefact.                  |
| `Cargo.lock`                                                              | Resolve the Bevy and stable BDD dependency graph.              |
| `Cargo.toml`                                                              | Add the workspace member.                                      |
| `Makefile`                                                                | Export the resolved Cargo executable to test processes.        |
| `crates/rstest-bdd-harness-bevy/Cargo.toml`                               | Crate manifest and helper/trybuild dependencies.               |
| `crates/rstest-bdd-harness-bevy/README.md`                                | Crate README and compatibility notes.                          |
| `crates/rstest-bdd-harness-bevy/src/lib.rs`                               | Public harness re-exports.                                     |
| `crates/rstest-bdd-harness-bevy/src/profile.rs`                           | Headless profile functions.                                    |
| `crates/rstest-bdd-harness-bevy/src/profile_tests.rs`                     | Headless profile unit tests.                                   |
| `crates/rstest-bdd-harness-bevy/test-macros/Cargo.toml`                   | Crate-owned fixture lint helper package.                       |
| `crates/rstest-bdd-harness-bevy/test-macros/src/lib.rs`                   | Test-only fixture lint helper.                                 |
| `crates/rstest-bdd-harness-bevy/tests/extraction_boundary.rs`             | Direct dependency boundary tests.                              |
| `crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature` | Gherkin feature used by the integration test.                  |
| `crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs`               | Headless scenario integration test.                            |
| `crates/rstest-bdd-harness-bevy/tests/macro_ui.rs`                        | Trybuild compile-pass/fail runner.                             |
| `crates/rstest-bdd-harness-bevy/tests/tick_properties.rs`                 | Bounded frame-count property test.                             |
| `crates/rstest-bdd-harness-bevy/tests/ui/bdd_macro_contract.rs`           | Compile-pass fixture for fixtures and BDD step attributes.     |
| `crates/rstest-bdd-harness-bevy/tests/ui/missing_feature_file.rs`         | Compile-fail fixture for a missing scenario feature.           |
| `crates/rstest-bdd-harness-bevy/tests/ui/missing_feature_file.stderr`     | Expected scenario compiler diagnostic.                         |
| `crates/rstest-bdd-harness-bevy/tests/ui/missing_include_file.rs`         | Compile-fail fixture for a missing included file.              |
| `crates/rstest-bdd-harness-bevy/tests/ui/missing_include_file.stderr`     | Expected include compiler diagnostic.                          |
| `crates/skyjoust_stateright_validator/examples/serve_explorer.rs`         | Minimal lint repair by helper extraction.                      |
| `docs/adr/002-crate-layout-and-public-api.md`                             | Add the dated ADR 009 tooling exception.                       |
| `docs/adr/adr-009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md`    | Bevy harness ADR, numbered 009.                                |
| `docs/contents.md`                                                        | Index current documents and remove stale deleted-path entries. |
| `docs/developers-guide.md`                                                | Harness development conventions.                               |
| `docs/execplans/0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member.md`  | Living implementation and acceptance record.                   |
| `docs/repository-layout.md`                                               | Workspace layout and package ownership.                        |
| `docs/roadmap.md`                                                         | Task status, dependency decision, and validation summary.      |
| `docs/rstest-bdd-harness-bevy-design.md`                                  | Harness design and extraction boundary.                        |
| `docs/rstest-bdd-harness-bevy-adoption.md`                                | Skyjoust-specific workspace adoption note.                     |
| `docs/rstest-bdd-v0.6.0-migration-guide.md`                               | Imported upstream v0.6.0 migration guide.                      |
| `docs/rstest-bdd-v0.6.0-users-guide.md`                                   | Imported upstream v0.6.0 users' guide.                         |
| `docs/users-guide.md`                                                     | Harness setup and public API example.                          |

*Table 2: The authoritative manifest of all 34 paths changed from `origin/main`
to the current candidate. The original 22-path tolerance was extended first to
24 by a bounded review authorization, then to 33 by subsequent explicit user
requests and the narrowly authorized lint repair, and to 34 by this adoption
note. No further scope is implied.*

## Revision note

Revision 2, 2026-08-15. Revised after a six-lens design review, and after four
further probes run to check the reviewers' claims rather than accept them.

What changed and why:

- **Manifest.** Licence corrected to ISC, matching the repository `LICENSE` and
  every published sibling. Marked the crate unpublishable so the incubating
  copy cannot claim the upstream-documented crate name. Switched to exact
  requirements, because a caret on a pre-release admits the `beta4` that this
  plan itself documents as breaking. Added Bevy's `std` feature and `tracing`
  as a direct dependency.
- **Public API.** The two constructors became `add_minimal_plugins` and
  `minimal_app`. The first draft claimed the constructors would become
  `configure` bodies; they could not, because `configure` mutates a borrowed
  application. `add_minimal_plugins` has that shape, so the carry-forward claim
  is now true. Added the Bevy re-export and the two unit-context aliases the
  cited GPUI precedent actually re-exports.
- **Milestone 2.** The specified red state could not occur: without the module
  declaration, rustc never compiles the test file and the run passes with zero
  tests. The declaration moved into the red state; only the re-export is
  withheld.
- **Gate commands.** All milestone test commands now carry denied warnings and
  the dev-fast configuration, matching what `make` does — the absence of the
  former is precisely why the first draft's probes missed the `unused_braces`
  trap. Replaced an inert log-path variable with a `gate` helper that
  propagates the pipeline's exit status, so a failing gate no longer reads as
  passing through `tee`.
- **New risks recorded.** The fixture deadlock between `unused_braces` and
  single-line formatting; the timestamp-counter clock without Bevy's `std`
  feature; workspace-wide feature unification defeating the headless
  constraint; the CI cache budget; and the floating-versus-pinned rustfmt split
  between `make fmt` and `make check-fmt`.
- **Added `Outcomes & retrospective`**, which the document's own opening
  paragraph required and the first draft omitted, and a section then named
  `Open decisions requiring approval` for the two questions the review could
  not settle from the repository.
- **Corrected figures.** The graph is 139 crates normal and 279 with
  development dependencies, not 98; that figure was the bevy-only probe. Build
  cost is now measured in seconds and gigabytes rather than asserted in
  adjectives. Coverage is met by the unit tests, not the doctests, because the
  CI coverage action uses nextest, which does not run them.
- **Scope trigger** raised from 18 files to 22, because the first draft's
  tolerance was set to exactly the file count it specified, and Table 2 became
  the authoritative manifest it is measured against.

Historical revision 3, 2026-08-15. Adopted the estate-mandated remedy for the
fixture expansion lint, which now lands in the layer beneath this plan in the
pull request stack rather than being worked around locally.

- The `let`-binding workaround is withdrawn. The scenario fixture keeps its
  natural single-expression form and carries `#[allow_fixture_expansion_lints]`
  from `skyjoust-test-macros`, mirroring `weaver-test-macros` in
  `leynos/weaver`. The workaround would have spread a bespoke idiom across
  every fixture the project writes instead of fixing the cause once.
- The harness ADR is renumbered 006 to 007, because the lower layer takes 006
  for the test-macro decision.
- At the time, `skyjoust-test-macros` was added as a development dependency and
  the extraction contract distinguished game code from test-only tooling. This
  exception was superseded on 2026-10-12 after standalone manifest resolution
  confirmed that the sibling path dependency violates extraction readiness.
- The original boundary guard permitted the test macro crate as a near-match.
  That remains the current canary behaviour: the predicate allows the exact
  macro-crate name, while the extraction invariant prohibits an actual sibling
  path-dependency edge. Direct manifest and transitive graph checks cover those
  distinct conditions.

Revision 4, 2026-08-17. Recorded the maintainer's answers to the open design
questions and reconciled their effects across the complete plan.

- Bevy now targets `0.19.1` with implicit caret syntax. All `0.17.3` probe
  results are labelled historical, and a Milestone 0 follow-up must repeat the
  API, feature, headless, Cranelift, lint, test, and cost checks before code
  work begins.
- The `rstest-bdd` family was exactly pinned to `=0.6.0-beta3`. This historical
  decision is superseded by the 2026-10-12 stable 0.6.0 adoption decision;
  stable compatibility evidence remains pending.
- Step functions now return `StepResult`, propagate `RefCell` borrow failures,
  and use functional transformations where those remain clear. This records why
  `.expect()` is invalid in macro-registered step functions even though test
  bodies may use it.
- Pull request #6 is a prerequisite: wait for automerge, rebase the stack onto
  `main`, and reconcile the new crate's `rstest` requirement with the merged
  root before Milestone 2.
- Added the mandatory `Conformance basis` and `Verification plan` living
  sections, including trace links, negative controls, external axioms, and
  discharge conditions for the new decisions.

Revision 5, 2026-08-21. Rebasing the implementation branch onto the pull
request's current `origin/main` base required a single `Cargo.lock` conflict.

- Retained `main`'s lockfile at the conflict, then regenerated it from the
  final manifests. The regenerated graph resolves Bevy `0.19.1` and the
  inspected `rstest-bdd` family at `0.6.0-beta3`.
- Repeated every repository gate and every ExecPlan acceptance command. The
  feature-file freshness A/B was rerun rather than inferred from earlier
  evidence.
- Replaced the obsolete force-push authorization blocker with the remaining
  remote-delivery condition. The branch may be published with the maintainer's
  explicit lease-protected authorization, but PR #51 remains pending until its
  merge state is clean and remote CI is non-skipped.

Revision 6, 2026-10-12. Adopted the published stable `rstest-bdd` 0.6.0 family
with implicit-caret requirements and imported its tagged user's and migration
guides. The migration guide identifies the stable `HarnessResult<T>` return
contract, Rust 1.88 minimum, and automatic feature-file rebuild dependency. At
this revision, compatibility validation, lockfile regeneration, and acceptance
gates remained pending. Review verification superseded the sibling test-macro
exception: fixture lint support is now owned by the nested
`rstest-bdd-harness-bevy-test-macros` package. The exact-name boundary guard
retains its synthetic near-match canary, while the actual test graph excludes
the sibling package. The trybuild pass fixture covers an `rstest` fixture and
BDD step attributes, not valid `#[scenario]` expansion or `_app` implicit
fixture normalization; `tests/headless_scenario.rs` compiles and exercises the
valid scenario binding. The compile-fail fixture covers a missing scenario
feature. No separate UI case for Rust's generic `include_str!` missing-file
behaviour is required. Later validation is recorded in the current progress
section; the roadmap task remains checked complete while final acceptance
remains pending.

Revision 7, 2026-10-12. Renumbered the Bevy harness decision to ADR 009 after
the rebased `main` added ADR 007 for the Rust build standard and ADR 008 for
CodeScene coverage publication. Updated active links and index entries while
retaining the original ADR 007 number only in dated implementation history.
Reconciled the developer and repository guides with stable 0.6.0's automatic
feature-file rebuild tracking: the old beta3 A/B result remains historical, and
the stable-version freshness A/B and compatibility gates remain pending. The
roadmap item 0.5.1.1 remains checked complete.

Revision 8, 2026-10-12. Reconciled the live version-coherence invariant and
historical root-`rstest` risk with the rebased `rstest = "0.27"` workspace and
the selected stable `rstest-bdd` 0.6.0 implicit-caret requirements. Updated the
remaining implementation guidance to treat stable feature-file rebuild tracking
as an acceptance A/B check, not as an established beta3 outcome. Compatibility,
lockfile resolution, and post-rebase acceptance evidence remain pending; no new
gate result is claimed.

Revision 9, 2026-10-12. Clarified that the direct manifest guard deliberately
permits `skyjoust-test-macros` as an exact-name near-match canary; only an
actual sibling dependency edge violates the extraction invariant. Corrected the
test description and distinguished historical 13-test results from the current
14-test suite, including the new `trybuild` runner and four integration
targets. Stable validation remains pending.

Revision 10, 2026-10-12. Corrected the current trybuild coverage description:
the pass fixture exercises an `rstest` fixture and `rstest-bdd` step macros,
not valid `#[scenario]` expansion or `_app` normalization. The successful
scenario binding remains covered by `tests/headless_scenario.rs`. Recorded that
the focused trybuild runner passed against stable 0.6.0; full crate acceptance
gates remain pending.

Revision 11, 2026-10-12. Recorded the stable 0.6.0 feature-freshness A/B: after
warm passes, a feature-only edit with no Rust-source touch triggered a rebuild
and the expected `Step not found` failure both with and without explicit
`include_str!`. The macros auto-track the bound feature; retain the explicit
line as a task-required documentary dependency, not as a load-bearing guard.
Updated the README, developer guide, design, and repository layout. Full
compatibility and acceptance gates remain pending.

Revision 12, 2026-10-12. Recorded the stable inverse dependency and standalone
crate-suite results, plus the `check-fmt` and state-graph results. Markdownlint
stopped on contents-index entries for two intentionally deleted paths; those
deletions are staged, and the remaining gates are pending. The maintainer
authorized a bounded extension from the original 22 touched paths to the 24
paths enumerated in Table 2. No final acceptance is claimed.

Revision 13, 2026-10-12. Updated the compile-time-test finding: retain the
scenario-macro missing-feature compile-fail case and add a separate direct
missing-`include_str!` case for the integration test's literal file inclusion.
At this revision the direct case had not yet been validated; Revision 14
records the later successful focused UI run. This supersedes the earlier
decision to omit the generic include case without changing the passed
feature-freshness A/B evidence.

Revision 14, 2026-10-12. Recorded the successful focused `TRYBUILD=overwrite`
run, covering the pass fixture and both compile-fail diagnostics, including the
missing scenario feature and literal `include_str!` target. The imported v0.6.0
guides' cross-document links were audited and now resolve to the imported guide
pair or the tagged upstream ADR. Full acceptance gates remain pending.

Revision 15, 2026-10-12. Reconciled validation after rebase: the six-gate green
run included a temporary edit to the explicitly excluded validator crate and is
not valid evidence for this task. After removing it, in-scope `make lint`
failed on `crates/skyjoust_stateright_validator/examples/serve_explorer.rs:12:4`
(cognitive complexity 15/9); the cited log and byte-identical comparison
against `origin/main` `d177781a276012da013531e7b6b9642fc0a5e0b8` are recorded in
`Progress` for candidate `6d5babe714d45179c632935bfa7611062216d3ad`. The
earlier markdownlint failure note was stale; final in-scope validation passed
markdownlint with zero errors. Review, acceptance, and publication remain
pending because of the lint failure and live PR state; the roadmap task remains
checked without claiming acceptance.

Revision 16, 2026-10-12. Recorded final in-scope results on candidate
`6d5babe714d45179c632935bfa7611062216d3ad`: `make test` passed 236/236 plus
workspace doctests; `make check-fmt`, markdownlint (zero errors), typecheck,
and nixie passed. These results supersede the temporary-edit gate run for those
targets. In-scope `make lint` remains failed on the unchanged origin/main
example. Review, acceptance, and publication remain pending due to lint and
live PR state.

Revision 17, 2026-10-12. At the time of this entry, the maintainer's direct
instruction superseded the earlier validator-crate exclusion solely for the
minimal cognitive-complexity repair in
`crates/skyjoust_stateright_validator/examples/serve_explorer.rs`. The example
now extracts tracing initialization and its two log events into small helpers.
This preserves their startup order and keeps `_checker` alive through
`std::thread::park()`. No lint suppression or threshold change was made. This
is a narrow lint-baseline exception, not a general expansion of the crate's
task scope. The pending-validation status in this entry is historical; Revision
20 records the focused and full reruns.

Revision 18, 2026-10-12. Recorded validation after the narrow example repair:
`make check-fmt`, `make typecheck`, `make lint`, markdownlint, and nixie
passed. The first full `make test` passed 234/236; both trybuild suites failed
because nested Cargo could not execute (`ENOENT`). A focused run with explicit
Cargo passed that stage but stopped at `build-limits-rustc` `EAGAIN` before
fixtures, so it is inconclusive. The Makefile now resolves and exports the
selected Cargo executable to test and doctest processes; that change awaits
validation. The earlier 236/236 test result predates this repair and is
historical. Do not claim the full test gate or acceptance complete until the
updated Makefile and test suite have been validated.

Revision 19, 2026-10-12. Recorded the later test rerun after the Makefile
change: the helper extraction's `make lint` and the other listed non-test gates
are green, but both compile-fail UI fixtures were reported as succeeding and a
separate `build_standard_contract` case failed with OS `EAGAIN`. Source
inspection of trybuild 1.0.121 shows that an absent matching rustc JSON error
defaults a case to success and `build_test`'s process exit status is ignored.
Resource exhaustion is a plausible explanation for the missing diagnostics, not
a proven cause. A constrained scrutineer reproduction of `macro_ui` is pending.
Keep acceptance incomplete until the UI cases and full test gate are validly
discharged; the roadmap's checked implementation milestone is not evidence that
PR acceptance or delivery is complete.

Revision 20, 2026-10-12. Reconciled this plan and the roadmap with the
successful focused and full reruns on rebased candidate
`f9a72c93b026e796ff025e71803e1a47a45ac31a` (tree
`515c4b3bac3c97cb776e10dd10eda54bfd89e112`). The constrained `macro_ui` run
passed one pass and two fail fixtures in 171 seconds; all six sequential
workspace gates passed, including `make test` at 236/236 plus doctests. The
earlier EAGAIN and unexpected-success outcomes remain historical; their
suspected resource-exhaustion explanation was not proven. The narrow
`serve_explorer.rs` helper extraction is accepted as the lint repair, without
suppression or threshold changes, and supersedes only the earlier prohibition
on editing that validator example. Updated Table 2 against the candidate diff:
all 33 changed paths appear exactly once. This includes the previous bounded
24-path follow-up plus later explicit user requests and the authorized lint
repair; it does not authorize further scope. The roadmap item remains checked
for implementation completion. Remote PR head, hosted checks, review, and final
acceptance remain pending, so local gates are not represented as hosted
evidence.

Revision 21, 2026-10-12. Added the Skyjoust-specific adoption note at
`docs/rstest-bdd-harness-bevy-adoption.md` and linked it from the documentation
contents and user's guide, superseding the earlier decision that no separate
adoption note was warranted. Reconciled Table 2 and the scope tolerance to 34
paths: the 33 paths in the candidate diff from `origin/main`, plus the new
untracked adoption note. The six gates and focused UI run in Revision 20 remain
successful pre-documentation evidence, not validation of this follow-up. At
that revision, documentation gates and the isolated/workspace graph, headless,
focused-test and feature-freshness acceptance checks were pending. The roadmap
item stays checked as implementation complete. PR acceptance, hosted checks,
publication and review remain pending.

Revision 22, 2026-10-12. Recorded the scrutineer's post-documentation results
for the docs gates and all requested acceptance checks in `Progress` above. The
outside-root standalone attempt first failed because stable Cargo 1.99 could
not parse the configured `codegen-backend` profile; an intervening
wrong-nightly attempt was stopped without a terminal result. The pinned
nightly-2026-03-26 standalone rerun passed, as did the in-workspace focused
suite. The isolated/workspace dependency and headless graphs passed, and the
feature freshness A/B passed from a disposable archive without tracked edits.
This supersedes Revision 21's pending validation status, not its 34-path
manifest. The roadmap item remains checked complete; publication, hosted
checks, CodeRabbit assessment and review, and final PR acceptance remain
pending.

Revision 23, 2026-10-12. Recorded publication of PR #51 at
`adfcb5c5e1bc435f6a58e8a0cdc5fa800fb23800`, based on `main`
`d177781a276012da013531e7b6b9642fc0a5e0b8`. Its source tree is the locally
validated `f9a72c9` tree; the publication follow-up changed documentation only.
At the latest status check, the PR was ready but `BLOCKED`, review decision was
`CHANGES_REQUESTED`, and build-test run `38309867260` was in progress. Existing
review findings refer to old head `6b185ee`; they are not current-head review
evidence and require reconciliation. Local gate and acceptance results remain
local evidence only. The roadmap implementation item remains checked; hosted
outcomes, current review, and final acceptance remain pending.

Revision 24, 2026-10-13. Recorded the local validation after the Nextest policy
Clippy fixes. `make check-fmt`, `make typecheck`, `make lint`,
`make markdownlint`, and `make nixie` passed. `make test` did not exercise the
test suite: Nextest could not spawn `validate_trace --list` and returned
`EAGAIN` (OS error 11). This is an infrastructure/resource failure, not a
product test assertion, so the test gate remains unpassed. In the first test
attempt, separately, a nested test helper's `make` invocation could not spawn
`uname` and `/bin/sh` under the same process pressure; that also was not a
product test assertion. Read-only resource checks on 2026-10-13 observed
`lody-daemon.service` at `TasksCurrent=8131` of `TasksMax=8192` and 8,776
global threads. No process was interrupted. Workspace Nextest configuration
inspection verified that both `macro_ui` and `fixture_expansion_lints_ui` are
in the `compile-ui` group with `max-threads = 1`. This establishes group
membership only: the full UI runtime and hosted coverage behaviour of the
mitigation remain unvalidated. PR acceptance remains pending.

Revision 25, 2026-10-13. The bounded isolated policy-test command

```sh
NEXTEST_TEST_THREADS=1 CARGO_BUILD_JOBS=1 RAYON_NUM_THREADS=1 \
  make test TEST_FLAGS='--package skyjoust --test nextest_policy'
```

passed all three policy tests and workspace doctests in 16 seconds. Log:
`/tmp/test-nextest-policy-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-pr51-repair.out`.
Read-only service observations were `TasksCurrent=8131/8192` before and
`8132/8192` after the run. This focused pass does not replace the full workspace
`make test`, which remains blocked while Nextest lists tests because the
service is at its task cap. The roadmap implementation item remains checked; PR
acceptance remains pending.

Revision 26, 2026-10-13. The bounded full-workspace command
`NEXTEST_TEST_THREADS=1 CARGO_BUILD_JOBS=1 RAYON_NUM_THREADS=1 make test`
passed in 18 seconds: 239/239 tests across 22 binaries plus workspace doctests.
Log:
`/tmp/test-workspace-constrained-rebase-0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member-20260928-pr51-repair.out`.
`build-limits` was healthy; read-only `TasksCurrent` observations were
8126/8192 before and 8123/8192 after. This is warm local validation and does
not prove the cold hosted coverage timeout is resolved. The roadmap item
remains checked, and PR acceptance remains pending.

# Add `rstest-bdd-harness-bevy` as a workspace member

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, and `Outcomes & retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

Approval gate: this plan must be approved before implementation begins. Do not
treat silence as approval. Two decisions in `Open decisions requiring approval`
need an explicit answer at that gate.

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

## Open decisions requiring approval

Two choices are recorded below with a default and a rationale, but both need an
explicit answer before Milestone 2. Each was raised independently by more than
one reviewer, and each is expensive to reverse later.

1. **Which Bevy version?** `0.17.3` appears in exactly one place in the
   repository — `docs/roadmap.md` line 43. [The technical
   design](../skyjoust-technical-design.md) never names a Bevy version. Bevy
   `0.19.1` is the current release, and `0.18.1` is already present in this
   machine's Cargo cache, so some part of the estate is past `0.17`. This is
   not a local test-tooling choice: roadmap `1.2.2` brings Bevy into the
   Skyjoust runtime, and `0.5.1.4` requires the harness to host a profile
   driving that runtime's `App`. Bevy types do not cross a major, so if the two
   diverge, `0.5.1.2` and `0.5.1.3` are wasted.
   Default taken: `0.17.3`, following the roadmap literally, verified working.
   The consequence, which must be accepted or rejected at the gate, is that the
   harness becomes the constraint on the runtime — roadmap `1.2.2` must adopt
   `0.17.3` too.

2. **Caret or exact requirements for `rstest-bdd`?** `AGENTS.md` mandates caret
   requirements. But `^0.6.0-beta3` admits `0.6.0-beta4`, which upstream has
   already made source-breaking (`StepContext::borrow_mut` now takes `&self`),
   and `.github/dependabot.yml` runs the Cargo ecosystem daily with automerge.
   Default taken: exact pins, `=0.6.0-beta3`, as a recorded deviation limited
   to pre-release requirements, reverting to `^0.6.0` when `0.6.0` final ships.
   `AGENTS.md` forbids `*` and `>=` and constrains `~`; it is silent on `=`,
   and the mandate's stated purpose — build stability and reproducibility — is
   served rather than undermined by pinning a pre-release.

## Constraints

These are hard invariants. If satisfying the objective would require violating
one, stop and escalate rather than working around it.

- The crate must not depend on `skyjoust`, `skyjoust-stateright-validator`, or
  any Lille crate, directly or transitively. This is the extraction contract
  from [the harness design](../rstest-bdd-harness-bevy-design.md) §11. The
  contract is about *game* code: gameplay rules, runtime state resources, and
  validator types must never reach this crate. Test-only tooling is a separate
  category — `skyjoust-test-macros` is a permitted development dependency, on
  the same footing as `rstest` or `googletest`, and is listed in the extraction
  note so the eventual move rewires it deliberately. Note the trap this
  creates: `skyjoust-test-macros` contains the substring `skyjoust`, so the
  boundary guard must match dependency *names* exactly rather than by
  substring, or it will reject a dependency the contract allows.
- The crate's own manifest must declare Bevy with `default-features = false`
  and enable no feature beyond `std` (see `Decision log`). Cargo unifies
  features across workspace members built in one invocation, and `make test`
  and `make lint` both build `--workspace --all-features`. This constraint
  therefore bounds what the crate *requests*, not what a workspace build
  resolves. Any other member that later adds `bevy` must also declare
  `default-features = false`, or the headless guarantee is lost for everyone.
- The harness crate's Bevy major version is a workspace-wide coupling, not a
  local choice. It must equal the version the Skyjoust runtime will use at
  roadmap `1.2.2`. Changing it is an ADR-level decision.
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
- Do not modify `crates/skyjoust_stateright_validator/`, `src/`, or
  `tests/makefile_contract.rs`.
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

- Scope: if the change touches more than 22 files, stop and escalate. Table 2
  is the authoritative file manifest to measure against.
- Public interface: this plan introduces exactly two public functions plus the
  re-export block described in `Interfaces and dependencies`. Anything named in
  design §§4-7 — `BevyScenario`, `BevyHarness`, `BevyProfile`,
  `BareBevyProfile`, `MinimalBevyProfile`, `BevyAttributePolicy` — belongs to
  roadmap task `0.5.1.2`; if one appears necessary here, stop and escalate.
- Dependencies: the dependency set in `Interfaces and dependencies` is fixed.
  If any additional crate is required, stop and escalate.
- Bevy features: `default-features = false` with `features = ["std"]` is the
  agreed declaration. If a further feature proves necessary, stop and escalate.
- `rstest-bdd` version: if `0.6.0-beta3` proves insufficient and a git
  dependency on `main` becomes necessary, stop and escalate.
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
  mutually exclusive formulations.
  Severity: high. Likelihood: certain (observed).
  Mitigation: retired by the layer beneath this one in the stack, which adds
  `crates/skyjoust_test_macros` and its `allow_fixture_expansion_lints`
  attribute — the estate-mandated approach, mirroring `weaver-test-macros`.
  Apply the attribute directly above `#[fixture]` and leave the fixture in its
  natural single-expression form. See
  [ADR 006](../adr/006-test-macro-crate-for-fixture-expansion-lints.md) and
  [the developer's guide](../developers-guide.md) §7.3. Verified clean under
  both gates.

- Risk: omitting Bevy's `std` feature replaces `std::time::Instant` with
  `bevy_platform::time::fallback::Instant`, whose x86-64 getter reads
  `core::arch::x86_64::_rdtsc()` and passes the raw tick count to
  `Duration::from_nanos`. `Time<Real>` then advances at roughly the timestamp
  counter frequency rather than wall-clock, and `Time<Virtual>` and
  `Time<Fixed>` derive from it.
  Severity: high. Likelihood: certain without `std` (verified in source).
  Mitigation: enable `features = ["std"]`. Verified from the resolved feature
  set that this does *not* re-enable `multi_threaded`, so the single-threaded
  executor is retained.

- Risk: `clippy::expect_used` is denied estate-wide, and
  `allow-expect-in-tests = true` in `clippy.toml` does **not** cover free
  functions in `tests/*.rs` that are not `#[test]`-annotated. `rstest-bdd` step
  functions are exactly such functions, so `.expect()` inside a step fails
  `make lint`.
  Severity: medium. Likelihood: high (observed).
  Mitigation: scenario state comes from an `rstest` fixture holding a
  `RefCell<App>` directly, so no step unwraps an `Option`.

- Risk: `^0.6.0-beta3` is not a pin. It admits `0.6.0-beta4` and `0.6.0`
  final, and `beta4` changes `StepContext::borrow_mut` to take `&self`. A bare
  `cargo update`, or Dependabot's daily run with automerge, would break every
  scenario in the workspace.
  Severity: high. Likelihood: medium.
  Mitigation: exact `=0.6.0-beta3` requirements (subject to the approval gate),
  plus a note in ADR 007 that `Cargo.lock` is load-bearing for the `rstest-bdd`
  family until `0.6.0` is stable.

- Risk: editing only a `.feature` file does not invalidate the build, so a
  changed scenario can appear to pass while the compiled step table is stale.
  Severity: medium. Likelihood: high.
  Mitigation: the scenario binding carries an `include_str!` of the feature
  file, the idiom used by `rstest-bdd`'s own fixtures. Milestone 4 proves the
  guard is load-bearing with an A/B test rather than a single observation.

- Risk: the crate adds a large graph to every workspace build — 139 crates for
  normal dependencies and 279 including development dependencies. `make lint`
  compiles it three times (`cargo doc`, `cargo clippy`, and the Whitaker Dylint
  driver, the last under its own toolchain with no Cranelift), and `make test`
  a fourth. CI caches no Cargo artefacts beyond the two shared-action caches.
  Severity: low, revised down after measurement. Likelihood: certain.
  Mitigation: measured on six cores with a warm registry — cold Cranelift
  `check --workspace --all-targets --all-features` 26 s, cold Cranelift test
  build 30 s, cold LLVM Clippy 19 s. Record the equivalent figures for the real
  workspace in `Outcomes & retrospective`.

- Risk: `target/` reaches 2.2 GB for this crate alone. Both CI caches — the
  `setup-rust` debug-target cache and the coverage action's whole-target cache
  — grow accordingly, against GitHub's 10 GB per-repository cap. Cache thrash
  degrades every run to cold without turning anything red.
  Severity: medium. Likelihood: medium.
  Mitigation: record the measured target size in `Outcomes & retrospective`. If
  the two caches together approach the cap, raise a follow-up to narrow the
  coverage job's cached path.

- Risk: `make fmt` runs `cargo +nightly fmt` (the floating nightly) while
  `make check-fmt` runs `cargo fmt` (the toolchain-file-pinned nightly). With
  `unstable_features`, `wrap_comments`, `format_strings`, and
  `format_code_in_doc_comments` all enabled, the two can disagree — and this
  plan adds doc comments carrying worked examples, which is exactly what
  `format_code_in_doc_comments` rewrites.
  Severity: low. Likelihood: low.
  Mitigation: in Milestone 5, run `mdformat-all` and `cargo fmt --all` (pinned)
  separately rather than `make fmt`, so the formatting that lands is the
  formatting `make check-fmt` will accept.

- Risk: `googletest`'s `expect_that!` requires an active test context and
  panics with a no-test-context message when the test is not annotated
  `#[gtest]`.
  Severity: low. Likelihood: high (observed).
  Mitigation: unit tests use `#[gtest]` above `#[rstest]`; step functions use
  `assert_that!`, which panics directly and needs no context.

- Risk: the workspace root package pins `rstest = "0.18"` while `rstest-bdd`
  `0.6.0-beta3` requires `rstest 0.26.1`.
  Severity: low. Likelihood: certain.
  Mitigation: accept both. They are semver-incompatible, so Cargo keeps them
  side by side and no crate sees two versions at once. Do not bump the root
  package's `rstest` as part of this task.

## Progress

Next action: obtain approval, including answers to the two items in
`Open decisions requiring approval`.

Last green gate: `markdownlint` and `nixie` — documentation only, commit
`69a903a`.

- [x] (2026-08-15) Milestone 0: orientation and evidence gathering. Six probes
      run; findings in `Surprises & discoveries` and `Artefacts and notes`.
- [ ] Milestone 1: record the decision.
  - [ ] Write the new ADR 007.
  - [ ] Amend the harness design document §§3, 9, 13 and its layout block.
  - [ ] Add a forward pointer from ADR 002.
  - [ ] Index ADR 007 and this ExecPlan in `docs/contents.md`.
  - [ ] Documentation gates green.
- [ ] Milestone 2 (red): scaffold and failing tests.
  - [ ] Add the workspace member.
  - [ ] Write the crate manifest.
  - [ ] Write `src/lib.rs` with the module declaration but no re-export of it.
  - [ ] Write `src/profile.rs` (module comment only) and `src/profile_tests.rs`.
  - [ ] Observe the red state; record the exact error text.
  - [ ] `make typecheck` as a whole-graph Cranelift smoke test.
- [ ] Milestone 3 (green): implement `src/profile.rs`; export its two functions.
- [ ] Milestone 4: behavioural, property, and boundary coverage.
  - [ ] `tests/features/headless_scenario.feature`.
  - [ ] `tests/headless_scenario.rs`.
  - [ ] Clippy immediately after the scenario binding first compiles.
  - [ ] `tests/tick_properties.rs`.
  - [ ] `tests/extraction_boundary.rs`.
  - [ ] A/B-prove the feature-file rebuild guard.
- [ ] Milestone 5: documentation.
  - [ ] Crate `README.md` including the Bevy compatibility table.
  - [ ] `docs/repository-layout.md`.
  - [ ] `docs/developers-guide.md`.
  - [ ] `docs/roadmap.md` — tick `0.5.1.1`, correct the stale sub-bullet.
- [ ] Milestone 6: full gate run, measurements, push, draft pull request.

## Surprises & discoveries

- Observation: `rstest-bdd` `0.6.0-beta3` was published to crates.io on
  2026-07-07, along with `rstest-bdd-harness`, `rstest-bdd-macros`,
  `rstest-bdd-harness-tokio`, and `rstest-bdd-harness-gpui` at the same
  version. The roadmap's instruction to use git dependencies against `main`
  "until v0.6.0-beta3 is published" is therefore already satisfied.
  Evidence: the crates.io versions endpoint lists `0.6.0-beta3` created at
  `2026-07-07T23:13:10Z`; `rstest-bdd-harness 0.6.0-beta3` at
  `2026-07-07T23:12:46Z`.
  Impact: this plan uses published requirements, not git dependencies. The
  roadmap sub-bullet and design §9 are amended accordingly.

- Observation: `rstest-bdd`'s `main` branch has moved to an unpublished
  `0.6.0-beta4` (head commit `12b9357`, 2026-08-14), well past the commit
  `21b67a4` cited in the design document. That version adds a
  `policy_conformance` module, a `testing` feature exposing `FailingHarness`,
  and guard-based fixture borrowing. It also changes the generated panic text
  from `initialise` to `initialize`.
  Evidence: `main`'s harness crate root exports `policy_conformance` and a
  feature-gated `FailingHarness`; the extracted `beta3` archive exports neither
  and has no features table at all.
  Impact: under `beta3`, requesting the `testing` feature will not resolve, and
  the policy-conformance helper is unavailable. Neither is needed here;
  `0.5.1.3` must plan around them. Recorded in the design document with an
  explicit migration trigger.

- Observation: `default-features = false` alone silently degrades timekeeping.
  `bevy_platform`'s time module selects `std::time` only when its `std`
  configuration predicate holds; otherwise it uses a fallback whose x86-64
  getter passes `_rdtsc()` straight to `Duration::from_nanos` — raw timestamp
  counter ticks reinterpreted as nanoseconds. `TimePlugin` calls the clock
  unconditionally.
  Evidence: `bevy_platform-0.17.3/src/time/mod.rs` selects the fallback module
  in the non-`std`, non-web branch; `src/time/fallback.rs` contains the
  `_rdtsc()` arm.
  Impact: the plan enables `features = ["std"]`. Confirmed from the resolved
  feature set that `bevy_ecs` then gains `async_executor`, `backtrace`,
  `bevy_reflect`, `default`, and `std` but *not* `multi_threaded`, so the
  single-threaded executor is retained. This finding also demolishes the
  determinism rationale the first draft was about to write into ADR 007.

- Observation: `clippy.toml`'s `allow-expect-in-tests` setting does not exempt
  non-`#[test]` free functions inside `tests/*.rs`. A step function calling
  `.expect()` fails Clippy with warnings denied.
  Evidence: a probe using a thread-local `Option` slot and `.expect()` in a
  `#[when]` step produced two `expect_used` errors.
  Impact: the behavioural test uses an `rstest` fixture holding the application
  directly. The same trap will recur in `0.5.1.2`; it is documented in the
  developer's guide.

- Observation: a single-expression `#[fixture]` body trips `unused_braces`
  under denied warnings, and `fn_single_line` reformats the multi-line repair
  straight back to the failing form.
  Evidence: `error: unnecessary braces around block return value` at the
  fixture, with `-D unused-braces implied by -D warnings`; a plain non-fixture
  function with the same shape does not fire.
  Impact: this is a known `rstest` issue with an estate-mandated remedy, not a
  problem for this plan to solve locally. The layer beneath this one in the
  stack adds `crates/skyjoust_test_macros` and its
  `allow_fixture_expansion_lints` attribute, mirroring `weaver-test-macros` in
  `leynos/weaver`. An earlier draft of this plan proposed a local
  `let`-binding workaround; that has been withdrawn, because it would have
  spread a bespoke idiom across every fixture the project writes rather than
  fixing the cause once. Verified clean under both Clippy and
  `cargo fmt --check` with the attribute applied and the fixture left in its
  natural form.

- Observation: `googletest 0.14.3`'s `expect_that!` requires the `#[gtest]`
  attribute; without it every assertion panics reporting no test context.
  Evidence: four unit tests annotated only `#[rstest]` failed that way; adding
  `#[gtest]` above `#[rstest]` made all four pass.
  Impact: unit tests use `#[gtest]` plus `#[rstest]`; step functions use
  `assert_that!`.

- Observation: the constructors proposed in the first draft could not become
  what the design says they become. Design §5 declares a `configure` that takes
  `&mut App` and returns a harness result — it mutates a borrowed application —
  whereas a constructor builds and returns one.
  Evidence: design §5's trait definition, compared with the draft's
  `pub fn minimal_app() -> App`.
  Impact: the public surface was reshaped. `add_minimal_plugins` takes
  `&mut App`, which is the shape `configure` calls, so
  `MinimalBevyProfile::configure` becomes two lines that call it. `bare_app`
  was dropped entirely: a plain `App::new()` needs no wrapper, and
  `BareBevyProfile::configure` has an empty body, so there was nothing to seed.

- Observation: `App::new()` is not plugin-free. It already adds
  `MainSchedulePlugin` and registers a message-update system in `First`.
  Evidence: `bevy_app-0.17.3/src/app.rs`.
  Impact: any test named "adds no plugins" would be false. The negative test is
  named for what it checks — that an unconfigured application omits
  `TimePlugin`. This is also why an unconfigured application can be updated at
  all.

- Observation: the first draft's build-cost figure of 98 crates was the
  bevy-only probe measured for normal dependencies. The real crate resolves 139
  crates for normal dependencies and 279 including development dependencies.
  Evidence: `cargo tree --workspace -e normal --prefix none | sort -u | wc -l`
  and the `-e normal,dev` equivalent, run against the full shape probe.
  Impact: the build-cost risk is restated with measured wall-clock rather than
  crate counts.

- Observation: the CI coverage action builds with *default* features via
  `cargo llvm-cov nextest`, and nextest does not run doctests.
  Evidence: the `generate-coverage` action's Rust runner script in
  `leynos/shared-actions`.
  Impact: doctests contribute nothing to the codecov figure. The patch target
  is still met, but by the unit tests — the first draft gave the wrong reason.

- Observation: `docs/rstest-bdd-users-guide.md` and
  `docs/ortho-config-users-guide.md`, cited in the earlier `1.1.1` execplan, do
  not exist in this repository.
  Evidence: repository-wide glob for both names returns nothing.
  Impact: this plan cites the upstream `rstest-bdd` users' guide by uniform
  resource locator (URL) instead, and does not create either local file.

## Decision log

- Decision: depend on published `rstest-bdd` `0.6.0-beta3` crates rather than
  git dependencies against `main`.
  Rationale: the roadmap's git-dependency instruction was explicitly
  conditional on `beta3` not yet being published; it now is. Published crates
  keep `Cargo.lock` stable and avoid pulling `rstest-bdd`'s vendored GPUI tree
  into this workspace's resolution. The later APIs are not needed here.
  Date/Author: 2026-08-15, planning pass.

- Decision: use exact `=0.6.0-beta3` requirements, deviating from `AGENTS.md`'s
  caret mandate for pre-release requirements only.
  Rationale: a caret on a pre-release admits `0.6.0-beta4`, which upstream has
  made source-breaking, and Dependabot runs daily with automerge here.
  `AGENTS.md` forbids `*` and `>=` and constrains `~` but is silent on `=`, and
  the mandate's stated purpose — build stability and reproducibility — is
  served by pinning a pre-release rather than floating across it. Revert to a
  caret when `0.6.0` final publishes. Subject to the approval gate.
  Date/Author: 2026-08-15, planning pass.

- Decision: declare Bevy with `default-features = false` plus
  `features = ["std"]`.
  Rationale: disabling default features is what the roadmap and design require,
  and it excludes the renderer, windowing, assets, audio, and the
  multi-threaded executor. `std` is added back because without it
  `bevy_platform` substitutes a clock that reinterprets timestamp counter ticks
  as nanoseconds, so real time — and fixed time beneath it — would run at
  roughly the counter frequency. That would be a booby trap for the fixed-tick
  Skyjoust profile at `0.5.1.4`. Determinism is explicitly *not* part of this
  rationale: nothing in this milestone has ambiguous system ordering. Cost is
  24 crates, from 115 to 139 normal dependencies.
  Date/Author: 2026-08-15, planning pass.

- Decision: pin Bevy `0.17.3`, and record that `0.19.1` is the current release.
  Rationale: the roadmap names `0.17.3` and this plan implements the roadmap.
  The version is not a local choice — it couples the harness to whatever Bevy
  the Skyjoust runtime adopts at roadmap `1.2.2` — so accepting `0.17.3` here
  means committing the runtime to it too. Raised at the approval gate as item 1
  of `Open decisions requiring approval`.
  Date/Author: 2026-08-15, planning pass.

- Decision: this milestone ships two public functions, `add_minimal_plugins`
  and `minimal_app`, in `src/profile.rs`, plus a re-export block in `lib.rs`.
  Rationale: the ExecPlan bar is demonstrably working behaviour, not a crate
  that merely compiles. `add_minimal_plugins` has exactly the shape design §5's
  `configure` calls, so `MinimalBevyProfile::configure` becomes two lines that
  call it in `0.5.1.2` — the carry-forward is real, not aspirational.
  `minimal_app` is the harness-free entry point for tests that want an
  application without a scenario, which Skyjoust and Lille will both want at
  `0.5.1.4`. Both stay public after `0.5.1.2`.
  Alternatives considered and rejected: (a) a `bare_app` constructor alongside
  them — a plain `App::new()` needs no wrapper and `BareBevyProfile::configure`
  has an empty body, so it would seed nothing; (b) shipping the whole
  `BevyProfile` trait seam now — the roadmap explicitly assigns `BevyProfile`,
  `BareBevyProfile`, and `MinimalBevyProfile` to task `0.5.1.2`, and this plan
  must not consume the next task's named deliverables; (c) an empty library
  with `bevy` and `rstest-bdd-harness` moved to development dependencies —
  viable, and the behavioural scenario would still run, but it defers every
  signature-shaped risk in design §5 and leaves the crate root undocumentable.
  Date/Author: 2026-08-15, planning pass.

- Decision: re-export `bevy` from the crate root.
  Rationale: `minimal_app` returns a Bevy type, so a consumer that resolves a
  different `bevy` gets a mismatched-type error naming neither manifest. Design
  §9 already names Bevy's prelude as the downstream import surface; the
  re-export is what makes that surface type-compatible.
  Date/Author: 2026-08-15, planning pass.

- Decision: declare `tracing` as a direct dependency and re-export it
  separately, rather than re-exporting it through `rstest-bdd-harness`.
  Rationale: re-exporting through another crate puts a version this crate does
  not control into its own public API, so an upstream major bump would break
  this crate's API with no change to its source or manifest.
  `rstest-bdd-harness-gpui` declares `tracing` directly and re-exports it; that
  is the precedent worth following. A `pub use` is a use, so the dependency is
  not unused.
  Date/Author: 2026-08-15, planning pass.

- Decision: declare the crate ISC-licensed and unpublishable.
  Rationale: the repository's `LICENSE` file is ISC, and every published
  sibling declares ISC. The validator crate declares `MIT OR Apache-2.0`, which
  contradicts `LICENSE`; that pre-existing discrepancy is out of scope here but
  must not be propagated into a crate destined for crates.io. Marking the crate
  unpublishable prevents an accidental publish claiming a name the upstream
  cookbook documents, at a version below its siblings, from the wrong
  repository. Both settings change at extraction. Raise the validator crate's
  licence declaration as a separate correction.
  Date/Author: 2026-08-15, planning pass.

- Decision: keep the crate directory hyphenated, diverging from the existing
  underscore-named validator directory.
  Rationale: the roadmap and design both name the hyphenated path; the
  extraction target repository is `leynos/rstest-bdd-harness-bevy`; and
  upstream `rstest-bdd` uses hyphenated crate directories throughout. Matching
  the extraction target keeps the eventual move a pure directory copy.
  Date/Author: 2026-08-15, planning pass.

- Decision: write a new ADR rather than amending ADR 002, and add a forward
  pointer from ADR 002's Consequences.
  Rationale: [the developer's guide](../developers-guide.md) §2 requires an ADR
  before workspace members change, and ADR 002 defers further crates "until a
  later ADR records a specific extraction". Adding a third crate is that
  trigger. ADR 002 is accepted; superseding text belongs in a new record, but
  without a pointer ADR 002 keeps telling readers to expect two crates.
  Date/Author: 2026-08-15, planning pass.

- Decision: ADR 007 must be accepted, not merely proposed, before the commit
  that changes workspace members lands.
  Rationale: the developer's guide rule is "before changing workspace members".
  Merging a membership change under a merely proposed ADR is what that rule
  guards against. ADR 002's precedent carries both a status line and a separate
  acceptance date, so acceptance is a distinct recorded event.
  Date/Author: 2026-08-15, planning pass.

- Decision: follow the repository's existing ADR file convention in preference
  to the literal template in
  [the documentation style guide](../documentation-style-guide.md).
  Rationale: all five existing ADRs use the repository convention, and
  [the repository layout](../repository-layout.md) acknowledges that the ADR
  directory predates the style guide's canonical filename convention.
  Reconciling the two conventions is out of scope.
  Date/Author: 2026-08-15, planning pass.

- Decision: state the crate boundary as an extension-seam rule rather than as
  hexagonal taxonomy.
  Rationale: the first draft framed the crate as a ports-and-adapters adapter.
  The label is strained — the harness trait is an extension point owned by
  `rstest-bdd`, not a port this crate defines against a domain it owns, and
  there is no local domain to invert dependencies around. The durable rule, and
  the one that will actually keep game code out at `0.5.1.4`, is: *the profile
  type is the single extension seam; every game-specific plugin, resource, and
  cleanup hook lives in a downstream implementation of it, never in this
  crate.* That is design §8 and §11 stated enforceably. The claim that the
  crate holds no domain logic survives as a consequence.
  Date/Author: 2026-08-15, planning pass.

- Decision: leave [the user's guide](../users-guide.md),
  [the development plan](../development-plan.md), and
  [the technical design](../skyjoust-technical-design.md) unchanged.
  Rationale: the user's guide is scoped to operators running the validator
  tooling, and this change alters none of those workflows. The development plan
  and technical design describe the *runtime* crate split accepted in ADR 002;
  this crate is test tooling, not runtime, so "one runtime crate beside the
  validator crate" remains accurate. The development plan's phase list predates
  roadmap phase 0.5; reconciling the two belongs to `0.5.1.5`.
  Date/Author: 2026-08-15, planning pass.

- Decision: include a `proptest` property over tick counts; do not use `kani`
  or `verus`; do not add `insta` snapshots.
  Rationale: an application updated *n* times must leave the frame count equal
  to *n*, which is a genuine invariant over a range and directly pre-figures
  design §5's `update_times`. Bounded model checking and deductive proof are
  disproportionate for a property that holds by Bevy's own frame counter, with
  no `unsafe` code and no unbounded state. Snapshots earn their keep when a
  multivariant output format must stay stable; this milestone emits none. The
  panic-diagnostic format in `0.5.1.3` is the artefact worth pinning.
  Date/Author: 2026-08-15, planning pass.

## Outcomes & retrospective

To be completed at Milestone 6. Record: the measured cold and warm times for
`make lint` and `make test` on the real workspace; the build-tree size; which
tolerances were approached; whether the feature-file rebuild guard proved
load-bearing under the A/B test; and what `0.5.1.2` should inherit — in
particular the answers taken at the approval gate and any lint trap discovered
while writing the scenario binding.

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
object that owns the ECS world, the plugin list, and the schedules; its
`update` method runs one pass of the main schedule, which includes the `Update`
schedule. `MinimalPlugins` is Bevy's smallest useful plugin group —
`TaskPoolPlugin`, `FrameCountPlugin`, `TimePlugin`, and `ScheduleRunnerPlugin`
— with no window and no renderer. Note that `App::new()` is not empty: it
already installs `MainSchedulePlugin`, which is why an unconfigured application
can be updated at all.

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
  adapter cookbook: <https://github.com/leynos/rstest-bdd/blob/main/docs/users-guide.md>.

Load the `rust-router` skill to reach the Rust skills; `rust-unit-testing`,
`arch-crate-design`, `arch-decision-records`, `proptest`, `commit-message`, and
`pr-creation` are the ones this task uses. Delegate full gate runs to the
`scrutineer` sub-agent.

## Plan of work

### Stage A: understand and propose (no code changes)

Completed during planning; evidence is in `Surprises & discoveries` and
`Artefacts and notes`. On approval, go straight to Milestone 1.

### Stage B: record the decision (Milestone 1)

Documentation only.

1. Create `docs/adr/007-in-tree-incubation-of-the-bevy-bdd-harness-crate.md`,
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
   - the dependency decisions and their evidence, including that `Cargo.lock`
     is load-bearing for the `rstest-bdd` family until `0.6.0` is stable;
   - that the Bevy major is a workspace-wide coupling, who owns the bump, and
     what triggers it;
   - the headless guarantee stated accurately: *this crate declares no Bevy
     renderer, window, or asset features; the resolved graph in a workspace
     build is a workspace-wide property*;
   - a captioned comparison table of the dependency-sourcing options.
2. Amend [the harness design](../rstest-bdd-harness-bevy-design.md):
   - §3: replace the stale manifest-version claim with the current position —
     `0.6.0-beta3` is published and carries the harness API this work targets;
     `main` has since moved to an unpublished `0.6.0-beta4`.
   - §9: replace the git-dependency instruction with the published
     requirements; note the later APIs unavailable under `beta3`; note that
     disabling default features needs `features = ["std"]` to keep the clock
     real, with the evidence; and add the migration trigger — *when
     `0.6.0-beta4` publishes, bump all `rstest-bdd` requirements together and
     adopt the policy-conformance helper in `0.5.1.3`. If `0.5.1.3` reaches the
     policy-conformance task first, escalate rather than adding a git
     dependency.*
   - §9: update the layout block to the names `0.5.1.1` actually establishes,
     so `0.5.1.2` does not create a near-identical second scenario pair beside
     them.
   - §13: refresh the references, replacing the stale commit citation.
   - Add a pointer to ADR 007.
3. Add a forward pointer to ADR 007 in ADR 002's Consequences.
4. Index ADR 007 and this ExecPlan in [the contents index](../contents.md).

Validation: the documentation gates (see `Concrete steps`). ADR 007 must be
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
     acknowledges the third, tooling-facing harness crate and cites ADR 007;
   - add a section documenting the harness crate's boundary rule and the
     testing traps this task uncovered: that the expect-in-tests allowance does
     not cover `rstest-bdd` step functions; that feature-file-only edits do not
     invalidate the build without the `include_str!` guard; that
     `expect_that!` needs `#[gtest]` while `assert_that!` does not; and that
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

Check free space before Milestone 2; the build tree reaches roughly 2.2 GB for
this crate alone:

```bash
df -h .
```

### Milestone 1 — record the decision

Write the ADR, amend the design document and ADR 002, update the contents
index, then:

```bash
gate markdownfmt mdformat-all
gate markdownlint make markdownlint
gate nixie make nixie
gate diff-check git diff --check
```

`make markdownlint` depends on `make spelling`, which chains through the
phrase check, the spelling-config check, and a pytest run — it is not a light
gate. If `typos` rejects new vocabulary, add narrow entries to
`typos.local.toml`, then run `make spelling-config-write` to regenerate
`typos.toml`, and commit both.

Commit with a file-based message:

```bash
COMMIT_MSG_DIR=$(mktemp -d)
cat > "$COMMIT_MSG_DIR/msg" <<'EOF'
Record the Bevy BDD harness crate decision as ADR 007

Add ADR 007 covering in-tree incubation of
`rstest-bdd-harness-bevy`, the extraction contract, and the
dependency decisions taken for roadmap task 0.5.1.1.

Amend the harness design document: `rstest-bdd` 0.6.0-beta3 is
published, so the crate uses published requirements rather than git
dependencies against `main`. Record the later APIs unavailable
under beta3, the Bevy standard-library feature requirement, and the
crate layout names this task establishes.
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

Now prove the rebuild guard is load-bearing with an A/B, not a single
observation. Observing only "guard present, edit, failure" is equally
consistent with the guard doing the work and with the macro already tracking
the path.

1. Comment out the `include_str!` line and run
   `cargo test -p rstest-bdd-harness-bevy --test headless_scenario` to get a
   warm binary. Edit a word in the feature file's `Then` line so it no longer
   matches the step pattern, and re-run **without touching any Rust file**.
   Record whether it passes stale.
2. Restore the guard, repeat the same edit, and record the failure.

Only "(1) passes and (2) fails" licenses the developer's-guide claim in
Stage F. If (1) also fails, the macro already tracks the path: either drop the
guard, or keep it and downgrade the guide text to belt and braces. Revert the
feature file afterwards.

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
report. The gates, following
[the development plan](../development-plan.md) §3, are:

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
must mention this ExecPlan by path, surface the two items from
`Open decisions requiring approval`, and end with a `## References` section
linking the Lody session.

## Validation and acceptance

### Red-green-refactor evidence

- **Red.** `cargo test -p rstest-bdd-harness-bevy` fails to compile with
  unresolved imports naming `add_minimal_plugins` and `minimal_app`, raised
  from `crates/rstest-bdd-harness-bevy/src/profile_tests.rs`. A green run with
  zero tests means the module declaration is missing, not that the step
  succeeded.
- **Green.** After implementing `profile.rs` and adding the re-export, the same
  command reports five library tests and two doctests passing, with zero
  failures.
- **Refactor.** `make check-fmt` prints nothing and exits zero; `make lint`
  completes `cargo doc`, `cargo clippy --workspace --all-targets --all-features`
  with warnings denied, and the Whitaker Dylint suite with no diagnostics.

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

2. Confirm the crate cannot reach Skyjoust or Lille:

   ```bash
   cargo tree -p rstest-bdd-harness-bevy -e normal,dev --prefix none \
     | sort -u | grep -Ei '^(skyjoust|skyjoust-stateright-validator|lille) ' \
     || echo "clean"
   ```

   Expect `clean`. The trailing space in the pattern matters: `cargo tree`
   prints `<name> v<version>`, so an unanchored `^skyjoust` would also match
   the permitted `skyjoust-test-macros`. This, not the manifest-text test, is
   the authority for the transitive half of the extraction constraint.

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
   failures across the library tests, the three integration targets, and the
   doctests — thirteen tests in total.

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

**Lockfile conflicts.** Adding roughly 139 crates rewrites a large contiguous
block of `Cargo.lock`, so every rebase onto a moving `main` will conflict
there. Never hand-merge it. Take `main`'s version wholesale, re-resolve with
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

### Evidence: the full crate shape passes every gate

A probe reproducing this plan's exact manifest, lint tables, `clippy.toml`,
`.rustfmt.toml`, and pinned toolchain was built at `~/.cache/shape-probe`
during planning:

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
precisely why the suppression belongs in a macro rather than at the call
site.

### Evidence: Bevy feature selection and cost

| Bevy dependency declaration                                            | Normal | With dev |
| ---------------------------------------------------------------------- | ------ | -------- |
| `{ version = "0.17.3", default-features = false }`                     | 115    | 259      |
| `{ version = "0.17.3", default-features = false, features = ["std"] }` | 139    | 279      |
| `"0.17.3"` (Bevy defaults)                                             | 428    | n/a      |

*Table 1: Resolved crate counts for the full harness crate, measured with
`cargo tree --workspace -e normal --prefix none | sort -u | wc -l` and the
dev-inclusive equivalent. The 428 figure is from the bevy-only probe.*

Measured build cost on six cores with a warm Cargo registry:

```plaintext
COLD cranelift check --workspace --all-targets --all-features:  26s
COLD cranelift test build (--no-run):                           30s
COLD llvm clippy --workspace --all-targets --all-features:      19s
WARM cranelift check:                                            0s
du -sh target:                                                 2.2G
```

Cranelift compiles the whole Bevy graph without incident, which is the
configuration `make test`, `make lint`, and `make typecheck` all use.

### Note on beta3 message wording

Published `0.6.0-beta3` generated code panics with the message
`harness failed to initialise scenario`, where `main` uses the `-ize` form.
Any future test asserting on that string must match the version in use, and
must keep the literal inside backticks — the spelling gate rejects the `-ise`
form in bare prose. This milestone asserts on no such string.

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
bevy = { version = "0.17.3", default-features = false, features = ["std"] }
# Exact pins: a caret requirement on a pre-release is not a pin. A caret on
# 0.6.0-beta3 admits 0.6.0-beta4, which changes StepContext borrowing.
rstest-bdd-harness = "=0.6.0-beta3"
tracing = "0.1"

[dev-dependencies]
googletest = "0.14.3"
pretty_assertions = "1"
proptest = "1"
rstest = "0.26.1"
rstest-bdd = "=0.6.0-beta3"
rstest-bdd-macros = "=0.6.0-beta3"
skyjoust-test-macros = { path = "../skyjoust_test_macros" }
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
- `rstest 0.26.1` matches what `rstest-bdd 0.6.0-beta3` expects, and is
  deliberately different from the root package's `rstest = "0.18"`.
- `skyjoust-test-macros` supplies `allow_fixture_expansion_lints`, without
  which the scenario fixture cannot satisfy `make lint` and `make check-fmt`
  at the same time. It is a path dependency on the crate added by the layer
  beneath this one in the stack, and is dev-only: the library never uses it.
  It is the one dependency here that does *not* travel to the extracted
  repository — see the extraction note in
  [ADR 006](../adr/006-test-macro-crate-for-fixture-expansion-lints.md).
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
    for _ in 0..ticks {
        app.update();
    }
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
fn given_minimal_app(app: &RefCell<App>) {
    assert_that!(app.borrow().is_plugin_added::<TimePlugin>(), eq(true));
}

#[when("the schedule advances once")]
fn when_schedule_advances_once(app: &RefCell<App>) { app.borrow_mut().update(); }

#[then("the frame count reads 1")]
fn then_frame_count_reads_one(app: &RefCell<App>) {
    let observed = app.borrow().world().resource::<FrameCount>().0;
    assert_that!(observed, eq(1_u32));
}

#[scenario(path = "tests/features/headless_scenario.feature", index = 0)]
fn minimal_app_advances_one_tick(app: RefCell<App>) {}
```

Two things a first reader will not guess. The fixture parameter must appear on
the `#[scenario]` function as well as on the steps, because that is how
`rstest` knows to construct it. And step functions use `assert_that!`, which
panics directly, rather than `expect_that!`, which needs a `#[gtest]` context
that generated step functions do not have.

The property test in `tests/tick_properties.rs` states the invariant that
`0.5.1.2`'s `update_times` must preserve, with an explicit case count:

```rust
proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

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
entry line and comparing whole strings. A substring scan would reject
`skyjoust-test-macros`, which is a permitted development dependency (see
`Constraints`), and would equally be fooled by a comment. Exact matching keeps
`skyjoust = { path = "../.." }` caught while letting the tooling crate through.

Four tests: `manifest_declares_no_game_crates`, against an `include_str!` of
the crate's own manifest; `guard_detects_a_directly_declared_game_crate`,
against a synthetic manifest, which is what proves the guard can fail;
`guard_ignores_game_names_outwith_dependency_tables`, which proves the scoping;
and `guard_permits_the_test_macro_crate`, which pins the exact-match behaviour
so a later "tightening" to substring matching fails loudly instead of silently
breaking the build. Use `pretty_assertions` for the vector comparisons so a
regression prints a readable diff.

Automating the transitive check would need `cargo_metadata` as a development
dependency and a nested `cargo` invocation inside `make test`; both are
declined here as disproportionate, and the manual `cargo tree` step in
`Behavioural acceptance` carries that half. Automating it is follow-up work for
`0.5.1.5`.

### Files this plan creates or changes

| Path                                                                                   | Change                                                         |
| -------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `Cargo.toml`                                                                           | Add the workspace member.                                      |
| `Cargo.lock`                                                                           | Regenerated; roughly 139 new entries.                          |
| `docs/adr/007-in-tree-incubation-of-the-bevy-bdd-harness-crate.md`                     | New ADR.                                                       |
| `docs/adr/002-crate-layout-and-public-api.md`                                          | Add a forward pointer to ADR 007 in Consequences.              |
| `crates/rstest-bdd-harness-bevy/Cargo.toml` (dev-dependency)                           | Path dependency on `skyjoust-test-macros`.                     |
| `docs/rstest-bdd-harness-bevy-design.md`                                               | Amend sections 3, 9 (dependencies and layout), and 13.         |
| `docs/contents.md`                                                                     | Index ADR 007 and this ExecPlan.                               |
| `docs/repository-layout.md`                                                            | Tree sketch, path responsibilities, workspace membership note. |
| `docs/developers-guide.md`                                                             | Amend section 2; add a harness-crate conventions section.      |
| `docs/roadmap.md`                                                                      | Mark 0.5.1.1 done; correct the git-dependency sub-bullet.      |
| `docs/execplans/0-5-1-1-add-rstest-bdd-harness-bevy-workspace-member.md`               | This living document, updated as work proceeds.                |
| `crates/rstest-bdd-harness-bevy/Cargo.toml`                                            | New crate manifest.                                            |
| `crates/rstest-bdd-harness-bevy/README.md`                                             | New crate README with the Bevy compatibility table.            |
| `crates/rstest-bdd-harness-bevy/src/lib.rs`                                            | New crate root.                                                |
| `crates/rstest-bdd-harness-bevy/src/profile.rs`                                        | New plugin configuration module.                               |
| `crates/rstest-bdd-harness-bevy/src/profile_tests.rs`                                  | New sibling unit tests.                                        |
| `crates/rstest-bdd-harness-bevy/tests/headless_scenario.rs`                            | New behavioural binding.                                       |
| `crates/rstest-bdd-harness-bevy/tests/features/headless_scenario.feature`              | New feature file.                                              |
| `crates/rstest-bdd-harness-bevy/tests/tick_properties.rs`                              | New property test.                                             |
| `crates/rstest-bdd-harness-bevy/tests/extraction_boundary.rs`                          | New boundary tripwire.                                         |
| `crates/rstest-bdd-harness-bevy/tests/proptest-regressions/`                           | Created only if the property test ever fails; committed.       |
| `typos.local.toml` and `typos.toml`                                                    | Only if spelling rejects new vocabulary; regenerate both.      |
| `docs/users-guide.md`, `docs/development-plan.md`, `docs/skyjoust-technical-design.md` | No change — see `Decision log`.                                |

*Table 2: The authoritative file manifest. The scope trigger in `Tolerances` is
measured against this table.*

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
  paragraph required and the first draft omitted, and
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

Revision 3, 2026-08-15. Adopted the estate-mandated remedy for the fixture
expansion lint, which now lands in the layer beneath this plan in the pull
request stack rather than being worked around locally.

- The `let`-binding workaround is withdrawn. The scenario fixture keeps its
  natural single-expression form and carries
  `#[allow_fixture_expansion_lints]` from `skyjoust-test-macros`, mirroring
  `weaver-test-macros` in `leynos/weaver`. The workaround would have spread a
  bespoke idiom across every fixture the project writes instead of fixing the
  cause once.
- The harness ADR is renumbered 006 to 007, because the lower layer takes 006
  for the test-macro decision.
- `skyjoust-test-macros` is added as a development dependency, and the
  extraction contract in `Constraints` is restated to distinguish game code,
  which is forbidden, from test-only tooling, which is not.
- The boundary guard now matches dependency names exactly rather than by
  substring, and the `cargo tree` acceptance pattern anchors on the trailing
  space. Both changes exist for the same reason: `skyjoust-test-macros`
  contains the substring `skyjoust`, so the guard as previously specified
  would have rejected a dependency the contract allows. A fourth boundary test
  pins that behaviour.

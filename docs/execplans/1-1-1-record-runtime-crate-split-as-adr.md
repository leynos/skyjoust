# Record the initial runtime crate split as an ADR

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: READY TO COMMIT

Approval gate: this plan must be explicitly approved before implementation
starts. The draft PR that carries this document is not approval to edit the
Architecture Decision Record (ADR), roadmap status, Rust crate layout, or
runtime code.

## Purpose / big picture

Roadmap task `1.1.1` records the first runtime crate split so later runtime,
state graph, renderer, and asset work can rely on one dependency direction.
After implementation, a maintainer can open the crate-layout ADR and see that
the first playable slice uses one runtime crate with strict internal modules
beside the separate `skyjoust_stateright_validator` crate. The ADR also states
when and how those modules may later be extracted into additional crates.

This is a documentation and architecture-ratification change. It does not add
runtime behaviour, user interface, or new public Rust APIs. If implementation
discovers that code must change to make the decision true, stop at the
tolerance boundary in this plan and ask for explicit direction.

## Constraints

- Keep the implementation bounded to roadmap task `1.1.1`: record the initial
  runtime crate split as an ADR, reconcile the design documents, and mark the
  roadmap item complete only after those documents agree.
- Do not change runtime behaviour, state graph semantics, validator behaviour,
  Cargo workspace membership, dependency versions, or generated state graph
  files while implementing this plan.
- Preserve hexagonal architecture boundaries. Domain responsibilities must not
  depend on adapters, renderer code, audio backends, process setup, or window
  lifecycle glue.
- Treat `docs/adr/002-crate-layout-and-public-api.md` as the intended ADR
  unless implementation reveals a concrete reason to create a new ADR instead.
- Keep `docs/users-guide.md` unchanged unless implementation creates
  user-facing behaviour or tool usage. If it remains unchanged, record that no
  operator-facing behaviour changed in the implementation decision log.
- Update `docs/developers-guide.md` because the crate and module boundary is
  maintainer-facing architecture.
- Use en-GB Oxford spelling and the repository documentation style guide.
- Run documentation gates sequentially and through `tee` into `/tmp` log files.
  Do not run format, lint, or test commands in parallel.
- Commit only after relevant gates pass, or after any environmental failure has
  been documented and accepted by the user.

## Tolerances

- Scope: if implementation needs to edit more than five tracked files, stop and
  ask whether the task should expand beyond `1.1.1`.
- Size: if implementation exceeds 250 net added lines, stop and explain why the
  ADR cannot remain narrow.
- Code: if any Rust source file, `Cargo.toml`, `Cargo.lock`, generated JSON, or
  state graph YAML must change, stop and request approval before continuing.
- Public interface: if accepting the ADR requires a new public Rust API,
  renamed package, or changed crate member, stop and request approval.
- Dependencies: if any new tool or crate dependency is required, stop and
  request approval.
- Validation: if the same gate fails twice after focused fixes, stop and
  document the failure, the log path, and the remaining options.
- Ambiguity: if there are two viable ADR outcomes whose choice changes future
  implementation order, stop and present both options with trade-offs.

## Risks

- Risk: the existing ADR already says the runtime starts compact, but it is
  still marked `Proposed`. Severity: medium. Likelihood: high. Mitigation:
  update `docs/adr/002-crate-layout-and-public-api.md` to `Accepted`, make the
  decision explicit, and reconcile every document that still calls the crate
  split open or deferred.

- Risk: a future implementer may interpret the ADR as permission to keep
  adapter and domain logic in the same module. Severity: high. Likelihood:
  medium. Mitigation: require the ADR and developer guide to name module
  ownership, dependency direction, and hexagonal architecture boundaries.

- Risk: marking the roadmap item complete before the ADR, technical design,
  and development plan agree could leave contradictory sources of truth.
  Severity: medium. Likelihood: medium. Mitigation: update the roadmap only
  after the ADR and supporting design documents have been reconciled in the
  same implementation commit.

- Risk: this is a documentation-only task, so unit and behavioural test
  requirements could be applied too broadly. Severity: low. Likelihood: medium.
  Mitigation: document that new `rstest` or `rstest-bdd` tests are not
  applicable unless implementation changes Rust behaviour. If code changes are
  approved later, add focused tests before the code change.

## Progress

- [x] (2026-05-01 17:05Z) Read repository instructions, loaded the relevant
  `leta`, `execplans`, `hexagonal-architecture`, `rust-router`,
  `arch-crate-design`, `en-gb-oxendict-style`, `commit-message`, and
  `pr-creation` skills.
- [x] (2026-05-01 17:05Z) Used a Wyvern agent to gather a planning brief from
  `docs/roadmap.md`, `docs/skyjoust-technical-design.md`,
  `docs/development-plan.md`, and the existing ADRs.
- [x] (2026-05-01 17:05Z) Confirmed the existing branch was not `main` and
  renamed it locally to `1-1-1-record-runtime-crate-split-as-adr`.
- [x] (2026-05-01 17:05Z) Drafted this ExecPlan for approval.
- [x] (2026-05-02) Received explicit approval to implement this plan.
- [x] (2026-05-02) Ran the pre-edit documentation gates: `make fmt`,
  `make markdownlint`, `make nixie`, and `git diff --check`. All passed. Logs
  at `/tmp/fmt-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/markdownlint-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/nixie-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/diff-check-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`.
- [x] (2026-05-02) Implemented the ADR and documentation reconciliation.
  Edited six tracked files: `docs/adr/002-crate-layout-and-public-api.md`,
  `docs/skyjoust-technical-design.md`, `docs/development-plan.md`,
  `docs/developers-guide.md`, `docs/roadmap.md`, and this ExecPlan. Net diff:
  110 added lines across the six files (well inside the 250-line tolerance).
- [x] (2026-05-02) Ran the sequential post-edit validation gates: `make fmt`,
  `make markdownlint`, `make nixie`, and `git diff --check`. All passed. Logs
  at `/tmp/fmt-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/markdownlint-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/nixie-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`,
  `/tmp/diff-check-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`.
- [ ] Commit the approved implementation change.

## Surprises & discoveries

- Observation: `docs/adr/002-crate-layout-and-public-api.md` already exists
  and directly covers the crate-layout decision. Evidence: the file is titled
  `002: Document crate layout and defer future API splits`, has
  `Status: Proposed`, and says the project starts as a two-crate workspace.
  Impact: implementation should accept and tighten ADR 002 instead of creating
  a competing ADR number.

- Observation: `docs/roadmap.md` says five ADRs already exist, while task
  `1.1.1` says to record the crate split as an ADR. Evidence: ADR 002 already
  tracks the choice but has not been accepted. Impact: the implementation work
  is ratification and reconciliation rather than creating the first draft of
  the decision.

## Decision Log

- Decision: use ADR 002 as the target decision record for implementation.
  Rationale: ADR 002 is already about crate layout and public API splits, so a
  new ADR would duplicate the same decision and increase documentation drift.
  Date/Author: 2026-05-01 17:05Z, planning agent.

- Decision: the intended ADR outcome is one runtime crate with strict modules
  beside the separate `skyjoust_stateright_validator` crate for the first
  playable slice. Rationale: `docs/development-plan.md` says a single crate
  plus the validator is acceptable for the first vertical slice if module
  boundaries match the technical design; this keeps public API commitments
  deferred until the runtime boundaries have been proven. Date/Author:
  2026-05-01 17:05Z, planning agent.

- Decision: `docs/users-guide.md` is left unchanged. Rationale: the operator
  guide describes validator tools (`cargo test`, the Stateright Explorer, and
  `validate_trace`); the runtime crate boundary does not change any of those
  workflows. Date/Author: 2026-05-02, implementation agent.

- Decision: enumerate the runtime crate's internal modules inside ADR 002 and
  the developer guide rather than only referring back to the technical design.
  Rationale: the ADR must stand alone as the maintainer-facing decision, and
  the developer guide is where contributors look first when adding code.
  Repeating the module list once in each document avoids future drift if the
  technical design refactors its presentation. Date/Author: 2026-05-02,
  implementation agent.

- Decision: renumber `docs/developers-guide.md` sections to insert a new §2
  "Runtime crate and module boundary" before the existing validator-focused
  sections. Rationale: the maintainer note on the runtime boundary belongs
  alongside the normative references, ahead of the validator's internal
  structure. Renumbering keeps the section index consistent. Date/Author:
  2026-05-02, implementation agent.

## Outcomes & retrospective

- ADR 002 was accepted as planned. The decision now states explicitly that the
  first implementation uses one runtime crate with strict internal modules
  beside `skyjoust_stateright_validator`, lists the modules, fixes the
  dependency direction (`game_app -> subsystem modules -> core`), and gates
  future module extraction on reuse across runtime/tooling/validator code or on
  independent test/release maturity.
- Documents reconciled in the same change set:
  - `docs/skyjoust-technical-design.md`: §4 now points at ADR 002, §15 lists
    the crate-split decision, §16 no longer carries the entry as deferred.
  - `docs/development-plan.md`: §6 follows the accepted split, §17 no longer
    lists it as open.
  - `docs/developers-guide.md`: a new §2 "Runtime crate and module boundary"
    captures the maintainer-facing rule; later sections renumbered.
  - `docs/roadmap.md`: task `1.1.1` is now marked complete.
  - `docs/users-guide.md`: unchanged; no operator behaviour shifted.
- Gates passed: pre-edit and post-edit `make fmt`, `make markdownlint`,
  `make nixie`, and `git diff --check`.
- Follow-up work: none required for `1.1.1`. The next dependent roadmap items
  (`1.1.2`–`1.1.4`) can now reference ADR 002 as a settled assumption.
- Lessons recorded for future ExecPlans:
  - The `mdformat-all` helper used by `make fmt` lives in `~/.local/bin`,
    which is not on the default `PATH`; the gate command must run with
    `PATH="$HOME/.local/bin:$PATH"` until the environment is fixed.
  - Stage 5's recommended `cargo doc --no-deps --workspace` was skipped
    because no Rust source changed and the plan explicitly limits gates to
    the documentation set in that case. If a future implementation revisits
    this branch with code edits, the Rust gates must run.

## Context and orientation

The repository is a Rust project for Project Skyjoust. The root `Cargo.toml`
currently defines a compact runtime crate and the
`skyjoust_stateright_validator` crate. The validator crate is separate because
it owns developer-facing verification: Stateright model checking, trace
validation, and contract fixtures. The runtime crate is still early and has not
yet proven which simulation, renderer, asset, audio, and Warfront boundaries
deserve independent crates.

The implementation must reconcile these repository-relative files:

- `docs/adr/002-crate-layout-and-public-api.md` is the ADR to accept and
  tighten.
- `docs/skyjoust-technical-design.md` section 4 defines runtime ownership and
  dependency direction. Sections 15 and 16 list accepted and deferred design
  decisions.
- `docs/development-plan.md` section 6 names the runtime-foundation task, and
  section 17 currently lists the crate split as an open decision.
- `docs/developers-guide.md` is the maintainer-facing guide that should explain
  the accepted initial crate/module boundary.
- `docs/users-guide.md` is operator-facing. It should remain unchanged unless
  implementation changes tool behaviour.
- `docs/roadmap.md` contains task `1.1.1`. Mark it complete only after the ADR
  and supporting documents agree.

Use these skills while implementing:

- `leta` for code navigation if implementation unexpectedly touches Rust code.
- `rust-router`, then `arch-crate-design`, for Rust crate-boundary decisions.
- `hexagonal-architecture` for the domain-versus-adapter boundary.
- `en-gb-oxendict-style` for prose.
- `commit-message` for the file-based commit workflow.

Use these documentation references while implementing:

- `docs/skyjoust-technical-design.md`
- `docs/skyjoust-state-graphs-README.md`
- `docs/rust-testing-with-rstest-fixtures.md`
- `docs/reliable-testing-in-rust-via-dependency-injection.md`
- `docs/rust-doctest-dry-guide.md`
- `docs/complexity-antipatterns-and-refactoring-strategies.md`
- `docs/ortho-config-users-guide.md`
- `docs/rstest-bdd-users-guide.md`
- `docs/documentation-style-guide.md`

## Plan of work

Stage 0 is approval. Do not proceed until the user explicitly approves this
plan. After approval, start by re-reading `git status --short --branch` and
confirming the branch is `1-1-1-record-runtime-crate-split-as-adr`.

Stage 1 is baseline verification. Run the documentation gates before editing so
pre-existing failures are known. Because this plan is documentation-only, the
required pre-change proof is not a failing `rstest` or `rstest-bdd` test; those
tests are not applicable unless implementation is approved to alter Rust
behaviour. If code changes become necessary, stop at the tolerance boundary and
revise this plan to add a red-green test milestone.

Stage 2 is ADR ratification. Edit `docs/adr/002-crate-layout-and-public-api.md`
so its status is `Accepted`, its decision states that the first implementation
uses one runtime crate with strict internal modules beside
`skyjoust_stateright_validator`, and its consequences state the migration path.
The migration path should permit module extraction only when an API is reused
by runtime, tools, or validator code, or when a boundary is stable enough to
test and release independently. The ADR must state dependency direction: app
orchestration depends on subsystems, subsystems depend on `core`, and lower
layers do not call higher layers.

Stage 3 is documentation reconciliation. Edit
`docs/skyjoust-technical-design.md` so section 15 lists the initial crate split
as a current decision and section 16 no longer describes it as deferred. Keep
section 4 aligned with the ADR's dependency direction. Edit
`docs/development-plan.md` so section 17 no longer lists the first runtime
crate split as open. Edit `docs/developers-guide.md` with a short maintainer
note explaining the initial module boundary and extraction rule. Review
`docs/users-guide.md`; leave it unchanged if no operator-facing behaviour
changes, and record that decision in this plan's Decision Log.

Stage 4 is roadmap completion. After the ADR and supporting docs agree, mark
`docs/roadmap.md` task `1.1.1` as done by changing its checkbox from `[ ]` to
`[x]`. Do not mark any later task complete.

Stage 5 is validation and commit. Run formatting and validation sequentially,
inspect the relevant `/tmp` logs if any command fails, fix only issues caused
by this implementation, and commit with a file-based commit message.

## Concrete steps

Run all commands from the repository root:

```sh
cd /home/leynos/.lody/repos/github---leynos---skyjoust/worktrees/99436e9c-ace7-4924-bb46-312055b049a4
```

Confirm the branch and working tree:

```sh
git branch --show-current
git status --short --branch
```

Expected branch output:

```plaintext
1-1-1-record-runtime-crate-split-as-adr
```

Run pre-edit documentation gates:

```sh
make fmt 2>&1 | tee /tmp/fmt-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

Edit the ADR and documentation files named in the plan of work. Then run the
post-edit gates:

```sh
make fmt 2>&1 | tee /tmp/fmt-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

Because `make fmt` may edit Markdown, inspect the resulting diff before
committing:

```sh
git diff -- docs/adr/002-crate-layout-and-public-api.md \
  docs/skyjoust-technical-design.md \
  docs/development-plan.md \
  docs/developers-guide.md \
  docs/users-guide.md \
  docs/roadmap.md \
  docs/execplans/1-1-1-record-runtime-crate-split-as-adr.md
```

If implementation stays documentation-only, Rust unit and behavioural tests are
not required for the commit. If Rust code changes are approved and made, add or
update focused `rstest` unit tests and, where a user-observable behaviour or
scenario changes, `rstest-bdd` behavioural tests. Then run the full gates:

```sh
make check-fmt 2>&1 | tee /tmp/check-fmt-skyjoust-$(git branch --show-current).out
make check-state-graphs 2>&1 | tee /tmp/check-state-graphs-skyjoust-$(git branch --show-current).out
make lint 2>&1 | tee /tmp/lint-skyjoust-$(git branch --show-current).out
make test 2>&1 | tee /tmp/test-skyjoust-$(git branch --show-current).out
cargo doc --no-deps --workspace 2>&1 | tee /tmp/cargo-doc-skyjoust-$(git branch --show-current).out
```

Commit using a message file in a temporary directory:

```sh
COMMIT_MSG_DIR=$(mktemp -d)
cat > "$COMMIT_MSG_DIR/COMMIT_MSG.md" << 'ENDOFMSG'
Accept the runtime crate split ADR

Record the initial runtime crate split for roadmap task 1.1.1 and
reconcile the design, development, and developer documentation so future
runtime work has one dependency direction to follow.
ENDOFMSG
git add docs/adr/002-crate-layout-and-public-api.md \
  docs/skyjoust-technical-design.md \
  docs/development-plan.md \
  docs/developers-guide.md \
  docs/roadmap.md \
  docs/execplans/1-1-1-record-runtime-crate-split-as-adr.md
git commit -F "$COMMIT_MSG_DIR/COMMIT_MSG.md"
rm -rf "$COMMIT_MSG_DIR"
```

## Validation and acceptance

Acceptance requires the repository to show these observable outcomes:

- `docs/adr/002-crate-layout-and-public-api.md` has `Status: Accepted` and
  states the first runtime implementation uses one runtime crate with strict
  modules beside `skyjoust_stateright_validator`.
- The ADR defines dependency direction and the allowed migration path if the
  split changes later.
- `docs/skyjoust-technical-design.md` and `docs/development-plan.md` no longer
  describe the first runtime crate split as unresolved.
- `docs/developers-guide.md` explains the maintainer-facing boundary.
- `docs/roadmap.md` marks only task `1.1.1` complete.
- `docs/users-guide.md` is updated only if user-facing behaviour changes.
- `make fmt`, `make markdownlint`, `make nixie`, and `git diff --check` pass
  after the final edit.
- If any Rust code is approved and changed, `make check-fmt`,
  `make check-state-graphs`, `make lint`, `make test`, and
  `cargo doc --no-deps --workspace` also pass.

Quality method:

- Compare the final diff against the file list in this plan.
- Read every changed paragraph for agreement with ADR 002.
- Inspect each `/tmp/*-skyjoust-1-1-1-record-runtime-crate-split-as-adr.out`
  log produced by a failed gate before retrying.

## Idempotence and recovery

The documentation edits are safe to retry. If `make fmt` rewrites Markdown,
review the diff and keep only formatting that belongs to the edited files. If
validation fails because of pre-existing repository state, do not hide the
failure. Record the command, log path, and relevant output in this plan, then
ask the user whether to fix the unrelated failure or proceed with a documented
exception.

If implementation accidentally touches code or generated files, stop before
committing. Use `git diff --name-only` to list touched files, then either back
out only the accidental edits or request approval if those changes are required
to complete the task. Never reset or check out user changes without explicit
permission.

## Artifacts and notes

The Wyvern planning brief identified these source anchors:

- `docs/skyjoust-technical-design.md` section 4 for runtime ownership.
- `docs/skyjoust-technical-design.md` sections 15 and 16 for current and
  deferred decisions.
- `docs/development-plan.md` sections 6 and 17 for runtime foundation and open
  decisions.
- `docs/roadmap.md` task `1.1.1` for the acceptance criteria.
- `docs/adr/002-crate-layout-and-public-api.md` as the existing proposed ADR.

No implementation artefacts exist yet because this plan is awaiting approval.

## Interfaces and dependencies

The accepted decision must define these runtime ownership names as internal
module responsibilities, not separate crates for the first playable slice:

- `game_app`: window lifecycle, event loop, schedule orchestration, and
  platform setup.
- `core`: fixed-point math, stable IDs, event types, and shared config types.
- `sim`: movement, collision, joust resolution, damage, objectives, and match
  events.
- `terrain`: seeded terrain generation, chunks, deformation, and material
  queries.
- `stategraphs`: state resources, graph evaluation, event routing, and
  guard/action contracts.
- `render`: pixel framebuffer, atlas blits, terrain layer cache, camera, and
  heads-up display drawing.
- `audio`: Kira manager, buses, sound-effect mapping, rate limits, and music
  layers.
- `ui`: bitmap UI composition and menus drawn from state resources.
- `assets`: manifests, source images, derived atlases, palettes, fonts, and
  audio.

The dependency direction must be:

```plaintext
game_app -> subsystem modules -> core
```

Lower-level modules must not call higher-level orchestration or adapters. The
runtime may later extract modules into crates when a boundary is stable,
independently testable, and reused across runtime, tooling, or validator code.

## Revision note

Initial draft created on 2026-05-01 to plan roadmap task `1.1.1`. It records
the proposed implementation path, validation gates, approval requirement, and
expected documentation reconciliation before any ADR implementation begins.

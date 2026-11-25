# Architectural decision record (ADR) 010: Bevy harness scenario failure contract

## Status

Accepted.

Amended by ADR 011 (2026-11-20): the never-finish paragraph's final sentence is
corrected by [ADR 011](adr-0011-correct-bevy-harness-never-finish-guidance.md).

## Date

2026-11-20.

## Context and problem statement

`rstest-bdd-harness-bevy` runs one headless Bevy `App` per Gherkin scenario.
The harness adapter (`BevyHarness<P>`) configures the application through a
profile type, hands the steps a cloneable `BevyScenario`, and then releases
scenario resources through the profile's cleanup hook. Three lifecycle phases
can fail independently: `configure`, `steps`, and `cleanup`.

The design document ([design §6](../rstest-bdd-harness-bevy-design.md))
requires that a step panic is re-raised with the harness type, feature path,
scenario name, and one-based line number so `cargo test` and `cargo nextest`
identify the failing scenario. It leaves open how the message reaches test
output, whether configure and cleanup panics are augmented, how profile errors
are surfaced through `rstest-bdd-harness` 0.6.0's single non-exhaustive error
variant, and which lifecycle behaviour is contractual. This record fixes those
decisions.

The upstream harness contract constrains the answer. `HarnessAdapter::run<T>`
returns `HarnessResult<T>`, and `HarnessError` exposes only
`RuntimeBuildFailed(std::io::Error)`. `std::panic::resume_unwind` does not
invoke the panic hook, and libtest prints only the panic hook's output, not a
failed test's payload. A harness that re-raises silently therefore produces a
failure message no one sees.

## Decision drivers

- The failure must be visible in ordinary `cargo test` and `cargo nextest`
  output without writing to standard error directly; the workspace denies
  `print_stderr`.
- Every scenario failure must name its lifecycle phase and scenario, because
  plugin-registration panics in `configure` and cleanup faults are otherwise
  indistinguishable from step failures.
- Profile cleanup must run exactly once after a successful configure, including
  when steps panic.
- A cleanup failure must never mask the step panic that caused the scenario to
  fail.
- The public API must match design §5, whose profile hooks return
  `HarnessResult<()>`.
- The mechanism must work under the default unwinding panic strategy and must
  not depend on globals, thread-local state, or process-wide locks.
- The harness must never call `App::run`, `App::finish`, or `App::cleanup`;
  finishing would forbid `add_plugins` inside steps, and `App::run` loops
  forever under `MinimalPlugins`.

## Requirements

### Functional requirements

- The runner is invoked only after `configure` returned `Ok`.
- Cleanup is invoked exactly once after a successful configure, after the
  runner terminates on either path, and never after a configure failure.
- A step panic is re-raised with phase `steps`, the harness type, feature path,
  one-based line, quoted scenario name, and the original payload text.
- Configure and cleanup panics are re-raised with their own phase tags.
- A profile `Err` result is returned on the normal path and logged on the panic
  path.
- The result value returned by the runner is propagated unchanged on success.

### Technical requirements

- Re-raise through `std::panic::panic_any` so the panic hook runs and the
  message reaches the test runner and `#[should_panic(expected = ...)]`.
- Wrap profile errors in a crate-private, phase-tagged error that keeps the
  upstream error as its `source`, because `HarnessError`'s display begins
  "failed to build runtime" even for cleanup.
- Never call `App::run`, `App::finish`, or `App::cleanup` from the harness.
- Keep `BevyProfile` unsealed so default-bodied hooks can be added later
  without breaking downstream implementations.
- Keep the crate free of `unsafe`, `Send`/`Sync` forwarding, globals, and
  process-wide locks.

## Options considered

### Option A: `resume_unwind` plus direct standard-error output

This is the GPUI harness's approach: augment the message and write it to
standard error before resuming the panic. `resume_unwind` preserves the
original payload, but it does not run the panic hook, and libtest does not
print a failed test's payload. The diagnostic therefore depends on a direct
write that the workspace denies (`print_stderr`), and the write bypasses
libtest's per-test capture.

### Option B: cleanup in a `Drop` guard

A guard would run cleanup while the step panic unwinds. If cleanup then panics,
the process aborts (a panic during unwinding). The lifecycle table's rows 7 and
8 require a cleanup failure to be recorded without masking the step panic,
which a guard cannot guarantee.

### Option C: hand steps a `&mut App` context

`HarnessAdapter` imposes only `Context: Any`; the macro moves the context into
a step context cell and drops it when the runner closure ends. A borrowed
context cannot be cloned for multiple steps, and the harness cannot reach the
application for cleanup once the cell is gone. Design §4 already chooses
`Rc<RefCell<App>>` for this reason.

### Option D: carry a configured profile value in the harness

Generated tests construct the harness with `<Harness as Default>::default()`,
so no configured profile instance exists. Profile state must live in associated
functions or in the application itself.

### Option E: return a crate-owned profile error type

Changing `configure` and `cleanup` to return a crate-owned error would make the
failing hook explicit in the type system. It also changes the design §5
signatures, which the ExecPlan's interface tolerance treats as an escalation.
The phase-tagged wrapper below achieves the diagnostic benefit without an
interface change.

### Option F: deterministic time by default

`TimeUpdateStrategy::FixedTimesteps(n)` makes each `App::update` run the fixed
loop a fixed number of times, which is attractive for deterministic tests. Bevy
0.20.1 initializes the clock on the first update with a zero delta, so that
first update runs no fixed steps; the steady state from the second update on is
`n` steps each. Adopting the strategy in `MinimalBevyProfile` would deviate
from design §5, which names `MinimalPlugins` as the default. The technique,
including the first-update exception, is documented for downstream profile
types instead.

| Option                         | Re-raise visibility           | Cleanup contract                | API impact             | Outcome                                        |
| ------------------------------ | ----------------------------- | ------------------------------- | ---------------------- | ---------------------------------------------- |
| A: `resume_unwind` plus stderr | Message invisible to libtest  | Explicit cleanup still possible | None                   | Rejected: denied output and silent payload     |
| B: `Drop` guard                | Hook runs                     | Cleanup panic aborts            | None                   | Rejected: cannot record cleanup failure safely |
| C: `&mut App` context          | Not applicable                | Harness cannot reach the app    | Signature change       | Rejected: conflicts with design §4             |
| D: profile value               | Not applicable                | Not applicable                  | Constructor change     | Rejected: generated tests only use `Default`   |
| E: crate-owned profile error   | Not applicable                | Explicit                        | Signature change       | Rejected for now: tolerance breach             |
| F: fixed timesteps by default  | Not applicable                | Not applicable                  | Profile default change | Rejected: deviates from design §5              |
| G: phase-tagged `panic_any`    | Hook prints augmented message | Explicit catch boundary         | None                   | Chosen                                         |

*Table 1: Options for the Bevy harness failure contract.*

## Decision outcome / proposed direction

### Lifecycle table

Let `c`, `r`, and `k` be the configure, runner, and cleanup outcome classes,
`m(phi, x)` the augmented panic message for phase `phi` and payload `x`, and
`w(phi, e)` the phase-tagged error wrapping. The contract is:

| configure  | steps       | cleanup    | result                  | runner calls | cleanup calls | error records |
| ---------- | ----------- | ---------- | ----------------------- | ------------ | ------------- | ------------- |
| `Err(e)`   | -           | -          | `Err(w(configure, e))`  | 0            | 0             | 0             |
| `Panic(q)` | -           | -          | panic `m(configure, q)` | 0            | 0             | 0             |
| `Ok`       | `Return(v)` | `Ok`       | `Ok(v)`                 | 1            | 1             | 0             |
| `Ok`       | `Return(v)` | `Err(e)`   | `Err(w(cleanup, e))`    | 1            | 1             | 0             |
| `Ok`       | `Return(v)` | `Panic(q)` | panic `m(cleanup, q)`   | 1            | 1             | 0             |
| `Ok`       | `Panic(p)`  | `Ok`       | panic `m(steps, p)`     | 1            | 1             | 1             |
| `Ok`       | `Panic(p)`  | `Err(e)`   | panic `m(steps, p)`     | 1            | 1             | 2             |
| `Ok`       | `Panic(p)`  | `Panic(q)` | panic `m(steps, p)`     | 1            | 1             | 2             |

*Table 2: Lifecycle outcome contract. Dashes mark phases that never run.*

The table is exhaustive over outcome classes. One representative per class
suffices because `run<T>` is parametric in `T` and never branches on the
runner's value, on the contents of an error, or on the scenario metadata.

### Phase-tagged messages

Every augmented panic is a `String` with this form, wrapped here for width:

```plaintext
rstest-bdd-harness-bevy scenario panicked: phase={phase}, harness={harness_type},
feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {payload_text}
```

`phase` is `configure`, `steps`, or `cleanup`. `harness_type` comes from
`std::any::type_name` and is informative, not a stable contract. Payload text
is the original text for `&'static str` and `String`; any other payload is
reported as `Box<dyn Any>`. The harness raises the message with
`std::panic::panic_any`, so the panic hook prints it and libtest's
`should_panic(expected = ...)` matches it. The hook also prints a location
inside the harness, which is acceptable.

### Phase-tagged error wrapping

Profile `Err` results are wrapped in a crate-private `ProfileHookError` that
holds the phase and the upstream `HarnessError`, implements `Display` as
`Bevy profile {phase} hook failed: {source}`, and returns the upstream error
from `source()`. The wrapper is carried inside
`HarnessError::RuntimeBuildFailed(std::io::Error::other(..))`, which preserves
the design §5 signatures and still names the failing hook.

### Panic-path cleanup and drop safety

On a step panic the harness builds the message, emits one `tracing::error!`
record naming the step panic, runs cleanup inside `catch_unwind`, records a
cleanup `Err` or panic as a second error record, drops its last scenario handle
inside `catch_unwind` (recording a panicking drop), forgets the original
payload so a panicking payload `Drop` cannot double-panic, and re-raises the
step message. Configure and cleanup panics take the same guarded-drop path.

### Never finish or run

The harness never calls `App::run`, `App::finish`, or `App::cleanup`, and it
advances the application only through `BevyScenario::update` and
`update_times`. A profile that needs plugin finalization may call `finish` and
`cleanup` at the end of `configure`, but a finished application can no longer
accept plugins, so that profile's steps must not call `add_plugins`. A profile
that needs `add_plugins` in steps must not finalize.

### SemVer coupling

`BevyProfile`, `BevyHarness`, and `BevyScenario` expose `bevy::app::App`,
`bevy::ecs::world::World`, `ScenarioMetadata`, `HarnessResult`, and
`HarnessError` in public signatures. A minor release of a `0.x` crate is
breaking, so a Bevy or `rstest-bdd-harness` minor bump is breaking for this
crate too. `BevyProfile` stays unsealed so new default-bodied hooks are
additive; adding a required method would be breaking.

## Goals and non-goals

- Goals:
  - Make every scenario failure identifiable by phase and scenario in ordinary
    test output.
  - Guarantee cleanup exactly once after a successful configure.
  - Preserve the runner's value and the step panic's identity.
  - Keep the crate's public surface at the design §5 shape.
- Non-goals:
  - Process-level stderr or tracing-output snapshots of generated scenarios
    (roadmap `0.1.3`).
  - A compile-pass `trybuild` fixture for a harness type alias (roadmap
    `0.1.3`).
  - Support for `panic = "abort"`; no cleanup can run under that strategy.
  - A strong-count warning when a step clone outlives its scenario (deferred
    with the `0.1.3` tracing-output work).

## Migration plan

The API is pre-1.0 and unreleased, so no migration is required. Downstream
profile authors adopt the documented idiom for returning hook errors:
`HarnessError::RuntimeBuildFailed(std::io::Error::other(message))`.

## Known risks and limitations

- A Bevy system that panics inside `App::update` leaves the `Main` schedule
  removed. `update` and `update_times` detect the missing schedule and panic
  with a crate-prefixed message that explains a system panicked earlier.
- A panicking `Drop` during re-raising would abort the test binary. The harness
  drops its scenario handle inside `catch_unwind` and forgets the original
  payload to bound that risk.
- `HarnessError`'s display text still begins "failed to build runtime" for a
  cleanup failure; the phase-tagged wrapper is what makes the hook visible.
- The panic hook prints a source location inside the harness rather than the
  feature file; the message itself carries the feature path and line.
- Under `panic = "abort"`, cleanup does not run and the hook path is
  unavailable. Tests use the default unwinding strategy.

## Outstanding decisions

None block implementation. The deferred items above are owned by roadmap
`0.1.3` and recorded in the ExecPlan.

## Architectural rationale

The decision keeps failure policy in one small, pure formatter (`src/panic.rs`)
and lifecycle orchestration in `src/harness.rs`, separated from the scenario
context (`src/context.rs`) and profile seam (`src/profile.rs`). It follows ADR
009's extension-seam rule: the harness owns the lifecycle and diagnostics,
while every framework- or game-specific hook stays in a downstream
`BevyProfile`. Using `panic_any` rather than a silent resume preserves the
project's observability rule that failures reach the ordinary test output.
Wrapping profile errors instead of changing the profile signatures preserves
the approved public API and its extractability.

## Amendments

### 2026-11-20: never-finish guidance corrected

The original never-finish paragraph ended with:

> A profile that needs plugin finalization calls `finish` and `cleanup` at the
> end of `configure`, which keeps `add_plugins` legal inside steps.

That clause contradicted Bevy's behaviour and the rule it was explaining:
finishing an application makes later `add_plugins` calls illegal.
[ADR 011](adr-0011-correct-bevy-harness-never-finish-guidance.md) replaces it
with the delivered rule — a profile that finalizes gives up in-step plugin
registration. The decision that the harness never finalizes is unchanged, and
no code or public API changed.

### 2026-11-21: hostile profile errors are formatted and disposed separately

The original panic-path paragraph said the harness "records a cleanup `Err` or
panic as a second error record", which did not say how a cleanup `Err` is
rendered. A profile's error is caller-supplied code, so both its `Display` and
its `Drop` may panic. Formatting it and releasing the wrapped temporary inside
one caught step is unsafe when an error does both: once `Display` panics, the
release runs while that panic unwinds, and a second panic there aborts the
process instead of unwinding to a catch boundary, losing the step panic the
path exists to preserve.

The delivered rule keeps this record's decision and adds a step boundary: the
wrapped error is formatted by borrow in one `catch_unwind`, then dropped in a
second `catch_unwind`, and a panicking disposal is the failure the second
record carries. The step panic, its phase tag, the payload-text rule, and the
public API are unchanged.

## References

- [Design §6](../rstest-bdd-harness-bevy-design.md#6-panic-and-cleanup-behaviour)
  defines the required diagnostic fields.
- [ADR 009](adr-0009-in-tree-incubation-of-the-bevy-bdd-harness-crate.md) sets
  the extension-seam and extraction contract.
- [ExecPlan for roadmap 0.1.2](../execplans/0-1-2-bevy-harness-public-api.md)
  records the approved lifecycle table, options, and acceptance evidence.
- Upstream `rstest-bdd-harness` 0.6.0 `adapter.rs`, `runner.rs`, and `error.rs`
  define `HarnessAdapter`, `ScenarioRunRequest`, and `HarnessError`.

## Approval provenance

The ExecPlan for roadmap `0.1.2` was approved by the maintainer on 2026-11-20,
who directed implementation to proceed to completion. That plan's
`Decision log` enumerates every decision this record captures; the maintainer's
approval of those decisions is the acceptance of this ADR.

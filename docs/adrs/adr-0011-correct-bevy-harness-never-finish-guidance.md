# Architectural decision record (ADR) 011: Correct the Bevy harness never-finish guidance

## Status

Accepted. Amends [ADR 010](adr-0010-bevy-harness-failure-contract.md)'s
never-finish paragraph.

## Date

2026-11-20.

## Context and problem statement

ADR 010 fixes that the harness never calls `App::run`, `App::finish`, or
`App::cleanup`. Its never-finish paragraph then says a profile that needs
plugin finalization "calls `finish` and `cleanup` at the end of `configure`,
which keeps `add_plugins` legal inside steps." That clause contradicts itself
and the rule it explains: Bevy rejects `add_plugins` once an application has
been finished, so a profile that finalizes during `configure` makes
`add_plugins` illegal in every later step.

The delivered harness (`src/harness.rs`) and the `BevyProfile` rustdoc already
state the correct rule: the harness never finalizes, and a profile that
finalizes gives up the ability to add plugins in steps. Only ADR 010's wording
is wrong, so this record corrects it without changing behaviour or the public
API.

## Decision drivers

- Preserve ADR 010's accepted decision; correct only the contradictory
  explanatory clause.
- Keep the documented contract consistent with `src/profile.rs`,
  `src/harness.rs`, the users' guide, and the crate README.
- Do not invent a compatibility mechanism or a new hook; the harness has no
  finalization responsibility.

## Decision outcome

The harness never calls `App::run`, `App::finish`, or `App::cleanup`, and it
advances the application only through `BevyScenario::update` and `update_times`.

A profile that needs plugin finalization may call `App::finish` and
`App::cleanup` at the end of its `configure` hook, but a finished application
can no longer accept plugins, so steps in that profile must not call
`add_plugins`. A profile that needs `add_plugins` in steps must not finalize.
The choice belongs to the downstream profile; the harness itself never makes it.

## Options considered

| Option                                  | Outcome                                                                                                                                         |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Leave ADR 010 unchanged                 | Rejected: the paragraph contradicts Bevy's behaviour and the delivered rustdoc.                                                                 |
| Amend ADR 010 in place without a record | Rejected: accepted ADR decisions are preserved as history, and the documentation style guide records targeted corrections through an amendment. |
| Change the harness to finalize          | Rejected: it would forbid `add_plugins` in steps and break design §7 and the profile seam.                                                      |

## Consequences

- ADR 010 gains an `Amended by ADR-011 (2026-11-20)` banner and an
  `## Amendments` section that preserves its original paragraph.
- No code changes follow; the delivered harness already implements this rule.
- Downstream profile authors must choose between in-step plugin registration
  and profile-level finalization, and document that choice in their profile.

## References

- [ADR 010: Bevy harness scenario failure contract](adr-0010-bevy-harness-failure-contract.md)
- [Design §4](../rstest-bdd-harness-bevy-design.md#4-architecture)
- [Design §5](../rstest-bdd-harness-bevy-design.md#5-public-api-contract)
- [ExecPlan for roadmap 0.1.2](../execplans/0-1-2-bevy-harness-public-api.md)

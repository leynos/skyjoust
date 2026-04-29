# 001: Use Stateright as the interaction validator

Status: Accepted

Date: 2026-04-26

## Context

Project Skyjoust needs a compact way to verify high-level interaction rules
before the runtime implementation is complete. The rules span match lifecycle,
ceremonies, scoring, rewards, and Warfront handoff. Unit tests catch individual
transitions, but they do not exhaustively explore action orderings.

The project also needs concrete trace replay so runtime logs can be checked
against the same contract used by the model checker.

## Decision

Use Stateright for bounded model checking of the Skyjoust interaction model.
Keep the model small, deterministic, and focused on contract-level state rather
than renderer or physics details.

The validator crate provides:

- a bounded `SkyjoustInteractionModel`,
- generated legal actions for each state,
- transition guards and side effects,
- `always` invariants and `sometimes` reachability properties,
- JSON trace replay through `validate_trace`,
- Explorer diagnostics for reachable states and counterexamples.

## Alternatives considered

Handwritten scenario tests were rejected as the only mechanism because they
cover selected examples rather than the action space.

Property tests using random action generation were rejected as the primary
validator because they can miss rare ordering failures unless seeds and search
budgets are carefully managed.

Runtime-only assertions were rejected as the primary validator because they
detect failures after implementation work has already coupled systems together.

## Consequences

The validator becomes a contract source that must be updated with the state
graph bundle and validator contract YAML whenever behaviour changes.

The model must remain intentionally smaller than the runtime. Renderer state,
entity-component system (ECS) details, and physics internals belong in runtime
tests, not in the Stateright model.

Laurels are event-only reward ledger currency in this contract. The validator
must keep Laurel grants behind tournament or special event completion instead
of treating Laurels as Warfront income or a substitute for Glory, Coin, or
Influence.

Depth bounds are part of the validation contract. Raising `max_depth` expands
the explored or replayed interaction horizon and can increase check time.

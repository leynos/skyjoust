# Documentation contents

- [Documentation contents](contents.md) - this index for the Skyjoust
  documentation set.

## User-facing documentation

- [Skyjoust user's guide](users-guide.md) - operator and integrator workflows
  for running validator checks, opening the Stateright Explorer, and replaying
  traces.
- [Skyjoust Product Requirements Document (PRD)](skyjoust-product-requirements.md)
  - product vision, game modes, platform constraints, and player-facing scope.

## Design and architecture

- [Project Skyjoust technical design](skyjoust-technical-design.md) - system
  architecture, runtime constraints, asset-pipeline expectations, and design
  rationale.
- [Project Skyjoust development plan](development-plan.md) - phased delivery
  plan and implementation sequencing for the current design.
- [Project Skyjoust roadmap](roadmap.md) - milestone-oriented work breakdown
  and outstanding implementation tasks.

## Maintainer guides and standards

- [Skyjoust developer's guide](developers-guide.md) - maintainer workflow,
  validator extension sequence, and local validation expectations.
- [Repository layout](repository-layout.md) - orientation to the repository
  tree, ownership boundaries, generated artefacts, and validation paths.
- [Documentation style guide](documentation-style-guide.md) - documentation
  spelling, structure, formatting, and standard document-type rules.
- [Scripting standards](scripting-standards.md) - conventions for repository
  scripts and command-line helper tooling.
- [Navigating code complexity](complexity-antipatterns-and-refactoring-strategies.md)
  - maintainability reference for recognizing complexity and planning
  refactors.

## Testing and Rust practice references

- [Mastering test fixtures in Rust with `rstest`](rust-testing-with-rstest-fixtures.md)
  - fixture, parameterization, and test-organization patterns for Rust tests.
- [Reliable testing in Rust via dependency injection](reliable-testing-in-rust-via-dependency-injection.md)
  - dependency-injection guidance for deterministic and isolated tests.
- [Effective, ergonomic, and dry doctests in Rust](rust-doctest-dry-guide.md)
  - doctest structure, reuse patterns, and anti-pattern avoidance.

## Architecture decision records

The current Architecture Decision Records (ADRs) live under `docs/adr/`.

- [ADR 001: Use Stateright as the interaction validator](adr/001-stateright-as-interaction-validator.md)
  - accepted decision to use Stateright for bounded interaction validation.
- [ADR 002: Document crate layout and defer future API splits](adr/002-crate-layout-and-public-api.md)
  - proposed crate boundary and public API timing decision.
- [ADR 003: Defer fixed-point type and scale](adr/003-fixed-point-type-and-scale.md)
  - deferred numeric representation decision pending prototype evidence.
- [ADR 004: Defer atlas packing format and manifest schema](adr/004-atlas-packing-format-and-manifest-schema.md)
  - deferred asset atlas and manifest schema decision.
- [ADR 005: Defer Warfront renderer map-art strategy](adr/005-warfront-renderer-map-art-strategy.md)
  - deferred rendering art strategy decision for Warfront maps.

## Execution plans

- [Record the initial runtime crate split as an ADR](execplans/1-1-1-record-runtime-crate-split-as-adr.md)
  - approval-gated plan for roadmap task `1.1.1`.

## State graph references and artefacts

- [Project Skyjoust state graph bundle](skyjoust-state-graphs-README.md) -
  overview and maintenance rules for the checked-in state graph files.
- [Canonical state graph YAML](skyjoust-state-graphs.yaml) - editable source
  of truth for high-level Skyjoust statecharts.
- [Generated state graph JSON](skyjoust-state-graphs.json) - generated JSON
  equivalent of the canonical YAML bundle.
- [System overview graph source](skyjoust-overview.dot) - Graphviz source for
  the high-level state graph overview.
- [System overview graph](skyjoust-overview.svg) - rendered overview diagram.
- [Match lifecycle graph source](skyjoust-match-lifecycle.dot) - Graphviz
  source for match lifecycle states.
- [Match lifecycle graph](skyjoust-match-lifecycle.svg) - rendered match
  lifecycle diagram.
- [Player action graph source](skyjoust-player-action.dot) - Graphviz source
  for per-player action states.
- [Player action graph](skyjoust-player-action.svg) - rendered player action
  diagram.
- [Event reward scoring graph source](skyjoust-event-reward-scoring.dot) -
  Graphviz source for ceremony, scoring, and reward flow.
- [Event reward scoring graph](skyjoust-event-reward-scoring.svg) - rendered
  ceremony, scoring, and reward-flow diagram.

# 002: Document crate layout and defer future API splits

Status: Proposed

Date: 2026-04-26

## Context

Project Skyjoust starts as a two-crate workspace: the compact runtime crate and
the focused `skyjoust_stateright_validator` crate. The validator crate is an
immediate exception to the general deferment rule because it owns tooling-only
responsibilities: Stateright Explorer support, trace validation, contract
snapshots, and exhaustive interaction tests.

Splitting crates too early would force public API commitments around renderer,
simulation, assets, and Warfront state before the first vertical slices prove
which seams remain stable.

## Decision

Keep the runtime implementation in a small workspace shape until the first
gameplay slice demonstrates stable module boundaries. The existing
`skyjoust_stateright_validator` crate remains separate because it is a
tooling-facing verification surface, not a runtime boundary. Promote additional
modules into crates only when an API is reused across runtime, tooling, or
validator code.

Candidate crates are:

- a runtime crate for simulation and app orchestration;
- an asset-tooling crate for manifests, slicing, and validation;
- a shared contract crate only if runtime and validator need common data types.

## Consequences

The first implementation pass can move faster without unstable public APIs.
Before crate extraction, module APIs must stay documented and testable, so the
eventual split remains mechanical rather than architectural.

The validator crate can publish a substantial API earlier than the runtime
because its consumers are developer tools, fixtures, and contract checks.
Runtime crate extraction remains deferred until gameplay slices prove stable
ownership boundaries.

# 002: Defer crate layout and public API split

Status: Proposed

Date: 2026-04-26

## Context

Project Skyjoust currently has a compact runtime crate and a focused
`skyjoust-stateright-validator` crate. The Minimum Viable Product still needs
early gameplay slices before stable runtime boundaries are obvious.

Splitting crates too early would force public API commitments around renderer,
simulation, assets, and Warfront state before the first vertical slices prove
which seams remain stable.

## Decision

Keep the implementation in a small workspace shape until the first gameplay
slice demonstrates stable module boundaries. Promote modules into crates only
when an API is reused across runtime, tooling, or validator code.

Candidate crates are:

- a runtime crate for simulation and app orchestration;
- an asset-tooling crate for manifests, slicing, and validation;
- a shared contract crate only if runtime and validator need common data types.

## Consequences

The first implementation pass can move faster without unstable public APIs.
Before crate extraction, module APIs must stay documented and testable so the
eventual split remains mechanical rather than architectural.

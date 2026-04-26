# 004: Defer atlas packing format and manifest schema

Status: Proposed

Date: 2026-04-26

## Context

The asset pipeline uses generated reference imagery, reviewed source sheets,
post-processing tools, and runtime manifests. The exact atlas packing format
depends on sprite slicing, palette limits, alpha handling, animation metadata,
and renderer constraints.

A premature schema would invite churn across generator prompts, validation
tools, and runtime loading code.

## Decision

Defer the final atlas format and manifest schema until the first reviewed
sprite sheets have passed slicing, alpha checks, palette checks, and runtime
loading in the placeholder renderer.

The first schema decision must cover:

- sprite frame rectangles and pivots;
- animation names and frame timing;
- palette and alpha requirements;
- source provenance and manifest validation rules.

## Consequences

Early manifests may be fixtures, but production manifests must be validated by
`validate_asset_manifest.*` tooling before the runtime treats them as
authoritative.

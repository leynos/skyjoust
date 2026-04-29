# 005: Defer Warfront renderer map-art strategy

Status: Proposed

Date: 2026-04-26

## Context

The Warfront screen needs readable region ownership, routes, pressure vectors,
event forecasts, and house control. Generated art can provide strong visual
identity, while deterministic map tiles can make state changes easier to render
and validate.

The project has not yet proven which approach gives better readability and
maintainability for live Warfront state.

## Decision

Defer the renderer map-art strategy until the first Warfront prototype compares
generated region base art with deterministic map tiles plus generated
ornamentation.

The comparison must evaluate:

- legibility of region ownership and routes;
- ease of updating control and pressure overlays;
- asset generation and validation cost;
- consistency with the reference design book.

## Consequences

Warfront state must remain renderable as data regardless of the final art
strategy. Art decisions must not require changes to the Warfront state graph or
validator contract.

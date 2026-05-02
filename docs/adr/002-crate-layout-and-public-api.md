# 002: Initial runtime crate split and deferred public API surface

Status: Accepted

Date: 2026-04-26

Accepted: 2026-05-02

## Context

Project Skyjoust starts as a two-crate workspace: one compact runtime crate and
the focused `skyjoust_stateright_validator` crate. The validator crate is an
immediate exception to the general deferment rule because it owns tooling-only
responsibilities: Stateright Explorer support, trace validation, contract
snapshots, and exhaustive interaction tests.

Splitting the runtime into multiple crates too early would force public API
commitments around renderer, simulation, assets, and Warfront state before the
first vertical slices prove which seams remain stable. At the same time, the
runtime must already preserve clear ownership boundaries so that future
extraction is mechanical rather than architectural.

## Decision

The first implementation of Project Skyjoust uses one runtime crate with strict
internal modules, beside the separate `skyjoust_stateright_validator` crate.
The validator crate remains separate because it is a tooling-facing
verification surface, not a runtime boundary.

The runtime crate organises responsibilities as internal modules, not as
separate crates, for the first playable slice:

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

Dependency direction inside the runtime crate is one-way:

```plaintext
game_app -> subsystem modules -> core
```

Subsystem modules (`sim`, `terrain`, `stategraphs`, `render`, `audio`, `ui`,
`assets`) depend on `core` and on each other only where the technical design
permits. Lower-level modules must not call higher-level orchestration or
adapters. Domain responsibilities must not depend on adapters, renderer code,
audio backends, process setup, or window lifecycle glue, in keeping with the
project's hexagonal architecture posture.

A module may be extracted into its own crate only when at least one of these
conditions holds:

- an API is reused across the runtime, developer tooling, or the validator
  crate;
- a boundary is stable enough to test and release independently of the
  runtime crate.

Candidate future crates remain:

- an asset-tooling crate for manifests, slicing, and validation, once tools
  outgrow runtime-private use;
- a shared contract crate only if the runtime and validator need common data
  types beyond what `skyjoust_stateright_validator` already exposes;
- subsystem-specific crates (for example `sim` or `render`) only after their
  module APIs prove stable and externally consumed.

## Consequences

The first implementation pass moves faster without committing to unstable
public APIs. Module APIs must still be documented and testable so the eventual
split remains mechanical rather than architectural.

The validator crate can publish a substantial API earlier than the runtime
because its consumers are developer tools, fixtures, and contract checks.
Runtime crate extraction remains deferred until gameplay slices prove stable
ownership boundaries.

Maintainers reading this ADR should expect the runtime to look like one crate
with named modules in the file tree, not a multi-crate workspace, until a later
ADR records a specific extraction. Roadmap task `1.1.1`, the technical design's
runtime ownership table, and the development plan's phase-2 entry all align
with this decision.

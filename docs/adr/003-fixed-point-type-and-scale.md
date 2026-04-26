# 003: Defer fixed-point type and scale

Status: Proposed

Date: 2026-04-26

## Context

Skyjoust requires deterministic physics and replayable input streams. Floating
point arithmetic risks platform divergence, but the exact coordinate range,
velocity range, and collision precision are not known until the movement
prototype exists.

Choosing the fixed-point representation before those measurements could either
waste range or under-specify collision precision.

## Decision

Defer the final fixed-point type and scale until the movement prototype records
position, velocity, terrain, and collision ranges under representative play.

The prototype must report:

- maximum required world coordinate range;
- smallest collision distance that affects joust outcomes;
- velocity and acceleration bounds;
- cost of fixed-tick collision updates at the target simulation rate.

## Consequences

Prototype code may use a temporary deterministic numeric wrapper, but it must
not become a public contract until benchmark and replay evidence select the
final scale.

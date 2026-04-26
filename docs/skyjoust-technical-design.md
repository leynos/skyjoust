# Project Skyjoust technical design

Status: Living design

Audience: developers implementing the runtime, asset pipeline, renderer,
simulation, and verification harness.

Companion documents:

- [Product Requirements Document (PRD)](skyjoust-product-requirements.md)
- [Development plan](development-plan.md)
- [State graph bundle](skyjoust-state-graphs-README.md)
- [Canonical state graph YAML](skyjoust-state-graphs.yaml)
- [Stateright validator](../crates/skyjoust_stateright_validator/README.md)
- Reference design book pages in `ref/`

## 1. Design context

Project Skyjoust is a side-scrolling arcade game about mounted aerial knights
fighting over destructible borderlands. The runtime must preserve immediate
Joust-style collision play while supporting Sopwith-style territory pressure,
terrain damage, and ceremonial rule shifts.

The reference images in `ref/` establish a stronger visual contract than the
original prose design. They define a 16-bit arcade presentation with navy and
brass frames, heraldic houses, metallic title treatment, readable mount
silhouettes, dense design-book panels, and heads-up display (HUD) elements for
altitude, brace timing, morale, ammo, objectives, and event banners. The
runtime must render the game as a playable pixel world, not as the design book
itself, but the book sets the palette, silhouette discipline, icon families,
and hierarchy.

The state graph bundle defines the authoritative high-level lifecycle. Runtime
systems may refine combat, terrain, input, and presentation details, but they
must preserve the graph's event order, scoring boundaries, and reward commit
rules. The Stateright validator models those handoffs and is a contract test
for the implementation.

## 2. Goals and non-goals

### Goals

- Run authoritative simulation on a fixed 120 Hz tick.
- Render at variable rate through a fixed-resolution pixel framebuffer.
- Keep simulation, rendering, audio, and user interface (UI) mutation
  boundaries separate.
- Model app runtime, match lifecycle, ceremony events, player actor state,
  objectives, scoring, rewards, and Warfront progression as explicit state
  resources.
- Validate high-level state transitions against
  `crates/skyjoust_stateright_validator`.
- Build a reproducible asset pipeline around built-in `imagegen`, manifests,
  post-processing, slicing, palette control, and runtime validation.
- Preserve the reference image style while drawing important gameplay text,
  counters, and state-dependent labels in the renderer.

### Non-goals

- Do not integrate Bevy's renderer for the initial runtime. Bevy supplies
  entity-component system (ECS) schedules and state resources; `pixels` owns
  presentation.
- Do not bake authoritative gameplay values into generated images.
- Do not treat generated sheets as deterministic sprite atlases until they pass
  slicing, palette, alpha, and manifest checks.
- Do not implement online multiplayer for the Minimum Viable Product (MVP).
  The simulation remains input-driven and deterministic enough to support later
  rollback work.
- Do not implement full wedding and banquet systems for the MVP. Their state
  graph entries remain design contracts for later phases.

## 3. Visual contract

Table 1: Reference image implications for implementation.

| Reference pages              | Implementation constraint                                                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `skyjoust-design-book-1.png` | The brand signal uses metallic block lettering, wing motifs, navy panels, brass trim, and large aerial joust scenes. Title and menu screens should use this language without making gameplay UI unreadable.  |
| `skyjoust-design-book-2.png` | Houses need distinct heraldry, colour ramps, champion silhouettes, role labels, and sprite-strip references. House data must drive both visual selection and gameplay doctrine.                              |
| `skyjoust-design-book-3.png` | Mounts need readable side-view silhouettes, action poses, stat bars, flap cadence notes, and small sprite references. Runtime mounts must remain legible at speed before detail is added.                    |
| `skyjoust-design-book-4.png` | Environments are cliffs, river gorges, aqueducts, shrine hills, villages, ridges, and material tiles. Terrain generation must expose material identity to rendering and collision.                           |
| `skyjoust-design-book-5.png` | The HUD must expose altitude advantage, brace window state, joust outcome, tactical actions, morale, ammo, objective state, and controls. These are runtime-drawn interface elements.                        |
| `skyjoust-design-book-6.png` | Battle scenes require vertical drama, bombing runs, keep assaults, rescue/capture beats, local versus chaos, and readable high-contrast effects. Camera and effects must preserve gameplay clarity.          |
| `skyjoust-design-book-7.png` | Tournament and duel events need arena lanes, brackets, honour rules, champion line-ups, dishonour penalties, and reward previews. Ceremony state must drive UI panels and temporary rules.                   |
| `skyjoust-design-book-8.png` | Wedding and banquet content requires alliance processions, resource exchange, treaty signing, truce status, breaking-truce penalties, and diplomatic upgrades. These are post-MVP, but must keep data slots. |
| `skyjoust-design-book-9.png` | Warfront UI needs regions, house control, supply routes, pressure arrows, event forecasts, active modifiers, recruit/deploy cards, and currency totals. Warfront state must be renderable as map data.       |

The art direction uses generated imagery for source references and production
inputs, then converts accepted outputs into deterministic runtime assets. The
gameplay renderer owns:

- live HUD text and counters,
- timers,
- input prompts,
- debug overlays,
- event state labels,
- health, morale, ammo, and score values,
- Warfront map overlays derived from current state.

Generated art may contain decorative headings in design-book pages and static
source sheets. It must not be the only source of gameplay-critical information.

## 4. Runtime architecture

The runtime uses a custom `winit` event loop. The loop updates input state,
steps Bevy schedules on a fixed accumulator, and then calls presentation
systems that draw into a `pixels` framebuffer and flush Kira audio events.

Table 2: Runtime ownership.

| Layer         | Owns                                                                     | Must not own                                     |
| ------------- | ------------------------------------------------------------------------ | ------------------------------------------------ |
| `game_app`    | Window lifecycle, event loop, schedule orchestration, platform setup     | Game rules, terrain algorithms, asset slicing    |
| `core`        | Fixed-point math, stable IDs, event types, shared config types           | Renderer state, audio backends, gameplay systems |
| `sim`         | Movement, collision, joust resolution, damage, objectives, match events  | Warfront persistence, framebuffer drawing        |
| `terrain`     | Seeded terrain generation, chunks, deformation, material queries         | Joust scoring, audio playback                    |
| `stategraphs` | State resources, graph evaluation, event routing, guard/action contracts | Fine-grained physics and rendering               |
| `render`      | Pixel framebuffer, atlas blits, terrain layer cache, camera, HUD drawing | Authoritative simulation mutation                |
| `audio`       | Kira manager, buses, SFX mapping, rate limits, music layers              | Score, reward, and match state mutation          |
| `ui`          | Bitmap UI composition and menus drawn from state resources               | Direct mutation of scoring or rewards            |
| `assets`      | Manifests, source images, derived atlases, palettes, fonts, audio        | Runtime state decisions                          |

The crate split may start smaller while the project is young. The dependency
direction still applies: orchestration depends on subsystems; subsystems depend
on `core`; lower layers do not call higher layers.

## 5. Fixed tick and presentation loop

Each platform runner follows the same order:

1. Ingest `winit` events into a raw input buffer.
2. Convert raw input into bounded `InputState`.
3. Accumulate elapsed time.
4. Run zero or more fixed simulation ticks while the accumulator is at least
   `SIM_DT`.
5. Run one presentation pass.
6. Present the framebuffer.

Simulation ticks never read wall-clock time. Systems receive tick index,
current input, deterministic random state, and ECS world state. The renderer
may interpolate visual positions, but interpolated values cannot feed back into
simulation resources.

## 6. State graph integration

`docs/skyjoust-state-graphs.yaml` defines parallel regions for:

- app runtime,
- Warfront,
- match lifecycle,
- ceremony events,
- objectives,
- one player actor graph per rider,
- scoring,
- rewards.

The runtime should represent each region as a resource with current node path,
activation state, and last processed tick. Graph evaluation runs during the
fixed tick after gameplay events are available and before presentation reads
state.

Table 3: Graph-to-runtime mapping.

| Graph region      | Runtime representation                            | Key outputs                                                                |
| ----------------- | ------------------------------------------------- | -------------------------------------------------------------------------- |
| `app_runtime`     | `AppRuntimeState` resource                        | mode changes, match start/end, title/results transitions                   |
| `warfront`        | `CampaignState` plus `WarfrontGraphState`         | region selection, battle context, event rolls, reward application          |
| `match_lifecycle` | `MatchState` plus `MatchLifecycleState`           | match construction, countdown, active play, round over, final score export |
| `ceremony_events` | `CeremonyState` plus temporary rule deltas        | tournament, duel, truce, banquet, cooldown, honour consequences            |
| `objectives`      | objective components and `ObjectiveState`         | keep breach, capture state, shrine buffs, supply route status              |
| `player_actor`    | player components plus per-player state resources | mobility, lance, ordnance, interaction, recovery                           |
| `scoring`         | `ScoreLedger` and deterministic event batch       | score deltas, morale deltas, victory progress, final snapshot              |
| `rewards`         | `RewardLedger` and persistence commands           | currencies, laurels, penalties, unlocks, committed results                 |

Graph guards are pure selectors over world snapshots. Graph actions enqueue
commands or publish events. The command queue applies in stable order at the
end of graph evaluation.

## 7. Verified contract

The Stateright validator in `crates/skyjoust_stateright_validator` is not a
physics model. It proves the high-level interaction contract that the runtime
must preserve.

The implementation must keep these properties visible in code and tests:

- Rewards cannot open or commit before a finalized score snapshot.
- Finalized score ledgers cannot accept later score writes.
- Committed rewards leave active match phases.
- Duel locks exist only during duel arena lock, active duel, duel resolution,
  or consequence resolution.
- Joust-only rules disable ordnance for match rules and player ordnance state.
- Active truces disable friendly fire.
- Broken truces create infamy and reward penalties.
- Laurels are event-only reward ledger currency granted only after tournament
  or special event completion; they do not transfer into Warfront Coin,
  Influence, or Glory ledgers.
- Duel rewards are granted only after a resolved duel.
- Temporary rules clear after cooldown when no truce remains active.
- Round-over and results-exported phases always have a winner.
- Warfront state does not mutate during active match simulation.

Runtime trace export should serialize high-level actions in the same shape as
the validator's `SkyAction` enum. A failing trace becomes a reproduction case:

```json
[
  "AssetsLoaded",
  "StartSkirmish",
  "StartBattle",
  "FinishConstructing",
  "SpawnReady",
  "CountdownDone",
  { "BombKeepBreach": { "team": "Red" } },
  "VictoryCheck",
  "ExportFinalScore",
  "TallyRewards",
  "CommitRewards"
]
```

## 8. Simulation model

The simulation owns authoritative gameplay facts:

- fixed-point or integer subpixel position,
- velocity,
- mount handling,
- rider state,
- lance state,
- ordnance state,
- collision shapes,
- terrain material occupancy,
- objective ownership,
- health, morale, score, and reward ledgers.

Joust contact is separate from physics contact. A collision can occur without
valid lance contact. The joust system emits `JOUST_OUTCOME` only when reach,
cone, brace timing, relative height, velocity, and rule constraints all pass.
Candidate engagements sort by
`(tick, min_stable_entity_id, max_stable_entity_id)` before resolution.

The advantage score includes:

- altitude difference,
- vertical velocity and dive state,
- brace timing quality,
- lance alignment,
- mount class modifier,
- broken-lance or stun debuffs,
- temporary ceremony modifiers.

Outcome categories match the PRD and state graph: knockback, unhorse, shatter,
and clean kill. The renderer may add slow frames, sparks, and camera impulse,
but the outcome is an ECS event consumed by recovery, scoring, ceremony, and
audio systems.

## 9. Terrain and objectives

Terrain uses a chunked 2D grid. Each cell stores a material such as air, dirt,
rock, stonework, wood, water, grassland, rubble, or decorative overlay. The
material set should match the reference book's visible terrain taxonomy: stone,
cliff rock, grassland, dirt or mud, wood, and water.

Chunks own:

- material cells,
- dirty rectangles for renderer cache invalidation,
- optional collision spans,
- deformation log entries for replay and debugging.

Generation is pure: seed plus biome and rules parameters produce the same
terrain, structures, objective placements, spawn corridors, and supply route
anchors. Fairness checks reject maps with unsafe spawn corridors, unreachable
objectives, unbreakable decisive chokepoints, or objective distance imbalance
outside the configured tolerance.

Objective state follows the state graph:

- keeps move from intact to outer works damaged, core exposed, and breached;
- outposts move through neutral, contested, capturing, and controlled states;
- shrines move through dormant, claimable, claimed, and cooldown states;
- supply routes move through secure, contested, blocked, and repaired states;
- hostages move through caged, released, carried, and delivered states.

## 10. Rendering and user interface

Rendering targets a fixed virtual resolution, scaled to the window with integer
scaling where possible. The first playable resolution should be chosen after a
HUD legibility test against the reference pages; candidates are `480x270`,
`512x288`, or `640x360`.

The renderer composes:

1. sky and far parallax,
2. midground structures and terrain cache,
3. destructible terrain chunks,
4. riders, mounts, lances, projectiles, and effects,
5. foreground silhouettes and particles,
6. HUD, event banners, menus, and debug overlays.

The HUD follows the reference page hierarchy:

- top strip: teams, timer, morale, and event state;
- left strip: altitude advantage and tactical hints;
- centre bottom: lance brace timing and current lance state;
- bottom or right strip: ammo, resupply, objective state, and controls;
- modal overlays: tournament bracket, duel prompt, truce status, reward
  preview, and Warfront action cards.

UI ornaments, heraldic frames, icon families, and panel backgrounds may come
from generated assets. Live text and numeric values must be drawn with a bitmap
font or renderer-owned glyph atlas.

## 11. Asset architecture

Generated art has three states:

- source reference: imagegen output preserved for art direction and review;
- production source: accepted generated image with manifest, prompt, input
  references, and evaluation notes;
- runtime asset: sliced, quantized, transparent or atlas-packed output with a
  manifest entry and validation result.

Recommended workspace layout:

```plaintext
assets/
  source/imagegen/
  derived/atlases/
  derived/cutouts/
  derived/ui/
  palettes/
  manifests/
prompts/
  templates/
  generated/
tools/
  slice_sheet.*
  quantize.*
  pack_atlas.*
  check_alpha.*
  check_palette.*
  validate_asset_manifest.*
```

Each generated asset needs a manifest. The manifest should include:

- stable asset ID,
- source prompt path,
- input image roles,
- built-in imagegen or confirmed fallback mode,
- generated source path,
- accepted output path,
- derived runtime path,
- dimensions,
- target palette,
- transparency method,
- slicing recipe,
- validation status,
- reviewer notes,
- consuming runtime module.

The default generation path is built-in `imagegen`. Command-line interface
(CLI) fallback is allowed only when explicitly requested or when true native
transparency is confirmed after the chroma-key path proves unsuitable.

## 12. Audio

Simulation emits audio events; presentation consumes them through Kira. Audio
events include lance clash, wing flap, explosion, capture complete, tournament
horn, duel drums, truce fanfare, banquet ambience, and victory fanfare.

The audio layer owns:

- one audio manager,
- music, sound-effect, and user-interface buses,
- rate limits for frequent sounds,
- spatial panning from camera-relative position,
- state-driven music layers for normal battle, tournament, duel, truce, and
  results.

Audio playback never mutates authoritative match state.

## 13. Persistence and replay

Warfront saves store:

- schema version,
- campaign seed,
- turn,
- regions and ownership,
- supply routes,
- house relations,
- queued events,
- active modifiers,
- currencies,
- unlocks,
- penalties.

Replay capture stores seed, configuration hash, asset manifest hash, and
per-tick validated inputs. High-level state actions should also be exportable
as a validator trace so lifecycle bugs can be checked without replaying every
pixel-level interaction.

## 14. Testing and gates

The implementation needs tests at three levels:

- unit tests for fixed math, terrain queries, joust scoring, ledger mutation,
  and asset manifest parsing;
- behavioural tests for match lifecycle, ceremony rules, reward commits, and
  Warfront transitions;
- property or model tests for terrain fairness, state graph invariants, and
  validator traces.

Required gates before code commits:

```bash
make check-fmt
make check-state-graphs
make lint
make test
cargo doc --no-deps --workspace
```

Required gates for documentation changes:

```bash
make fmt 2>&1 | tee /tmp/markdownfmt-skyjoust-$(git branch --show-current).out
cargo doc --no-deps --workspace 2>&1 | tee /tmp/cargo-doc-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

`make nixie` applies whenever Markdown contains Mermaid diagrams or when edited
documents are part of the graph/design set.

## 15. Design decisions

Table 4: Current decisions.

| Decision                                                      | Rationale                                                                                                                                                             |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Use Bevy ECS without Bevy rendering for MVP                   | The state graph and simulation need ECS scheduling. The visual target needs a fixed pixel framebuffer with direct control over scaling and blitting.                  |
| Use fixed 120 Hz simulation                                   | The Product Requirements Document (PRD) requires consistent collision feel and low-latency input. A fixed tick also supports replay and later rollback work.          |
| Treat Stateright as the lifecycle contract                    | The validator already proves the high-level scoring, reward, ceremony, and Warfront handoffs. Runtime tests should align with it instead of inventing parallel rules. |
| Draw gameplay text at runtime                                 | Generated text is not reliable enough for counters, timers, prompts, and state-dependent values. Runtime text keeps UI correct and localizable later.                 |
| Use imagegen outputs as production sources, not final atlases | Generated sheets need human review, slicing, palette normalization, transparency validation, and manifests before deterministic runtime use.                          |

## 16. Deferred decisions

- Final virtual resolution after HUD legibility tests.
- Exact crate split for the first implementation pass is tracked in
  [ADR 002](adr/002-crate-layout-and-public-api.md).
- Fixed-point type and scale after movement prototype benchmarks is tracked in
  [ADR 003](adr/003-fixed-point-type-and-scale.md).
- Atlas packing format and manifest schema version is tracked in
  [ADR 004](adr/004-atlas-packing-format-and-manifest-schema.md).
- Whether Warfront map rendering uses generated region base art or
  deterministic map tiles plus generated ornamentation is tracked in
  [ADR 005](adr/005-warfront-renderer-map-art-strategy.md).
- Whether weddings and banquets ship as visible preview content or remain
  hidden post-MVP graph states until diplomacy is implemented.

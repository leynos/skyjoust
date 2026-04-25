## High-level technical design (Rust + Bevy ECS/app + Pixels + Kira)

This design treats Bevy primarily as an ECS + scheduling + state machine framework, while Pixels owns the actual “draw a framebuffer to a window” job, and Kira owns audio. That combo stays lean, keeps the retro feel honest (a real pixel framebuffer), and avoids fighting Bevy’s renderer when you don’t want meshes, cameras, and shaders doing interpretive dance.

### 1. Goals and non-goals

Goals:

* Deterministic-ish arcade feel: collisions resolve consistently, inputs feel immediate, simulation runs on a fixed timestep.
* Procedural terrain with local deformation (craters, collapsed structures).
* Clean separation between simulation (authoritative) and presentation (render/audio/UI).
* Single-player + local multiplayer first; online later without painting ourselves into a corner.

Non-goals (initially):

* Deep Bevy render integration, sprite pipelines, or 3D. Pixels is the renderer.
* Perfect cross-platform determinism to the last bit (we’ll design towards it, but don’t overpay upfront).

### 2. Runtime architecture overview

Process anatomy:

* **winit event loop** drives the application.
* Each frame:

  1. pump winit events → update an `InputState` resource
  2. run **0..N fixed simulation ticks** (e.g. 120 Hz) until caught up
  3. run a **presentation pass** (render to Pixels framebuffer + audio flush + UI)
  4. ask Pixels to present

Bevy’s `App` exists, but we provide a **custom runner** so winit + Pixels remains in charge of the window and swapchain. Bevy becomes the world/schedule engine.

Key timing choice:

* `SIM_DT = 1/120s` fixed update
* Rendering runs “as fast as sensible” (vsync), interpolating visuals if you want smoothness.

### 3. Crate/module layout

A pragmatic split (single workspace, multiple crates if it grows):

* `game_app/`

  * `main.rs` (winit runner + Pixels surface init)
  * `app.rs` (Bevy App construction, plugins, schedules)

* `core/`

  * `math` (fixed-point or subpixel integer types, helpers)
  * `ids` (stable ordering keys)
  * `events` (typed gameplay events)
  * `config` (tuning constants, loadable)

* `sim/`

  * `movement` (mount flight model)
  * `physics` (integration, forces)
  * `collision` (broadphase + narrowphase + resolution)
  * `combat` (joust resolution, ordinance, damage)
  * `objectives` (keeps/outposts/morale)
  * `special_events` (tournament/duel/truce logic)

* `terrain/`

  * `gen` (seeded procedural generation)
  * `chunks` (storage, dirty tracking)
  * `deform` (bomb craters, collapse)
  * `terrain_collision` (sampling + contact)

* `render/`

  * `framebuffer` (Pixels target, scaling)
  * `atlas` (sprite sheets, font)
  * `draw` (layers, culling, blitting)
  * `effects` (screen shake, flashes, palette tweaks)

* `audio/`

  * `kira_backend` (AudioManager, buses, handles)
  * `sfx` (event→sound mapping, pooling, rate-limits)
  * `music` (stateful layering per game state/event)

* `ui/`

  * `hud` (altitude advantage, morale, objectives)
  * `menus` (simple bitmap UI)
  * `debug` (fps, seed display, collision overlays)

* `save/` (Warfront map persistence, replays later)

### 4. Data model: ECS components and resources

Core components (representative, not exhaustive):

* `Transform2D { pos: IVec2, rot: i16 }`
  Positions in **subpixel integer space** (e.g. 1 unit = 1/256 pixel). Rotation can be coarse (fixed angle) if you want strict determinism.

* `Velocity2D { v: IVec2 }`

* `Mount { kind, flap_power, stall_speed, drag… }`

* `Rider { house_id, stamina, unhorsed_timer… }`

* `Lance { state, brace_window, reach, cone_angle… }`

* `Health { hp, invuln_timer… }`

* `Team { id }`

* `Collider { shape }` (capsule-ish for riders, AABB for projectiles, etc.)

* `Structure { kind, integrity, destructible }`

* `Objective { kind, capture_progress… }`

* `Projectile/Bomb { fuse, radius, impulse, terrain_damage… }`

* `Renderable { sprite_id, layer, flip, tint }`

* `AnimationState { frame, timer }`

* `CameraTarget` (for follow logic)

* `AIController` (later)

Key resources:

* `GameClock { sim_dt, accumulator }`
* `InputState { players: [PlayerInput; N] }`
* `RngState` (seed + deterministic RNG instance)
* `TerrainWorld` (chunk store + metadata)
* `CollisionWorld` (spatial hash grid + scratch buffers)
* `MatchRules` (current ruleset modifiers; event-driven)
* `MatchState` (morale, score, timers)
* `CampaignState` (Warfront map, ownership, queued events)
* `AssetStore` (sprite atlases, fonts, sfx buffers)
* `AudioState` (Kira managers, buses, handles)

### 5. Scheduling: system sets and update phases

You want ruthless discipline here; it’s what keeps arcade games from becoming soup.

Suggested phases per simulation tick (fixed update):

1. **Input Sampling**

* Convert current `InputState` into per-entity intents (`ControlIntent` components), with deadzones, buffering, and optional “grace windows”.

2. **Control → Forces**

* Translate intent into forces/impulses: flap, dive, brake, lance brace, fire ordinance.
* Emit `GameplayEvent`s (e.g. `BraceStarted`, `BombDropped`).

3. **Physics Integrate**

* Apply gravity, drag, impulses.
* Integrate positions using integer arithmetic (or carefully clamped f32 if you accept mild nondeterminism).
* Clamp to world bounds.

4. **Collision Build (Broadphase)**

* Populate spatial hash with entity AABBs expanded by velocity.
* Gather potential pairs (entity-entity, entity-terrain).

5. **Collision Resolve (Narrowphase)**

* Resolve penetrations with minimal impulses.
* Terrain contacts: sample terrain grid/heightfield to compute contact normals and slide.

6. **Combat Resolution**

* Joust detection: lance cones + collision timing.
* Determine outcome: unhorse / knockback / shatter / kill.
* Apply damage and impulses.
* Spawn particles, emit audio events.

7. **Ordinance + Terrain Deformation**

* Explosions modify terrain chunks (circle carve, scorch, rubble).
* Mark chunks dirty for renderer + collision cache refresh.

8. **Objectives + Morale**

* Capture points, structure damage, morale drain/gain.

9. **Special Events**

* Tournament/duel/truce state machines mutate `MatchRules` and UI prompts.

10. **Cleanup**

* Despawn dead entities, expire timers, compact buffers.

Presentation pass (variable update, once per rendered frame):

* Camera smoothing, animation frame advance, particle visual updates
* Draw layers into Pixels framebuffer
* Audio: consume queued audio events and play via Kira
* HUD/menu drawing last

### 6. Terrain subsystem design (procedural + deformable)

For a side-scroller, you can get 80% of the “Sopwith feel” with a terrain representation that is simple, fast, and destructible.

Representation:

* **Chunked 2D tile grid** where each cell encodes `Material` (Air, Dirt, Rock, Wood, Stonework…).
* Chunk size: e.g. `64x64` or `128x64` cells for cache friendliness.
* World storage: `HashMap<ChunkCoord, Chunk>` or a slab + sparse index if you want to be fancy.
* Each chunk maintains:

  * `cells: Vec<Material>`
  * `dirty_rect: Option<Rect>` for incremental redraw
  * `collision_cache` (optional: run-length encoded “solid spans” per column)

Procedural generation:

* Seeded noise fields generate:

  * base ground height per x
  * rock layers below a depth
  * occasional overhang/caves via a secondary mask (optional; you can defer)
  * structure placement (keeps/outposts) respecting fairness constraints
* Important: generation must be **pure** (seed + parameters → identical result), and must not depend on hash iteration order.

Deformation:

* Explosion at `(x,y)` with radius `r`:

  * iterate affected chunks
  * for each cell in bounding box, carve if within radius (and material destructible)
  * optionally deposit rubble materials around rim (purely visual or mild collision)
* Post-deform:

  * update chunk dirty rect
  * invalidate collision spans for affected columns
  * emit `TerrainChanged` event for renderer

Collision queries:

* Fast path: sample tile solidity at subpixel position → contact resolution.
* Optional optimisation: per-column solid spans to detect surface collisions quickly.

### 7. Collision and joust resolution (the “don’t feel random” bit)

Broadphase:

* Spatial hash grid keyed by cell coordinate in world units (not pixels). Each tick rebuild or incrementally update.

Narrowphase:

* Prefer simple shapes:

  * riders: capsule or two-circle composite
  * projectiles: small circles/AABBs
  * structures: AABBs, with optional per-part hitboxes

Joust contact:

* Separate “physics collision” from “joust interaction”. You can graze someone without a meaningful lance contact.
* Detect joust opportunity when:

  * rider A is within rider B’s lance cone (and vice versa if you want mutual)
  * within reach distance
  * brace window state influences effectiveness

Resolution model (deterministic ordering):

* Compute an `Engagement` record for candidate pairs and sort by `(tick_index, min(entity_id), max(entity_id))`.
* For each engagement:

  * calculate advantage score:

    * height delta (signed)
    * relative vertical velocity
    * brace timing quality
    * mount class modifiers
    * debuffs (broken lance, stunned)
  * map score to outcome and apply:

    * impulses (knockback)
    * state transitions (unhorse, stun, lance broken)
    * damage
* Emit explicit events: `JoustHit { winner, loser, outcome, score }`

That “sort engagements” step matters more than people expect; it prevents “pair resolution order” from becoming a hidden random number generator.

### 8. Rendering with Pixels (framebuffer-first)

Logical resolution:

* Pick a fixed internal resolution (e.g. `480x270` or `320x180`) and scale to window. This keeps art consistent and performance predictable.

Render pipeline (pure CPU blit into framebuffer):

1. Clear sky gradient / background
2. Parallax layers (pre-rendered strips; x offset from camera)
3. Terrain draw

   * either redraw only dirty rects into a cached terrain layer buffer, then composite
   * or draw visible chunks each frame (often fine at low resolutions)
4. Entities

   * sort by `layer` then `y` (optional) for readability
   * sprite blit with alpha (straight alpha, avoid fancy blending initially)
5. Particles and screen-space effects (flash, vignette)
6. HUD/UI (bitmap font)

Camera:

* World pos is integer subpixels; camera computes a top-left origin in the same space.
* Convert world→screen by subtracting camera origin and shifting by subpixel scale.

Debug overlays (dev build):

* collision bounds, lance cones, terrain solidity heatmap, seed display

### 9. Audio with Kira (event-driven, rate-limited)

Kira setup:

* One `AudioManager` resource.
* Three buses:

  * `MusicBus`
  * `SfxBus`
  * `UiBus`
* SFX loaded into `StaticSoundData` (or streamed if huge, but SFX shouldn’t be huge).

Audio event ingestion:

* Simulation emits `AudioEvent`s (e.g. `LanceClash`, `WingFlap`, `Explosion`, `CaptureComplete`, `TournamentHorn`).
* Presentation step consumes events and plays sounds:

  * Apply panning based on (sound_x - camera_x)
  * Apply attenuation by distance
  * Apply rate limits (no 200 wing flaps a second, please)
  * Use small pools for common sounds to avoid handle churn

Music:

* State machine keyed off `AppState` + `SpecialEventState`
* Crossfade layers when entering tournament/duel/truce
* Optional stingers for decisive joust outcomes

### 10. Asset strategy (including Web builds)

For portability:

* `include_bytes!` for sprite atlases, fonts, and short audio, so WASM packaging is painless.
* Keep an “unpacked assets” mode for desktop dev with hot reload later (not mandatory at first).

Sprite atlas:

* Load PNG → RGBA8 buffer once at startup.
* Precompute sprite rects and optional collision masks if you want pixel-perfect hits (I’d avoid at MVP).

### 11. Save/load and replays (design now, implement later)

Warfront save:

* Store: world seed, region ownership, upgrades, queued special events, player unlocks.
* Keep it versioned (`schema_version`) and robust to additions.

Replay groundwork:

* If you keep simulation fixed-timestep and input-driven, replays can be “seed + input stream”.
  That’s a big future win (debugging, sharing ridiculous jousts).

### 12. Testing and tooling

* Deterministic terrain generation tests: seed → hash of generated chunks/structures.
* Property tests for “fairness constraints” (spawn corridors, objective reachability).
* Golden tests for joust resolution: controlled initial conditions → expected outcome.
* Benchmarks around terrain deformation and chunk redraw.

### 13. Minimal runner shape (pseudocode)

Just to make the integration idea concrete:

```rust
// Pseudocode: winit owns loop; Bevy app updates inside it; Pixels presents.
event_loop.run(move |event, _, control_flow| {
    input_state.ingest_winit(&event);

    if matches!(event, Event::MainEventsCleared) {
        let now = Instant::now();
        clock.accumulator += now - clock.last;
        clock.last = now;

        while clock.accumulator >= SIM_DT {
            app.world_mut().resource_mut::<InputState>().clone_from(&input_state);
            app.update(); // runs FixedUpdate schedule once (or you run your own fixed schedule)
            clock.accumulator -= SIM_DT;
        }

        // Presentation (variable)
        render_into_pixels_framebuffer(&mut app.world_mut(), pixels.frame_mut());
        audio_pump(&mut app.world_mut());
        pixels.render().unwrap();
    }
});
```

In practice you’ll likely split Bevy schedules so `app.update()` can run only the relevant sets, or you maintain two schedules (fixed sim vs presentation) and call them explicitly.

### 14. Future-proofing for online multiplayer

If you decide to go online later, this architecture adapts:

* Keep simulation authoritative and input-driven.
* Make entity ordering stable (sorted engagements, stable RNG).
* Avoid using wall-clock time in sim logic.
* Introduce rollback/netcode later by capturing world snapshots or deltas at fixed ticks.

Even if you never ship online, the discipline pays off: fewer heisenbugs, easier balancing, and replays “for free”.


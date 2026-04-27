# Project Skyjoust development plan

Status: Living plan

Scope: implement the Project Skyjoust Minimum Viable Product (MVP) described in
the Product Requirements Document (PRD) and technical design, while keeping the
runtime consistent with the state graph bundle, Stateright validator, and
reference design book images in `ref/`.

## 1. Source-of-truth order

When documents disagree, use this order:

1. `AGENTS.md` and repository style rules.
2. `docs/skyjoust-product-requirements.md`.
3. `docs/skyjoust-state-graphs.yaml` and
   `crates/skyjoust_stateright_validator`.
4. `docs/skyjoust-technical-design.md`.
5. Reference images in `ref/`.
6. This development plan.

The plan is an execution guide. If implementation discovers that this plan
contradicts the technical design or state graph, update the plan in the same
change that resolves the contradiction.

## 2. MVP boundaries

### In scope

- Skirmish.
- Warfront-lite.
- Local versus.
- Two or three houses.
- Four to six mount types.
- Fixed-tick arcade flight.
- Joust contact with knockback, unhorse, shatter, and clean-kill outcomes.
- Bomb and bolt ordnance.
- Procedural terrain with limited deformation.
- Keeps, outposts, shrines, supply routes, and morale.
- Tournament and duel ceremony events.
- Warfront currencies: glory, coin, influence, and laurels.
- Runtime heads-up display (HUD), event banners, menus, and Warfront map
  interface.
- Imagegen-based source asset generation with manifests and validation.

### Out of scope for MVP

- Online multiplayer.
- Ranked ladder.
- Full wedding and banquet diplomacy.
- Large roster expansion.
- Console and mobile ports.
- Full localization.
- Generated images used directly as gameplay-critical user interface (UI).

Wedding and banquet states remain in the data model and state graph contract,
so the runtime does not need a structural rewrite later.

## 3. Branch and gate rules

- Work on a feature branch, not `main`.
- Commit each logical change after its relevant gates pass.
- Run commands through `tee` into `/tmp` logs using the project and branch name
  in the file name.
- Do not run format, lint, or tests in parallel.
- Prefer Makefile targets over raw tool commands.
- Keep unrelated local changes unstaged.

Code gates:

```bash
make check-fmt 2>&1 | tee /tmp/check-fmt-skyjoust-$(git branch --show-current).out
make check-state-graphs 2>&1 | tee /tmp/check-state-graphs-skyjoust-$(git branch --show-current).out
make lint 2>&1 | tee /tmp/lint-skyjoust-$(git branch --show-current).out
make test 2>&1 | tee /tmp/test-skyjoust-$(git branch --show-current).out
```

Documentation gates:

```bash
make fmt 2>&1 | tee /tmp/markdownfmt-skyjoust-$(git branch --show-current).out
cargo doc --no-deps --workspace 2>&1 | tee /tmp/cargo-doc-skyjoust-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/markdownlint-skyjoust-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-skyjoust-$(git branch --show-current).out
git diff --check 2>&1 | tee /tmp/diff-check-skyjoust-$(git branch --show-current).out
```

## 4. Phase 0: repository and documentation baseline

Goal: make the current design set reliable enough to build from.

Tasks:

- Confirm the current branch and working-tree status.
- Read `AGENTS.md`, the PRD, technical design, state graph bundle, validator
  README, and reference image set.
- Replace generated or stale planning prose with the current technical design
  and this development plan.
- Ensure the development plan names the imagegen asset pipeline and Stateright
  verification path.
- Run documentation gates.

Exit criteria:

- `docs/skyjoust-technical-design.md` names visual, state, runtime, and asset
  contracts.
- `docs/development-plan.md` provides a phase-by-phase implementation path.
- Documentation gates pass, or failures are recorded with exact log paths.

## 5. Phase 1: asset pipeline foundation

Goal: create a repeatable path from reference images and prompts to validated
runtime assets.

### 5.1 Directory layout

Create this structure before generating project-bound art:

```plaintext
assets/
  source/imagegen/style_guides/
  source/imagegen/characters/
  source/imagegen/mounts/
  source/imagegen/terrain/
  source/imagegen/ui/
  source/imagegen/ceremony/
  source/imagegen/warfront/
  derived/atlases/
  derived/cutouts/
  derived/ui/
  palettes/
  manifests/
prompts/
  templates/
  generated/style_guides/
  generated/characters/
  generated/mounts/
  generated/terrain/
  generated/ui/
  generated/ceremony/
  generated/warfront/
tools/
```

### 5.2 Prompt templates

Create prompt templates before live generation:

- `prompts/templates/README.md`
- `prompts/templates/style-guide-page.md`
- `prompts/templates/house-sheet.md`
- `prompts/templates/mount-sheet.md`
- `prompts/templates/animation-sheet.md`
- `prompts/templates/terrain-material-sheet.md`
- `prompts/templates/environment-sheet.md`
- `prompts/templates/ui-ornament.md`
- `prompts/templates/event-panel.md`
- `prompts/templates/warfront-map-panel.md`
- `prompts/templates/transparent-chromakey.md`
- `prompts/templates/edit-invariants.md`

Each template uses the shared imagegen schema:

```plaintext
Use case:
Asset type:
Primary request:
Input images:
Scene/backdrop:
Subject:
Style/medium:
Composition/framing:
Lighting/mood:
Colour palette:
Materials/textures:
Text (verbatim): ""
Constraints:
Avoid:
```

Edit templates add:

```plaintext
Change:
Preserve:
Constraints:
```

Rules:

- Use built-in `imagegen` by default.
- Do not use command-line interface (CLI) fallback unless explicitly requested
  or confirmed for true native transparency.
- Label every input image by role, for example `style reference`, `layout
  reference`, `subject reference`, or `edit target`.
- Keep gameplay-critical text out of generated images.
- Put unavoidable decorative text in `Text (verbatim)` and require no duplicate
  text.
- Avoid vague prompt language such as "epic", "stunning", "masterpiece", or
  "insane detail". Describe visible materials, silhouettes, lighting, and
  composition instead.

### 5.3 Manifest schema

Create a versioned manifest format. TOML is preferred because it is readable in
reviews and easy to parse in Rust.

Required fields:

```toml
schema = "skyjoust.asset_manifest.v1"
asset_id = "mount.roc.side_sheet.v1"
status = "source-approved"
tool_mode = "built-in-imagegen"
source_prompt = "prompts/generated/mounts/roc-side-sheet-v1.md"
generated_source = "assets/source/imagegen/mounts/roc-side-sheet-v1.png"
runtime_output = "assets/derived/atlases/mounts/roc-v1.png"
palette = "assets/palettes/skyjoust-master-v1.json"
transparency = "none"
slicing_recipe = "tools/slice_sheet.toml#mount-side-sheet"
consuming_module = "render::atlas"

[[input_images]]
path = "ref/skyjoust-design-book-3.png"
role = "style and layout reference"

[[checks]]
name = "visual_review"
status = "pass"
notes = "Silhouette and house colours match the design-book page."
```

Allowed `status` values:

- `draft`
- `source-approved`
- `needs-iteration`
- `derived`
- `runtime-approved`
- `rejected`

### 5.4 Generation waves

Run generation in waves. Do not slice sprites until the relevant source sheet
has passed visual review.

Wave 1: style anchors

- overview design-book page,
- house and hero sheet,
- mount guide sheet,
- borderland environment sheet,
- core gameplay HUD sheet,
- ceremony event sheet,
- Warfront map sheet.

Use the existing `ref/` images as the first approved style anchors. Generate
new anchors only when the runtime needs missing views or corrected details.

Wave 2: deterministic UI source

- brass and navy panel frames,
- corner ornaments,
- house shields,
- role icons,
- action icons,
- currency icons,
- objective icons,
- event badges,
- Warfront route markers.

These assets may be generated as source art, but final runtime forms should be
clean cutouts, palette-normalized tiles, or renderer-composed panels.

Wave 3: mount and rider sheets

- side-view mount sheets,
- action-pose mount sheets,
- rider-on-mount sheets by house,
- brace, dive, flap, hit, unhorse, and recovery reference sheets.

Generated sheets are reference material until slicing and manual cleanup pass.
The sprite-strip look in the reference images should guide silhouette and frame
count, not bypass validation.

Wave 4: terrain and environments

- material tiles for stone, cliff rock, grassland, dirt or mud, wood, water,
  rubble, scorch, and crater rims,
- keep walls, breached keep walls, outpost towers, shrines, bridges, aqueduct
  segments, villages, cages, and supply carts,
- parallax backdrops for cliffs, valleys, waterfalls, castles, and ridges.

Terrain collision data remains generated by code. Art tiles decorate material
cells and structures.

Wave 5: ceremony and Warfront interfaces

- tournament bracket panels,
- duel prompt panels,
- honour and dishonour icons,
- reward preview ornaments,
- truce status panels,
- treaty and banquet source art for post-MVP,
- Warfront map frames, action-card ornaments, pressure arrows, and event
  forecast icons.

Live values, active modifiers, route states, timers, and currency totals are
drawn by the renderer.

### 5.5 Transparent asset path

Use the built-in imagegen chroma-key path for simple opaque subjects:

1. Prompt for a perfectly flat chroma-key background.
2. Copy the generated source into `assets/source/imagegen/...`.
3. Run the installed chroma-key removal helper.
4. Save the alpha output under `assets/derived/cutouts/...`.
5. Validate transparent corners, subject coverage, and edge fringing.
6. Record the source and derived paths in the manifest.

Ask before using CLI fallback for true native transparency. Complex subjects
such as feathers, smoke, glass, translucent fabric, and reflective armour may
need manual cleanup even after chroma-key removal.

### 5.6 Post-processing tools

Add tools only when a generated asset needs them:

- `tools/validate_asset_manifest.*` checks manifest paths, statuses, and
  required fields.
- `tools/quantize.*` maps images to a named palette.
- `tools/slice_sheet.*` extracts frames from approved sheets.
- `tools/pack_atlas.*` creates runtime atlases and metadata.
- `tools/check_alpha.*` validates cutout transparency.
- `tools/check_palette.*` reports out-of-palette pixels.

Each tool must have focused tests before it becomes a gate.

Exit criteria:

- Template files exist.
- Manifest schema exists with at least one sample.
- Asset source, derived, prompt, and manifest directories exist.
- Documentation explains built-in imagegen, chroma-key transparency, manifests,
  and runtime text ownership.

## 6. Phase 2: runtime foundation

Goal: start the game with a fixed virtual resolution and placeholder panels.

Tasks:

- Decide the initial crate split. A single crate plus
  `skyjoust_stateright_validator` is acceptable for the first vertical slice if
  module boundaries match the technical design.
- Add runtime dependencies behind explicit feature choices.
- Implement a `winit` runner.
- Initialize `pixels` with fixed virtual resolution.
- Add a Bevy app for entity-component system (ECS) world, resources,
  schedules, and state resources.
- Add placeholder draw layers for sky, terrain, riders, HUD, and debug overlay.
- Implement integer scaling and resize handling.
- Draw a placeholder HUD shaped like the reference page:
  - team strip,
  - timer,
  - renderer-owned debug seed text labelled `Seed`,
  - altitude indicator,
  - brace window,
  - morale,
  - ammo,
  - objective strip.
- Run code gates.

Exit criteria:

- The application opens a window.
- The placeholder renderer is nonblank and stable under resize.
- No authoritative simulation state is mutated during presentation.

## 7. Phase 3: state graph runtime shell

Goal: make the state graph executable before gameplay systems grow around it.

Tasks:

- Define state resources for app runtime, match lifecycle, ceremony events,
  objectives, scoring, rewards, and Warfront.
- Define the deterministic event bus used by graph evaluation.
- Implement pure guard functions for the selectors in
  `docs/skyjoust-state-graphs.yaml`.
- Implement action queues that apply after graph evaluation in stable order.
- Emit trace actions matching the validator's `SkyAction` names where the
  abstraction level matches.
- Add trace-export tests that validate known-good paths through
  `validate_trace`.
- Run the Stateright validator in the test suite.

Exit criteria:

- A skirmish lifecycle can move from assets loaded through match start,
  countdown, victory, final score, reward tally, and reward commit.
- Rewards cannot commit before final score export.
- Graph tests fail on deliberately illegal score-after-final and
  reward-before-final traces.

## 8. Phase 4: flight, collision, and joust prototype

Goal: prove the fun before implementing full Warfront content.

Tasks:

- Implement fixed-point or integer subpixel position.
- Add rider, mount, lance, velocity, team, health, and control-intent data.
- Implement flap, dive, drag, stall, landing, and respawn basics.
- Add simple terrain floor and world bounds.
- Add broadphase candidate collection with stable ordering.
- Add lance reach and cone checks.
- Resolve joust outcomes from altitude, vertical velocity, brace quality,
  lance alignment, mount modifier, and debuffs.
- Emit `JOUST_OUTCOME` into recovery, scoring, audio, and visual effects.
- Add golden tests for knockback, unhorse, shatter, and clean kill.

Exit criteria:

- Two riders can launch, collide, and produce deterministic joust outcomes.
- Replaying the same seed and inputs produces the same outcome sequence.
- Joust events route to score atoms without bypassing the scoring graph.

## 9. Phase 5: terrain and objectives

Goal: turn the prototype arena into a destructible battlefield.

Tasks:

- Implement chunked material storage.
- Implement seeded terrain generation for one biome.
- Add keep, outpost, shrine, supply route, and hostage anchors.
- Add fairness validation for spawn safety, objective reachability, route
  reachability, and distance balance.
- Add bomb explosions that deform terrain and mark dirty rectangles.
- Add structure damage, core exposure, and keep breach.
- Add outpost capture and shrine claim interactions.
- Add terrain render cache invalidation after deformation.

Exit criteria:

- A generated map can be rejected when fairness constraints fail.
- Bombs can deform destructible materials without touching indestructible
  materials.
- Keep breach emits victory progress and enters round-over flow through the
  state graph.

## 10. Phase 6: rendering and asset integration

Goal: replace placeholders with validated runtime assets without weakening
gameplay readability.

Tasks:

- Implement atlas loading from derived assets and manifests.
- Add palette-aware sprite blitting.
- Add bitmap font rendering for live gameplay text.
- Add HUD ornaments from approved UI source assets.
- Add mount and rider placeholder sprites from approved source sheets after
  slicing and validation.
- Add terrain tiles and structure sprites from validated material sheets.
- Add debug overlays for collision bounds, lance cones, terrain solidity, seed,
  and event bus.
- Add screenshot-based smoke tests for nonblank rendering where practical.

Exit criteria:

- Runtime can load only manifest-approved derived assets.
- Missing assets fail with named diagnostics.
- HUD values are drawn by the renderer, not read from generated text.

## 11. Phase 7: audio

Goal: provide timing and impact feedback without coupling audio to simulation
mutation.

Tasks:

- Add Kira manager and music, sound-effect, and UI buses.
- Map gameplay events to audio events.
- Add rate limits for wing flaps, impacts, and repeated UI sounds.
- Add camera-relative panning and distance attenuation.
- Add state-driven music layers for normal battle, tournament, duel, results,
  and Warfront.

Exit criteria:

- Audio events consume simulation output in presentation.
- Muting or audio-device failure cannot change match outcomes.
- Event transitions trigger the correct stingers.

## 12. Phase 8: ceremony MVP

Goal: ship tournament and duel as mechanics, not cutscenes.

Tournament tasks:

- Build an arena-lane ruleset.
- Disable or limit ordnance through `MatchRules`.
- Add registration, bracket seed, round countdown, round active, round score,
  and champion declared states.
- Award laurels only after tournament completion.
- Render bracket and reward preview panels from state.

Duel tasks:

- Add challenge, accept, arena lock, active duel, honour violation, and
  resolve-duel states.
- Lock duelists and block non-duelist scoring.
- Emit dishonour penalties for interference.
- Award duel rewards only after resolved duel.
- Render duel prompt, rule summary, and consequence panels.

Exit criteria:

- Stateright properties for joust-only ordnance, duel lock, laurels, and duel
  rewards remain green.
- Ceremony events change rules and rewards through state graph actions.

## 13. Phase 9: Warfront-lite

Goal: connect matches into a readable campaign layer.

Tasks:

- Implement region ownership and adjacency.
- Implement supply routes, pressure vectors, and queued events.
- Implement glory, coin, influence, and laurels.
- Implement recruit/deploy, repair/supply, and political move card shells.
- Implement event forecast from campaign state.
- Apply committed match rewards to Warfront only after `REWARDS_COMMITTED`.
- Draw the Warfront map with deterministic overlays and generated ornaments.

Exit criteria:

- Winning a match changes region state only after reward commit.
- Warfront does not mutate during active match simulation.
- The map communicates control, supply, pressure, events, active modifiers,
  and next action choices.

## 14. Phase 10: accessibility, tuning, and release polish

Goal: make the MVP playable by new players and maintainable by developers.

Tasks:

- Add remappable controls.
- Add toggle or hold options for brace and interaction where appropriate.
- Add reduced screen-shake and high-contrast objective options.
- Add tutorial prompts for altitude, brace, resupply, and objectives.
- Add performance counters for simulation tick, render frame, terrain
  deformation, collision, and audio event processing.
- Tune match length toward six to ten minutes.
- Tune house and mount balance against win-rate bands from local test data.
- Add save versioning and basic replay capture.

Exit criteria:

- New players can complete a tutorial loop.
- Median skirmish length lands in the target range during local tests.
- Debug overlays can explain a surprising joust outcome.

## 15. Verification matrix

Table 1: Required checks by subsystem.

| Subsystem     | Required checks                                                                                   |
| ------------- | ------------------------------------------------------------------------------------------------- |
| State graph   | Stateright exhaustive smoke test, trace validator tests, illegal transition tests                 |
| Flight        | fixed-tick replay tests, movement bounds tests, stall and recovery tests                          |
| Joust         | golden outcome tests, stable ordering tests, brace-window tests                                   |
| Terrain       | seed determinism tests, fairness property tests, deformation tests                                |
| Objectives    | keep breach, outpost capture, shrine claim, supply block tests                                    |
| Rewards       | final-score-before-reward tests, tournament laurels tests, duel reward tests, truce penalty tests |
| Assets        | manifest parser tests, path existence checks, alpha checks, palette checks, slicing checks        |
| Rendering     | nonblank frame smoke tests, resize tests, HUD live-text tests                                     |
| Audio         | event mapping tests, rate-limit tests, audio-disabled safety test                                 |
| Documentation | make fmt, markdownlint, nixie when diagrams are touched, diff check                               |

## 16. First implementation slice

The first code slice should be deliberately narrow:

1. Runtime window and framebuffer.
2. Placeholder HUD matching the reference hierarchy.
3. Minimal state resources.
4. Skirmish lifecycle trace from start to reward commit.
5. Validator-backed trace test.
6. One manifest schema sample and parser test.

This slice proves the runtime loop, visual hierarchy, graph contract, and asset
pipeline before combat complexity arrives.

## 17. Open decisions

- Initial virtual resolution: `480x270`, `512x288`, or `640x360`.
- Whether the first runtime crate split is modular crates or one crate with
  strict modules.
- Fixed-point scale and numeric type.
- Palette size and final colour ramps.
- Whether atlas metadata is TOML, RON, or JSON.
- Whether Warfront region base art is generated as a single source map or
  assembled from deterministic tiles plus generated ornamentation.

Resolve each decision in the design document or a narrow Architecture Decision
Record before implementation relies on it.

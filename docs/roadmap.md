# Project Skyjoust roadmap

This roadmap translates the Product Requirements Document (PRD), technical
design, state graph bundle, Stateright validator contract, and reference design
book into an outcome-oriented delivery sequence. It does not promise dates.
Each phase carries one testable idea at the GIST level; the steps underneath
answer sequencing questions and leave behind usable game functionality instead
of isolated layers. Task counts are intentionally uneven: a step is split only
where the build work has distinct acceptance surfaces, and thin validation work
stays attached to the delivery task it proves.

The primary sources are `docs/skyjoust-product-requirements.md`,
`docs/skyjoust-technical-design.md`, `docs/skyjoust-state-graphs.yaml`,
`crates/skyjoust_stateright_validator/spec/validator_contract.yaml`, and the
reference images in `ref/`. No RFCs or ADRs exist yet; early roadmap tasks
create narrow Architecture Decision Records (ADRs) where the design still names
unresolved choices.

## 1. Foundational contracts and playable spine

Idea: if Skyjoust settles its runtime contracts, state graph shell, and asset
pipeline before feature work broadens, later slices can prove arcade feel and
Warfront progression without repeatedly rewriting interfaces.

This phase is foundational, but it must still produce a running slice: a
windowed application with a placeholder heads-up display (HUD), a skirmish
lifecycle trace, and a validated asset manifest sample.

### 1.1. Ratify decisions that would otherwise force rework

This step answers which unresolved choices block the first playable loop. Its
outcome informs crate layout, fixed-point math, asset metadata, and renderer
setup. See `docs/skyjoust-technical-design.md` §§15-16 and
`docs/development-plan.md` §17.

- [ ] 1.1.1. Record the initial runtime crate split as an ADR.
  - Decide whether the first implementation uses multiple runtime crates or
    one crate with strict modules beside `skyjoust_stateright_validator`.
  - See `docs/skyjoust-technical-design.md` §4 and
    `docs/development-plan.md` §6.
  - Success: the ADR defines dependency direction and the allowed migration
    path if the split changes later.
- [ ] 1.1.2. Record the first virtual resolution and scaling policy as an ADR.
  - Requires 1.1.1.
  - Compare `480x270`, `512x288`, and `640x360` against HUD legibility and
    integer scaling.
  - See `docs/skyjoust-technical-design.md` §§10 and 16.
  - Success: placeholder HUD text, counters, and panels fit at the chosen
    resolution without relying on generated text.
- [ ] 1.1.3. Record the fixed-point position scale and numeric boundary.
  - Requires 1.1.1.
  - Define the simulation unit, conversion points, and where floating-point
    values are permitted in presentation only.
  - See `docs/skyjoust-product-requirements.md` §§4 and 15 and
    `docs/skyjoust-technical-design.md` §§5 and 8.
  - Success: movement, collision, camera, and replay code share one numeric
    contract.
- [ ] 1.1.4. Record the atlas metadata and asset manifest format decision.
  - Requires 1.1.1.
  - Decide TOML, RON, or JSON for runtime metadata and document schema
    versioning rules.
  - See `docs/skyjoust-technical-design.md` §§11 and 16.
  - Success: later asset tooling can validate manifests before the renderer
    loads assets.

### 1.2. Build the repository spine for a running placeholder game

This step answers whether the chosen runtime shape can open a window, draw a
stable pixel framebuffer, and execute a fixed tick without simulation mutation
from presentation. See `docs/skyjoust-technical-design.md` §§4-5 and
`docs/development-plan.md` §6.

- [ ] 1.2.1. Implement the runtime entrypoint, `winit` loop, and `pixels`
  framebuffer.
  - Requires 1.1.1 and 1.1.2.
  - Draw placeholder sky, terrain, riders, and debug layers.
  - Success: the application opens, renders a nonblank frame, preserves the
    chosen virtual resolution under resize, and reports input events.
- [ ] 1.2.2. Add Bevy entity-component system (ECS) resources and schedules
  for fixed simulation ticks.
  - Requires 1.2.1 and 1.1.3.
  - Keep presentation systems from writing authoritative simulation resources.
  - See `docs/skyjoust-state-graphs.yaml` `execution_model`.
  - Success: a fixed tick counter advances independently of render frame rate.
- [ ] 1.2.3. Draw the first placeholder HUD hierarchy.
  - Requires 1.2.1.
  - Include team strip, timer, altitude indicator, brace window, morale, ammo,
    objective strip, and debug seed text.
  - See `docs/skyjoust-product-requirements.md` §12 and
    `docs/skyjoust-technical-design.md` §§3 and 10.
  - Success: all live labels and numeric values come from renderer-owned text,
    not baked image text.

### 1.3. Make the high-level state graph executable

This step answers whether the app can follow the state graph before combat and
Warfront systems exist. Its outcome informs every gameplay slice because score,
reward, and Warfront handoffs must already be constrained. See
`docs/skyjoust-state-graphs.yaml` `root_parallel` and
`crates/skyjoust_stateright_validator/spec/validator_contract.yaml`.

- [ ] 1.3.1. Define state resources for app runtime, match lifecycle,
  ceremony, objectives, scoring, rewards, and Warfront.
  - Requires 1.2.2.
  - See `docs/skyjoust-technical-design.md` §6.
  - Success: each state graph region has an explicit runtime owner.
- [ ] 1.3.2. Implement a deterministic event bus and graph action queue.
  - Requires 1.3.1.
  - Sort transition commands in stable order before applying ECS mutations.
  - See `docs/skyjoust-state-graphs.yaml` `execution_model`.
  - Success: tests can inspect ordered events by tick and topic.
- [ ] 1.3.3. Export validator-compatible high-level action traces.
  - Requires 1.3.2.
  - Cover the path from `AssetsLoaded` through `CommitRewards`.
  - See `crates/skyjoust_stateright_validator/README.md` "Validate an engine
    trace".
  - Success: known-good traces pass `validate_trace`, and illegal
    reward-before-score traces fail.
- [ ] 1.3.4. Add graph-transition fixtures for illegal ordering cases.
  - Requires 1.3.3.
  - Cover reward-before-score, Warfront mutation during match, duel reward
    before resolution, and tournament laurels before completion.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties`.
  - Success: runtime trace tests fail for each validator contract violation.

### 1.4. Establish the imagegen asset pipeline contract

This step answers whether generated art can enter the repository without
becoming an uncontrolled runtime dependency. It informs renderer loading,
review workflow, and future source art generation. See
`docs/skyjoust-technical-design.md` §11 and `docs/development-plan.md` §5.

- [ ] 1.4.1. Create the asset, prompt, manifest, and tool directory skeleton.
  - Requires 1.1.4.
  - Include `assets/source/imagegen`, `assets/derived`, `assets/palettes`,
    `assets/manifests`, `prompts/templates`, `prompts/generated`, and `tools`.
  - Success: project-bound generated images have a clear source and derived
    location before live generation starts.
- [ ] 1.4.2. Write the imagegen prompt template set.
  - Requires 1.4.1.
  - Include style-guide, house, mount, animation, terrain, user interface (UI)
    ornament, event panel, Warfront map, chroma-key, and edit-invariant
    templates.
  - See `docs/development-plan.md` §5.2.
  - Success: templates use the shared schema and state that runtime text owns
    gameplay-critical values.
- [ ] 1.4.3. Implement the v1 asset manifest sample and validator test.
  - Requires 1.4.1 and 1.1.4.
  - Validate required fields, paths, status values, input image roles, and
    consuming modules.
  - See `docs/development-plan.md` §5.3.
  - Success: a sample manifest fails when paths or statuses drift.
- [ ] 1.4.4. Add a checked sample source asset and derived placeholder output.
  - Requires 1.4.3.
  - Use a tiny committed fixture rather than generated production art.
  - See `docs/development-plan.md` §§5.3 and 5.6.
  - Success: pipeline tests can prove source-to-derived path validation before
    real image generation starts.
- [ ] 1.4.5. Document the built-in imagegen and chroma-key transparency flow.
  - Requires 1.4.2.
  - Include the rule that command-line interface (CLI) fallback requires
    explicit confirmation.
  - See `docs/development-plan.md` §5.5.
  - Success: a developer can move from prompt to accepted source image to
    derived runtime asset without relying on unstated tool behaviour.

## 2. Vertical slice 1: Skirmish joust that feels fair

Idea: if the first playable skirmish can launch two riders, resolve joust
contacts deterministically, and explain the outcome through HUD and trace data,
Skyjoust proves its core arcade promise before terrain, ceremony, and Warfront
scope expand.

This slice should be fun in a blank or simple arena. It validates the movement,
brace, collision, scoring, recovery, audio-event, and debug loops that every
later slice depends on.

### 2.1. Prove fixed-tick flight and rider state

This step answers whether the motion model can express altitude advantage
without simulation-grade flight. It informs joust scoring and camera behaviour.
See `docs/skyjoust-product-requirements.md` §§7-8.1 and
`docs/skyjoust-technical-design.md` §8.

- [ ] 2.1.1. Implement rider, mount, lance, velocity, team, and health data.
  - Requires steps 1.1-1.3.
  - Align runtime fields with the `PlayerActor` component group in
    `docs/skyjoust-state-graphs.yaml`.
  - Success: a test fixture can spawn two riders with deterministic starting
    state.
- [ ] 2.1.2. Implement input intent capture and replay fixture wiring.
  - Requires 2.1.1.
  - Convert bindings into per-tick intent components before simulation runs.
  - See `docs/skyjoust-technical-design.md` §§5 and 13.
  - Success: replaying the same intent stream produces the same input state
    sequence.
- [ ] 2.1.3. Implement launch, flap, dive, stall, landing, and respawn basics.
  - Requires 2.1.1 and 2.1.2.
  - See `docs/skyjoust-state-graphs.yaml` `player_actor.mobility`.
  - Success: replaying the same input stream produces the same position and
    state sequence.
- [ ] 2.1.4. Draw flight debug overlays for altitude, velocity, and rider
  state.
  - Requires 2.1.3 and 1.2.3.
  - See `docs/skyjoust-product-requirements.md` §§12 and 18.
  - Success: a surprising altitude outcome can be diagnosed from one captured
    frame and event trace.

### 2.2. Resolve joust contact as a teachable event

This step answers whether collision plus brace timing produces deterministic
outcomes that players can learn. It informs scoring, recovery, SFX, and
visual-effects timing. See `docs/skyjoust-product-requirements.md` §8.2 and
`docs/skyjoust-technical-design.md` §8.

- [ ] 2.2.1. Implement stable broadphase candidate ordering for rider contact.
  - Requires 2.1.3.
  - Sort candidates by tick and stable entity IDs.
  - See `docs/skyjoust-technical-design.md` §8.
  - Success: pair resolution order is identical across repeated runs.
- [ ] 2.2.2. Implement lance reach, cone checks, brace windows, and recovery.
  - Requires 2.2.1.
  - See `docs/skyjoust-state-graphs.yaml` `player_actor.lance`.
  - Success: invalid contact produces physics response only, while valid lance
    contact emits `JOUST_OUTCOME`.
- [ ] 2.2.3. Implement outcome selection for knockback, unhorse, shatter, and
  clean kills.
  - Requires 2.2.2.
  - See `docs/skyjoust-product-requirements.md` §8.2 and
    `docs/skyjoust-state-graphs.yaml` `scoring_rules.score_atoms`.
  - Success: golden tests cover all four outcomes.
- [ ] 2.2.4. Route joust outcomes into recovery, scoring, and debug events.
  - Requires 2.2.3.
  - Feed recovery, scoring, audio-event, and debug systems from the same
    ordered outcome event.
  - See `docs/skyjoust-state-graphs.yaml` `event_routes`.
  - Success: one trace can explain the physical result, score atom, and
    recovery timer for a contact.
- [ ] 2.2.5. Add visual and audio event placeholders for joust outcomes.
  - Requires 2.2.4.
  - Emit events for lance clash, wing flap, stun, unhorse, shatter, and kill
    without letting presentation mutate simulation.
  - See `docs/skyjoust-technical-design.md` §§10 and 12.
  - Success: disabling presentation or audio does not alter replay results.

### 2.3. Close the skirmish score and reward loop

This step answers whether the simplest match can start, score, end, export a
final snapshot, and commit rewards without violating the validator contract. See
 `docs/skyjoust-state-graphs.yaml` `match_lifecycle`, `scoring`, and `rewards`.

- [ ] 2.3.1. Map joust outcomes into score atoms and morale deltas.
  - Requires 2.2.4 and 1.3.2.
  - See `docs/skyjoust-state-graphs.yaml` `scoring_rules`.
  - Success: scores update only through the scoring graph.
- [ ] 2.3.2. Implement victory by timer decision and debug-forced win.
  - Requires 2.3.1.
  - See `docs/skyjoust-state-graphs.yaml` `scoring.victory_conditions`.
  - Success: round-over states always include a winner.
- [ ] 2.3.3. Export the final score snapshot and freeze post-final scoring.
  - Requires 2.3.2.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties`.
  - Success: final-score snapshots reject later score writes in tests and
    traces.
- [ ] 2.3.4. Commit skirmish rewards after final score export.
  - Requires 2.3.3.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties`.
  - Success: validator traces prove rewards never open or commit before the
    score ledger finalizes.

## 3. Vertical slice 2: Destructible borderland objective match

Idea: if a skirmish can move from a simple arena into a generated borderland
with destructible terrain and objectives, the game can prove the Sopwith half
of its premise without depending on Warfront campaign systems.

This slice keeps the user-facing loop match-local: players fly, joust, bomb,
capture, breach, and win on one generated battlefield.

### 3.1. Generate a fair battlefield that code can reject

This step answers whether procedural terrain can create readable battlefields
without deciding the result at generation time. It informs objective placement,
camera bounds, renderer caching, and replay. See
`docs/skyjoust-product-requirements.md` §§8.3-8.4 and
`docs/skyjoust-technical-design.md` §9.

- [ ] 3.1.1. Implement chunked material storage and deterministic terrain
  generation for one biome.
  - Requires phase 2.
  - Include material cells, dirty rectangles, optional collision spans, and a
    deformation log.
  - See `docs/skyjoust-technical-design.md` §9.
  - Success: terrain chunks expose stable material and collision queries, and
    seed plus biome parameters regenerate identical chunks.
- [ ] 3.1.2. Place keeps, outposts, shrines, supply anchors, and spawn
  corridors.
  - Requires 3.1.1.
  - See `docs/skyjoust-state-graphs.yaml` `shared_context.ObjectiveState`.
  - Success: generated maps expose every objective anchor needed by the state
    graph.
- [ ] 3.1.3. Implement fairness validation for spawns, objectives, and routes.
  - Requires 3.1.2.
  - Reject unsafe spawns, unreachable objectives, decisive chokepoints, and
    distance imbalance beyond configured tolerance.
  - See `docs/skyjoust-product-requirements.md` §§15 and 18.
  - Success: property tests find and reject deliberately unfair seeds.

### 3.2. Make bombs reshape terrain and objectives

This step answers whether ordnance can change the battlefield while preserving
deterministic collision and score routing. It informs terrain art, dirty-region
rendering, and keep-breach victory. See `docs/skyjoust-state-graphs.yaml`
`event_routes` and `docs/development-plan.md` §9.

- [ ] 3.2.1. Implement bomb and bolt ordnance states.
  - Requires 3.1.1 and 2.2.5.
  - Gate firing through `match.can_spawn_ordnance`.
  - See `docs/skyjoust-state-graphs.yaml` `player_actor.ordnance`.
  - Success: tournament-style joust-only rules can disable ordnance.
- [ ] 3.2.2. Implement impact resolution against riders, terrain, and
  structures.
  - Requires 3.2.1.
  - Route rider hits, terrain hits, and structure hits through distinct events.
  - See `docs/skyjoust-state-graphs.yaml` `event_routes`.
  - Success: impact tests can distinguish splash damage, terrain deformation,
    and structural damage.
- [ ] 3.2.3. Implement terrain deformation and dirty-rectangle invalidation.
  - Requires 3.2.2.
  - Preserve indestructible materials and record deformation events.
  - See `docs/skyjoust-technical-design.md` §9.
  - Success: repeated seed and bomb inputs produce the same terrain hash.
- [ ] 3.2.4. Implement keep damage, core exposure, and keep-breach victory.
  - Requires 3.2.3 and 2.3.2.
  - See `docs/skyjoust-state-graphs.yaml` `objectives.keeps`.
  - Success: keep breach emits `VICTORY_CONDITION_MET` through scoring, not
    directly from ordnance code.

### 3.3. Deliver capture, shrine, and supply objective play

This step answers whether objective play can compete with jousting without
diluting arcade pace. It informs HUD priority and Warfront reward inputs. See
`docs/skyjoust-product-requirements.md` §§5 and 8.3.

- [ ] 3.3.1. Implement outpost capture and contest state.
  - Requires 3.1.3 and 2.3.1.
  - See `docs/skyjoust-state-graphs.yaml` `objectives.outposts`.
  - Success: outpost capture posts score, morale, and ammo-income events.
- [ ] 3.3.2. Implement shrine claim and buff expiry.
  - Requires 3.3.1.
  - See `docs/skyjoust-state-graphs.yaml` `objectives.shrines`.
  - Success: shrine buffs affect the match through explicit rule or component
    state and expire deterministically.
- [ ] 3.3.3. Implement supply route block and repair states.
  - Requires 3.3.1.
  - See `docs/skyjoust-state-graphs.yaml` `objectives.supply_routes`.
  - Success: route state changes can reduce resupply rate and coin income for
    later Warfront use.

## 4. Vertical slice 3: Reference-faithful runtime presentation

Idea: if validated imagegen-derived assets can replace placeholders without
weakening readability or determinism, Skyjoust can adopt the reference design
book's visual identity while keeping gameplay state renderer-owned.

This slice exercises the asset pipeline end to end. It should deliver a
playable match that looks recognizably like Skyjoust while refusing unapproved
or malformed assets.

### 4.1. Convert source art into deterministic runtime assets

This step answers whether the imagegen workflow can produce auditable runtime
assets. It informs atlas loading, review practice, and later content creation.
See `docs/skyjoust-technical-design.md` §§3 and 11.

- [ ] 4.1.1. Generate or approve the first style-anchor and UI-source assets.
  - Requires phase 1.
  - Use the existing `ref/` images as style references and generate only
    missing or corrected views.
  - See `docs/development-plan.md` §5.4.
  - Success: each accepted source image has a prompt file, manifest, input
    image roles, and review note.
- [ ] 4.1.2. Implement palette and alpha validation tools for source assets.
  - Requires 1.4.3 and 4.1.1.
  - Cover `quantize`, `check_alpha`, and `check_palette` as needed.
  - See `docs/development-plan.md` §5.6.
  - Success: invalid palette or transparency output fails before runtime
    loading.
- [ ] 4.1.3. Implement slicing and atlas packing validation tools.
  - Requires 4.1.2 and 1.4.4.
  - Cover `validate_asset_manifest`, `slice_sheet`, and `pack_atlas`.
  - See `docs/development-plan.md` §5.6.
  - Success: frame bounds, pivots, tags, and atlas metadata fail fast when
    source art drifts.
- [ ] 4.1.4. Load only runtime-approved assets through manifests.
  - Requires 4.1.3 and 1.2.1.
  - See `docs/skyjoust-technical-design.md` §11.
  - Success: missing or unapproved assets fail with named diagnostics.

### 4.2. Replace placeholders with readable mounted combat

This step answers whether riders, mounts, lances, terrain, and effects remain
legible at speed after source art lands. It informs animation scope and camera
tuning. See `docs/skyjoust-product-requirements.md` §13.

- [ ] 4.2.1. Integrate first rider, mount, lance, and effect atlases.
  - Requires 4.1.4 and phase 2.
  - Start with limited animation states: idle/perched, launch, flap, dive,
    brace, hit, and recovery.
  - See `docs/development-plan.md` §5.4 wave 3.
  - Success: each combat state has a recognizable silhouette at gameplay
    scale.
- [ ] 4.2.2. Integrate combat effect atlases for contact and ordnance events.
  - Requires 4.2.1 and 3.2.2.
  - Cover lance clash, hit sparks, shatter, stun, bomb trails, and impact
    flashes.
  - See `docs/skyjoust-technical-design.md` §§10 and 12.
  - Success: effects communicate the event type without hiding rider
    silhouettes.
- [ ] 4.2.3. Integrate terrain materials, keep, outpost, shrine, and route
  assets.
  - Requires 4.1.4 and phase 3.
  - See `docs/development-plan.md` §5.4 wave 4.
  - Success: terrain visuals reflect collision material and deformation state.
- [ ] 4.2.4. Add camera, shake, flash, and debug overlays that preserve clarity.
  - Requires 4.2.1, 4.2.2, and 4.2.3.
  - See `docs/skyjoust-product-requirements.md` §§12-14.
  - Success: visual effects can be reduced or disabled without hiding gameplay
    state.

### 4.3. Deliver the first runtime-owned HUD and audio feedback pass

This step answers whether the reference HUD hierarchy can communicate live
state during actual play. It informs ceremony panels and Warfront UI. See
`docs/skyjoust-technical-design.md` §§10 and 12.

- [ ] 4.3.1. Replace placeholder HUD panels with validated ornaments and
  renderer-owned glyphs.
  - Requires 4.1.4 and 1.2.3.
  - Cover altitude, brace, morale, ammo, objectives, timer, teams, and event
    banner shell.
  - See `docs/skyjoust-product-requirements.md` §12.
  - Success: HUD values are live, readable, and independent of generated
    image text.
- [ ] 4.3.2. Implement Kira audio buses and event-to-sound mapping.
  - Requires 2.2.5.
  - Include music, sound-effect, and UI buses with rate limits and
    device-failure handling.
  - Cover lance clash, wing flap, stun, unhorse, shatter, kill, capture, and
    menu confirmation events.
  - See `docs/skyjoust-technical-design.md` §12.
  - Success: audio-device failure or cue rate limiting cannot change match
    outcome or event order.
- [ ] 4.3.3. Add nonblank rendering and resize smoke tests.
  - Requires 4.3.1.
  - See `docs/development-plan.md` §15.
  - Success: automated checks catch blank framebuffer, broken scaling, and
    missing HUD glyph atlas failures.

## 5. Vertical slice 4: Ceremony that changes play

Idea: if tournament and duel events can alter rules, scoring, rewards, and UI
without bypassing the state graph, Skyjoust proves that ceremony is a mechanic
rather than a cutscene.

This slice implements the Minimum Viable Product (MVP) ceremony layer only.
Wedding and banquet states stay visible as deferred contracts.

### 5.1. Deliver tournament as a playable ruleset

This step answers whether a tournament can reduce ordnance, emphasize joust
skill, award laurels, and return safely to normal match flow. See
`docs/skyjoust-product-requirements.md` §10.1 and
`docs/skyjoust-state-graphs.yaml` `ceremony_events.Tournament`.

- [ ] 5.1.1. Implement tournament arena build, registration, and rule
  application states.
  - Requires phases 2-4.
  - Success: tournament setup applies temporary rules through
    `EVENT_RULES_APPLIED`.
- [ ] 5.1.2. Implement bracket seed, round countdown, and round transition
  states.
  - Requires 5.1.1.
  - See `docs/skyjoust-state-graphs.yaml` `ceremony_events.Tournament`.
  - Success: each round starts from graph state rather than presentation
    timers.
- [ ] 5.1.3. Implement round scoring and honour audit.
  - Requires 5.1.2 and 2.3.3.
  - See `docs/skyjoust-product-requirements.md` §10.1.
  - Success: tournament scoring rejects ordnance-only or invalid-contact
    wins.
- [ ] 5.1.4. Implement champion declaration and laurel rewards.
  - Requires 5.1.3 and 2.3.4.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties.laurels_only_after_tournament_completion`.
  - Success: laurels cannot be granted before tournament completion.
- [ ] 5.1.5. Render tournament bracket, rule summary, and reward preview
  panels.
  - Requires 5.1.4 and 4.3.1.
  - See `docs/skyjoust-technical-design.md` §§3 and 10.
  - Success: tournament UI reflects current graph state and contains no
    gameplay-critical baked values.

### 5.2. Deliver duel as a constrained champion challenge

This step answers whether a duel can lock participants, block interference,
resolve a decisive joust, and award consequences without corrupting scoring. See
 `docs/skyjoust-product-requirements.md` §10.2 and
`docs/skyjoust-state-graphs.yaml` `ceremony_events.Duel`.

- [ ] 5.2.1. Implement duel issue, accept, and refuse states.
  - Requires phases 2-4.
  - See `docs/skyjoust-product-requirements.md` §10.2.
  - Success: a refused duel exits through a visible consequence path rather
    than silently cancelling.
- [ ] 5.2.2. Implement arena lock and duel-active participant constraints.
  - Requires 5.2.1.
  - See `docs/skyjoust-state-graphs.yaml` selectors `event.duel_locked` and
    `player.lance_contact_valid`.
  - Success: non-duelist scoring is blocked while duel lock is active.
- [ ] 5.2.3. Implement duel resolution, interference, and honour violation
  outcomes.
  - Requires 5.2.2.
  - See `docs/skyjoust-state-graphs.yaml` `ceremony_events.Duel`.
  - Success: interference creates an explicit penalty event without resolving
    the duel as a clean win.
- [ ] 5.2.4. Implement duel reward and penalty queueing.
  - Requires 5.2.3 and 2.3.4.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties.duel_reward_only_after_resolved_duel`.
  - Success: duel rewards require a resolved duel before reward commit.
- [ ] 5.2.5. Render duel prompts, honour rules, and consequence panels.
  - Requires 5.2.4 and 4.3.1.
  - See `docs/skyjoust-technical-design.md` §10.
  - Success: players can see who is locked, what is forbidden, and what the
    duel result changes.

### 5.3. Prove temporary rules restore correctly

This step answers whether ceremony modifiers can be applied and removed without
hidden state leaks. It informs all post-MVP diplomacy events. See
`docs/skyjoust-state-graphs.yaml` `ceremony_events.ConsequenceResolution`.

- [ ] 5.3.1. Add tests for temporary rule application, cooldown, and baseline
  restoration.
  - Requires 5.1.4 and 5.2.4.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties.temporary_rules_cleared_after_cooldown`.
  - Success: tournament and duel modifiers clear after cooldown when no truce
    remains active.
- [ ] 5.3.2. Export ceremony traces for tournament champion and clean duel win.
  - Requires 5.3.1 and 1.3.3.
  - Success: validator traces cover tournament laurels and duel influence
    reachability.

## 6. Vertical slice 5: Warfront-lite campaign loop

Idea: if a match result can change a readable Warfront map only after rewards
commit, Skyjoust can deliver "war as a system" while preserving the arcade
match as the unit of play.

This slice connects skirmish results into regions, routes, pressure, currency,
and next-battle selection without implementing deep diplomacy.

### 6.1. Build a campaign map that can launch real matches

This step answers whether region data can select a battle, freeze the Warfront
turn, and resume only after match result and reward commit. See
`docs/skyjoust-product-requirements.md` §§5-6 and
`docs/skyjoust-state-graphs.yaml` `warfront`.

- [ ] 6.1.1. Implement region ownership and adjacency data.
  - Requires phase 3.
  - See `docs/skyjoust-state-graphs.yaml` `shared_context.CampaignState`.
  - Success: a campaign seed creates a deterministic control map.
- [ ] 6.1.2. Implement supply routes and pressure vectors.
  - Requires 6.1.1 and 3.3.3.
  - See `docs/skyjoust-product-requirements.md` §§5 and 11.
  - Success: route state can affect battle pressure without mutating active
    match state.
- [ ] 6.1.3. Implement battle preview, battle lock, and match launch from a
  selected region.
  - Requires 6.1.2 and phase 2.
  - See `docs/skyjoust-state-graphs.yaml` `warfront.BattlePreview`.
  - Success: Warfront state freezes while the match is active.
- [ ] 6.1.4. Apply match results to region control after reward commit.
  - Requires 6.1.3 and 2.3.4.
  - See `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`
    `always_properties.warfront_not_mutated_during_match`.
  - Success: no Warfront mutation occurs before `REWARDS_COMMITTED`.

### 6.2. Deliver the Warfront economy and action cards

This step answers whether glory, coin, influence, laurels, and simple strategic
actions can create a campaign-lite loop without deep strategy. See
`docs/skyjoust-product-requirements.md` §11.

- [ ] 6.2.1. Implement glory, coin, influence, and laurel ledgers.
  - Requires 6.1.1.
  - See `docs/skyjoust-state-graphs.yaml` `reward_rules`.
  - Success: reward commit updates currencies through one auditable ledger
    path.
- [ ] 6.2.2. Implement recruit, deploy, repair, and supply cards.
  - Requires 6.2.1.
  - See `docs/skyjoust-product-requirements.md` §11.
  - Success: rewards can be spent on sidegrade-style actions without pure stat
    inflation.
- [ ] 6.2.3. Implement event forecast and active modifier summaries.
  - Requires 6.2.2 and phase 5.
  - See `docs/skyjoust-product-requirements.md` §12.
  - Success: upcoming events appear as probabilities or forecasts, not a fixed
    timetable.
- [ ] 6.2.4. Persist campaign state, unlocks, and penalties.
  - Requires 6.1.4 and 6.2.2.
  - See `docs/skyjoust-technical-design.md` §13.
  - Success: reloading a save preserves region ownership, currencies, queued
    events, and active modifiers.

### 6.3. Render Warfront as an inspectable tactical map

This step answers whether the reference Warfront page can become a usable
interface rather than a static illustration. It informs final UI polish. See
`docs/skyjoust-technical-design.md` §§3 and 10.

- [ ] 6.3.1. Render map control, route state, pressure arrows, and region
  selection overlays.
  - Requires 6.1.2 and 4.1.4.
  - See `docs/development-plan.md` §5.4 wave 5.
  - Success: every displayed marker comes from campaign state or manifest
    assets.
- [ ] 6.3.2. Render recruit, repair, political move, event forecast, active
  modifier, and summary panels.
  - Requires 6.2.3 and 6.3.1.
  - See `docs/skyjoust-product-requirements.md` §12.
  - Success: the Warfront screen supports selecting the next battle and
    spending available resources.

## 7. Vertical slice 6: MVP hardening and release discipline

Idea: if the MVP is replayable, tuneable, accessible, and observable before
content expands, Skyjoust can stabilize its first release around core feel
rather than hiding uncertainty behind more assets or modes.

This slice turns the playable systems into a maintainable MVP. It includes
accessibility, replay, performance, balancing, and documentation tasks because
those concerns must validate the completed slices, not a theoretical design.

### 7.1. Make play accessible without changing physics

This step answers whether accessibility options can improve usability while
preserving deterministic combat. See `docs/skyjoust-product-requirements.md`
§14.

- [ ] 7.1.1. Implement remappable controls and hold/toggle options.
  - Requires phase 2.
  - Cover brace, interaction, movement, and ordnance inputs.
  - Success: changing control bindings does not change replayed input
    semantics.
- [ ] 7.1.2. Implement reduced screen shake and high-contrast objective
  options.
  - Requires phase 4.
  - See `docs/skyjoust-product-requirements.md` §§12-14.
  - Success: visual clarity options preserve all gameplay information.
- [ ] 7.1.3. Add tutorial prompts for altitude, brace, and resupply.
  - Requires phases 2-3.
  - See `docs/skyjoust-product-requirements.md` §16.
  - Success: a new player can complete a guided launch, joust, and resupply
    loop.
- [ ] 7.1.4. Add tutorial prompts for objectives and ceremony rules.
  - Requires phases 3 and 5.
  - See `docs/skyjoust-product-requirements.md` §16.
  - Success: a new player can complete a guided objective and ceremony
    interaction loop without reading external documentation.

### 7.2. Make bugs reproducible and performance visible

This step answers whether developers can explain simulation and rendering
failures from captured data. See `docs/skyjoust-technical-design.md` §§13-14.

- [ ] 7.2.1. Implement replay capture and loading with seed, configuration
  hash, asset manifest hash, and per-tick inputs.
  - Requires phases 2-6.
  - See `docs/skyjoust-technical-design.md` §13.
  - Success: a captured bug report can launch into the same match state and
    tick sequence, then reproduce match events and validator traces.
- [ ] 7.2.2. Add performance counters for simulation, collision, terrain,
  rendering, and audio event processing.
  - Requires phases 3-4.
  - See `docs/development-plan.md` §14.
  - Success: debug overlay and logs identify the subsystem exceeding budget.
- [ ] 7.2.3. Add release gate documentation for code, asset, and doc checks.
  - Requires phase 1.
  - See `docs/development-plan.md` §§3 and 15.
  - Success: contributors can run the same validation set used for commits.

### 7.3. Tune the MVP against product success criteria

This step answers whether the MVP matches the PRD's pacing and balance targets
instead of only compiling. See `docs/skyjoust-product-requirements.md` §§16-18.

- [ ] 7.3.1. Build a local tuning corpus from deterministic seeds and bot
  scripts.
  - Requires phases 2, 3, and 5.
  - See `docs/skyjoust-product-requirements.md` §§16-18.
  - Success: balance changes can be compared against the same repeatable
    match set.
- [ ] 7.3.2. Tune skirmish and objective scoring toward six to ten minute
  matches.
  - Requires 7.3.1.
  - Success: local test runs produce median match length inside the PRD target
    band.
- [ ] 7.3.3. Tune house and mount sidegrades against win-rate and readability
  checks.
  - Requires 7.3.1 and phases 4 and 6.
  - See `docs/skyjoust-product-requirements.md` §§9, 11, and 16.
  - Success: no house/loadout exceeds the configured balance band in local
    test data.
- [ ] 7.3.4. Audit ceremony pacing for interruption risk.
  - Requires phase 5.
  - See `docs/skyjoust-product-requirements.md` §18.
  - Success: tournament and duel prompts can be queued, accepted, declined, and
    completed without obscuring match objectives.

## 8. Deferred extensions after the MVP promise

Idea: if the core MVP is already trustworthy and boring to operate, Skyjoust
can evaluate broader extensions on their product value instead of letting them
destabilize the first release.

This phase collects explicitly deferred scope. These tasks should stay out of
the MVP unless a new design decision changes the product boundary.

### 8.1. Evaluate wedding and banquet diplomacy

This step preserves the graph contracts for alliance and negotiation while
keeping full diplomacy outside MVP. See `docs/skyjoust-product-requirements.md`
§§10.3-10.4 and `docs/skyjoust-technical-design.md` §16.

- [ ] 8.1.1. Decide whether wedding alliance and banquet negotiation graduate
  from hidden graph states.
  - Requires phase 7.
  - Success: an ADR either promotes the feature with MVP-independent
    acceptance criteria or keeps it deferred.
- [ ] 8.1.2. Prototype truce, treaty, and infamy UI only after the decision.
  - Requires 8.1.1.
  - See `docs/skyjoust-state-graphs.yaml` `ceremony_events.WeddingAlliance`
    and `ceremony_events.Banquet`.
  - Success: truce breaks retain the validator properties for friendly fire,
    infamy, and penalties.

### 8.2. Evaluate online, ranked, and expanded content

This step keeps post-MVP competitive and content expansion from contaminating
core delivery. See `docs/skyjoust-product-requirements.md` §§6.2 and 17.2.

- [ ] 8.2.1. Decide whether online play uses rollback, lockstep, or a separate
  architecture.
  - Requires phase 7.
  - See `docs/skyjoust-technical-design.md` §§5, 8, and 13.
  - Success: the decision cites replay evidence from the MVP rather than
    speculative networking assumptions.
- [ ] 8.2.2. Decide whether ranked tournament play needs a separate ruleset.
  - Requires phase 5.
  - Success: ranked work does not change MVP tournament scoring semantics.
- [ ] 8.2.3. Plan expanded houses, biomes, structures, and mod-friendly seed
  sharing as separate roadmaps.
  - Requires phase 7.
  - See `docs/skyjoust-product-requirements.md` §17.2.
  - Success: content expansion tasks include asset, balance, and fairness
    gates before implementation starts.

### 8.3. Evaluate localization and broader accessibility

This step keeps internationalization and advanced accessibility visible without
blocking the MVP's renderer-owned text and controls. See
`docs/skyjoust-product-requirements.md` §§14 and 17.2.

- [ ] 8.3.1. Decide the localization boundary after runtime-owned text is
  stable.
  - Requires 4.3.1 and phase 7.
  - Success: localization scope covers renderer text and excludes generated
    gameplay-critical image text by design.
- [ ] 8.3.2. Evaluate screen-reader, subtitling, and one-handed control
  support.
  - Requires phase 7.
  - Success: accepted accessibility extensions include testable input, audio,
    and UI acceptance criteria.

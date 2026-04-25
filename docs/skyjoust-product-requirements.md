# Skyjoust PRD

## 1. Summary

*Project Skyjoust* is a side-scrolling, arcade-action game that fuses:

- **Sopwith**-style territory warfare over deformable terrain (bases,
  objectives, bombing runs, supply, frontline push), and
- **Joust**-style aerial duelling where **altitude + timing** decide the winner
  in a collision (“lance contact”) between mounted knights.

Players ride “avian terrors” (terror birds, rocs, gryphons, etc.), fighting for
rival noble houses across a procedurally generated borderland. Matches remain
fast and skill-driven, but the surrounding war evolves: castles fall, frontiers
move, alliances form, and occasionally the battlefield pauses for
**ritual**—tournaments, duels, weddings, and banquets that create temporary
rules shifts, truce windows, and political consequences.

## 2. Vision and pillars

**Vision:** “Arcade jousting meets total war soap opera—played at 60 FPS.”

Gameplay pillars:

1. **Height is might:** altitude advantage translates to joust dominance and
   tactical leverage.
2. **Terrain matters:** the world is not a backdrop; it’s cover, hazard, and an
   objective you can reshape.
3. **War as a system, not a menu:** territory control and politics create
   emergent match-to-match narrative.
4. **Ceremony as mechanics:** tournaments and negotiations are not cutscenes;
   they alter constraints and rewards.

## 3. Target audience

- Players who enjoy retro arcade action (Joust, Sopwith, Luftrausers-style
  immediacy).
- Competitive-ish players who like skill expression without long commitment
  (5–12 minute matches).
- Co-op and couch-versus fans (local play is a first-class requirement).
- “Strategy-curious” players who enjoy a light meta-layer (campaign map)
  without grand-strategy complexity.

## 4. Platforms and constraints (initial)

Primary target: **PC (Windows/macOS/Linux)** and **Web (modern browsers)**.
Input: **keyboard + mouse**, **gamepad**, and **two-player local** minimum.

Hard requirements:

- **60 FPS** locked on target hardware; physics and collision must feel
  consistent.
- **Low-latency input**; buffered actions and generous coyote-time where
  appropriate.
- Deterministic simulation strongly preferred (especially if online multiplayer
  lands).

## 5. Core game loop

### Moment-to-moment loop (micro)

1. Launch / flap / dive to gain position.
2. Engage: joust collisions, ranged harassment, bombing runs.
3. Reposition using terrain (ridge lines, ruins, towers, wind pockets).
4. Claim objectives (outposts, supply drops, shrines, hostage cages).
5. Cash in advantage: break enemy morale, destroy keep, or win the event
   condition.

### Match loop (macro)

- Each match occurs in a **region** on a procedural border map.
- Winning shifts control of that region and unlocks/denies resources for the
  next battle.
- Between battles: spend “Glory” and “Coin” on upgrades, recruits, and
  political moves.

## 6. Game modes

### 6.1 MVP modes

- **Skirmish (single match):** quick play with selectable rulesets.
- **Warfront (campaign-lite):** a string of matches across a shifting
  territorial map.
- **Local Versus:** split-screen or shared-screen variants.

### 6.2 Later

- **Online Versus / Co-op Warfront**
- **Ranked Jousting Circuit** (tournament-focused ladder)

## 7. Player fantasy and roles

You are a **mounted knight-pilot**, not a modern fighter ace. Skill expression
comes from:

- energy management (altitude/speed),
- collision geometry (angle, timing, lance alignment),
- terrain reading,
- opportunistic objective play.

Team roles (optional but encouraged via loadouts):

- **Lancer:** collision specialist, joust damage and knockbacks.
- **Harrier:** ranged pressure, denial, chase-down.
- **Siegebird:** bombing and structure damage.
- **Standard-bearer:** objective capture speed, morale buffs, support tools.

## 8. Core mechanics

### 8.1 Movement model (the “feel”)

- **Arcade flight**, not sim flight: simple pitch control, flap impulse, dive
  acceleration.
- **Altitude advantage** affects joust outcome (see combat).
- Mounts have distinctive handling profiles (turn radius, flap cadence, stall
  behaviour).

### 8.2 Combat model

Two primary damage channels:

#### A) Joust contact (signature mechanic)

- When riders collide within a lance cone, resolve contact by:

  - relative vertical position (height advantage),
  - relative speed and approach angle,
  - lance alignment timing (a “brace window”).
- Outcomes:

  - **Unhorse** (drop rider briefly; vulnerable recovery),
  - **Knockback** (slam into terrain/structures),
  - **Shatter** (break opponent’s lance; temporary disadvantage),
  - **Clean kill** (rare; requires strong advantage or debuffs).

#### B) Ordinance and arms

- Limited-ammo tools that complement jousting:

  - bombs/firepots (terrain/structure damage, area denial),
  - light ranged (bolts/arrows) for chip damage and finishing,
  - utility (nets, smoke, caltrops-in-air as a “shrapnel cloud”, decoys).
- Resupply via landing/perching at friendly structures or grabbing supply drops.

### 8.3 Territory and objectives

Map contains:

- **Keeps** (team base; morale anchor; destructible outer works).
- **Outposts** (capture points generating coin/ammo or spawning allied patrols).
- **Shrines / Relics** (temporary buffs; contested).
- **Supply routes** (caravans or aerial drops; disrupt to starve enemy).

Victory conditions (ruleset-dependent):

- **Keep breach** (destroy core structure),
- **Morale collapse** (kills + objective control drains morale),
- **Glory threshold** (tournament variants).

### 8.4 Procedural terrain and destruction

- Regions are generated from a seed with biome parameters (hills, cliffs, river
  gorges, ruined aqueducts).
- Terrain supports **local deformation** (craters, collapsed bridges, blasted
  towers).
- Fairness constraints:

  - symmetrical or “mirrored bias” placement for competitive modes,
  - guaranteed safe spawn corridors,
  - no unbreakable chokepoints that decide matches at generation time.

## 9. Noble houses and identity

Each house includes:

- heraldry (colours, sigil),
- signature mount species,
- a passive doctrine (e.g., siege prowess, joust superiority, diplomacy
  leverage),
- a roster of unlockable kit pieces.

Examples (placeholder):

- **House Gyrfalcon:** speed and chase-down; fragile structures.
- **House Rocmere:** heavy mounts; siege bonus; slower recovery.
- **House Nightheron:** stealth/smoke tools; strong duellists.
- **House Wyrmwing:** risky ordinance; volatile but high ceiling.

## 10. Special events system (the “ceremony layer”)

Events trigger within Warfront and sometimes mid-match as “edicts”. They must
*change play*, not just add flavour.

### 10.1 Jousting tournament

- Temporary ruleset: reduced ordinance, emphasise joust contacts.
- Bracket or score attack: players earn **Laurels** used for upgrades and
  political influence.
- Arena spawns with clear lanes and minimal terrain randomness to keep it
  skill-first.

### 10.2 Duel challenge

- A champion mechanic: one player can issue/accept a duel under constraints.
- Duel stakes:

  - winner earns a region-wide buff (morale surge, supply boon),
  - loser risks a temporary penalty (taxation hit, slower respawn).
- Duel can be interrupted only under specific “dishonour” rules (which have
  consequences).

### 10.3 Wedding (alliance event)

- Warfront-level event: two houses may form a **temporary alliance**.
- Gameplay effects during truce:

  - shared resupply structures,
  - friendly fire disabled between allied houses,
  - joint objectives appear (escort, relic recovery).
- Political effects:

  - breaking the truce creates an infamy debuff and can rally neutral factions
    against you.

### 10.4 Banquet / peace negotiation

- Short intermission phase between battles:

  - spend resources to propose terms (tribute, territory swap, hostage
    exchange),
  - a lightweight negotiation mini-system (risk/reward, bluff via limited
    information).
- Successful negotiation can:

  - prevent a difficult battle,
  - re-route the front,
  - unlock “diplomatic” upgrades (better supply, mercenaries, event control).

## 11. Progression and economy (non-predatory)

Currencies:

- **Glory:** earned by joust wins, kills, objectives; spent on gear and mount
  parts.
- **Coin:** earned via territory/supply; spent on structures, mercs, repairs.
- **Influence:** earned via events/negotiation; spent to steer Warfront events.

Progression principles:

- Avoid pure stat inflation. Prefer **sidegrades** and expressive kit choices.
- Competitive modes should offer standardized loadouts or tightly bounded
  differences.

## 12. UX/UI requirements

In-match HUD must communicate fast:

- altitude advantage indicator (subtle but readable),
- lance brace window cue,
- ammo/ordinance and resupply hints,
- objective state (outposts, morale, keep integrity),
- event banner with rules change summary (one line + iconography).

Warfront map UI:

- regions with control markers,
- supply lines and predicted pressure,
- upcoming event probability (in a fog-of-war style, not a timetable).

## 13. Art direction and audio

Art: readable silhouette-first 2D, chunky animation, strong heraldic identity,
terrain with high-contrast edges for collision readability.

Mounts: each must be readable at speed and have a distinctive flap cadence.

Audio:

- lance impacts and wingbeats are core feedback.
- musical stings for ceremony triggers (tournament horn, banquet strings).
- positional audio cues for incoming dive attacks and bombs.

## 14. Accessibility and difficulty

- Remappable controls, toggle/hold options.
- Aim assist options for ranged weapons (separate from joust mechanics).
- Visual clarity options (reduced screen shake, high-contrast objectives).
- Difficulty presets affect AI aggression and resource scarcity, not physics.

## 15. Technical requirements (high level)

- Deterministic-ish physics for consistent collision outcomes; avoid “floaty
  ambiguity”.
- Procedural generation that guarantees:

  - spawn safety,
  - objective reachability,
  - bounded match length.
- Replayable seeds (shareable codes).
- If online: design towards rollback-friendly simulation (limited entity
  counts, deterministic state, input-driven updates).

## 16. Analytics and success metrics

MVP success metrics:

- Median match length within target (e.g., 6–10 minutes).
- New-player retention: tutorial completion rate, first-win rate,
  time-to-first-“that felt amazing” joust moment.
- Balance health: no house/loadout exceeds target win-rate bands in telemetry.
- Procedural fairness: low rate of “unwinnable terrain” reports correlated to
  specific generators/seeds.

## 17. Scope definition

### 17.1 MVP (ship a fun core)

- 2–3 houses
- 4–6 mount types (including variants per house)
- Skirmish + Warfront-lite
- Joust contact system + 2 ordinance types
- Procedural terrain with limited deformation
- 2 special events (tournament + duel)

### 17.2 Post-MVP

- Weddings/banquets fully implemented
- Deeper diplomacy and alliance mechanics
- Expanded biomes and structures
- Online multiplayer
- Mod-friendly seed sharing and custom rulesets

Non-goals (initially):

- Deep grand strategy.
- Simulation-grade flight.
- Large roster bloat before the core feel is perfect.

## 18. Risks and mitigations

- **Risk:** joust collision feels random.
  **Mitigation:** strict, teachable resolution rules + strong feedback (brace
  timing, clear cones, slow-motion micro-sting on decisive contacts).

- **Risk:** procedural maps decide outcomes.
  **Mitigation:** fairness constraints, mirrored placement in competitive
  modes, generator test harness with seeded property tests.

- **Risk:** ceremony events feel like interruptions.
  **Mitigation:** keep them short, mechanically meaningful, and optionally
  “queued” between battles unless explicitly chosen as a mid-match edict.

- **Risk:** too many systems dilute arcade purity.
  **Mitigation:** ship MVP with one meta-layer (territory) and one ceremony
  layer (tournament/duel); add diplomacy once the action loop is proven.

## 19. Milestones (example)

- **Prototype:** movement + joust contact + one arena (prove the fun).
- **Vertical slice:** one procedural biome + one keep/outpost + tournament
  event.
- **Alpha:** Warfront-lite, 2 houses, balance pass, tutorial.
- **Beta:** content expansion, accessibility, performance, polish.
- **Release:** stability, replayability, telemetry-driven tuning.

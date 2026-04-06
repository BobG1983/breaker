# 5r: Evolution VFX — Entity Effects

## Summary

Implement bespoke VFX for five evolutions that fundamentally change how entities look or behave visually: Phantom Bolt, Voltchain, ArcWelder, FlashStep, and Second Wind. These evolutions share a theme of persistent or dramatic entity-level visual transformations — spectral ghost bolts, electric coronas, crackling tethers, teleportation disintegration, and salvation-moment wall materialization. Two shared infrastructure systems (electric corona, afterimage) serve multiple evolutions.

## Context

Key architecture changes from the LEGACY plan:
- **No recipe system.** Each evolution VFX is a direct Rust function/system.
- **rantzsoft_particles2d** for particle effects (electric arcs, sparks, afterimage particles).
- **rantzsoft_postprocess** for screen effects (radial distortion, screen flash, slow-mo).
- **Visual types from visuals/ domain** (5e) — Hue, GlowParams, VisualModifier, ModifierStack.
- **DR-9 corrections**:
  - **Phantom Breaker renamed Phantom Bolt**: the current mechanic spawns a PhantomBolt (ghost bolt), not a ghost breaker. Future Phase 7 will add BOTH a Phantom Bolt evolution (current) and a separate Phantom Breaker evolution (ghost breaker that mirrors movement). This phase implements VFX for the current PhantomBolt behavior only.
  - **ArcWelder**: VFX is enhanced crackling tether between bolts (TetherBeam), NOT bolt-to-cells arcs or Tesla coil. The beam connects consecutive bolts in sequence (1->2->3->4).
  - **FlashStep**: mechanic is unimplemented (no working FlashStep effect leaf). RON file exists but the `Do(FlashStep)` effect leaf may be a stub. VFX should be authored so it connects when the mechanic is implemented in Phase 7.

Current RON files:
- `phantom_bolt.evolution.ron` (name: "Phantom Bolt"): Bump -> `Do(SpawnPhantom(duration: 5.0, max_active: 5))` — Ingredients: Wide Breaker x2 + Bump Force x2
- `voltchain.evolution.ron` (name: "Voltchain"): CellDestroyed -> `Do(ChainLightning(arcs: 6, range: 128.0, damage_mult: 0.5))` — Ingredients: Chain Reaction x1 + Aftershock x2
- `arcwelder.evolution.ron` (name: "Arcwelder"): Bumped -> `Do(TetherBeam(damage_mult: 1.5, chain: true))` — Ingredients: Tether x2 + Amp x2
- `flashstep.evolution.ron` (name: "FlashStep"): `Do(FlashStep)` on Breaker — Ingredients: Breaker Speed x2 + Reflex x1
- `second_wind.evolution.ron` (name: "Second Wind"): BoltLost -> `Do(SecondWind)` — Ingredients: Wide Breaker x2 + Last Stand x1

## Evolutions in This Batch

### Phantom Bolt

**Mechanic summary**: On any bump (early, late, or perfect), spawns a phantom bolt with 5s duration and up to 5 active. Phantom bolts have infinite piercing and a lifespan timer.

**VFX design (from DR-9 and effects-particles.md)**:
- Ghost bolt: translucent/phasing appearance with alpha oscillation (0.3-0.8 range, sine wave at ~3 Hz).
- Non-white core color: spectral blue-violet (SlateBlue/MediumSlateBlue), distinct from normal bolt white.
- Spectral shader effect: flickering — brief visibility drops every few frames, conveying "unstable/ethereal."
- Afterimage trail: multiple fading copies of the bolt behind it (the afterimages ARE the trail, no separate wake trail).
- No wake trail (unlike normal bolts). The visual distinction is immediate — phantom bolts look ghostly.
- On phantom bolt despawn (duration expires or lost): quiet dim spark dissolve — no dramatic bolt-lost VFX. Ghosts fade, they do not explode.

**What to implement**:
1. Phantom bolt visual modifiers: on SpawnPhantom, apply VisualModifiers to the phantom bolt entity:
   - AlphaOscillation (min: 0.3, max: 0.8, frequency: 3.0 Hz)
   - ColorShift to spectral blue-violet (SlateBlue)
   - AfterimageTrail (spawn fading copies at bolt position every N frames)
   - Flicker (brief alpha drops to ~0.1 at random intervals)
2. Afterimage trail system: spawns semi-transparent copies of the bolt at its previous positions, each copy fades over ~0.2s. No physics — purely visual entities.
3. Phantom despawn VFX: dim spark dissolve (small RadialBurst, low HDR ~0.4, SlateBlue, ~6 particles). Suppress the normal bolt-lost VFX for phantom bolts.
4. No wake trail: phantom bolts should not receive the standard bolt wake trail (from 5f bolt visuals).

### Voltchain

**Mechanic summary**: On cell destroyed, fires chain lightning with 6 arcs, 128 range, 0.5x damage per arc. Each arc jumps to a nearby cell within range.

**VFX design (from DR-9 and effects-particles.md)**:
- Enhanced chain lightning — visually louder than base chain lightning from 5l.
- 6 arcs with LARGE max jump distance (128 range). Arcs are brighter, thicker, and more spark-dense than base.
- Each arc: jagged/angular line from source to target cell, with rapid brightness fluctuation (flicker at ~12 Hz).
- Sparks at branch points (small RadialBurst at each junction).
- Brief bloom flash at each target cell on hit.
- Bolt gains a persistent electric corona while Voltchain is active: additional glow layer with jagged edges around the bolt. Conveys "this bolt is electrically charged."
- Density comes from many cell-destroys in quick succession (each triggering 6 arcs), not from a single trigger.

**What to implement**:
1. Voltchain arc VFX function: when ChainLightning fires with Voltchain attribution, use enhanced parameters:
   - Arc width: wider than base
   - Arc HDR: 1.5 (vs base ~0.8)
   - Arc color: DodgerBlue
   - Spark count at branch points: higher than base
   - Flicker rate: 12 Hz brightness oscillation
   - Bloom flash at each target cell
2. Electric corona modifier: on bolt when Voltchain evolution is gained, apply VisualModifier for GlowIntensity increase (1.8x) + ColorShift (DodgerBlue). Creates the "electrically charged" look.
3. Distinction from base: base chain lightning (5l) uses fewer arcs, dimmer color, no corona on bolt. Voltchain is unmistakably more electric.

### ArcWelder

**Mechanic summary**: On bump, creates TetherBeam connections between all active bolts in sequence (chain: true means 1->2->3->4, not all-pairs). Beams damage cells they intersect. 1.5x damage multiplier.

**VFX design (from DR-9 and effects-particles.md)**:
- Crackling electric tether beams connecting ALL active bolts in sequence.
- Each tether beam: energy line with elasticity — stretches when bolts are far apart, slackens when close.
- Animated energy flow along the beam (brightness wave traveling from one end to the other at ~5 units/sec).
- Electric crackling visual: jagged edges on the beam, small spark particles along the length.
- Electric corona on ALL connected bolts (similar to Voltchain but applied to every bolt in the chain).
- When many bolts are active, the tethers form a visible electric web across the playfield.
- Tether snap VFX: flash + sparks when a tether breaks (bolt lost or out of range).
- Distinct from base TetherBeam (5l): more electric, more crackling, corona on bolts. Base tether is a clean energy beam; ArcWelder tether is violent and electric.

**What to implement**:
1. ArcWelder tether VFX function: when TetherBeam fires with ArcWelder attribution:
   - Crackling electric beam material (jagged edges, brightness fluctuation)
   - Energy flow animation (brightness wave, speed ~5 units/sec)
   - Spark particles along beam length (ContinuousEmitter, low rate, DodgerBlue)
   - Elasticity visual: beam width/tension varies with bolt distance
2. Electric corona on connected bolts: VisualModifier for GlowIntensity (1.5x) + ColorShift (DodgerBlue) on all bolts in the chain.
3. Tether snap VFX: on tether break, flash at midpoint + small spark burst + corona removal from disconnected bolts.
4. Multi-tether rendering: when 3+ bolts are connected, multiple tethers render simultaneously, creating the "electric web" aesthetic.

### FlashStep

**Mechanic summary**: Dash reversal during settling becomes a teleport. Breaker disintegrates at departure point and rematerializes at arrival point instantly. Mechanic is unimplemented — `Do(FlashStep)` effect leaf may be a stub. VFX should be authored to connect when the mechanic is wired in Phase 7.

**VFX design (from DR-9 and effects-particles.md)**:
- Teleportation disintegration/rematerialization — the VFX IS the spectacle, not the travel.
- Departure: breaker disintegrates into energy streak particles (dissolve outward from center, ~0.15s).
- Departure afterimage: ghostly copy of breaker at departure position, fades over ~0.3s.
- Light-streak: bright line connecting departure and arrival positions, visible for 1-2 frames only.
- Arrival: radial distortion burst at arrival position + particles converge inward to form breaker (reverse of disintegration, ~0.15s).
- Small screen shake on arrival.
- Total movement is one frame — the breaker teleports, then the VFX plays to give it visual weight.

**What to implement**:
1. Departure VFX function: triggers when FlashStep fires. Takes departure position as input.
   - Particle disintegration: RadialBurst from breaker position (outward, ~15 particles, white, HDR 1.5, short lifetime).
   - Afterimage entity: semi-transparent copy of breaker at departure position, fades over ~0.3s.
2. Light-streak entity: thin bright beam from departure to arrival position, HDR 1.5, despawns after 2 frames.
3. Arrival VFX function: triggers after breaker is repositioned. Takes arrival position as input.
   - Converging particles: directional burst inward toward arrival position (~12 particles).
   - Radial distortion at arrival (intensity 0.3, duration 0.15s).
   - Small spark burst at arrival.
   - Small screen shake.
4. Integration hooks: the VFX functions accept departure/arrival positions as parameters. When the FlashStep mechanic is implemented in Phase 7, it calls departure VFX, teleports the breaker, then calls arrival VFX. Until then, the VFX functions exist but are not triggered during gameplay.

### Second Wind

**Mechanic summary**: On bolt loss, an invisible wall catches the bolt — cheat death once per node. The bolt bounces back from a momentarily visible barrier along the bottom edge.

**VFX design (from DR-9 and effects-particles.md)**:
- Salvation moment — the most dramatic single-moment VFX in the game.
- Invisible wall materializes with a bright flash along the bottom playfield edge (HDR >2.0).
- Brief slow-mo (~0.1s, time factor ~0.3) — "time stops for a split second" salvation feeling.
- Wall is visible for a split second, then fades.
- Bolt bounces back from the materialized wall.
- Expanding energy ring at the save point.
- Spark burst along the wall edge.
- Screen flash (white, high intensity).
- Medium screen shake.
- Single-use per trigger — one dramatic save, then gone.
- Distinct from Shield barrier (5i): Shield is persistent with hexagonal pattern; Second Wind is a sudden flash appearance that vanishes.

**What to implement**:
1. Wall materialization VFX: bright horizontal line along bottom playfield edge, HDR >2.0, appears instantly, fades over ~0.25s. Rendered as a wide beam entity or bright line mesh.
2. Salvation moment screen effects:
   - Screen flash (white, intensity 1.5, ~4 frames)
   - Slow-mo trigger (factor 0.3, duration 0.25s)
   - Medium screen shake
3. Particle effects at save point:
   - Expanding energy ring from bolt bounce position (speed 600, radius 80, white, HDR 2.0)
   - Spark burst along wall edge (~16 sparks, white, HDR 1.2, outward)
4. Wall fade system: the materialized wall entity fades from full brightness to invisible over ~0.25s, then despawns.

## What to Build

### 1. Afterimage System (Shared: Phantom Bolt + FlashStep)

Both evolutions use afterimage/ghosting effects. Build a shared afterimage utility:

- `spawn_afterimage(commands, source_entity, position, color, fade_duration)`: spawns a semi-transparent copy of the source entity's visual at the given position. The copy fades over `fade_duration` then despawns.
- `AfterimageTrail` system: for entities with an AfterimageTrail modifier, periodically spawns afterimage entities at the entity's position. Used by Phantom Bolt for continuous trail.
- `afterimage_fade` system: updates afterimage alpha over time, despawns when fully faded.
- Differentiation: Phantom Bolt uses continuous trail (spectral blue-violet afterimages). FlashStep uses one-shot afterimage (single white afterimage at departure point).

### 2. Electric Corona System (Shared: Voltchain + ArcWelder)

Both evolutions use electric effects on bolts. Build shared electric corona infrastructure:

- Electric corona VisualModifier: GlowIntensity increase + ColorShift to DodgerBlue.
- Apply/remove corona when evolution is gained/lost or tether connects/disconnects.
- Differentiation: Voltchain = triggered burst of arcs to target cells, bolt always has corona. ArcWelder = persistent tether between bolts, all connected bolts get corona while tethered.

### 3. Phantom Bolt Visual Modifiers

- Alpha oscillation system: entities with AlphaOscillation modifier have alpha set by sin(time * frequency) mapped to [min, max].
- Flicker system: entities with Flicker modifier have random brief alpha drops (~0.1 for 1-2 frames).
- Suppress wake trail: phantom bolt entities do not receive the standard bolt trail.
- Phantom despawn: dim SlateBlue spark dissolve, suppress normal bolt-lost VFX.

### 4. Voltchain Enhanced Arcs

- Voltchain arc VFX function: enhanced ChainLightning visual (wider, brighter, DodgerBlue, more sparks at branch points).
- Per-target bloom flash: brief HDR flash at each target cell hit by an arc.
- Electric corona applied to bolt on evolution gain.

### 5. ArcWelder Tether VFX

- Crackling tether beam material: electric jagged-edge beam with brightness fluctuation and energy flow animation.
- Spark emitter along beam: ContinuousEmitter, low rate, DodgerBlue, positioned along beam length.
- Elasticity visual: beam width scales with distance between connected bolts (tighter when close, stretched when far).
- Tether snap VFX: flash + sparks at midpoint when tether breaks.
- Corona on all connected bolts while tethered.
- Multi-tether rendering: multiple simultaneous tethers for 3+ bolt chains.

### 6. FlashStep Teleportation VFX

- Departure function: particle disintegration + afterimage spawn. Accepts departure position parameter.
- Light-streak: thin bright beam entity, 2-frame lifetime.
- Arrival function: converging particles + radial distortion + spark burst + screen shake. Accepts arrival position parameter.
- Functions are callable but not triggered until FlashStep mechanic is implemented in Phase 7.

### 7. Second Wind Salvation VFX

- Wall materialization: bright horizontal beam along bottom edge, instant appear, fade over ~0.25s.
- Screen effects: white screen flash (intensity 1.5), slow-mo (factor 0.3, duration 0.25s), medium shake.
- Energy ring at save point: expanding ring particle (speed 600, radius 80, HDR 2.0).
- Spark burst along wall edge.
- Wall fade + despawn system.

### 8. Tests

- Phantom Bolt entities have alpha oscillation, color shift, afterimage trail, and flicker modifiers
- Phantom Bolt afterimage trail spawns fading copies at bolt positions
- Phantom Bolt despawn plays dim spark dissolve (not normal bolt-lost VFX)
- Phantom Bolt entities do not receive standard wake trail
- Voltchain arcs are visually enhanced (brighter, wider, more sparks) vs base chain lightning
- Voltchain bolt has electric corona modifier while evolution is active
- ArcWelder tethers render between consecutive bolt pairs
- ArcWelder tether snap plays flash + sparks
- ArcWelder connected bolts have electric corona
- ArcWelder multi-tether renders correctly for 3+ bolts
- FlashStep departure function spawns disintegration particles + afterimage
- FlashStep arrival function spawns converging particles + distortion + shake
- FlashStep light-streak entity despawns after 2 frames
- Second Wind wall materializes with HDR >2.0 flash
- Second Wind triggers slow-mo, screen flash, screen shake
- Second Wind wall fades and despawns after ~0.25s
- Afterimage system correctly spawns and fades copies for both Phantom Bolt and FlashStep
- Electric corona system applies/removes correctly for both Voltchain and ArcWelder
- Attribution routing correctly identifies each evolution

## What NOT to Do

- Do NOT implement the FlashStep mechanic (dash-reversal teleport). The VFX functions are authored and callable, but the mechanic is Phase 7. Do not create a FlashStep trigger system.
- Do NOT create a ghost breaker entity for Phantom Bolt. The current mechanic spawns PhantomBolt (ghost bolt), not a ghost breaker. Phantom Breaker (ghost breaker) is a separate future evolution.
- Do NOT make ArcWelder a Tesla coil (bolt-to-cells arcs). ArcWelder is bolt-to-bolt tethers only. Cell damage happens because cells intersect the tether beam — the VFX is the beam, not arcs to cells.
- Do NOT modify base Chain Lightning or TetherBeam VFX from 5l. Voltchain and ArcWelder use separate code paths with enhanced parameters.
- Do NOT create a recipe system. All VFX is direct Rust function calls.
- Do NOT make Second Wind look like the Shield barrier. Shield (5i) is persistent with hexagonal pattern. Second Wind is a sudden flash that vanishes.
- Do NOT create FlashStep particle effects that require the breaker to actually teleport. The VFX functions accept position parameters and can be tested independently of the teleport mechanic.

## Dependencies

- **5l** (combat effect VFX): base Chain Lightning VFX (Voltchain enhances it), base TetherBeam VFX (ArcWelder enhances it), base SecondWind VFX (if any), base SpawnPhantom VFX (if any). Evolution variants are separate higher-tier code paths.
- **5c** (rantzsoft_particles2d): spark bursts, afterimage particles, electric arc particles, energy ring particles.
- **5d** (rantzsoft_postprocess): radial distortion (FlashStep arrival, Second Wind), screen flash (Second Wind, FlashStep), slow-mo trigger (Second Wind).
- **5e** (visuals/ domain): VisualModifier types (AlphaOscillation, ColorShift, GlowIntensity, AfterimageTrail, Flicker), ModifierStack.
- **5f** (bolt visuals): bolt wake trail system (to suppress for Phantom Bolt), bolt visual attachment (for corona application).
- **5g** (breaker visuals): breaker visual system (for FlashStep afterimage/disintegration).
- **5k** (bump/failure VFX): screen shake infrastructure, bolt-lost VFX (to suppress for Phantom Bolt).
- **5q** (evo beams): beam entity infrastructure (for light-streak in FlashStep, wall materialization in Second Wind). Evolution VFX routing mechanism.

## Verification

- Phantom Bolt is visually distinct: translucent, spectral blue-violet, afterimage trail, flickering
- Phantom Bolt has no wake trail (afterimages serve as the trail)
- Phantom Bolt despawn is quiet (dim spark dissolve, no dramatic bolt-lost VFX)
- Voltchain arcs are visually louder and brighter than base Chain Lightning
- Voltchain bolt has visible electric corona (DodgerBlue glow)
- ArcWelder shows crackling electric tether beams between bolts
- ArcWelder and Voltchain are visually distinguishable (arcs-to-cells vs tether-between-bolts)
- ArcWelder electric web is visible when 3+ bolts are connected
- ArcWelder tether snap has visible flash + sparks
- FlashStep departure shows disintegration + afterimage
- FlashStep arrival shows convergence + distortion + shake
- FlashStep light-streak briefly connects departure and arrival positions
- Second Wind wall materializes dramatically with HDR flash and slow-mo
- Second Wind is visually distinct from Shield barrier (sudden flash vs persistent hexagonal)
- All evolution VFX are visually a tier above base effects
- Afterimage system works for both continuous (Phantom) and one-shot (FlashStep) use cases
- Electric corona system works for both always-on (Voltchain) and tether-linked (ArcWelder) cases
- All existing tests pass

## Status: NEEDS DETAIL

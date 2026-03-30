# 5w: Evolution VFX Batch 4 — Entity Effects

**Goal**: Implement bespoke VFX for evolutions that fundamentally change how an entity looks or behaves visually — persistent entity modifications, spectral effects, electric tethers, and teleportation.

## Evolutions

### Phantom Breaker

**Behavior**: Bump → SpawnPhantom (spawns a PhantomBolt — ghost bolt with duration, max 1 active). NOTE: Future plans include BOTH a Phantom Bolt evolution (current mechanic) and a separate Phantom Breaker evolution (ghost breaker that mirrors movement and bumps). For Phase 5, implement VFX for the current PhantomBolt behavior.

**VFX direction**: Ghost bolt with spectral shader.
- Translucent/phasing bolt — alpha oscillation (0.4-0.8 range, sine wave)
- Non-white core color (spectral blue-violet, distinct from normal bolt white)
- Spectral shader: flickering effect, brief visibility drops
- Afterimage trail (multiple fading copies behind, like Chrono's afterimage but ghostly)
- No wake trail (unlike normal bolts) — the afterimages ARE the trail
- On loss: fades out with dim spark dissolve (no dramatic bolt-lost VFX)

### Voltchain

**Behavior**: Cell destroy → chain lightning (3 arcs, 96 range, 0.5x damage).

**VFX direction**: Enhanced chain lightning — louder than base, large max jumps.
- Bright, vivid electric arcs from destroyed cell to 3 targets (+ large max jumps per arc)
- Arcs are visually louder than base Chain Lightning (5m) — thicker, brighter, more sparks at branch points
- Bolt gains electric corona (additional glow layer with jagged edges) while Voltchain is active
- Density comes from many cell-destroys in quick succession, not from a single trigger
- Brief bloom flash at each target cell

### ArcWelder

**Behavior**: Bump → TetherBeam (1.5x damage tether between two bolts).

**VFX direction**: Enhanced crackling tether between bolts (NOT bolt-to-cells arcs).
- Crackling electric energy along the tether beam
- Tether has elasticity — stretches when bolts are far apart, slackens when close
- Animated energy flowing along beam (brightness traveling end to end)
- Electric corona on both connected bolts
- Cells between the bolts take damage from the beam (this is the gameplay — VFX shows the beam path clearly)
- Flash + sparks when tether snaps
- Distinct from base TetherBeam — more electric, more crackling, corona on bolts

### FlashStep

**Behavior**: Dash becomes teleport — breaker disintegrates and rematerializes at dash target. NOTE: No RON file yet — mechanic needs implementing in Phase 7. Ingredients: Breaker Speed + Reflex.

**VFX direction**: Teleportation disintegration/rematerialization.
- On trigger: Breaker disintegrates into energy streak particles
- Light-streak connects departure and arrival positions (1-2 frames)
- Departure: afterimage of breaker fades ~0.3s
- Arrival: radial distortion burst + rematerialization (particles converge to form breaker)
- Total movement is one frame — the VFX is the spectacle, not the travel
- Small screen shake on arrival

### Second Wind

**Behavior**: Bolt loss → invisible wall catches the bolt (cheat death once).

**VFX direction**: Invisible wall materialization — salvation moment.
- When bolt would be lost: invisible wall materializes with bright flash along bottom edge (HDR >2.0)
- Brief slow-mo (~0.1s) — "salvation moment" feeling
- Wall visible for a split second, then fades
- Bolt bounces back from the materialized wall
- Single-use per trigger — dramatic one-time save
- Distinct from Shield barrier (5j) — Second Wind is a sudden flash appearance, shield is persistent with hexagonal pattern

## Shared Infrastructure

### Electric Corona System (Voltchain + ArcWelder)

Both use electric effects on the bolt:
- Electric arc particle system that renders corona around bolt
- Differentiated by: Voltchain = triggered burst of arcs to targets, ArcWelder = persistent tether between bolts

### Afterimage System (Phantom + FlashStep)

Both use afterimage/ghosting:
- Afterimage entity spawner (copies entity visual at position, then fades)
- Shared fade/dissolve animation
- Differentiated by: Phantom = continuous on bolt (spectral), FlashStep = one-shot on breaker (teleport)

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: bloom, distortion), 5e (particles: Electric Arc, Spark, Trail), 5g (bolt visuals for corona layer), 5h (breaker visuals for FlashStep disintegration), 5k (screen effects: shake, slow-mo, flash)
- **Enhanced by**: 5m (base Chain Lightning and TetherBeam VFX as reference for Voltchain/ArcWelder)
- DR-9 resolved: ArcWelder corrected (tether, not Tesla coil), Phantom Breaker clarified (ghost bolt, not ghost breaker)

## Verification

- Phantom bolt is visually distinct (translucent, spectral, afterimage trail)
- Voltchain arcs are louder/brighter than base Chain Lightning
- ArcWelder shows crackling electric tether between bolts
- Voltchain and ArcWelder are visually distinguishable (arcs-to-targets vs tether-between-bolts)
- FlashStep shows disintegration → streak → rematerialization
- Second Wind wall materializes dramatically on save, distinct from shield barrier
- All evolution VFX are visually a tier above base effects
- All existing tests pass

# 5w: Evolution VFX Batch 4 — Entity Effects

**Goal**: Implement bespoke VFX for evolutions that fundamentally change how an entity looks or behaves visually — persistent entity modifications, spectral effects, electric coronas, and teleportation.

## **DECISION REQUIRED: DR-9 (partial)**

VFX directions exist for all five evolutions in this batch.

## Evolutions

### 1. Phantom Breaker (SpawnPhantom)

**VFX direction**: Ghost bolt with spectral shader.

- Translucent/phasing bolt — alpha oscillation (0.4-0.8 range, sine wave)
- Non-white core color (spectral blue-violet, distinct from normal bolt white)
- Spectral shader: flickering effect, brief visibility drops
- Afterimage trail (multiple fading copies behind, like Chrono's afterimage but ghostly)
- No wake trail (unlike normal bolts) — the afterimages ARE the trail
- On loss: fades out with dim Spark dissolve (no dramatic bolt-lost VFX)

Partially built in 5g (PhantomBolt visual stub) — this step adds the full spectral shader and afterimage system.

### 2. Voltchain

**VFX direction**: Dense branching lightning web filling the screen.

- Persistent Electric Arc particles crackling from bolt to all cells in range
- Arcs are dense — many more than base Chain Lightning (5m)
- Screen fills with electric arcs between all targets simultaneously
- Bolt gains electric corona (additional glow layer with jagged edges)
- Arcs refresh geometry every 3-4 frames (organic flickering)
- Cells under arc damage show surface flicker
- Distinct from base Chain Lightning — persistent, denser, full-screen

### 3. ArcWelder

**VFX direction**: Moving Tesla coil.

- Persistent Electric Arc particles from bolt to nearby cells
- Jagged, flickering, refreshing geometry every 3-4 frames
- Cells show surface flicker/glow under arc damage
- Bolt gains electric corona (shared with Voltchain but different color/intensity)
- Distinct from Voltchain — ArcWelder is proximity-based continuous, Voltchain is triggered burst

### 4. FlashStep

**VFX direction**: Teleportation disintegration/rematerialization.

- On trigger: Breaker disintegrates into energy streak particles
- Light-streak connects departure and arrival positions (1-2 frames)
- Departure: afterimage of breaker fades ~0.3s
- Arrival: radial distortion burst + rematerialization (particles converge to form breaker)
- Total movement is one frame — the VFX is the spectacle, not the travel
- Small screen shake on arrival

### 5. Second Wind

**VFX direction**: Invisible wall salvation moment.

- When bolt would be lost (no shield, no lives would end run): invisible wall catches it
- Wall materializes with bright flash along bottom edge (HDR >2.0)
- Brief slow-mo (~0.1s) — "salvation moment" feeling
- Wall visible for a split second, then fades
- Bolt bounces back from the materialized wall
- Single-use per trigger — dramatic one-time save
- Distinct from Shield barrier (5j) — Second Wind is a sudden appearance, shield is persistent

## Shared Infrastructure

### Electric Corona System (Voltchain + ArcWelder)

Both use persistent electric effects on the bolt:
- Electric Arc particles system that targets nearby entities
- Corona shader layer on bolt Material2d
- Differentiated by: density, color, trigger condition (Voltchain = triggered burst, ArcWelder = continuous proximity)

### Afterimage System (Phantom + FlashStep)

Both use afterimage/ghosting:
- Afterimage entity spawner (copies entity visual at position, then fades)
- Shared fade/dissolve animation
- Differentiated by: opacity, color, lifetime

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: bloom, distortion), 5e (particles: Electric Arc, Spark, Trail), 5g (bolt visuals: PhantomBolt stub, bolt shader for corona layer), 5h (breaker visuals for FlashStep disintegration), 5k (screen effects: shake, slow-mo, flash)
- **Enhanced by**: 5m (base Chain Lightning VFX as reference for Voltchain/ArcWelder)

## Catalog Elements Addressed

From `catalog/evolutions.md`:
- Phantom Breaker: NONE → full spectral shader + afterimage
- Voltchain: NONE → dense lightning web
- ArcWelder: NONE → persistent Tesla coil
- FlashStep: NONE → teleportation disintegrate/rematerialize
- Second Wind: NONE → wall materialization salvation

## Verification

- Phantom bolt is visually distinct (translucent, spectral, afterimage trail)
- Voltchain fills screen with electric arcs
- ArcWelder shows persistent proximity arcs
- Voltchain and ArcWelder are visually distinguishable
- FlashStep shows disintegration → streak → rematerialization
- Second Wind wall materializes dramatically on save
- All evolution VFX are visually a tier above base effects
- All existing tests pass

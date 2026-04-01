# Evolution VFX

Each evolution requires bespoke VFX that looks fundamentally different from base chip effects — a visual tier above everything else. These are the crown jewels.

| Evolution | Status | Juice | Style Guide | VFX Direction |
|-----------|--------|-------|-------------|---------------|
| Nova Lance | NONE | High | COVERED | Massive beam along trajectory — appears at max width, shrinks over time. Heavy bloom + distortion. |
| Voltchain | NONE | High | COVERED | Enhanced chain lightning — 6 arcs, large max jumps, brighter than base. Electric corona on bolt. |
| Phantom Bolt | NONE | High | COVERED | Ghost bolt — translucent/phasing, spectral blue-violet core, afterimage trail, flickering. |
| Supernova | NONE | High | COVERED | Cascade marker — base effects with Supernova visual distinction (brighter ring, extra sparks). Spectacle is emergent. |
| Dead Man's Hand | NONE | High | COVERED | Pending mechanic rework. Provisional: dramatic shockwave from loss position + speed-up on remaining bolts. |
| Gravity Well (evo) | NONE | High | COVERED | Larger distortion lens — wider radius (160), stronger warping, 4 active, more glow motes. |
| Second Wind | NONE | High | COVERED | Invisible wall materializes with bright flash on save — salvation moment. Brief slow-mo. |
| Entropy Engine | NONE | High | COVERED | Prismatic flash per cell destroy, then selected effect fires. Bolt has persistent prismatic shimmer. No counter gauge. |
| Shock Chain | NONE | High | COVERED | Recursive shockwaves with escalating intensity per generation. Chromatic aberration scales with depth. |
| Circuit Breaker | NONE | High | COVERED | Three-node triangle charge indicator. Each perfect bump lights a node. On completion: nodes collapse, circuit closes, spawn + shockwave. |
| Split Decision | NONE | High | COVERED | Fission effect: cell splits, halves condense into bolt orbs. Energy filaments. Prismatic birth trails. |
| ArcWelder | NONE | High | COVERED | Crackling electric tether beams connecting ALL bolts in sequence. Electric corona on all connected bolts. Web aesthetic. |
| FlashStep | NONE | High | COVERED | Breaker disintegrates on dash-reversal during settling. Departure afterimage. Arrival burst + distortion. Light-streak. |
| Mirror Protocol | NONE | High | COVERED | Prismatic flash at bolt's impact point. Mirrored bolt emerges from flash with prismatic birth trail. Flash orientation reflects mirror axis (horizontal or vertical). |
| Anchor | NONE | Medium | COVERED | Charging glow while timer counts down. Ground-anchor lock-in flash when planted. Concentrated impact flash on planted bump. Glow dissipates on movement. |
| Resonance Cascade | NONE | High | COVERED | Persistent pulse aura — visible expanding rings at fixed interval. Larger bolt = larger rings. |

## Implementation

### Nova Lance

Recipe `"evo_nova_lance"`: `Beam(direction: Forward, range: 400.0, width: 8.0, hdr: 2.5, color: White, shrink_duration: 0.4, afterimage_duration: 0.2) + RadialDistortion(intensity: 0.4, duration: 0.4) + ScreenShake(tier: Medium) + ChromaticAberration(intensity: 0.2, duration: 0.3) + SparkBurst(count: 20, velocity: 300.0, hdr: 1.5, color: White, gravity: 0.0, lifetime: 0.3)`. Phase 2 (AfterPhase(0)): `ScreenFlash(color: White, intensity: 1.0, duration_frames: 3)`.

### Voltchain

No recipe — code-composed like base chain lightning but with enhanced params. For each arc: `SpawnElectricArc { start, end, jitter: 5.0, flicker_rate: 12.0, hdr: 1.5, color: DodgerBlue, lifetime: 0.2 }`. 6 arcs instead of base 3. Plus modifier on bolt: `AddModifier(GlowIntensity(1.8), source: "voltchain")` + `AddModifier(ColorShift(DodgerBlue), source: "voltchain")` for electric corona.

### Phantom Bolt

No recipe for the entity itself — modifier-driven. On PhantomBolt spawn: `AddModifier(AlphaOscillation { min: 0.3, max: 0.8, frequency: 3.0 }, source: "phantom")` + `AddModifier(AfterimageTrail(true), source: "phantom")` + `AddModifier(ColorShift(SlateBlue), source: "phantom")`. `AttachVisuals` uses a spectral color from the phantom bolt definition RON.

### Supernova

No bespoke recipe — uses base shockwave/bolt-spawn recipes with a visual marker. Supernova-triggered shockwaves get a variant recipe `"shockwave_supernova"` with brighter ring (`hdr: 1.8` vs base `1.2`) and more sparks (`count: 16` vs base `8`). The spectacle is emergent from cascade overlap.

### Dead Man's Hand

**Mechanic pending rework (Phase 7).** Provisional recipe `"evo_dead_mans_hand"`: `ExpandingRing(speed: 500.0, max_radius: 80.0, thickness: 4.0, hdr: 2.0, color: OrangeRed, lifetime: 0.4) + RadialDistortion(intensity: 0.5, duration: 0.4) + ScreenShake(tier: Heavy) + ScreenFlash(color: OrangeRed, intensity: 1.5, duration_frames: 4)`. Plus modifier on remaining bolts: `AddModifier(TrailLength(2.0), source: "dead_mans_hand")`.

### Gravity Well (evolution)

Same approach as base gravity well (anchored primitives) but larger params. Recipe `"evo_gravity_well"`: `AnchoredDistortion(entity: Source, radius: 160.0, intensity: 0.6, rotation_speed: 0.3) + AnchoredGlowMotes(entity: Source, count: 12, drift_speed: 30.0, radius: 120.0, hdr: 0.5, color: MidnightBlue, inward: true)`.

### Second Wind

Recipe `"evo_second_wind_save"`: `ExpandingRing(speed: 600.0, max_radius: 80.0, thickness: 4.0, hdr: 2.0, color: White, lifetime: 0.25) + ScreenFlash(color: White, intensity: 1.5, duration_frames: 4) + ScreenShake(tier: Medium) + SparkBurst(count: 16, velocity: 250.0, hdr: 1.2, color: White, gravity: 0.0, lifetime: 0.3)`. Plus `TriggerSlowMotion { factor: 0.3, duration: 0.25 }`. Salvation moment — bigger than base Second Wind.

### Entropy Engine

Recipe `"evo_entropy_flash"`: `SparkBurst(count: 12, velocity: 180.0, hdr: 0.8, color: White, gravity: 0.0, lifetime: 0.15)` (multi-colored sparks, prismatic). Fires on each cell destroy before the selected random effect fires its own VFX. Plus persistent modifier on bolt: `AddModifier(ColorCycle { colors: [Red, Gold, DodgerBlue, MediumOrchid], speed: 3.0 }, source: "entropy_engine")`.

### Shock Chain

**Mechanic pending (Phase 7).** Uses recursive `"shockwave"` recipe. Each generation gets brighter: `hdr: 1.2 + generation * 0.3`. Plus per-generation: `TriggerChromaticAberration { intensity: generation * 0.1, duration: 0.2 }`. Total chain size drives `TriggerScreenShake` tier (3+ cells = Small, 6+ = Medium, 10+ = Heavy).

### Circuit Breaker

No recipe for the charge indicator — code-composed. Three `AnchoredRing` entities in triangle layout around bolt. On perfect bump: one ring brightens (modifier). On completion: recipe `"evo_circuit_close"`: `ExpandingRing(speed: 400.0, max_radius: 40.0, thickness: 3.0, hdr: 2.0, color: Gold, lifetime: 0.3) + SparkBurst(count: 20, velocity: 250.0, hdr: 1.5, color: Gold, gravity: 40.0, lifetime: 0.3) + ScreenShake(tier: Medium) + ScreenFlash(color: Gold, intensity: 0.8, duration_frames: 3)`.

### Split Decision

Recipe `"evo_split_decision"`: `Split(entity: Source, axis: (0.0, 1.0), drift_speed: 100.0, hdr: 1.2, color: White, lifetime: 0.3) + SparkBurst(count: 10, velocity: 150.0, hdr: 0.8, color: White, gravity: 20.0, lifetime: 0.2)`. Phase 2 (AfterPhase(0)): `ExpandingRing(speed: 200.0, max_radius: 16.0, thickness: 1.5, hdr: 1.0, color: White, lifetime: 0.2)` at each new bolt position. Game spawns bolt entities after the Split visual.

### ArcWelder

No recipe — code-composed. For each consecutive bolt pair in the tether sequence: `ExecuteRecipe { recipe: "evo_arcwelder_tether", source: bolt_a, target: bolt_b }`. Recipe: `AnchoredBeam(entity_a: Source, entity_b: Target, width: 2.0, hdr: 1.0, color: DodgerBlue, energy_flow_speed: 5.0, elasticity: 0.3)`. Plus modifier on all connected bolts: `AddModifier(GlowIntensity(1.5), source: "arcwelder")` + `AddModifier(ColorShift(DodgerBlue), source: "arcwelder")`.

### FlashStep

**Mechanic pending (Phase 7).** Departure recipe `"evo_flashstep_depart"`: `Disintegrate(entity: Source, duration: 0.15) + Beam(direction: Forward, range: 300.0, width: 2.0, hdr: 1.5, color: White, shrink_duration: 0.1, afterimage_duration: 0.05)`. Arrival recipe `"evo_flashstep_arrive"`: `ExpandingRing(speed: 400.0, max_radius: 24.0, thickness: 2.0, hdr: 1.2, color: White, lifetime: 0.15) + RadialDistortion(intensity: 0.3, duration: 0.15) + SparkBurst(count: 12, velocity: 200.0, hdr: 1.0, color: White, gravity: 30.0, lifetime: 0.2)`. Game handles breaker teleport between departure and arrival.

### Mirror Protocol

Recipe `"evo_mirror_flash"`: `ExpandingRing(speed: 300.0, max_radius: 20.0, thickness: 2.0, hdr: 1.5, color: MediumOrchid, lifetime: 0.2) + SparkBurst(count: 8, velocity: 150.0, hdr: 0.8, color: MediumOrchid, gravity: 0.0, lifetime: 0.2)`. Fired at impact point. Mirrored bolt spawns with `AttachVisuals` from definition + `AddModifier(ColorShift(MediumOrchid), source: "mirror")` for prismatic birth trail (fades after ~0.5s via timed `RemoveModifier`).

### Anchor

Charging phase: `SetModifier(GlowIntensity(1.0 + charge_fraction * 1.0), source: "anchor_charge")` each FixedUpdate while timer counts down. Lock-in: recipe `"evo_anchor_plant"`: `ScreenFlash(color: White, intensity: 0.4, duration_frames: 2) + SparkBurst(count: 6, velocity: 100.0, hdr: 0.6, color: White, gravity: 40.0, lifetime: 0.15)`. Planted bump: recipe `"evo_anchor_impact"`: `ExpandingRing(speed: 200.0, max_radius: 16.0, thickness: 2.5, hdr: 1.8, color: White, lifetime: 0.1) + ScreenShake(tier: Small)`. Movement: `RemoveModifier(source: "anchor_charge")` — glow dissipates.

### Resonance Cascade

Recipe `"evo_resonance_cascade"` with repeating phase: Phase 0 (Immediate): `AnchoredRing(entity: Source, radius: 20.0, thickness: 1.0, hdr: 0.4, color: White, rotation_speed: 0.5)`. Phase 1 (Delay(0.5), repeat: (interval: 0.5)): `ExpandingRing(speed: 300.0, max_radius: 40.0, thickness: 1.5, hdr: 0.6, color: White, lifetime: 0.4)`. Repeats indefinitely until source entity despawns. Larger bolt → game sends `SetModifier(ShapeScale(bolt_scale), source: "resonance")` to scale the ring.

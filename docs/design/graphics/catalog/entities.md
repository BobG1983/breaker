# Entities

## Bolt

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Bolt (base) | PLACEHOLDER | Critical | High | High | COVERED | `Circle::new(1.0)` flat HDR yellow. No glow, no halo, no bloom shader. |
| Bolt wake/trail | NONE | Critical | High | High | COVERED | No trailing wake or path visualization. |
| Bolt speed state | NONE | Critical | Medium | High | COVERED | Speed is invisible. Style guide: wake length + core brightness scale with speed. |
| Bolt piercing state | NONE | Important | Medium | High | COVERED | `ActivePiercings` is logic-only. Style guide: sharper angular glow, energy spikes. |
| Bolt damage-boosted state | NONE | Important | Medium | High | COVERED | `EffectiveDamageMultiplier` is logic-only. Style guide: core shifts amber/white. |
| Bolt shield aura | NONE | Important | Medium | High | COVERED | `ShieldActive` on bolt is logic-only. Style guide: additional aura ring. |
| Bolt size-boosted state | PARTIAL | Low | Low | Low | COVERED | `Scale2D` scales correctly. No glow scaling. |
| Bolt serving (hover) | NONE | Important | Low | Medium | COVERED | Pulsing orb at ~70% brightness, no wake trail, halo breathes on 1.5s sine wave. Snaps to full on launch. |
| ExtraBolt distinction | NONE | Important | Medium | High | COVERED | Same white core but halo tinted with archetype accent (~40% sat). Shorter/thinner wake. Dissolves into dim sparks on loss. |
| ChainBolt + tether | NONE | Critical | Medium | High | COVERED | Thin energy filament (~0.4 HDR) to anchor. Brightens at max stretch. Simpler than evo TetherBeam. Flash + sparks on snap. |
| PhantomBolt | NONE | Critical | High | High | COVERED | `SpawnPhantom` creates bolt with no ghost/spectral visual. |
| Bolt lifespan indicator | NONE | Important | Low | Medium | COVERED | Below 30%: brightness/halo diminish. Below 15%: flicker with increasing frequency, wake shortens. Expiry: soft inward implosion of sparks. |
| Bolt spawn moment | NONE | Important | Medium | Medium | COVERED | Brief energy ring at spawn point (~0.1s), bolt materializes from point-source flash. Multi-spawns overlap additively. |

### Implementation

**Bolt (base):** `AttachVisuals` from bolt definition RON. `EntityVisualConfig { shape: Circle, color: White, glow: (core_brightness: 1.0, halo_radius: 2.5, halo_falloff: 2.0, bloom: 0.7), trail: ShieldEnergy(...) }`. Replaces Mesh2d/MeshMaterial2d.

**Bolt wake/trail:** Part of `AttachVisuals` — `trail: ShieldEnergy(width: 1.5, fade_length: 40.0, color: White, intensity: 0.8)` in the bolt RON rendering block. Trail entity spawned by the crate.

**Bolt speed state:** No recipe. Modifier-driven. `bolt/` domain sends each FixedUpdate: `SetModifier { entity, modifier: TrailLength(speed / max_speed * 2.0), source: "bolt_speed" }`.

**Bolt piercing state:** No recipe. Modifier-driven. `SetModifier { entity, modifier: SpikeCount(piercing_count), source: "bolt_piercing" }`.

**Bolt damage-boosted state:** No recipe. Modifier-driven. Effect `fire()` sends `AddModifier { entity, modifier: ColorShift(Gold), source: "damage_boost" }`. Effect `reverse()` sends `RemoveModifier`.

**Bolt shield aura:** No recipe. Modifier-driven. Effect `fire()` sends `AddModifier { entity, modifier: GlowIntensity(1.5), source: "bolt_shield" }`.

**Bolt size-boosted state:** No recipe. Modifier-driven. Effect `fire()` sends `AddModifier { entity, modifier: ShapeScale(size_multiplier), source: "size_boost" }`.

**Bolt serving (hover):** No recipe. Modifier-driven. While `BoltServing` is present: `SetModifier { entity, modifier: CoreBrightness(0.7), source: "bolt_serving" }`. On launch: remove modifier (brightness returns to 1.0).

**ExtraBolt distinction:** `AttachVisuals` from same bolt definition. Then immediately: `AddModifier(GlowIntensity(0.7), source: "extra_bolt")` + `AddModifier(TrailLength(0.6), source: "extra_bolt")`.

**ChainBolt + tether:** Recipe `"chain_tether"` with `AnchoredBeam(entity_a: Source, entity_b: Target, width: 0.8, hdr: 0.4, color: White, energy_flow_speed: 2.0, elasticity: 0.5)`. Fired with `source: chain_bolt, target: anchor_bolt`. On snap: recipe `"tether_snap"` with `SparkBurst + ScreenFlash`.

**PhantomBolt:** `AttachVisuals` from a phantom bolt definition RON (spectral color). Then: `AddModifier(AlphaOscillation { min: 0.3, max: 0.8, frequency: 3.0 }, source: "phantom")` + `AddModifier(AfterimageTrail(true), source: "phantom")`.

**Bolt lifespan indicator:** No recipe for dimming. Modifier-driven by a system reading `BoltLifespan`: `SetModifier(CoreBrightness(fraction.max(0.3)), source: "bolt_lifespan")`. Below 15%: `SetModifier(AlphaOscillation { min: 0.5, max: 1.0, frequency: 8.0 }, source: "bolt_lifespan_flicker")`. At expiry: `ExecuteRecipe { recipe: "bolt_expiry" }`.

**Bolt spawn moment:** Recipe `"bolt_spawn"`: `ExpandingRing(speed: 300.0, max_radius: 20.0, thickness: 1.5, hdr: 0.8, color: White, lifetime: 0.15)`. Fired by bolt/ on spawn.

---

## Breaker

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Breaker — Aegis | PLACEHOLDER | Critical | High | High | COVERED | `Rectangle::new(1.0, 1.0)` flat cyan. Same shape as all archetypes. |
| Breaker — Chrono | PLACEHOLDER | Critical | High | High | COVERED | Identical to Aegis. No angular shape, no amber color, no time distortion. |
| Breaker — Prism | PLACEHOLDER | Critical | High | High | COVERED | Identical to Aegis. No crystalline shape, no magenta color, no prismatic split. |
| Breaker aura (idle) | NONE | Important | High | High | COVERED | No ambient aura. Style guide: per-archetype aura (shield/time/prismatic). |
| Breaker moving state | PARTIAL | Low | Low | Low | COVERED | Tilt visual exists (rotation). No aura intensification. |
| Breaker dash state | NONE | Important | High | High | COVERED | No trail. Style guide: per-archetype trail (shield/afterimage/spectral). |
| Breaker settling state | NONE | Low | Low | Low | COVERED | Visually identical to idle. Style guide: trail fading, aura returning. |
| Breaker bump pop | PARTIAL | Low | Medium | Low | COVERED | Y-offset pop animation exists. No scale overshoot or flash at peak. |
| Breaker width boost | PARTIAL | Low | Low | Low | COVERED | `Scale2D` scales correctly. No stretch animation or glow pulse. |
| Breaker speed boost visual | NONE | Low | Low | Medium | COVERED | Aura stretches in movement direction, trailing wisps. Dash trail activates at lower intensity during normal movement. Speed lines at high stacks. |
| Breaker bump force visual | NONE | Low | Low | Medium | COVERED | Front face gains intensified archetype-color glow, pulsing slowly. White-hot at high stacks (HDR >1.0). Flares sharply on bump contact. |

### Implementation

**Breaker — Aegis/Chrono/Prism:** `AttachVisuals` from breaker archetype RON rendering block. Each archetype has shape, color, glow, aura, trail. Example Aegis: `{ shape: Shield, color: CadetBlue, glow: (...), aura: ShieldShimmer(...), trail: ShieldEnergy(...) }`.

**Breaker aura:** Part of `AttachVisuals` — crate spawns aura child entity with `AuraMaterial` variant uniform. No recipe.

**Breaker moving/dash/settling state:** No recipe. Modifier-driven. `breaker/` sends each FixedUpdate: `SetModifier(TrailLength(speed_fraction), source: "breaker_speed")`. During dash: `SetModifier(GlowIntensity(1.5), source: "breaker_dash")`.

**Breaker bump pop:** No recipe for the pop itself. `SquashStretch { x_scale: 1.2, y_scale: 0.8 }` modifier (shader-only, 2-3 frames). Plus the bump grade recipe fires (see Feedback catalog).

**Breaker width/speed/force boost:** No recipe. Modifier-driven. Each effect's `fire()` sends `AddModifier` (e.g., `ShapeScale(width_mult)` for width, `TrailLength(1.3)` for speed, `CoreBrightness(1.5)` for force). Effect `reverse()` sends `RemoveModifier`.

**Bump grade recipes:** See Feedback catalog — `"bump_perfect"`, `"bump_early"`, `"bump_late"`.

---

## Cells

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Standard Cell | PLACEHOLDER | Important | Medium | Medium | COVERED | `Rectangle` flat HDR magenta. Has color dimming on damage. No rounded corners. |
| Tough Cell | PLACEHOLDER | Important | Medium | High | COVERED | Same rectangle as Standard, different color (purple). Should be hexagonal. |
| Lock Cell | PLACEHOLDER | Critical | Medium | High | COVERED | Same rectangle, blue. No lock glyph, no octagonal shape, wrong color (should be amber). |
| Regen Cell | PLACEHOLDER | Important | Medium | High | COVERED | Same rectangle, green. No circular shape, no pulsing animation. |
| Shield Cell (orbiting) | PARTIAL | Important | Medium | Medium | COVERED | Small rectangles orbit parent. No orbit ring trail, no brightness distinction. |
| Cell damage — full health | PLACEHOLDER | Important | Medium | Medium | COVERED | Max glow. Needs to match target shape per type. |
| Cell damage — damaged | PARTIAL | Important | Medium | Medium | COVERED | Color dimming exists. No fractures/cracks, no shape destabilization. |
| Cell damage — near death | PARTIAL | Important | High | Medium | COVERED | Same dimming. No flickering, no heavy fracturing. |
| Cell destruction | NONE | Critical | High | High | COVERED | Cells despawn instantly. No dissolve, shatter, or energy release. |
| Cell hit impact | NONE | Important | Medium | Medium | COVERED | 4-8 sparks from impact point (bolt angle). Cell micro-flash (HDR ~1.3, 1-2 frames). Fracture cells gain crack at impact. Scales with damage. |
| Lock cell unlock | NONE | Important | Medium | Medium | COVERED | Lock glyph fractures outward (~0.2s), golden shard particles scatter. Cell transitions amber→true color. Energy ring + flash (HDR ~1.5). |
| Regen cell healing pulse | NONE | Low | Low | Medium | COVERED | Green glow pulse outward on each heal tick (~1.3x cell radius). Glow motes drift upward. Fracture cracks visibly seal with green glow. |
| Cell Powder Keg modifier | NONE | Important | High | Medium | COVERED (partial) | Pending chip. Style guide mentions "flickering, sparking, volatile." |

### Implementation

**Standard/Tough/Lock/Regen Cell:** `AttachVisuals` from cell definition RON. Each cell type has shape + color + glow in its rendering block.

**Cell damage state:** No recipe. Modifier-driven. `cells/` sends on health change: `SetModifier(CoreBrightness(0.3 + health_fraction * 0.7), source: "cell_health")`. The 0.3 floor ensures cells are never too dim to read — a 1-HP cell at last hit is still 30% brightness before despawning.

**Cell destruction — single:** Recipe `"cell_death_single"`: `Disintegrate(entity: Source, duration: 0.3) + SparkBurst(count: 6, velocity: 150.0, hdr: 0.6, color: White, gravity: 30.0, lifetime: 0.2)`.

**Cell destruction — combo:** Recipe `"cell_death_combo"`: `Fracture(entity: Source, shard_count: 6, velocity: 200.0, hdr: 0.8, color: White, lifetime: 0.3) + SparkBurst(count: 12, velocity: 200.0, hdr: 0.8, color: White, gravity: 40.0, lifetime: 0.25) + ScreenShake(tier: Micro)`.

**Cell destruction — chain:** Recipe `"cell_death_chain"`: `Fracture(entity: Source, shard_count: 8, velocity: 250.0, hdr: 1.2, color: White, lifetime: 0.35) + ExpandingRing(speed: 400.0, max_radius: 24.0, thickness: 2.0, hdr: 1.0, color: White, lifetime: 0.2) + SparkBurst(count: 16, velocity: 250.0, hdr: 1.0, color: White, gravity: 50.0, lifetime: 0.3) + ScreenShake(tier: Small)`.

**Cell hit impact:** Recipe `"cell_hit"`: `SparkBurst(count: 6, velocity: 180.0, hdr: 0.5, color: White, gravity: 20.0, lifetime: 0.15)`. Fired with `direction` from bolt velocity.

**Lock cell unlock:** Recipe `"cell_unlock"`: `ShardBurst(count: 8, velocity: 150.0, rotation_speed: 3.0, hdr: 1.0, color: Gold, lifetime: 0.3) + ExpandingRing(speed: 300.0, max_radius: 20.0, thickness: 2.0, hdr: 1.5, color: Gold, lifetime: 0.2) + ScreenFlash(color: Gold, intensity: 0.6, duration_frames: 2)`.

**Regen cell healing pulse:** Recipe `"cell_regen_pulse"`: `ExpandingRing(speed: 100.0, max_radius: 18.0, thickness: 1.0, hdr: 0.5, color: LimeGreen, lifetime: 0.3) + GlowMotes(count: 4, drift_speed: 30.0, radius: 3.0, hdr: 0.4, color: LimeGreen, lifetime: 0.5)`. Fired on each heal tick.

**Shield cell orbit:** `AttachVisuals` with brighter glow. Plus: `ExecuteRecipe { recipe: "orbit_ring" }` with `AnchoredRing(entity: Source, radius: [orbit_radius], thickness: 0.5, hdr: 0.3, color: White, rotation_speed: 0.0)`.

**Powder Keg modifier:** No recipe. Modifier-driven. `effect/` sends on cell: `AddModifier(AlphaOscillation { min: 0.6, max: 1.0, frequency: 5.0 }, source: "powder_keg")`.

---

## Walls & Background

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Walls (left, right, ceiling) | NONE | Low | Medium | High | COVERED | Invisible collision entities. No mesh, no glow. |
| Wall bolt-impact flash | NONE | Low | Medium | Medium | COVERED | No visual response on wall hit. |
| Bottom wall shield barrier | NONE | Critical | High | High | COVERED | `SecondWind` + `ShieldActive` have no barrier visual. |
| Background grid | NONE | Low | Medium | High | COVERED | Pure void. No grid, no spatial reference, no energy sprites. |
| Background energy sprites | NONE | Low | Low | Medium | COVERED | No ambient grid animation. |
| Void background color | PARTIAL | Low | Low | Low | COVERED | `ClearColor` dark blue-purple. Close to target but slightly more purple. |

### Implementation

**Walls:** `AttachVisuals` at wall spawn. `EntityVisualConfig { shape: Rectangle, color: SlateGray, glow: (core_brightness: 0.3, halo_radius: 1.5, halo_falloff: 3.0, bloom: 0.2) }`. Temperature tint via `SetModifier(ColorShift(...))` from `run/`.

**Wall bolt-impact flash:** Recipe `"wall_impact"`: `SparkBurst(count: 4, velocity: 120.0, hdr: 0.5, color: White, gravity: 20.0, lifetime: 0.1) + ExpandingRing(speed: 200.0, max_radius: 12.0, thickness: 1.0, hdr: 0.4, color: White, lifetime: 0.1)`. Fired at bolt-wall collision position.

**Shield barrier:** Not a recipe — dedicated `shield.wgsl` Material2d entity. Spawned by `effect/effects/shield/` when `ShieldActive` is added. Damage progression via shader uniforms (crack seeds). See `docs/architecture/rendering/walls_and_background.md`.

**Background grid:** Not a recipe — dedicated `grid.wgsl` Material2d on a single quad entity. Spawned by `screen/playing/` on enter. Color driven by `RunTemperature`.

**Background energy sprites:** Continuous `GlowMotes` emitter entity placed at grid level. `ParticleEmitter(Continuous { rate: 0.5 })` with very slow drift and long lifetime.

**Void background color:** `ClearColor(Color::linear_rgb(0.02, 0.02, 0.06))`. Set at startup.

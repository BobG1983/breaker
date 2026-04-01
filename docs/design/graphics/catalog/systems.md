# Systems & Infrastructure

## Temperature Palette System

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Run temperature resource | NONE | High | COVERED | No system tracks node progression or adjusts colors. All colors are static. |
| Temperature application | NONE | High | COVERED | Grid tint, cell glow, particle color, wall border, ambient bloom should all shift cool→hot. |

### Implementation

Not recipe-driven — infrastructure systems.

**RunTemperature resource:** `RunTemperature(f32)` in `run/` domain. Updated on node transition: `temperature = (node_index as f32 / expected_nodes_per_act as f32).clamp(0.0, 1.0)`. Instant snap — transition animation masks the change. See `docs/architecture/rendering/temperature.md`.

**Temperature application:** Systems read `RunTemperature` and send `SetModifier(ColorShift(...))` on wall entities. Grid shader reads temperature via uniform. Bloom tints via camera settings. No recipe involved.

---

## Transitions

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Flash transition | PLACEHOLDER | Medium | COVERED | Full-screen alpha fade. Functional but no bloom spike or temperature tinting. |
| Sweep transition | PLACEHOLDER | Medium | COVERED | Full-screen rect sweep. Functional but solid color, not energy beam edge. |
| Glitch transition | NONE | High | COVERED | Not implemented. Screen corruption + static + distortion. |
| Collapse/Rebuild transition | NONE | High | COVERED | Not implemented. Elements build outward/collapse inward. |
| Random transition selection | NONE | Medium | COVERED | Not implemented. Should randomly pick from pool per transition. |

### Implementation

Transitions use **direct primitives**, not recipes. Each transition style is a system in `screen/transition/` that drives FullscreenMaterial components and/or primitive messages.

**Flash:** Out: `TriggerScreenFlash { color: [temperature-tinted White], intensity: 2.0, duration_frames: 4 }`. In: same flash reveals the new scene.

**Sweep:** Out: `SpawnBeam { position, direction: E, range: screen_width, width: 4.0, hdr: 1.5, color: White, shrink_duration: 0.0, afterimage_duration: 0.3 }` traveling across screen. `SpawnSparkBurst` trailing the beam edge. In: reverse direction.

**Glitch:** Out: `TriggerChromaticAberration { intensity: ramp(0.0→0.5), duration: 0.4 }` + `TriggerRadialDistortion { origin: center, intensity: ramp(0.0→0.3), duration: 0.4 }` + `TriggerScreenFlash { color: White, intensity: 0.5, duration_frames: 2 }` at the end. In: reverse ramps.

**Collapse/Rebuild:** Dedicated `collapse_rebuild.wgsl` FullscreenMaterial on the camera. `progress` uniform animated 0.0→1.0 over duration. Out: tiles shrink toward center. In: tiles expand from center. See `docs/architecture/rendering/transitions.md`.

**Random selection:** `GameRng` picks a `TransitionStyle` for In and Out independently. Seeded for deterministic replay.

---

## Audio System

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Audio plugin | NONE | High | COVERED | `AudioPlugin` exists but is empty stub. No sound, no music. |
| Per-event SFX (15+ events) | NONE | High | COVERED | Every visual event needs a sound. None exist. |
| Layered intensity music | NONE | High | COVERED | 4-layer adaptive music (ambient→full). Not implemented. |
| Timer critical heartbeat | NONE | Critical | COVERED | Accelerating pulse sound. Primary chip-select pressure signal. |
| Music temperature shift | NONE | Medium | COVERED | Cool early → warm late paralleling visual temperature. |

### Implementation

**Phase 6 — not Phase 5.** No audio work in Phase 5. VFX systems do not emit audio events or include audio stubs. Audio will hook into the same game events that VFX hooks into (BumpPerformed, CellDestroyed, BoltLost, etc.) but through its own message system.

---

## Data-Driven Composition Enums

| Element | Status | Cohesion | Style Guide | Current |
|---------|--------|----------|-------------|---------|
| CellShape enum | NONE | High | COVERED | All cells are rectangles. Needs: Rectangle, RoundedRect, Hexagon, Octagon, Circle, Diamond. |
| CellColor enum | NONE | High | COVERED | Colors hardcoded per type. Needs: TemperatureDefault, fixed colors. |
| DamageDisplay enum | NONE | High | COVERED | Only color dimming exists. Needs: Fracture, Fade, Flicker, Shrink, ColorShift. |
| DeathEffect enum | NONE | High | COVERED | Cells just despawn. Needs: Dissolve, Shatter, EnergyRelease, Custom. |
| BreakerShape enum | NONE | High | COVERED | All breakers are rectangles. Needs: Shield, Angular, Crystalline. |
| ColorAccent enum | NONE | High | COVERED | All breakers same cyan. Needs: BlueCyan, Amber, Magenta. |
| AuraType enum | NONE | High | COVERED | No aura system. Needs: ShieldShimmer, TimeDistortion, PrismaticSplit. |
| TrailType enum | NONE | High | COVERED | No trail system. Needs: ShieldEnergy, Afterimage, PrismaticSplit. |
| VisualModifier system | NONE | High | COVERED | No modifier stacking. Needs: trail_length, glow_intensity, color_shift, particle_emitter, shape_modifier with diminishing returns. |
| evolution_vfx field | NONE | High | COVERED | Evolution RON files have no VFX reference field. |

### Implementation

All enums live in `rantzsoft_vfx` (game-agnostic). These are **not recipes** — they are types used by `AttachVisuals`, modifiers, and recipes.

**Shape, Hue, Aura, Trail, GlowParams, VisualModifier, ModifierKind:** Defined in step 5f. See `docs/architecture/rendering/types.md` for full type definitions.

**Note on naming:** The architecture uses unified enums (not cell/breaker prefixed). `Shape` (not CellShape/BreakerShape), `Hue` (not CellColor/ColorAccent), `Aura` (not AuraType), `Trail` (not TrailType). Any entity can use any variant.

**DamageDisplay/DeathEffect:** These catalog entries map to **recipes** defined in cell RON rendering blocks (`damage_recipe`, `death_recipe`, `hit_recipe`). There are no DamageDisplay or DeathEffect enums — each cell definition names the recipe strings directly.

**VisualModifier system:** Step 5n. `ModifierStack` + `ModifierConfig` + DR computation. See `docs/architecture/rendering/modifiers.md`.

**evolution_vfx field:** Evolution RON files reference recipe names. The evolution's `fire()` function sends `ExecuteRecipe` with the named recipe.

---

## Post-Processing Pipeline

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Bloom (tunable per-entity) | PARTIAL | High | COVERED | Camera bloom exists. No per-entity control or debug tuning. |
| Additive blending | NONE | High | COVERED | Default blending used. Style guide: additive for all light-on-dark. |
| Screen distortion shader | NONE | High | COVERED | Needed for shockwave, gravity well, explosion. Not implemented. |
| Glitch text shader | NONE | Medium | COVERED | Scan line + chromatic split + jitter for typography. Not implemented. |
| Holographic card shader | NONE | High | COVERED | For evolution rarity chip cards. Prismatic/Balatro polychrome. Not implemented. |

### Implementation

All post-processing is infrastructure — built in step 5d as `FullscreenMaterial` components on the camera.

**Bloom:** `VfxConfig.bloom_intensity` drives camera `Bloom` settings. Per-entity bloom via HDR material values >1.0.

**Additive blending:** `Material2d::specialize()` pattern — `BlendFactor::One` for dst_factor. Applied to entity_glow, particle, aura, beam, ring, glow_line materials. Not a recipe.

**Screen distortion:** `distortion.wgsl` FullscreenMaterial. 16-source fixed array. Triggered by `TriggerRadialDistortion` message.

**Glitch text shader:** `glitch_text.wgsl` Material2d. Child overlay entity on Text2d. Used by `GlitchText` PrimitiveStep.

**Holographic card shader:** `holographic.wgsl` Material2d. Applied to Evolution-rarity card background entities. See `docs/architecture/rendering/shaders.md`.

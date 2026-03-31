# UI & Screens

## HUD Elements

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Node timer | PLACEHOLDER | Critical | High | COVERED | Plain text number in status panel. Color shifts on time fraction. Uses proportional font (should be monospace). |
| Lives display | NONE | Critical | Medium | COVERED (partial) | `LivesCount` tracked but never displayed. |
| Node progress (e.g., 3/8) | NONE | Important | Low | COVERED (partial) | `RunState.node_index` tracked but not displayed. |
| Active chips display | NONE | Low | Low-Med | COVERED (partial) | Left "AUGMENTS" panel exists as empty container. |
| Side panels (structure) | PLACEHOLDER | Low | Low | COVERED | Thin glowing border matching neon dashboard aesthetic. Semi-transparent background. Temperature-following tint. Frames playfield without competing. |
| HUD style (diegetic vs dashboard) | NONE | Critical | Medium | COVERED | Resolved: diegetic/integrated. |

### Implementation

**Node timer:** Not a recipe — dedicated `timer_wall.wgsl` shader entity overlaid on the top wall. System in `screen/playing/hud/timer_wall.rs` updates `fill_level` uniform each FixedUpdate. Small monospace `Text2d` child for numeric readout. See `docs/architecture/rendering/hud.md`.

**Lives display:** Not a recipe for the orbs themselves — each orb is spawned via `AttachVisuals { shape: Circle, color: [archetype tint], glow: (...) }`. On life loss: recipe `"life_orb_dissolve"`: `Disintegrate(entity: Source, duration: 0.2) + SparkBurst(count: 4, velocity: 80.0, hdr: 0.5, color: White, gravity: 30.0, lifetime: 0.15)`. On life gain: recipe `"life_orb_birth"`: `ExpandingRing(speed: 150.0, max_radius: 10.0, thickness: 1.0, hdr: 0.6, color: White, lifetime: 0.15) + GlowMotes(count: 4, drift_speed: 15.0, radius: 8.0, hdr: 0.4, color: White, lifetime: 0.3)`. System in `screen/playing/hud/life_orbs.rs`.

**Node progress:** Not a recipe — small `entity_glow` Rectangle entities spawned along one side wall. Current node bright, completed dim, upcoming very dim. Driven by modifiers. System in `screen/playing/hud/node_progress.rs`.

**Active chips display:** Not implemented as explicit HUD — the build is communicated through entity visual state (bolt appearance, breaker aura, modifier effects). Remove the empty side panel container.

**Side panels:** Removed. Playfield takes full width with thin wall borders.

---

## Chip Cards

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Card shape ("cyber chip" outline) | NONE | Important | Medium | COVERED | Cards are plain rectangles. Should be angular/circuit-board-inspired. |
| Common rarity treatment | NONE | Important | Low | COVERED | White/silver glow border, no effects. |
| Rare rarity treatment | NONE | Important | Medium | COVERED | Electric blue glow, subtle pulse. |
| Epic rarity treatment | NONE | Important | Medium-High | COVERED | Magenta glow, shimmer wave animation. |
| Legendary rarity treatment | NONE | Important | High | COVERED | Gold glow (thicker), particle aura, animated energy. |
| Evolution rarity treatment | NONE | Critical | High | COVERED | Prismatic/holographic shifting border, Balatro polychrome shader. |
| Card icon/illustration | NONE | Important | Medium | COVERED | Abstract geometric symbols per DR-5. |
| Card selection animation | NONE | Important | Medium | COVERED | Selection tracked but no scale/animation response. |
| Card confirm animation | NONE | Important | Medium | COVERED | No absorption/collapse animation on confirm. |
| Card shatter (timer expired) | NONE | Important | High | COVERED | Timer expiry just ends selection. No card shatter. |
| Timer pressure (50% pulse) | NONE | Important | High | COVERED | No card pulsing at 50%. |
| Timer pressure (25% encroach) | NONE | Important | High | COVERED | No void encroachment at 25%. |
| Timer pressure (10% destabilize) | NONE | Important | High | COVERED | No card flickering at 10%. |

### Implementation

**Card shape:** Entity composition — parent entity with children. Card background is `entity_glow` with `Shape::RoundedRectangle { corner_radius: 0.15 }`. See `docs/architecture/rendering/chip_cards.md`.

**Rarity treatments:** Driven by the card background's glow params and optional shader overlays. Not recipes — `AttachVisuals` with rarity-specific `GlowParams`. Evolution cards get `holographic.wgsl` Material2d on a child overlay entity.

**Card icon:** `entity_glow` child entity with chip-category Shape (Circle for AoE, Diamond for Speed, etc.). Not a recipe.

**Card selection:** Not a recipe. Modifier-driven: selected card gets `SetModifier(GlowIntensity(1.5))`, unselected get `SetModifier(GlowIntensity(0.4))`. Transform animation (scale 1.0 → 1.1) on selection.

**Card confirm:** Recipe `"card_confirm"`: `ScreenFlash(color: White, intensity: 0.3, duration_frames: 2) + SparkBurst(count: 6, velocity: 100.0, hdr: 0.6, color: White, gravity: 20.0, lifetime: 0.15)`. Unselected cards fade out via modifier.

**Card shatter (timer expired):** Recipe `"card_shatter"`: `Fracture(entity: Source, shard_count: 6, velocity: 150.0, hdr: 0.8, color: White, lifetime: 0.3)`. Fired on each remaining card entity.

**Timer pressure:** Modifier-driven on card entities. `screen/chip_select/` system reads timer and sends:
- <50%: `SetModifier(GlowIntensity(1.0 + sin(time * 2.0) * 0.2), source: "timer_pressure")` (pulse)
- <25%: `SetModifier(GlowIntensity(0.6 + sin(time * 4.0) * 0.3), source: "timer_pressure")` (faster pulse, dimmer baseline)
- <10%: `SetModifier(AlphaOscillation { min: 0.5, max: 1.0, frequency: 8.0 }, source: "timer_destabilize")` on unselected cards

---

## Screens

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Loading screen | PLACEHOLDER | Low | Low-Med | COVERED | "Loading..." + cyan progress bar. No style treatment. |
| Main menu | PLACEHOLDER | Important | High | COVERED | Orbitron title + menu items. No interactive idle (bolt/breaker playground). |
| Run setup (breaker select) | PLACEHOLDER | Important | High | COVERED | Text-only cards. No archetype visual preview, no color coding. |
| Chip select screen | PLACEHOLDER | Critical | High | COVERED | Functional but plain. No rarity treatments, no card shape, no timer escalation. |
| Pause menu | PLACEHOLDER | Low | Low | COVERED | "PAUSED" + options. No desaturation overlay, no glitch text. |
| Run-end screen | PLACEHOLDER | Important | High | COVERED | Text-only stats + highlights. No animation, no seed display, no "almost unlocked." |
| Run-end "almost unlocked" | NONE | Important | Medium | COVERED | Not implemented. Evolution/achievement teases on defeat. |
| Meta-progression screen | NONE | Low | Low | DEFERRED | `GameState::MetaProgression` exists but no screen. Phase 10 feature. |

### Implementation

Screens are not recipe-driven — they are entity compositions spawned on state enter, despawned on exit. VFX within screens use recipes and primitives.

**Loading:** Minimal entity composition. Styled progress indicator (animated `entity_glow` entity) + game logo (`Text2d`). Void background with subtle grid.

**Main menu:** Interactive idle — spawn breaker + bolt + playfield (grid + walls, no cells). Bolt bounces autonomously. Menu options as `Text2d` entities (or Bevy UI). Title with GlitchText overlay.

**Run setup (breaker select):** Each archetype shown via `AttachVisuals` with full shape/aura/trail preview. Selection highlights via modifiers. Bolt preview alongside.

**Chip select:** Card entity composition (see Chip Cards section above). Timer pressure system. Selection/confirm animations.

**Pause:** `TriggerDesaturation { camera, target_factor: 0.7, duration: 0.2 }` on enter. GlitchText overlay on "PAUSED" title. Menu options. `TriggerDesaturation { camera, target_factor: 0.0, duration: 0.2 }` on exit.

**Run-end (victory):** Stats entities slam in with `ScreenShake(tier: Micro)` per reveal. Highlights animate with recipes. Flux earned display. Run seed in monospace `Text2d`.

**Run-end (defeat):** Hologram treatment — stats appear one by one with subtle fade-in. "Almost unlocked" evolution teases if applicable. Calm presentation.

**Meta-progression:** Deferred to Phase 10. No implementation needed in Phase 5.

---

## Typography

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Display font (Orbitron-Bold) | PARTIAL | Low | Medium | COVERED | Font loaded. Used for title only. No glitch overlay effects. |
| Body font (Rajdhani-Medium) | PARTIAL | Low | Low | COVERED | Font loaded. Used for menu items only. No scan line treatment. |
| Data font (monospace) | NONE | Critical | Low | COVERED | No monospace font in assets. Timer/seed need monospace for readability. |
| Glitch text shader (scan lines) | NONE | Low | Medium | COVERED | Not implemented. Style guide: scan lines + chromatic split + jitter. |

### Implementation

**Display/Body fonts:** Already loaded. Glitch overlay applied via GlitchText primitive where appropriate (titles, highlight labels). No recipe needed for the fonts themselves.

**Data font (monospace):** Placeholder TTF at `assets/fonts/mono.ttf`. Final font chosen in Phase 7 polish pass. Used by timer readout, seed display, numeric data.

**Glitch text shader:** `glitch_text.wgsl` in `rantzsoft_vfx`. `GlitchText` PrimitiveStep + `SpawnGlitchText` message. Child overlay entity on `Text2d`. See `docs/architecture/rendering/shaders.md` — glitch_text.wgsl section.

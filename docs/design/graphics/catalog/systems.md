# Systems & Infrastructure

## Temperature Palette System

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Run temperature resource | NONE | High | COVERED | No system tracks node progression or adjusts colors. All colors are static. |
| Temperature application | NONE | High | COVERED | Grid tint, cell glow, particle color, wall border, ambient bloom should all shift cool→hot. |

## Transitions

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Flash transition | PLACEHOLDER | Medium | COVERED | Full-screen alpha fade. Functional but no bloom spike or temperature tinting. |
| Sweep transition | PLACEHOLDER | Medium | COVERED | Full-screen rect sweep. Functional but solid color, not energy beam edge. |
| Glitch transition | NONE | High | COVERED | Not implemented. Screen corruption + static + distortion. |
| Collapse/Rebuild transition | NONE | High | COVERED | Not implemented. Elements build outward/collapse inward. |
| Random transition selection | NONE | Medium | COVERED | Not implemented. Should randomly pick from pool per transition. |

## Audio System

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Audio plugin | NONE | High | COVERED | `AudioPlugin` exists but is empty stub. No sound, no music. |
| Per-event SFX (15+ events) | NONE | High | COVERED | Every visual event needs a sound. None exist. |
| Layered intensity music | NONE | High | COVERED | 4-layer adaptive music (ambient→full). Not implemented. |
| Timer critical heartbeat | NONE | Critical | COVERED | Accelerating pulse sound. Primary chip-select pressure signal. |
| Music temperature shift | NONE | Medium | COVERED | Cool early → warm late paralleling visual temperature. |

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

## Post-Processing Pipeline

| Element | Status | Juice | Style Guide | Current |
|---------|--------|-------|-------------|---------|
| Bloom (tunable per-entity) | PARTIAL | High | COVERED | Camera bloom exists. No per-entity control or debug tuning. |
| Additive blending | NONE | High | COVERED | Default blending used. Style guide: additive for all light-on-dark. |
| Screen distortion shader | NONE | High | COVERED | Needed for shockwave, gravity well, explosion. Not implemented. |
| Glitch text shader | NONE | Medium | COVERED | Scan line + chromatic split + jitter for typography. Not implemented. |
| Holographic card shader | NONE | High | COVERED | For evolution rarity chip cards. Prismatic/Balatro polychrome. Not implemented. |

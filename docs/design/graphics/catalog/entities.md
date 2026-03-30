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

## Walls & Background

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Walls (left, right, ceiling) | NONE | Low | Medium | High | COVERED | Invisible collision entities. No mesh, no glow. |
| Wall bolt-impact flash | NONE | Low | Medium | Medium | COVERED | No visual response on wall hit. |
| Bottom wall shield barrier | NONE | Critical | High | High | COVERED | `SecondWind` + `ShieldActive` have no barrier visual. |
| Background grid | NONE | Low | Medium | High | COVERED | Pure void. No grid, no spatial reference, no energy sprites. |
| Background energy sprites | NONE | Low | Low | Medium | COVERED | No ambient grid animation. |
| Void background color | PARTIAL | Low | Low | Low | COVERED | `ClearColor` dark blue-purple. Close to target but slightly more purple. |

# Color & Palette

## Palette Philosophy: Evolving Temperature

The game does not have a single fixed palette. The palette **evolves across a run**, communicating escalation (Pillar 1) through color temperature:

- **Early nodes (1-3)**: Cool spectrum. Deep blues, teals, cyans. The screen feels cold, calm, controlled. The player is building.
- **Mid nodes (4-6)**: Transitional. Cyan bleeds into purple, violet appears, early hints of magenta. Tension is rising.
- **Late nodes (7-9)**: Hot spectrum. Magentas, ambers, warm whites. The screen radiates heat. Everything feels urgent.
- **Final/boss nodes**: White-hot. Dominant whites and golds with magenta/amber accents. Visual climax matches gameplay climax.

This temperature shift applies to:
- Background grid tint
- Default cell glow color
- Particle base color
- Wall border tint
- Ambient bloom color

It does NOT apply to (these maintain identity regardless of temperature):
- Bolt core color (must always be trackable)
- Breaker archetype colors (identity > temperature)
- Rarity tier colors (semantic meaning > temperature)
- UI elements (readability > atmosphere)

## The Void

The base of everything is **near-black** — not pure #000000 but a very deep blue-black (#050510 range) that allows for even darker elements (gravity well voids) to register. The void never changes — it is the constant anchor that makes all light visible.

## Semantic Colors

Certain colors carry fixed meaning across the entire game, regardless of temperature shift:

| Semantic | Color | Usage |
|----------|-------|-------|
| Player (bolt core) | Bright white with warm glow | Always visible, cuts through all effects |
| Perfect bump | Gold/white flash | Instant positive feedback — "you nailed it" |
| Early/Late bump | Archetype-tinted flash (dimmer) | Lower-intensity feedback — "acceptable but not great" |
| Danger/damage | Red-orange | Bolt lost, life lost, timer critical |
| Shield active | Distinct from player color (TBD — see decisions-required.md) | Must not be confused with bolt or breaker glow |
| Healing/regen | Green | Regen cells, life restoration |

## Rarity Tier Colors

Chip cards and evolution offerings use rarity colors that are fixed and immediately recognizable:

| Rarity | Color | Treatment |
|--------|-------|-----------|
| Common | White/silver | Clean glow border, no special effects |
| Uncommon | (TBD) | Moderately brighter glow |
| Rare | Electric blue | Brighter glow, subtle pulse |
| Evolution | Prismatic/holographic | Multi-color shift, unique holographic shader (Balatro polychrome reference) |

Evolution rarity receives special visual treatment on chip cards — see `ui-screens.md` for card styling details.

## Additive Blending

Because everything is light-on-dark, most visual elements use **additive blending** (or screen blending). This means:
- Overlapping glows naturally combine and brighten
- Dense particle areas create hot spots of white
- No need for alpha sorting in most cases
- Visual density = brightness, which reads as intensity/power

This is a core technical choice that reinforces the "light is the material" identity pillar.

## HDR and Bloom

The game uses HDR rendering with bloom. Elements can exceed the standard brightness range:
- Bolt core: >1.0 HDR brightness (blooms into surrounding space)
- Perfect bump flash: Brief >2.0 spike (visible bloom halo)
- Shockwave edge: >1.0 (expanding ring of bloom)
- Background grid: <0.3 (never competes with gameplay elements)

Bloom radius and intensity should be configurable in the debug menu for tuning.

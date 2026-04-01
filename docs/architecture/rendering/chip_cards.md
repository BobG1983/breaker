# Chip Card Rendering

## Overview

Chip cards are shown during the chip select screen (`screen/chip_select/`). Each card displays: chip name, rarity treatment, abstract symbol icon, and timer pressure visual.

## Rendering Approach: Entity Composition + Shaders

Each card is a **parent entity** with children:

| Child | Rendering | Purpose |
|-------|-----------|---------|
| Card background | `entity_glow` (Shape::RoundedRectangle) with rarity-colored border | Card frame |
| Icon | `entity_glow` with chip-specific Shape | Abstract geometric symbol |
| Name text | Bevy `Text2d` with monospace font | Chip name label |
| Rarity label | `Text2d` + GlitchText overlay (Evolution rarity only) | Rarity indicator |
| Holographic overlay | `holographic.wgsl` Material2d (Evolution rarity only) | Prismatic shimmer |

### Rarity Treatments

| Rarity | Border Color | Background | Special Effect |
|--------|-------------|------------|----------------|
| Common | Dim white/silver | Dark, minimal glow | None |
| Rare | Electric blue | Faint blue tint | Subtle pulse animation |
| Epic | Magenta | Faint magenta tint | Shimmer wave animation |
| Legendary | Gold (thicker) | Warm amber tint | Particle aura around card edges |
| Evolution | Prismatic shifting | `holographic.wgsl` shimmer | GlitchText on rarity label |

### Abstract Symbol Icons

Geometric shapes representing effects — consistent with the abstract neon aesthetic (per DR-5). Each chip maps to one of the existing Shape enum variants or a simple composition:

| Effect Category | Icon Shape | Examples |
|----------------|-----------|----------|
| AoE | Circle | Shockwave, Explode, Pulse |
| Speed | Diamond | Speed Boost, Quick Stop |
| Protection | Shield | Shield, Second Wind |
| Damage | Angular | Damage Boost, Glass Cannon |
| Chain/Arc | Hexagon | Chain Lightning, Voltchain |
| Spawn | Octagon | Split Decision, Feedback Loop |

Icons are `entity_glow` entities with the appropriate Shape. Small scale (~16-24px), centered on the card.

### Timer Pressure

As the chip select timer runs down, card borders pulse faster. Driven by `SetModifier` on each card entity:
- Normal: `GlowIntensity` at baseline, no pulse
- <50% time: subtle pulse begins (modifier oscillation)
- <25% time: rapid pulse, color shifts toward red-orange
- Timer expired: auto-select (game logic, not rendering)

## Card Layout

Cards arranged horizontally, centered in the playfield. Typically 3 cards. Spacing and scale TBD during implementation — depends on playfield dimensions and card content.

**Selection feedback:** Selected card pulses brightly + scale up (Transform animation). Unselected cards dim (modifier). On confirm: selected card plays a recipe (brief flash + sparks), unselected cards fade out.

## What Lives Where

| Concern | Owner |
|---------|-------|
| Card entity composition + layout | `screen/chip_select/` |
| Card selection logic + timer | `screen/chip_select/` |
| `holographic.wgsl` shader | `rantzsoft_vfx` |
| GlitchText overlay system | `rantzsoft_vfx` (generic primitive) |
| Chip data (name, rarity, effects) | `chips/` domain |

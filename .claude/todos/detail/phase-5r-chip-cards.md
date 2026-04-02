# 5r: Chip Cards

**Goal**: Transform chip selection cards into styled cards with rarity treatments, abstract symbol icons, and timer pressure visualization.

Architecture: `docs/architecture/rendering/chip_cards.md`

## What to Build

### 1. Card Entity Composition

Each card is a parent entity with children:
- Card background: `entity_glow` (Shape::RoundedRectangle) with rarity-colored border
- Icon: `entity_glow` with chip-specific Shape (abstract geometric symbol per DR-5)
- Name text: Bevy `Text2d` with monospace font
- Rarity label: `Text2d` + GlitchText overlay (Evolution rarity only)
- Holographic overlay: `holographic.wgsl` Material2d (Evolution rarity only)

All card systems live in `screen/chip_select/`.

### 2. Rarity Treatments

| Rarity | Border | Special Effect |
|--------|--------|----------------|
| Common | Dim white/silver | None |
| Rare | Electric blue | Subtle pulse animation |
| Epic | Magenta | Shimmer wave animation |
| Legendary | Gold (thicker) | Particle aura around card edges |
| Evolution | Prismatic shifting | `holographic.wgsl` shimmer + GlitchText on rarity label |

### 3. Abstract Symbol Icons

Geometric shapes representing effects:

| Category | Icon Shape |
|----------|-----------|
| AoE | Circle |
| Speed | Diamond |
| Protection | Shield |
| Damage | Angular |
| Chain/Arc | Hexagon |
| Spawn | Octagon |

Icons are `entity_glow` entities with the appropriate Shape enum variant.

### 4. Timer Pressure

Cards react to chip select timer via `SetModifier`:
- Normal: baseline glow
- <50% time: subtle border pulse begins
- <25% time: rapid pulse, color shifts toward red-orange
- Timer expired: auto-select (game logic)

### 5. Selection Animation

- Selected: scale up + `SetModifier(GlowIntensity(1.5))` + energy pulse recipe
- Unselected: `SetModifier(GlowIntensity(0.5))` — dim
- Confirmed: brief flash recipe, unselected cards fade out

## Dependencies

- **Requires**: 5c (crate), 5d (bloom), 5f (types), 5o (GlitchText for evolution labels)
- DR-5 resolved: abstract geometric symbols

## Verification

- Each rarity tier visually distinct
- Evolution cards show holographic shimmer
- Selection animation responds to input
- Timer pressure visible at each threshold
- All existing tests pass

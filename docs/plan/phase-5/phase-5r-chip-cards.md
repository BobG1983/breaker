# 5r: Chip Cards

**Goal**: Transform chip selection cards from plain rectangles into styled "cyber chip" cards with rarity treatments and timer pressure visualization.

## Icon Strategy: Abstract Geometric Symbols

Each chip card gets an abstract geometric symbol representing its effect type — circle for AoE, arrow for speed, shield for protection, etc. Consistent with the abstract neon aesthetic. Scales well across 20+ chips without per-chip art. Icons are simple geometric compositions, not illustrations.

## What to Build

### 1. Card Shape

Current: Plain rectangles.

Target:
- "Cyber chip" outline — angular/circuit-board-inspired, not a standard rectangle
- Glowing line outline, not solid fill
- Semi-transparent interior (void shows through)

### 2. Rarity Treatments

| Rarity | Border | Background | Special Effect |
|--------|--------|------------|----------------|
| Common | White/silver glow line | Near-transparent | None |
| Rare | Electric blue glow line | Faint blue tint | Subtle pulse animation |
| Epic | Magenta glow line | Faint magenta tint | Shimmer wave animation |
| Legendary | Gold glow line (thicker) | Warm amber tint | Particle aura around card edges |
| Evolution | Prismatic/holographic shifting border | Holographic background shader | Full holographic treatment |

### 3. Holographic Card Shader

For Evolution rarity:
- Prismatic color shifts with selection position / cursor movement
- Rainbow reflections
- Balatro polychrome reference
- Uses the glitch text shader from 5o for card name

### 4. Card Icons (Abstract Symbols)

Each chip gets an abstract geometric symbol:
- AoE effects (Shockwave, Pulse, Explode): expanding circle/ring
- Speed effects: arrow/streak
- Damage effects: angular burst/spike
- Defensive effects (Shield, Second Wind): curved barrier shape
- Utility effects: effect-specific geometric shape
- Symbols are simple compositions of lines, circles, and angles — not illustrations

### 5. Card Selection Animation

| State | Visual |
|-------|--------|
| Unselected | Base rarity treatment, slightly dimmer/smaller |
| Selected (hovering) | Scale up, border brightens, rarity animation intensifies, energy pulse from center outward |
| Confirmed | Flash bright, card collapses/absorbs into player's build |

### 6. Timer Pressure Visualization

Progressive urgency escalation during chip select:

| Timer Phase | Visual Effect |
|-------------|--------------|
| 50% remaining | Cards pulse in sync with a rhythm (border brightness fluctuation) |
| 25% remaining | Void encroaches on card edges — borders dim, darkness creeps inward. Pulse accelerates. Timer shifts to danger color. |
| 10% remaining | Unselected cards flicker/destabilize — glitch artifacts, scan line distortion. Only selected card remains stable. |
| 0% (expired) | Remaining cards shatter — fracture into Shard particles scattered into void. Selection lost. |

### 7. Card Shatter on Timer Expiry

When timer hits zero during chip select:
- Cards fracture into Shard particles (from 5e)
- Shards scatter into void
- Quick desaturation pulse

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing for bloom, holographic shader), 5e (particles: Shard for shatter), 5o (glitch text shader for evolution card names)
- **Enhanced by**: 5k (screen effects for timer pressure vignette)
- DR-5 resolved: abstract geometric symbols

## Catalog Elements Addressed

From `catalog/ui-screens.md` (Chip Cards):
- Card shape: NONE → cyber chip outline
- Common rarity: NONE → implemented
- Rare rarity: NONE → implemented
- Epic rarity: NONE → implemented
- Legendary rarity: NONE → implemented
- Evolution rarity: NONE → holographic shader
- Card icon/illustration: NONE → abstract geometric symbols
- Card selection animation: NONE → implemented
- Card confirm animation: NONE → implemented
- Card shatter (timer expired): NONE → implemented
- Timer pressure (50% pulse): NONE → implemented
- Timer pressure (25% encroach): NONE → implemented
- Timer pressure (10% destabilize): NONE → implemented

From `catalog/systems.md`:
- Holographic card shader: NONE → implemented

## Verification

- Each rarity tier has a visually distinct card treatment
- Evolution cards show holographic shifting effect
- Selection animation responds to hover/confirm
- Timer pressure escalation is visible at each threshold
- Card shatter on timeout produces particles
- All existing tests pass

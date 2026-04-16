# 5m: Chip Cards

## Summary

Transform chip selection cards into styled entity-composition cards with rarity-specific visual treatments, abstract symbol icons (per DR-5), holographic shader for evolution-rarity cards, timer pressure visualization, and selection animations. Cards are world-space entity hierarchies using EntityGlowMaterial — not Bevy UI nodes. Each rarity tier is instantly distinguishable at a glance.

## Context

The chip select screen is a key decision moment with a countdown timer (Pillar 5 — decisions under pressure). Cards must be readable fast, feel premium, and communicate rarity instantly. The current implementation (if any) uses basic UI elements. This phase replaces them with fully styled world-space entities that participate in the game's visual identity.

Key architecture change from the LEGACY plan: the old plan referenced `AttachVisuals` messages and `ExecuteRecipe` for card animations (energy pulse recipe, dissolve recipe). The new plan uses direct entity composition — each card is a parent entity with child entities for background, icon, text, and optional holographic overlay, all constructed via a card builder function. Animations are direct modifier applications and tween systems, not recipes.

The holographic shader (`holographic.wgsl`) and `HolographicMaterial` were built in 5e (visuals domain). The abstract symbol icon approach was decided in DR-5.

## What to Build

### 1. Card Entity Composition

Each chip card is a parent entity with the following children:

**Card background**:
- `EntityGlowMaterial` with `Shape::RoundedRectangle { corner_radius }` — the "cyber chip" outline shape
- Border color and thickness driven by rarity tier
- Interior is semi-transparent (alpha ~0.1-0.2) — the void shows through slightly
- Angular notches or connection points along the border (achieved via the RoundedRectangle SDF with additional masking, or via Custom shape with notch vertices)

**Icon**:
- `EntityGlowMaterial` with a chip-specific `Shape` variant representing the effect category (abstract geometric symbol per DR-5)
- Centered within the card area
- Color matches the chip's effect identity, glow params tuned for readability

**Name text**:
- Bevy `Text2d` child entity with Heading-level typography (medium, moderate glitch treatment if GlitchMaterial is applied)
- Positioned at the top of the card
- Monospace variant not needed here — Heading font is fine

**Description text**:
- Bevy `Text2d` child entity with Body-level typography (small, minimal glitch — scan lines only, readability first)
- Positioned below the icon
- Brief, one-line description of the chip's effect

**Rarity label** (Evolution rarity only):
- `Text2d` with GlitchMaterial overlay (from 5m's GlitchText pattern)
- "EVOLUTION" text with full glitch treatment (scan lines + chromatic split + jitter)

**Holographic overlay** (Evolution rarity only):
- Child entity with `HolographicMaterial` (`holographic.wgsl`) covering the card background
- Prismatic foil shimmer based on UV position + time
- Renders on top of the card background, beneath the icon and text

Card builder function:
```
fn spawn_chip_card(
    commands: &mut Commands,
    chip_definition: &ChipDefinition,
    position: Vec2,
    ...
) -> Entity
```

All card systems live in the chip select state module (e.g., `state/run/chip_select/` or wherever chip selection ends up).

### 2. Rarity Treatments

Each rarity tier has a distinct visual treatment applied to the card background entity:

| Rarity | Border Color | Border Brightness | Background Tint | Special Effect |
|--------|-------------|-------------------|-----------------|----------------|
| Common | White/Silver | HDR ~0.8 (dim) | Near-transparent (alpha ~0.05) | None — clean and simple |
| Uncommon | (TBD) | HDR ~1.0 | Near-transparent (alpha ~0.06) | Subtle glow |
| Rare | Electric blue (CornflowerBlue/DodgerBlue) | HDR ~1.2 | Faint blue tint (alpha ~0.08) | Subtle pulse: `AlphaOscillation { min: 0.9, max: 1.1, frequency: 1.5 }` on border |
| Evolution | Prismatic shifting border | HDR ~2.0+ | HolographicMaterial background | Full holographic treatment via `holographic.wgsl` — color shifts with time, rainbow reflections. GlitchText rarity label. Border color cycles through spectral hues. |

Rarity treatment is applied by the card builder function based on `ChipDefinition.rarity`.

### 3. Abstract Symbol Icons

Geometric shapes representing effect categories per DR-5:

| Category | Icon Shape | Shape Enum Variant | Examples |
|----------|-----------|-------------------|----------|
| AoE (area damage) | Circle | `Shape::Circle` | Shockwave, Pulse, Explode |
| Speed/Movement | Diamond | `Shape::Diamond` | Speed Boost, Quick Stop |
| Protection/Defense | Shield | `Shape::Shield` | Shield, Second Wind |
| Direct Damage | Angular | `Shape::Angular` | Damage Boost, Piercing Beam |
| Chain/Arc/Multi-target | Hexagon | `Shape::Hexagon` | Chain Lightning, Tether Beam, ArcWelder |
| Spawn/Creation | Octagon | `Shape::Octagon` | Split Decision, Supernova |
| Utility/Misc | Rectangle | `Shape::Rectangle` | Time Penalty, Attraction |

Icons are `EntityGlowMaterial` entities with the appropriate Shape variant. Icon color matches the chip's primary color (from ChipDefinition or a category-to-color mapping). Icon glow is moderate — bright enough to read against the semi-transparent card background.

The mapping from chip to icon category should be defined as a function or lookup, not hardcoded per-chip, so new chips automatically get an icon based on their effect category.

### 4. Timer Pressure Visualization

Cards react to the chip select countdown timer, creating escalating visual urgency (Pillar 5):

**Normal (>50% time remaining)**:
- Baseline rarity treatment
- Cards are stable and readable

**At 50% remaining**:
- Cards begin to pulse in sync with an accelerating rhythm
- Border brightness oscillates: `AlphaOscillation` modifier with increasing frequency
- Pulse is subtle — a brightness fluctuation on borders, not a size change

**At 25% remaining**:
- Void encroaches: card backgrounds darken from edges inward (reduce alpha on card background, or add a darkening overlay child entity that grows inward)
- Rapid pulse: border oscillation frequency increases
- Timer text (if visible) shifts to danger color (OrangeRed)
- Only the selected card maintains full brightness — unselected cards are visibly dimming

**At 10% remaining**:
- All unselected cards flicker and destabilize: `AddModifier` with `SquashStretch` jitter (small random values each frame) + `AlphaOscillation` with high frequency + aggressive `GlowIntensity` fluctuation
- GlitchMaterial effects intensify on unselected card text (scan lines thicken, chromatic split widens)
- Only the currently-selected card remains stable and readable
- Player's eye is drawn to the one stable card — the selection

**At 0% (time expired)**:
- Remaining unselected cards shatter: each card's EntityGlowMaterial ramps `dissolve_threshold` to 1.0 over ~0.2s
- `ParticleEmitter` with `Burst { count: 16 }` per card — shard particles scattering outward
- Game logic handles auto-selection

Timer pressure system reads the chip select timer resource and applies the appropriate modifications each frame.

### 5. Selection Animation

Cards respond to selection state (cursor/indicator position):

**Unselected**:
- Base rarity treatment, slightly recessed
- `SetModifier` with `GlowIntensity(0.6)` — dimmer than selected
- Scale: 0.95x (slightly smaller)

**Selected (hovering — current selection before confirmation)**:
- Card scales to 1.05x (slightly larger) with smooth lerp
- Border brightens: `SetModifier` with `GlowIntensity(1.5)`
- Rarity animation intensifies: pulse faster, shimmer brighter, particle rate increases
- Energy pulse: `ParticleEmitter` with `Burst { count: 8 }` from card center outward — brief "activation" burst on selection change

**Confirmed (player confirms the selection)**:
- Card flashes bright: `AddModifier` with `GlowIntensity(4.0)` for 0.1s
- `TriggerScreenFlash` brief, matching card rarity color
- Unselected cards fade out: `FadeOut` component with 0.3s duration
- Selected card collapses: scale shrinks to 0.0 over 0.2s with a final spark burst — the chip "absorbs" into the player's build

Selection system responds to input events (left/right to change selection, confirm to choose).

### 6. Card Layout System

Positions cards on screen based on the number of options:

- Cards are centered horizontally in the viewport
- Spacing between cards is proportional to card width (e.g., 1.2x card width between centers)
- Cards may have a slight arc layout (curved row) or straight row — tune during implementation
- Layout recalculates if the number of cards changes (e.g., some rare events might offer fewer choices)
- Cards spawn with a staggered entry animation: each card fades in (reverse dissolve) with a 0.05s delay between them, left to right

## What NOT to Do

- Do NOT implement cards as Bevy UI nodes (Node, Button, Interaction) — all cards are world-space entity hierarchies with EntityGlowMaterial
- Do NOT implement chip effect preview or "try before you buy" — that is a future feature
- Do NOT implement the full chip select game logic (timer, selection rules, chip offering algorithm) — that already exists. This phase only adds visuals to the existing logic.
- Do NOT implement per-chip unique illustrations — DR-5 resolved this as abstract geometric symbols, not illustrations
- Do NOT create per-chip art assets — icons are procedural EntityGlowMaterial shapes

## Dependencies

- **Requires**: 5e (visuals domain — EntityGlowMaterial, HolographicMaterial, GlitchMaterial, Shape enum, Hue, GlowParams, modifier messages, PunchScale, FadeOut), 5j (modifier computation — for dynamic card effects; temperature palette for base color tinting)
- **Enhanced by**: 5c (rantzsoft_particles2d — shard particles for timer-expired shatter, energy bursts), 5m (GlitchText — for Evolution rarity label treatment)
- **Required by**: 5p (screens — breaker select and run-end screens reference chip card visual patterns)

## Verification

- Each rarity tier is visually distinct at a glance — a player can identify rarity without reading text
- Common: clean, simple, white/silver border, no effects
- Uncommon: slightly brighter glow (TBD treatment)
- Rare: blue border, subtle pulse
- Evolution: holographic prismatic background, shifting border, GlitchText label
- Abstract symbol icons correctly represent effect categories
- Timer pressure visualization escalates correctly at 50%, 25%, 10%, and 0% thresholds
- At 10% remaining: unselected cards visibly destabilize, selected card remains stable
- At 0%: unselected cards shatter with dissolve + shard particles
- Selection animation: unselected cards are dimmer/smaller, selected card is brighter/larger
- Selection change triggers energy pulse burst
- Confirmation triggers flash + fade of unselected cards + selected card collapse
- Cards spawn with staggered entry animation
- Card layout is centered and evenly spaced
- HolographicMaterial renders prismatic shimmer on Evolution cards
- All text is readable against card backgrounds at all rarity levels
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL

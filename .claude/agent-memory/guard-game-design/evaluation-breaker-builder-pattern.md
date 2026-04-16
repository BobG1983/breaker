---
name: Breaker Builder Pattern Design Evaluation
description: Typestate builder, LivesSetting, ClampRange, BreakerDefinition 35+ fields, Visual dimension — all approved, no design constraints
type: project
---

## Evaluation: Breaker Builder Pattern (2026-04-02)

### Approved — No Design Concerns

Pure structural refactor collapsing 4-system spawn pipeline into single `build()` call. No gameplay impact, no new mechanics, no changes to feel.

### Key Findings

1. **Typestate builder does not constrain future gameplay**. Abilities live in `Vec<RootNode>` (optional data), bolt types are separate entities via `bolt: String` field, new dimensions can be added as optional fields without typestate change. The `definition()` convenience transitions 5 dimensions at once but runtime modifications (chips) should modify components, not spawn params.

2. **LivesSetting enum correctly replaces `Option<Option<u32>>`**. Unset/Infinite both produce `LivesCount(None)`. Count(n) produces `LivesCount(Some(n))`. All three archetypes (Aegis lives, Chrono infinite, Prism infinite) covered. Future resource-based archetypes would use new components, not extend LivesSetting.

3. **ClampRange (min/max) is the right foundation for size-boost chips**. Multiplicative stacking composes clean. Default range 0.5x-5.0x base prevents degenerate states (too small = unplayable, too large = no skill). Per-archetype overrides supported via RON `min_w`/`max_w`/`min_h`/`max_h`.

4. **BreakerDefinition 35+ fields maps exactly to gameplay knobs**: position (movement + dash), angle (tilt + spread), timing (bump windows + cooldowns). All `#[serde(default)]` + `deny_unknown_fields`. Adding future fields won't break existing RON.

5. **Visual Rendered/Headless binary split is sufficient**. Custom shaders replace material type (method refactor, not typestate). Per-archetype meshes = method signature change. Trail/glow = additional components. Third visual mode would need more terminal impls but is architecturally feasible.

### Expansion Paths Validated
- New breaker abilities: effect chains, no builder change
- Size-boost chips: ClampRange + effective_size ready
- Per-archetype bump juice: feedback params in definition
- New bolt types: separate builder, connected by `bolt: String`
- New resource systems (energy/charges): new optional component

**Why:** Tracks design evaluation of breaker builder migration to prevent revisiting.
**How to apply:** Reference when evaluating future breaker archetype design, size-boost chips, or visual rendering changes.

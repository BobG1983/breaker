---
name: QuickStop Replaces TiltControl
description: QuickStop (breaker deceleration multiplier) replaces removed TiltControl effect — enables speed+precision builds
type: project
---

## Decision: Replace TiltControl with QuickStop

**Why:** TiltControl (tilt sensitivity increase) removed — control scheme doesn't support it. QuickStop fills the breaker modifier slot with movement mastery that rewards aggressive play.

**QuickStop effect:**
- Parameter: decel_mult (f32, 1.x format — 2.0 = 2x deceleration)
- fire(): Push decel_mult to ActiveDecelBoosts vec
- reverse(): Remove matching entry from ActiveDecelBoosts
- Stacking: Multiplicative (like SpeedBoost)

**Rarity progression (design-from-Rare):**
- Common "Steady": decel_mult 1.4
- Uncommon "Firm": decel_mult 1.75
- Rare "Precise": decel_mult 2.0 + SpeedBoost(1.1) — synergy hook
- max_taken: 3

**How to apply:**
- Enables "faster but more precise" build fantasy — take speed chips without losing control
- Rare opens SpeedBoost scaling path (interacts with all speed-related chips)
- Three Rare stacks = 8x decel + 1.33x speed = teleport-and-plant playstyle
- Evolution candidates: QuickStop x2 + Breaker Speed x2 -> "Flash Step" (micro-teleport on perfect bump), or QuickStop x2 + Bump Force x2 -> "Anchor" (plant-and-hit force boost)
- The "plant" moment (full speed to dead stop) needs visual juice: glow, micro-shake, decel streak

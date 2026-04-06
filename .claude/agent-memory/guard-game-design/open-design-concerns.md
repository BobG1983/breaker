---
name: Open Design Concerns
description: Unresolved design concerns from full verification reviews — breaker differentiation, Surge stacking, Attraction gate
type: project
---

## Open Design Concerns

### 1. Breaker archetype differentiation
Aegis and Chrono have identical bump speed profiles (PerfectBumped 1.5x, Early/Late 1.1x). No stat_overrides used. Need at minimum different speed/width/force profiles.

**Why:** Breaker choice is meaningless if stats are identical — only abilities differ.
**How to apply:** Address when breaker or chip tuning work begins. Each breaker should have a distinct mechanical identity beyond its active ability.

### 2. Surge permanent stacking
Surge applies permanent SpeedBoost per PerfectBumped with no expiry. 3 Rare Surges = potential 3.375x permanent speed. Likely makes Overclock obsolete. Needs Until node or value reduction.

**Why:** Unbounded permanent stacking breaks the power curve and obsoletes other upgrade paths.
**How to apply:** Address before Phase 8 (content & variety) — any balance pass needs this resolved first.

### 3. Attraction(Breaker) gate
System supports Attraction(Breaker) but no chip uses it yet. Must be Legendary-only with trade-off when shipped.

**Why:** Breaker-attraction is a powerful mechanic that trivializes positioning if not gated appropriately.
**How to apply:** Gate at Legendary rarity with a meaningful downside when this chip is eventually created.

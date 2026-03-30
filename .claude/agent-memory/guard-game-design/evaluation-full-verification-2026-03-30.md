---
name: Full Verification Review 2026-03-30
description: Comprehensive design review of develop branch against all 9 pillars — 2 blockers, 3 concerns, 6 watch items
type: project
---

## Full Verification Tier Design Review — 2026-03-30

### Blockers
1. **BASE_BOLT_DAMAGE hardcoding** — 6 combat effects (Shockwave, Pulse, Explode, ChainLightning, PiercingBeam, TetherBeam) use const 10.0 instead of EffectiveDamageMultiplier. Breaks Pillar 2 synergy web. Combat effects must scale with bolt's effective damage.
2. **Chip catalog doc drift** — `docs/design/chip-catalog.md` values use old additive format, RON files use correct 1.x multiplicative format. Affects ~10 chips.

### Concerns
3. **Breaker archetype differentiation** — Aegis and Chrono have identical bump speed profiles (PerfectBumped 1.5x, Early/Late 1.1x). No stat_overrides used. Need at minimum different speed/width/force profiles.
4. **Surge permanent stacking** — Surge applies permanent SpeedBoost per PerfectBumped with no expiry. 3 Rare Surges = potential 3.375x permanent speed. Likely makes Overclock obsolete. Needs Until node or value reduction.
5. **Attraction(Breaker) gate** — System supports it but no chip uses it yet. Must be Legendary-only with trade-off when shipped.

### Watch Items (Not Blocking)
- Transition timing: 0.5s out + 0.3s in = 0.8s dead air. Appropriate.
- Difficulty curve: HP 1.0x-2.5x, timer 1.0x-0.6x, active ratio 0%-100%. Good escalation.
- Cell type wiring: Lock and Regen defined but not in difficulty curve introduced_cells. Phase 7.
- Entropy Engine counter: No visual indicator for counter progress. Phase 5 item.
- Multiplicative stacking visuals: No feedback for damage/speed multiplier magnitude. Phase 5 critical.
- Legendaries: Strong design. Tempo, Feedback Loop, Ricochet Protocol, Deadline are standouts.

### Chip Catalog Health
- 31 templates, 11 evolutions, 3 breakers
- Synergy coverage: >30% cross-chip interaction (Cascade+Flux->Entropy Engine, Impact+Tether, Splinter+Piercing->Split Decision, etc.)
- Rarity design: Rare variants consistently add synergy hooks per chip-rarity-rework decision

**Why:** Comprehensive state-of-game design checkpoint. Reference for future reviews.
**How to apply:** Check blockers 1-2 status before any future merge to main. Reference concerns 3-4 when breaker or chip tuning work begins.

---
name: Toughness + HP Scaling Evaluation
description: Evaluation of exponential HP scaling (1.2^tier), three toughness levels, boss multiplier, guardian fraction — APPROVED with 4 tuning recommendations
type: project
---

## Toughness + HP Scaling — Design Evaluation (2026-04-08)

### Verdict: APPROVED

Core design is correct: exponential tier scaling, three toughness levels (Weak 10 / Standard 20 / Tough 30), config-driven tuning via RON, guardian HP as fraction of parent.

### Key Numbers
- Formula: `base * 1.2^tier * (1.0 + 0.05 * node_index)`
- Bolt base damage: 10.0
- Standard cell at Tier 0: 20 HP = 2 hits minimum at base damage
- Standard cell at Tier 8: ~86 HP = ~9 hits at base damage
- Boss multiplier: flat 3x across all toughness levels and tiers

### Tuning Recommendations (not blockers)
1. **Standard HP vs bolt damage ratio**: Standard=20 + bolt_damage=10 means 2-hit minimum from the start. Consider bumping bolt base damage to 15-20 OR accept this as intentional. Impacts early-game feel significantly.
2. **Damage economy validation at Tier 8**: 86 HP Standard cells need 9 hits at base. Verify node timer budget supports no-damage-investment play, or damage chips become mandatory (narrows build diversity).
3. **Escalating boss multiplier**: Flat 3x misses "tension only goes up." Consider 2x (T1-3) / 3x (T4-6) / 4x (T7-8) / 4x+hazards (T9+).
4. **Guardian HP fraction unspecified**: Recommend 0.3-0.4x of parent HP. Too low = trivial guardians, too high = tedious.

### Phase 5 Dependency
Multi-hit cells require progressive visual damage states. Without visible degradation, high-HP cells feel like sponges. This is load-bearing for the system's feel.

**Why:** First HP scaling system — sets the baseline for all future cell HP tuning and damage economy.
**How to apply:** Reference these recommendations when implementing toughness config, when tuning bolt damage, and when designing Phase 5 cell damage visuals.

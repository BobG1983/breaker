---
name: Phase 4+5 Effect Roster Evaluation
description: Full pillar evaluation of 15 runtime effects — 14 approved, Shield flagged for redesign, two implementation gaps identified
type: project
---

## Evaluation: 15 Effects Against Design Pillars

**Date**: 2025-03-28
**Scope**: Shockwave, Pulse, Explode, Shield, SecondWind, Attraction, SpawnPhantom, ChainBolt, ChainLightning, PiercingBeam, SpawnBolts, TetherBeam, TimePenalty, RandomEffect, EntropyEngine

### Results

- **14 of 15 effects approved** — serve the core identity of speed, tension, build variety
- **Shield flagged for redesign** — violates Pillar 1 (Escalation) and Pillar 5 (Pressure Not Panic) by creating a safe harbor with zero skill expression

### Shield Redesign Recommendation

Current: timed damage/bolt-loss immunity. Problem: helps everyone equally, no skill expression, relieves tension.

Proposed: Shield absorbs N hits before breaking, each hit while shielded triggers a counter-effect (shockwave, speed boost). This makes Shield an offensive tool (seek hits while shielded), preserves tension, and opens synergy paths.

Alternative: Keep timed immunity but cap at 0.5-1.0s (dodge-frame feel, not vacation).

### Implementation Gaps Found

1. **DamageCell source_chip: None everywhere** — combat effects (Shockwave, Pulse, Explode, ChainLightning, PiercingBeam) all send DamageCell with source_chip: None. Blocks damage attribution for run-end highlights.

2. **BASE_BOLT_DAMAGE hardcoding** — Shockwave and Pulse use BASE_BOLT_DAMAGE constant instead of effective damage multiplier. DamageBoost stacking doesn't amplify these effects. Weakens synergy web.

### Tuning Watch

- **Attraction(Breaker)** — high force values could make bolt loss trivially avoidable, destroying core tension. Recommend Legendary-only or hard force cap.
- **Pulse interval** — hardcoded at 0.5s, should be parameterized for stack-based frequency scaling.

### Highest-Praise Effects

- **PiercingBeam** — highest skill ceiling in roster (beam direction = bolt velocity = bump aim)
- **TetherBeam** — creates new control axis (position two bolts for beam sweep)
- **EntropyEngine** — perfect evolution capstone (escalating chaos per node)
- **SecondWind** — model for how to do defensive effects (one save, tension increases after use)

**Why:** Guards against revisiting approved decisions and ensures Shield redesign doesn't get lost.
**How to apply:** Reference when evaluating new defensive effects or Shield rework proposals. Ensure implementation gaps are tracked for fix.

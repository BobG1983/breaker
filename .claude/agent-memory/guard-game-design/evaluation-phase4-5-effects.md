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

### Shield Status (Updated 2026-04-02)

Refactored AGAIN from charge-based `ShieldActive` to timed visible floor wall (`ShieldWall`). See `evaluation-shield-wall-refactor.md` for full pillar evaluation. APPROVED with tuning recommendations (reduce duration from 5s to 3s, add visual decay, add timer cost per bolt reflected). Cell shielding via Shield effect REMOVED (cell types retain their own ShieldBehavior orbits). Phase 7 enrichment: partial-width wall centered on breaker position.

### Implementation Gaps

1. **RESOLVED: DamageCell source_chip threading** — All combat effects now thread chip attribution via `EffectSourceChip` component. Closed 2026-03-29.

2. **RESOLVED: BASE_BOLT_DAMAGE hardcoding** — Resolved in feature/breaker-builder-pattern (2026-04-02). `BoltDefinition.base_damage` replaces the hardcoded const. Combat effects (Shockwave, Pulse, etc.) now read `base_damage` from the registry/definition; `ActiveDamageBoosts.multiplier()` scales it. Synergy web intact. Do NOT re-flag.

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

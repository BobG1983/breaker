---
name: Runtime Effects Round 2 Evaluation
description: Evaluated source_chip threading, Shield charge model, Chain Lightning sequential arcs — all approved with tuning notes
type: project
---

## Evaluation: Runtime Effects Implementation Changes (2026-03-29)

### Source Chip Threading — APPROVED
DamageCell now carries `source_chip: Option<String>` attribution from all combat effects via `EffectSourceChip` component. Closes prior gap. Enables build storytelling (P2, P7, P9). Zero performance concern at current scale.

### Shield Charge-Based Absorption — APPROVED WITH FLAG
- **Breaker**: Per-bolt charge cost with independent multi-bolt handling. Correct defensive model.
- **Cell**: Pure absorption (effectively extra HP). Works as baseline but lacks skill expression and synergy. Flagged for Phase 7 — needs secondary behavior (reflection, timer penalty, adjacency buff).

### Chain Lightning Sequential Arcs — APPROVED
Changed from instant batch to arc-by-arc chaining with `ChainState` state machine. Visual identity transformed from invisible to spectacular. Emergent timing skill added (trigger order matters in multi-effect builds).

**Tuning critical**: `arc_speed` must stay 600-1200 world units/sec. Full 8-arc chain must complete under 0.75 seconds. Slow arcs create dead air (violates litmus test 6).

### Dispatch Infrastructure — APPROVED (2026-03-29)
- Chip dispatch: recursive `On` target resolution, immediate `Do` firing, `When` to BoundEffects. Clean.
- Breaker dispatch: same pattern, fires at game start. Clean.
- Cell dispatch: marker-based (`CellEffectsDispatched`) prevents double-dispatch. Clean.
- Wall dispatch: structural no-op, correct placeholder for future wall effects.
- Maxed chips still dispatch: `add_chip` returns false but effects fire anyway. Correct defensive fallback — offering system is the real gatekeeper for max stacks. Warn log must remain.

### Chain Lightning 50-Arc Invariant — APPROVED (2026-03-29)
Scenario runner `ChainArcCountReasonable` with default `max_chain_arc_count: 50`. Single chain = ~10 arcs max, so 50 allows ~5 concurrent chains. Generous enough for power fantasy, strict enough to catch unbounded entity accumulation. Configurable per-scenario via `InvariantParams` RON.

### Open Items
- Cell Shield enrichment for Phase 7
- Chain Lightning `arc_speed` tuning bounds
- `BASE_BOLT_DAMAGE` hardcoding still present in Shockwave/Pulse
- Future: wall effects are interesting design space (speed boost on bounce, charge mechanics, destructible walls) — evaluate when proposed

**Why:** Tracks incremental evaluation decisions on runtime effects as they ship.
**How to apply:** Reference when tuning Chain Lightning speed, designing cell archetypes (Phase 7), evaluating future defensive or wall effects, or debugging maxed-chip dispatch behavior.

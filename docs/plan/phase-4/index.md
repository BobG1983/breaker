# Phase 4: Vertical Slice — Mini-Run

**Goal**: A playable multi-tier run that proves the architecture and feels like the game. Seeded determinism, functional chip effects with stacking and evolution, weighted offering system with pool depletion, procedural node escalation with boss nodes, animated transitions, and a full run summary.

## Stages

| Stage | Name | Depends On | Deliverable |
|-------|------|-----------|-------------|
| [4a](phase-4a-seeded-rng.md) | Seeded RNG & Run Seed | — | Deterministic run foundation, user-selectable seed |
| [4b](phase-4b-chip-effects.md) | Chip Effect System | — | Component markers with values, ChipSelected -> apply effects, hot-reload propagation |
| [4c](phase-4c-chip-pool.md) | Chip Pool & Rarity | 4b | 16-20 chips across rarity tiers, rarity affects stats + weights, stacking with per-chip caps |
| [4d](phase-4d-trigger-effect.md) | Trigger/Effect Architecture | 4b | Recursive RON-defined trigger chains, bolt behaviors domain, Surge overclock |
| [4e](phase-4e-node-escalation.md) | Node Sequence & Escalation | 4a | Procedural tier system, node types (Passive/Active/Boss), difficulty curve, timer scaling |
| [4f](phase-4f-chip-offerings.md) | Chip Offering System | 4a, 4c | Weighted random pool, Isaac-style weight decay, seeded determinism, no-duplicate-per-node |
| [4g](phase-4g-node-transitions.md) | Node Transitions & VFX | 4e | Animated transition states, multiple transition styles, random selection per seed |
| [4h](phase-4h-chip-evolution.md) | Chip Evolution | 4c, 4d, 4e | Boss rewards, evolution recipes, 3-4 evolved chips |
| [4i](phase-4i-run-stats.md) | Run Stats & Summary | 4e, 4f | RunStats resource, message-driven accumulation, full run-end screen |

## Dependency Graph

```
4a (Seeded RNG) ──────┬──── 4e (Node Escalation) ──┬── 4g (Transitions)
                      │                             │
                      └──── 4f (Chip Offerings) ────┤
                                                    │
4b (Chip Effects) ──┬── 4c (Chip Pool) ─────────────┤── 4h (Evolution)
                    │                               │
                    └── 4d (Trigger/Effect) ────────┘
                                                    │
                                                    └── 4i (Run Stats)
```

Stages 4a and 4b have no dependencies and can start in parallel. The graph fans out from there.

## Build Order Rationale

**Seeded RNG (4a)** comes first because every subsequent system that involves randomness (node selection, chip offerings, transition selection) must be deterministic from day one. Retrofitting seeds is more expensive than building on them.

**Chip effects (4b)** runs parallel to 4a — it's the other foundation. No point offering chips if they don't do anything.

**Chip pool (4c)** needs the effect system to define what each chip actually does. 16-20 chips across Common/Uncommon/Rare/Legendary with flat stacking and per-chip caps.

**Trigger/effect architecture (4d)** is the most complex single system — recursive RON-defined trigger chains for overclocks. Depends on 4b for the effect application mechanism.

**Node escalation (4e)** needs seeded RNG. Builds the procedural tier system that drives the run structure.

**Chip offerings (4f)** needs both the chip pool (what to offer) and seeded RNG (deterministic selection).

**Node transitions (4g)** needs node escalation to have something to transition between. Multiple animated transition styles, randomly selected per seed.

**Chip evolution (4h)** needs the chip pool (ingredients), trigger/effect architecture (evolved overclocks), and node escalation (boss nodes as evolution triggers).

**Run stats (4i)** comes last — needs all gameplay systems working to measure them.

## Design Decisions

All design decisions for Phase 4 are documented in `../../design/decisions/`:
- [Chip Stacking](../design/decisions/chip-stacking.md)
- [Chip Evolution](../design/decisions/chip-evolution.md)
- [Chip Offering System](../design/decisions/chip-offering-system.md)
- [Chip Selection Timeout](../design/decisions/chip-timeout.md)
- [Node Escalation](../design/decisions/node-escalation.md)
- [Seeded Determinism](../design/decisions/seeded-determinism.md)
- [Chip Synergies](../design/decisions/chip-synergies.md)

# Phase 4e: Node Sequence & Escalation

**Goal**: Procedural node sequence from seed with tier-based difficulty escalation.

## Dependencies

- 4a (Seeded RNG) — node sequence must be deterministic

## What to Build

### Node Types

Three node types with distinct gameplay:

| Type | Characteristics |
|------|----------------|
| **Passive** | Classic breakout — static cells, no special mechanics |
| **Active** | Cells have behaviors (movement, shields, spawning, etc.) |
| **Boss** | Single tough entity or complex pattern, drops evolution rewards |

Each type has its own pool of RON layouts.

### Tier System

A run progresses through tiers. Each tier defines:
- Number of nodes before the boss
- Mix ratio of passive vs active nodes
- Cell HP multiplier
- Timer multiplier (applied after boss kill)
- Which cell types are introduced

Example progression:
- **Tier 1**: 5 passive nodes -> boss. Timer: 100%. Simple cells only.
- **Tier 2**: 5 nodes (mostly passive, 1-2 active) -> boss. Timer: 90%. Lock cells introduced.
- **Tier 3**: 5 nodes (mixed passive/active) -> boss. Timer: 80%. Tough cells introduced.
- **Tier N**: All active nodes -> boss. Timer: 60%. All cell types.

### Procedural Sequence Generation

Algorithm (seeded):
1. Read difficulty curve params from RON
2. For each tier: roll node count from a range (e.g., 4-6 instead of fixed 5) using seeded RNG
3. Select N layouts from the appropriate node-type pools (seeded)
4. Roll for rare structural events (see below)
5. Append a boss layout
6. Apply scaling multipliers (HP, timer)
7. Advance to next tier

### Structural Variance

The tier cadence must not be perfectly predictable. Seeded RNG introduces variance:

- **Variable node count per tier**: 4-6 nodes instead of fixed 5. Some tiers are short sprints, others are marathons.
- **Rare events** (low probability, seeded): early boss (boss appears after 2-3 nodes instead of full tier), bonus chip node (extra chip offering mid-tier), double boss (two boss nodes back-to-back at tier end).
- **Tier composition jitter**: the `active_ratio` in RON defines a target, but actual selection has variance (a 0.2 active ratio might produce 0 or 2 active nodes in a 5-node tier, not always exactly 1).

These deviations create stories: "I got a boss on node 3 and barely survived" or "seed 42 has a double boss in tier 2 — good luck."

### Difficulty Curve RON

```ron
// assets/config/difficulty.ron
(
    tiers: [
        (nodes: 5, active_ratio: 0.0, hp_mult: 1.0, timer_mult: 1.0),
        (nodes: 5, active_ratio: 0.2, hp_mult: 1.3, timer_mult: 0.9),
        (nodes: 5, active_ratio: 0.4, hp_mult: 1.6, timer_mult: 0.8),
        (nodes: 5, active_ratio: 0.6, hp_mult: 2.0, timer_mult: 0.7),
        (nodes: 5, active_ratio: 1.0, hp_mult: 2.5, timer_mult: 0.6),
    ],
    boss_hp_mult: 3.0,
    timer_reduction_per_boss: 0.1,
)
```

### Layout Pools

- `assets/nodes/passive/` — passive node layouts
- `assets/nodes/active/` — active node layouts
- `assets/nodes/boss/` — boss node layouts

Existing layouts (`corridor.node.ron`, `fortress.node.ron`, `scatter.node.ron`) move to `passive/`.

### Cell Types for the Vertical Slice

Without mechanically distinct cell types, every node plays identically (break cells, don't lose bolt). The vertical slice must include at least 2-3 cell types beyond basic cells:

| Cell Type | Mechanic | Introduced |
|-----------|----------|-----------|
| **Basic** | Takes N hits to destroy | Tier 1 |
| **Tough** | Higher HP, may require multiple bounces | Tier 1-2 |
| **Lock** | Cannot be damaged until adjacent cells are cleared | Tier 2 |
| **Regen** | Regenerates 1 HP per N seconds if not destroyed | Tier 3+ |

These create meaningfully different situations per node — a Lock Cell layout requires targeting order strategy, Regen cells create time pressure within the time pressure. Additional cell types (Portal, Twin, Gate) are Phase 7+ content.

## Acceptance Criteria

1. Same seed produces identical node sequence
2. Tier progression works: passive nodes -> boss -> tougher nodes -> boss
3. Timer gets shorter after each boss kill
4. Cell HP scales with tier
5. Active/passive ratio shifts as described
6. Difficulty curve is fully RON-configurable and hot-reloadable
7. Node count per tier varies (not always exactly 5)
8. At least 3 mechanically distinct cell types exist (Basic, Tough, Lock minimum)
9. New cell types are introduced at tier thresholds as defined in difficulty RON

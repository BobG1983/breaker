# Phase 4e: Node Sequence & Escalation

**Goal**: Procedural node sequence from seed with tier-based difficulty escalation.

**Wave**: 2 (after 4a) — parallel with 4c and 4d

## Dependencies

- 4a (Seeded RNG) — node sequence must be deterministic

## Sub-Stages

This is the largest single stage in Phase 4. It spans data structures, a generation algorithm, new cell types, and asset reorganization. Split into four sub-stages across two sessions.

### 4e.1: Tier Data Structures + Difficulty Curve RON (Session 3)

**Domain**: run/

Define the data model for tier-based escalation:

- `TierDefinition` struct: node count range, active_ratio, hp_mult, timer_mult, introduced cell types
- `DifficultyCurve` resource loaded from RON
- `NodeType` enum: Passive, Active, Boss

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

**Delegatable**: Yes — writer-tests → writer-code, scoped to run/ domain. Pure data types + RON parsing.

### 4e.2: Procedural Sequence Generation (Session 3)

**Domain**: run/

The algorithm that builds a node sequence from a seed:

1. Read difficulty curve params from RON
2. For each tier: roll node count from range (e.g., 4-6) using seeded RNG
3. Select N layouts from the appropriate node-type pools (seeded)
4. Roll for rare structural events
5. Append a boss layout
6. Apply scaling multipliers (HP, timer)
7. Advance to next tier

**Structural variance** (creates stories):
- **Variable node count per tier**: 4-6 nodes instead of fixed 5
- **Rare events** (low probability, seeded): early boss, bonus chip node, double boss
- **Tier composition jitter**: active_ratio defines a target with variance

**Delegatable**: Yes — pure logic with tests. Can run in parallel with 4e.1 if types are defined first, or as a follow-up in the same session.

### 4e.3: New Cell Types (Session 4)

**Domain**: cells/

Without mechanically distinct cell types, every node plays identically. The vertical slice needs at least 2 beyond Basic/Tough:

| Cell Type | Mechanic | Introduced |
|-----------|----------|-----------|
| **Basic** | Takes N hits to destroy | Tier 1 (exists) |
| **Tough** | Higher HP | Tier 1-2 (exists) |
| **Lock** | Cannot be damaged until adjacent cells are cleared | Tier 2 |
| **Regen** | Regenerates 1 HP per N seconds if not destroyed | Tier 3+ |

Lock and Regen are independent of each other — **can parallelize** as two writer-tests → writer-code pairs within cells/ domain (separate system files).

**Delegatable**: Yes — one pair per cell type.

### 4e.4: Layout Pool Reorganization (Session 4)

**Domain**: assets/, run/

Reorganize node layouts into type-based pools:
- `assets/nodes/passive/` — passive node layouts
- `assets/nodes/active/` — active node layouts
- `assets/nodes/boss/` — boss node layouts

Existing layouts (`corridor.node.ron`, `fortress.node.ron`, `scatter.node.ron`) move to `passive/`.

Update the layout loader to discover layouts by pool directory. Add 1-2 layouts per new pool (active, boss) for the vertical slice.

**Delegatable**: Partially — asset moves are manual, loader updates can be delegated.

## Node Types

Three node types with distinct gameplay:

| Type | Characteristics |
|------|----------------|
| **Passive** | Classic breakout — static cells, no special mechanics |
| **Active** | Cells have behaviors (movement, shields, spawning, etc.) |
| **Boss** | Single tough entity or complex pattern, drops evolution rewards |

Each type has its own pool of RON layouts.

## Scenario Coverage

### New Invariants
- **`CellHealthNonNegative`** — cell HP never goes below 0 (especially important with Regen cells that modify HP). Checked every frame.
- **`LockCellDamageBlocked`** — Lock cells with active locks never take damage. Ensures the lock mechanic is correctly enforced under chaos input.
- **`TierProgressionMonotonic`** — current tier index never decreases during a run. Sanity check on the node sequence system.

### New Scenarios
- `mechanic/lock_cell_targeting.scenario.ron` — Layout with Lock cells surrounded by standard cells. Scripted input to clear adjacents first, then chaos. Verifies `LockCellDamageBlocked` during locked phase, then cells become damageable.
- `mechanic/regen_cell_timer.scenario.ron` — Layout with Regen cells. Chaos input over extended frames. Verifies `CellHealthNonNegative`, cells regenerate as expected, and node can still be cleared.
- `stress/multinode_escalation.scenario.ron` — Long chaos run (2000+ frames) through multiple tiers. Verifies `NoEntityLeaks` across node transitions, `TimerNonNegative` with timer scaling, `ValidStateTransitions` across tier boundaries.
- `mechanic/boss_node_clear.scenario.ron` — Scripted input to clear a boss node. Verifies boss HP scaling, evolution reward trigger, and state transition to ChipSelect/EvolutionReward.

### Layout Pool Scenarios
- At least 1 scenario per node pool (passive, active, boss) to verify layouts parse and play correctly.

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

# Node Types

How the three node pools differ architecturally. For the design rationale (timer philosophy, dismissed alternatives, pillar alignment), see `docs/design/decisions/node-type-differentiation.md`.

## NodePool

```rust
// state/run/node/definition/types.rs
pub enum NodePool {
    Passive,  // non-combat or early-game nodes
    Active,   // higher-difficulty with active cells
    Boss,     // end-of-tier encounters
}
```

Each `NodeLayout` (loaded from `.node.ron`) declares its pool. The node sequence generator assigns layouts from the matching pool — passive nodes never accidentally contain active cells, and boss encounters only appear at tier boundaries.

## TierDefinition

```rust
// state/run/definition/types.rs
pub struct TierDefinition {
    pub nodes:            TierNodeCount,   // how many nodes this tier contains
    pub active_ratio:     f32,             // fraction of nodes that are active (0.0–1.0)
    pub timer_mult:       f32,             // multiplier on node timer durations
    pub introduced_cells: Vec<char>,       // cell-type aliases introduced in this tier
}
```

`TierDefinition` is loaded from `defaults.difficulty.ron` as part of the `DifficultyCurve` config. It controls per-tier knobs:

- **`active_ratio`** — drives the proportion of Active-pool nodes in the tier's sequence. Higher tiers have a larger ratio.
- **`timer_mult`** — tightens timers as tiers progress. Applied as a multiplier on the layout's `timer_secs`.
- **`introduced_cells`** — cell-type aliases that become available in this tier. Earlier tiers see only basic types; later tiers introduce complex behaviors (locked, regen, guarded, etc.).

## Toughness

```rust
// cells/definition/data.rs
pub(crate) enum Toughness {
    Weak,       // 10 HP fallback
    Standard,   // 20 HP fallback (default)
    Tough,      // 30 HP fallback
}
```

Each `CellTypeDefinition` declares a `Toughness` level. HP is computed from `ToughnessConfig` (loaded from `defaults.toughness.ron`). The fallback values above are only used in tests without the config resource.

`ToughnessConfig` is the RON-tunable HP table — it replaces the old `TierDefinition.hp_mult` field. All HP scaling parameters live in one place rather than scattered across tier and cell definitions.

## HP Scaling Pipeline

```
CellTypeDefinition.toughness (Weak | Standard | Tough)
    + ToughnessConfig (RON — base HP per toughness level, boss multiplier)
    + node context (is this a boss node?)
    → computed HP for the cell entity at spawn time
```

The cell builder reads `Toughness` from the definition and `ToughnessConfig` from the resource, applies the boss multiplier if the current node is a boss, and inserts the result as the cell's `Hp` component.

## Complexity Scaling

As tiers progress:

| Knob | What changes | Controlled by |
|---|---|---|
| Grid size | More cells per layout | Per-layout RON |
| Cell variety | New cell types appear | `TierDefinition.introduced_cells` |
| Active ratio | More active-pool nodes | `TierDefinition.active_ratio` |
| Timer pressure | Shorter timers | `TierDefinition.timer_mult` |
| Entity scale | Smaller breaker/bolt | Per-layout `entity_scale` → `NodeScalingFactor` (see `scaling.md`) |

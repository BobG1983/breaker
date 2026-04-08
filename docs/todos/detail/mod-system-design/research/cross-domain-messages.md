# Cross-Domain Messages for Protocols & Hazards

New messages needed by the protocol and hazard systems. Each is owned by the CONSUMING domain (command message pattern).

---

## `HealCell` — owned by `cells`

```rust
// cells/messages.rs
#[derive(Message, Clone, Debug)]
pub(crate) struct HealCell {
    /// The cell entity to heal.
    pub cell: Entity,
    /// Amount of HP to restore.
    pub amount: f32,
}
```

**Sent by**: Cascade, Renewal, Volatility, Sympathy, Momentum hazard systems
**Consumed by**: A new `apply_cell_healing` system in the cells domain that adds HP up to max.

---

## `SpawnGhostCell` — owned by `cells`

```rust
// cells/messages.rs
#[derive(Message, Clone, Debug)]
pub(crate) struct SpawnGhostCell {
    /// World position to spawn the ghost cell.
    pub position: Vec2,
    /// HP of the ghost cell (scales with Echo Cells stack level).
    pub hp: f32,
    /// Grid position for adjacency queries.
    pub grid_col: u32,
    pub grid_row: u32,
}
```

**Sent by**: Echo Cells hazard system (on cell death, after delay)
**Consumed by**: A new `spawn_ghost_cells` system in the cells domain. Ghost cells are standard cells with no special behaviors — they don't carry the original cell's modifiers (no locks, no regen, no shields). They ARE `RequiredToClear`.

---

## `ApplyBoltForce` — owned by `bolt`

```rust
// bolt/messages.rs
#[derive(Message, Clone, Debug)]
pub(crate) struct ApplyBoltForce {
    /// The bolt entity to apply force to. If None, applies to all active bolts.
    pub bolt: Option<Entity>,
    /// Force vector in world units per second.
    pub force: Vec2,
}
```

**Sent by**: Drift hazard (constant directional wind), Gravity Surge hazard (pull toward gravity well position)
**Consumed by**: A new `apply_external_forces` system in the bolt domain that accumulates forces and applies them to bolt velocity in `FixedUpdate`.

---

## `ApplyBreakerShrink` — owned by `breaker`

```rust
// breaker/messages.rs
#[derive(Message, Clone, Debug)]
pub(crate) struct ApplyBreakerShrink {
    /// Amount to shrink the breaker width by (in world units).
    pub amount: f32,
}
```

**Sent by**: Erosion hazard (per-tick shrinkage)
**Consumed by**: A new `apply_breaker_shrink` system in the breaker domain. Respects `min_w` from `BreakerDefinition`. Also scales bump window height proportionally.

---

## `ApplyBreakerRestore` — owned by `breaker`

```rust
// breaker/messages.rs
#[derive(Message, Clone, Debug)]
pub(crate) struct ApplyBreakerRestore {
    /// Fraction of lost width to restore (0.0 to 1.0).
    pub restore_fraction: f32,
}
```

**Sent by**: Erosion hazard (on non-whiff bump: 25%, on perfect bump: 50%)
**Consumed by**: Same `apply_breaker_shrink` system (or a paired `apply_breaker_restore`).

---

## Existing messages reused by hazards

| Message | Existing in | Used by hazard |
|---------|-------------|----------------|
| `ApplyTimePenalty { seconds }` | `state/run/node/messages.rs` | Decay (positive seconds = drain faster) |
| `BumpPerformed { grade, bolt, breaker }` | `breaker/messages.rs` | Erosion (reads grade for restore amount) |
| `DamageDealt<Cell>` (after effect refactor) | `new_effect/damage/` | Resonance sends for wave-hit cells |

---

## Messages NOT needed (cell damage pipeline integration)

These hazards do NOT send their own messages. Instead, the cell damage system (`apply_damage::<Cell>`) reads their config resources and handles the behavior internally:

| Hazard | Why no message | Cell damage system reads |
|--------|---------------|------------------------|
| Diffusion | Redistributes incoming damage to adjacents | `Res<DiffusionConfig>` + `Res<ActiveHazards>` |
| Tether | Shares damage between linked pairs | `Res<TetherConfig>` + `Query<&TetherLink>` |
| Momentum | Non-lethal hits add HP, split at 2x | `Res<MomentumConfig>` + sends `HealCell` |
| Sympathy | Damage dealt heals adjacents | `Res<SympathyConfig>` + sends `HealCell` |

The hazard domain provides the config resources. The cells domain reads them. This avoids message interception and keeps the damage pipeline in one place.

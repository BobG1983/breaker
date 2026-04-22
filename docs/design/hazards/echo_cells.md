# Hazard: Echo Cells

## Game Design

Destroyed cells leave a ghost after 1.5 seconds at the victim's world position. Ghosts don't carry original cell rules (no unlocking, no special behaviors). 1 HP base, **doubling per level** (1/2/4/8...). Cleanup tax — the player must revisit cleared areas. The exponential HP curve is the steepest of any hazard: "looks easy at stack 1, terrifying at stack 3+".

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct EchoCellsConfig {
    pub respawn_delay: f32,         // 1.5
    pub base_hp: f32,               // 1.0
    pub hp_doubles_per_level: bool, // true
}
```

Populated from `HazardTuning::EchoCells`.

## Components

### `PendingGhost`

```rust
#[derive(Component, Debug)]
pub(crate) struct PendingGhost {
    pub world_position: Vec2,
    pub timer: f32,
}
```

Marker entity tracking a death site (world position snapshot) until the respawn timer expires.

### `GhostCell`

```rust
#[derive(Component, Debug, Default)]
pub(crate) struct GhostCell;
```

Marker on spawned ghost cells. Used to filter ghost-of-ghost recursion.

## Messages
**Reads**: `CellDestroyed { entity, position }` (cells-domain companion with pre-despawn world position), filtered to exclude `GhostCell` entities.
**Sends**: None as messages. Uses `Cell::builder().ghost(world_position, hp).spawn()` via `Commands`.

## Systems

### `echo_cells_track_deaths`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`.
- **run_if**: `hazard_active(HazardKind::EchoCells)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `CellDestroyed`. For each victim that was NOT a `GhostCell`, spawns a `PendingGhost` entity with the victim's world position + `timer = respawn_delay`. Ghost-of-ghost filtering requires looking up the victim's `GhostCell` component before despawn — cache this in a side `Query<Entity, With<GhostCell>>` snapshot earlier in the tick, or use a boolean flag on the `CellDestroyed` message (cells-domain choice).

### `echo_cells_spawn_ghosts`
- **Schedule**: `FixedUpdate`, `.after(echo_cells_track_deaths)`.
- **run_if**: `hazard_active(HazardKind::EchoCells)` + `in_state(NodeState::Playing)`.
- **Behavior**: Ticks all `PendingGhost.timer -= delta_secs`. When `timer <= 0`:
  1. Compute `hp = base_hp * 2^(stack - 1)` if `hp_doubles_per_level`, else `base_hp * stack`.
  2. Spawn ghost via `Cell::builder().ghost(world_position, hp).spawn()`.
  3. Despawn the `PendingGhost` marker.

## Pipeline position (dmg crate)

- **Not in the death pipeline as a producer.** Echo Cells reacts to `CellDestroyed` (derived from `rantzsoft_dmg::Destroyed<Cell>`) but runs `.after(DeathPipelineSystems::ApplyKill)` — outside the damage/heal chain.
- **Trigger**: `CellDestroyed` from the cells domain (companion to `rantzsoft_dmg::Destroyed<Cell>`).
- **Emits**: no dmg-crate messages. Spawns new cell entities via builder.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` / `VulnerableStack` involvement.

## Stacking Behavior

Exponential: `ghost_hp = base_hp * 2^(stack - 1)` (uses `HpScaling::Doubles` shape post-TODO #9).

| Stack | Ghost HP | Notes |
|-------|----------|-------|
| 1 | 1 | Trivial — one hit |
| 2 | 2 | Two hits per ghost |
| 3 | 4 | Real obstacles |
| 5 | 16 | Tankier than some originals |
| 8 | 128 | Boss-tier debris |

`respawn_delay` is stack-independent — ghosts always appear 1.5s after death.

## Cross-Domain Dependencies
- **cells**: `Cell::builder().ghost(world_position, hp).spawn()`. Reads `CellDestroyed`.
- **damage crate (`rantzsoft_dmg`)**: Upstream `Destroyed<Cell>` drives `CellDestroyed` emission.

## Expected Behaviors (for test specs)

1. **Ghost spawns after delay at stack 1** — `respawn_delay: 1.5`, cell destroyed at world `(100, 200)`, 1.5s elapsed: ghost spawned at `(100, 200)` with HP 1.
2. **Ghost HP doubles at stack 3** — stack 3, `base_hp: 1.0`, `hp_doubles_per_level: true`: ghost spawned with HP 4.
3. **Ghosts of ghosts are NOT spawned** — `GhostCell` entity destroyed: no `PendingGhost` created.
4. **Multiple deaths same frame spawn multiple pending markers** — 3 cells destroyed: 3 `PendingGhost` entities with independent timers.
5. **Ghost not spawned before delay expires** — `timer: 1.5`, 1.0s elapsed: `timer = 0.5`, no spawn yet.
6. **Works for animated boss cells** — a boss cell destroyed at `(250.4, 93.1)` while moving: ghost spawns at that snapshotted position, not snapped to a grid.

## Edge Cases
- **Echo Cells + Volatility**: uncleared ghost grows via Volatility's timer; 2-HP ghost becomes 3-HP, etc.
- **Echo Cells + Resonance**: clearing ghosts counts as kills — can trigger Resonance waves.
- **Echo Cells + Fracture**: both spawn new cells. Fracture reacts same tick, Echo Cells delays 1.5s — no same-frame conflicts.
- **Position overlap**: ghost may spawn on top of another cell (e.g., a debris spawned later). The cells domain handles the overlap (collision-based; either allow or nudge — domain's choice, not Echo Cells' concern).
- **Cleanup**: `EchoCellsConfig` removed at run end. `PendingGhost` markers despawned via cleanup markers.
- **Fixed respawn delay**: keeps the mechanic predictable — only HP scales.

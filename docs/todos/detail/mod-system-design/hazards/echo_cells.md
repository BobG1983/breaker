# Hazard: Echo Cells

## Game Design

Destroyed cells leave a ghost after 1.5 seconds that must be cleared. Ghosts don't carry original cell rules (no re-unlocking, no special behaviors). They have 1 HP, doubling per level (1/2/4/8...). This creates a cleanup tax — the player must revisit cleared areas to deal with ghost debris. At higher stacks, ghosts become increasingly tanky, turning what was "free cleanup" into real work.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct EchoCellsConfig {
    /// Delay in seconds before a ghost cell spawns after the original is destroyed.
    pub respawn_delay: f32,
    /// Base HP for ghost cells at stack 1.
    pub base_hp: f32,
    /// Whether HP doubles per stack level (true = exponential: 1, 2, 4, 8...).
    pub hp_doubles_per_level: bool,
}
```

Extracted from `HazardTuning::EchoCells { respawn_delay, base_hp, hp_doubles_per_level }` at activation time.

## Components

### `PendingGhost`

```rust
/// Tracks a destroyed cell position waiting to spawn a ghost.
#[derive(Component, Debug)]
pub(crate) struct PendingGhost {
    /// Grid position where the ghost will spawn.
    pub position: Vec2,
    /// Countdown timer until ghost spawns.
    pub timer: f32,
}
```

Spawned as a marker entity when a cell is destroyed. After the timer expires, the system sends `SpawnGhostCell` and despawns the marker.

### `GhostCell` (marker component)

```rust
/// Marks a cell as a ghost spawned by the Echo Cells hazard.
/// Ghost cells do not carry original cell rules (no lock behaviors, no special types).
#[derive(Component, Debug, Default)]
pub(crate) struct GhostCell;
```

Attached to ghost cell entities by the cells domain when it processes `SpawnGhostCell`. Used by other systems to distinguish ghosts from regular cells (e.g., Resonance checks, Echo Cells itself should NOT spawn ghosts of ghosts).

## Messages

**Reads**: Cell death message — `DamageDealt<Cell>` result or a dedicated `CellDestroyed { cell: Entity, position: Vec2 }` message. (The exact message name depends on the effect refactor — todo #2. `DamageDealt<Cell>` is a placeholder; the actual name may be `DamageDealt<Cell>` or similar.)
**Sends**: `SpawnGhostCell { position: Vec2, hp: f32 }` — new message, owned by `cells` domain.

## Systems

### `echo_cells_track_deaths`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::EchoCells).and(in_state(NodeState::Playing))`
- **Ordering**: After cell death processing (after the system that sends cell destroyed messages)
- **Behavior**: Reads cell death messages. For each destroyed cell that is NOT a `GhostCell`, spawns a `PendingGhost` entity with the cell's grid position and `respawn_delay` as the timer.

### `echo_cells_spawn_ghosts`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::EchoCells).and(in_state(NodeState::Playing))`
- **Ordering**: After `echo_cells_track_deaths`
- **Behavior**: Tick all `PendingGhost` timers. When a timer expires:
  1. Compute ghost HP: if `hp_doubles_per_level`, HP = `base_hp * 2^(stack-1)`; otherwise HP = `base_hp * stack`.
  2. Send `SpawnGhostCell { position, hp }`.
  3. Despawn the `PendingGhost` marker entity.

## Stacking Behavior

Exponential HP scaling (doubles per stack): `ghost_hp = base_hp * 2^(stack - 1)`

| Stack | Ghost HP | Notes |
|-------|----------|-------|
| 1 | 1 | Trivial cleanup — one hit |
| 2 | 2 | Two hits per ghost |
| 3 | 4 | Four hits — ghosts become real obstacles |

At stack 5, ghosts have 16 HP. At stack 8, ghosts have 128 HP — tankier than many original cells. The exponential curve is the steepest of all hazards, making this a "looks easy at stack 1, terrifying at stack 3+" trap.

The `respawn_delay` does NOT change with stacking. Ghosts always appear 1.5s after the original cell dies.

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `cells` | reads from | Cell death message (cell destroyed notification) |
| `cells` | sends to | `SpawnGhostCell` — requests ghost cell creation at position |

The cells domain owns cell spawning. Echo Cells sends a request; the cells domain creates the entity with `GhostCell` marker, the specified HP, and no special cell type behaviors.

## Expected Behaviors (for test specs)

1. **Ghost spawns after delay at stack 1**
   - Given: Echo Cells active at stack 1, `respawn_delay=1.5`, `base_hp=1.0`, cell destroyed at position `(100.0, 200.0)`
   - When: 1.5 seconds elapse
   - Then: `SpawnGhostCell { position: Vec2(100.0, 200.0), hp: 1.0 }` is sent

2. **Ghost HP doubles at stack 3**
   - Given: Echo Cells active at stack 3, `base_hp=1.0`, `hp_doubles_per_level=true`, cell destroyed
   - When: Ghost spawn timer expires
   - Then: `SpawnGhostCell { position, hp: 4.0 }` is sent (1.0 * 2^2)

3. **Ghosts of ghosts are NOT spawned**
   - Given: Echo Cells active, a `GhostCell` entity is destroyed
   - When: `echo_cells_track_deaths` processes the death
   - Then: No `PendingGhost` is created (ghost deaths are filtered out)

4. **Multiple cells destroyed spawn multiple ghosts**
   - Given: Echo Cells active, 3 cells destroyed in same frame at different positions
   - When: `echo_cells_track_deaths` runs
   - Then: 3 `PendingGhost` entities created with independent timers

5. **Ghost not spawned before delay expires**
   - Given: `PendingGhost` with `timer=1.5`, 1.0 seconds elapsed
   - When: `echo_cells_spawn_ghosts` runs
   - Then: Timer is 0.5, no `SpawnGhostCell` sent yet

## Edge Cases

- **Echo Cells + Volatility synergy**: Ghosts at stack 1 have 1 HP but grow via Volatility if not cleared immediately. A ghost that goes untouched for 5 seconds gains 1 HP (from Volatility's `hp_per_interval`), making it a 2-HP ghost. At higher Echo Cells stacks, the base HP is already high, and Volatility growth compounds.
- **Echo Cells + Resonance synergy**: Clearing ghost cells counts as kills and can trigger Resonance slow-waves. A cluster of ghosts cleared quickly generates a wave.
- **Node-end cleanup**: `EchoCellsConfig` resource removed at run end. All `PendingGhost` entities despawned via standard entity cleanup. Ghost cells are regular entities and despawn with the node.
- **Position occupied**: If the ghost's target position is already occupied by another cell (e.g., from Fracture spawns), the cells domain decides behavior — likely skip the spawn or find the nearest empty position. This is a cells-domain concern, not Echo Cells' responsibility.
- **Respawn delay is fixed**: The 1.5s delay does not scale with stacks. This is intentional — the player always knows when ghosts will appear. Only the ghost HP increases.

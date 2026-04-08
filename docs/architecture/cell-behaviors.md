# Cell Behaviors

How to add a new cell behavior to the game.

## Architecture

Each behavior is a self-contained package under `cells/behaviors/<name>/`:

```
cells/behaviors/
  mod.rs              — module declarations
  locked/
    mod.rs
    components.rs     — LockCell, Locked, Locks(Vec<Entity>), Unlocked
    systems/
      check_lock_release/
  regen/
    mod.rs
    components.rs     — RegenCell, Regen, RegenRate(f32), NoRegen
    systems/
      tick_cell_regen.rs
  guarded/
    mod.rs
    components.rs     — GuardedCell, GuardianCell, GuardianSlot, SlideTarget, GuardianSlideSpeed, GuardianGridStep
    systems/
      slide_guardian_cells.rs
```

## Component Marker Pattern

Every behavior uses a capability/state/data split:

| Role | Example (Regen) | Example (Locked) | Example (Guarded) |
|------|----------------|-------------------|-------------------|
| **Capability** (permanent) | `RegenCell` | `LockCell` | `GuardedCell` / `GuardianCell` |
| **State** (current) | `Regen` | `Locked` | `GuardianSlot(u8)` |
| **Data** | `RegenRate(f32)` | `Locks(Vec<Entity>)` | `GuardianSlideSpeed(f32)`, `GuardianGridStep` |
| **Future state** | `NoRegen` | `Unlocked` | `SlideTarget(u8)` |

Queries filter on capability + state: `(With<RegenCell>, With<Regen>, Without<NoRegen>)`.

## Adding a New Behavior

### 1. Define the variant

Add to `CellBehavior` enum in `cells/definition.rs`:

```rust
enum CellBehavior {
    Regen { rate: f32 },
    Guarded(GuardedBehavior),
    NewBehavior { param: f32 },  // <- add variant
}
```

Add validation in the `validate()` match arm.

### 2. Create the behavior folder

```
cells/behaviors/new_behavior/
  mod.rs
  components.rs    — marker + state + data components
  systems/
    mod.rs
    new_behavior_system.rs  — system + inline tests
```

### 3. Wire components

In `cells/behaviors/mod.rs`: add `pub(crate) mod new_behavior;`

In `cells/components/mod.rs`: re-export components needed cross-domain.

### 4. Handle in builder

In `cells/builder/core/terminal.rs`, add a match arm in `spawn_inner()`:

```rust
CellBehavior::NewBehavior { param } => {
    entity.insert((NewBehaviorCell, NewBehaviorData(param)));
}
```

### 5. Register the system

In `cells/plugin.rs`, add the system to `FixedUpdate` with appropriate ordering and `run_if(in_state(NodeState::Playing))`.

### 6. Update RON

Create `assets/cells/new_behavior.cell.ron` with the behavior in the `behaviors` list.

## Guard Cells (3x3 Grid Model)

Guard cells are a special case — the parent (`Gu`) spawns child entities (`gu`) via the builder's `.guarded()` method rather than inline component insertion.

### Grid Encoding

```
gu  .  gu
gu  Gu gu
.   gu gu
```

- `Gu` = guarded parent (center of 3x3)
- `gu` = guardian child position (ring slot)
- `.` = gap (no guardian)

### Ring Positions (clockwise from top-left)

```
0  1  2
7  X  3
6  5  4
```

### Guardian Properties

- Square dimensions: `cell_height x cell_height`
- Slide between adjacent ring positions at `slide_speed` units/sec
- Auto-despawn when parent dies (via `ChildOf`)
- NOT part of the lock system — parent is fully damageable

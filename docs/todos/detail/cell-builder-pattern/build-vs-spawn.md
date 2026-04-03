# Cell Builder — build() vs spawn()

## What build() does

Returns a component bundle (impl Bundle). Does NOT interact with the World.

1. Resolves all optional fields (override > definition > default)
2. Creates core components: `Cell`, `Spatial2D`, `Position2D`, `Scale2D`, `Aabb2D`, `CollisionLayers`, `GameDrawLayer::Cell`
3. Creates health: `CellHealth::new(hp)`
4. Creates dimension components: `CellWidth::new(width)`, `CellHeight::new(height)`
5. Creates visual components (if Rendered): `Mesh2d`, `MeshMaterial2d`
6. Creates damage visuals: `CellDamageVisuals` (if set)
7. Creates alias: `CellTypeAlias(char)` (if set)
8. Creates required_to_clear: `RequiredToClear` marker (if true)
9. Does NOT insert behavior components (Locked, CellRegen, ShieldParent, etc.)
10. Does NOT dispatch effects
11. Does NOT spawn orbit children

## What spawn() does

Spawns the entity into the World via Commands. Does EVERYTHING.

1. Calls `build()` to get the component bundle
2. Spawns the entity: `commands.spawn(bundle)`
3. Inserts behavior components based on resolved `Vec<CellBehavior>`:
   - `CellBehavior::Regen { rate }` → inserts `CellRegen { rate }`
   - `CellBehavior::Shielded(config)` → inserts `ShieldParent`, `Locked`, `LockAdjacents(orbit_ids)`, spawns orbit children as Bevy children
4. Dispatches effects via `commands.push_bound_effects(entity, entries)` (if effects are present)
5. Returns `Entity`

## Why spawn() must be the primary API

`build()` cannot:
- Dispatch effects (requires `Commands`)
- Spawn orbit children (requires `Commands`)
- Insert `LockAdjacents` with orbit entity IDs (requires spawning children first)
- Set up Bevy parent-child hierarchy (requires `Commands`)

If a caller uses `build()` and manually spawns, they skip effect dispatch and behavior setup. This is the exact bug currently in the wall builder — shield and second_wind effects use `.build()` and miss effect dispatch.

## Removing build() from public API — all builders

**Decision: remove `build()` from the public interface of all entity builders (Cell, Wall, Bolt, Breaker).**

`build()` becomes `pub(in crate::<domain>::builder)` — only used internally by `spawn()`. External callers must use `spawn()`, which guarantees:
- Effects are dispatched
- Behavior components are inserted
- Child entities are spawned
- Parent-child hierarchy is set up
- All post-spawn wiring is complete

### Migration for existing build() call sites

| File | Current | After |
|------|---------|-------|
| `effect/effects/shield/system.rs:48` | `Wall::builder()...build()` then manual spawn | `Wall::builder()...spawn()` — needs refactor since this is an exclusive World system, not a Commands system |
| `effect/effects/second_wind/system.rs:28` | `Wall::builder()...build()` then `world.spawn(bundle)` | `Wall::builder()...spawn()` — same exclusive World issue |
| `cells/systems/cell_wall_collision.rs` (tests) | `Wall::builder()...build()` then manual spawn | Tests are fine — walls in collision tests don't need effects |
| `breaker/systems/breaker_wall_collision.rs` (tests) | `Wall::builder()...build()` then manual spawn | Tests are fine — same reason |

### The exclusive World problem

Shield and second_wind effects run as exclusive systems with `&mut World`. They can't use `Commands` directly, but they can get a `Commands` from `World`:

```rust
let mut commands = world.commands();
let entity = Wall::builder().floor(&pf).timed(duration).spawn(&mut commands);
commands.entity(entity).insert((ShieldWall, ShieldWallTimer(...), ReflectionCost(...)));
world.flush_commands();
```

`spawn()` returns `Entity`. The caller appends domain-specific markers (`ShieldWall`, `SecondWindWall`, etc.) themselves. No `with_additional_components()` on the builder — the builder's job is wall completeness, not domain-specific extras.

### Test helpers

Test helpers that use `.build()` for headless entities without effects are fine — they're testing collision/physics behavior, not effect dispatch. But they should use `.spawn()` going forward for consistency. If a test truly needs a bare bundle without effects, it can construct components manually (not use the builder at all).

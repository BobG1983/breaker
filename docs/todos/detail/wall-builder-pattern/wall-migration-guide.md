# Wall Builder Migration Guide

Every wall construction site and how it maps to the builder.

## Builder API (flattened)

```rust
// Entry point
Wall::builder()

// Side dimension (required — exactly one)
.left(playfield: &PlayfieldConfig)
.right(playfield: &PlayfieldConfig)
.ceiling(playfield: &PlayfieldConfig)
.floor(playfield: &PlayfieldConfig)

// Visual dimension (optional — default Invisible)
.visible(meshes, materials)
.invisible()                          // explicit no-op, for clarity

// Lifetime (floor only — default Permanent)
.timed(duration: f32)                 // stamps TimedWall(f32) marker
.one_shot()                           // stamps OneShotWall marker + bakes in When(Impacted(Bolt), Do(Die))

// Definition shortcut (optional — sets defaults, specific beats definition)
.definition(def: &WallDefinition)

// Optional overrides (any order, specific always wins)
.at_position(pos: Vec2)              // override position computed by Side
.with_size(half_w: f32, half_h: f32) // override size computed by Side
.with_thickness(half_thickness: f32)
.with_effects(effects: Vec<RootEffect>)
.with_color(rgb: [f32; 3])

// Terminal methods (available when Side is set)
.build()                              -> impl Bundle
.spawn(commands: &mut Commands)       -> Entity  // dispatches effects via dispatch_initial_effects
```

**Internals:** Builder uses `Spatial::builder()` for Position2D, Scale2D, Aabb2D, Spatial2D, PreviousPosition, etc. — same as breaker/bolt builders. `WallSize` is deleted (empty struct, zero readers).

## New Types

### Effect domain
- `EffectKind::Die` — resolves target entity type, sends the appropriate `KillYourself<T>` message
- `KillYourself<Wall>(Entity)` — domain message, wall domain listens
- `KillYourself<Bolt>(Entity)` — domain message, bolt domain listens
- `KillYourself<Cell>(Entity)` — domain message (replaces/unifies existing cell death?)

### Wall domain
- `TimedWall(f32)` — component, countdown timer. Wall system ticks it, sends `KillYourself<Wall>` at 0
- `OneShotWall` — marker. Builder bakes in `When(Impacted(Bolt), Do(Die))` effect
- `WallDestroyed(Entity)` — message, broadcast when wall is despawned (for `Died` trigger on wall)
- `WallDestroyedAt(Entity, Vec2)` — message, broadcast with position (for spatial VFX)

### Death chain
```
[one-shot]  bolt hits wall → Impacted(Bolt) → Do(Die) → KillYourself<Wall>(entity)
[timed]     TimedWall(f32) ticks to 0 → KillYourself<Wall>(entity)
[both]      → wall domain system receives → plays death anim → despawns → WallDestroyed + WallDestroyedAt
```

The effect system listens for `WallDestroyed`/`WallDestroyedAt` to fire `Died`/`Death` triggers on wall targets. Same pattern as cells.

## Production Site Migrations

### 1. `spawn_walls` system (`wall/systems/spawn_walls/system.rs`)

**Current:** 3x `commands.spawn((Wall, WallSize{}, Position2D, Scale2D, Aabb2D, CollisionLayers, GameDrawLayer))`

**After:**
```rust
fn spawn_walls(
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    registry: Res<WallRegistry>,
    mut walls_spawned: MessageWriter<WallsSpawned>,
) {
    let def = registry.get("Wall").unwrap();

    Wall::builder().definition(def).left(&playfield).invisible().spawn(&mut commands);
    Wall::builder().definition(def).right(&playfield).invisible().spawn(&mut commands);
    Wall::builder().definition(def).ceiling(&playfield).invisible().spawn(&mut commands);

    walls_spawned.write(WallsSpawned);
}
```

### 2. `second_wind::fire` (`effect/effects/second_wind/system.rs`)

**Current:** `world.spawn((SecondWindWall, Wall, WallSize{}, Position2D, Scale2D, Aabb2D, CollisionLayers, CleanupOnNodeExit))`

**After:**
```rust
let entity = Wall::builder()
    .floor(&playfield)
    .one_shot()
    .invisible()
    .spawn(world);  // exclusive &mut World path, like bolt builder
world.entity_mut(entity).insert(SecondWindWall);
```

`SecondWindWall` stays as an extra marker for the "at most one" guard. `.one_shot()` handles the despawn-on-contact lifecycle via `Do(Die)`.

## Test Site Migrations

### 3. `bolt_wall_collision` helper (`bolt/systems/bolt_wall_collision/tests/helpers.rs`)

**Current:** `Wall, Aabb2D, CollisionLayers(WALL_LAYER, BOLT_LAYER), Position2D, GlobalPosition2D, Spatial2D, GameDrawLayer::Wall`

**After:**
```rust
fn spawn_wall(app: &mut App, x: f32, y: f32, half_w: f32, half_h: f32) -> Entity {
    let def = WallDefinition::default();
    app.world_mut().spawn(
        Wall::builder()
            .definition(&def)
            .left(&PlayfieldConfig::default())  // any side for defaults
            .at_position(Vec2::new(x, y))       // override position
            .with_size(half_w, half_h)           // override size
            .invisible()
            .build()
    ).id()
}
```

No post-spawn overrides needed — builder provides spatial components via `Spatial::builder()`. `GlobalPosition2D` is derived by spatial plugin; tests that need it insert post-spawn.

### 4. `bolt_cell_collision` helper (`bolt/systems/bolt_cell_collision/tests/helpers.rs`)

Same pattern as #3.

### 5. `cell_wall_collision` helper (`cells/systems/cell_wall_collision.rs`)

Same as #3, then post-spawn override for collision mask:
```rust
app.world_mut().entity_mut(entity).insert(CollisionLayers::new(WALL_LAYER, CELL_LAYER));
```

### 6. `breaker_wall_collision` helper (`breaker/systems/breaker_wall_collision.rs`)

Same as #3, then post-spawn override:
```rust
app.world_mut().entity_mut(entity).insert(CollisionLayers::new(WALL_LAYER, BREAKER_LAYER));
```

### 7. Effect dispatch test helpers (dispatch_wall_effects, dispatch_chip_effects, dispatch_bolt_effects, dispatch_cell_effects, resolve_all_targets, resolve_entity_targets, dispatch_initial_effects)

**Current:** bare `Wall` or `Wall + BoundEffects + StagedEffects`

**After:**
```rust
let def = WallDefinition::default();
let entity = app.world_mut().spawn(
    Wall::builder()
        .definition(&def)
        .left(&PlayfieldConfig::default())
        .invisible()
        .build()
).id();
app.world_mut().entity_mut(entity).insert((BoundEffects::default(), StagedEffects::default()));
```

### 8. `wall::components` tests (`wall/components.rs`)

**Current:** bare `Wall` for `#[require]` tests

**After:** Use builder — same as #7 without effect components.

### 9. Scenario runner — `entity_tagging` tests

**Current:** bare `Wall`

**After:** Same as #8.

### 10. Scenario runner — `SpawnExtraSecondWindWalls` mutation

**Current:** `commands.spawn(SecondWindWall)` — bare marker, no Wall

**Keep as-is.** This is a self-test that deliberately spawns malformed entities to test the invariant checker. Not a real wall construction site.

## Design Decisions Summary

| Decision | Choice |
|----------|--------|
| `WallSize` | Delete — empty struct, Aabb2D carries geometry |
| `SelectedWall` resource | No — read by name from registry |
| `GlobalPosition2D` in builder | No — derived by spatial plugin at runtime |
| `.one_shot()` mechanism | Bakes `When(Impacted(Bolt), Do(Die))` effect + `OneShotWall` marker |
| `.timed()` mechanism | Stamps `TimedWall(f32)` marker, wall system ticks + sends `KillYourself<Wall>` |
| Death chain | `Die` effect → `KillYourself<T>` message → domain cleanup → `WallDestroyed`/`WallDestroyedAt` |
| Collision mask in tests | Post-spawn `.insert()` override — test-specific, not builder's concern |
| `SecondWindWall` marker | Kept — added post-spawn for "at most one" guard |

# Bolt Birthing Animation

## Summary
All bolts go through a brief "birthing" animation before becoming active, preventing single-frame bolt explosions from cascade effects and giving visual feedback on spawn.

## Context
Split Decision + ArcWelder (chain mode) can cascade: cell death → 2 bolts → new beams → more cell deaths → more bolts → all in one frame. This creates a single-frame bolt explosion that's visually jarring and gameplay-breaking. Birthing adds a brief delay before new bolts become active, spreading the cascade over time.

Primary bolts also animate in — the serve bolt pops in with the same visual treatment for consistency. Birthing and serving overlap: the bolt scales up while sitting above the breaker. Player can't bump during birthing.

## Decisions

- **Duration**: 0.3 seconds
- **Lifespan paused during birthing**: lifespan timer does not tick until birthing completes
- **Applies to ALL bolts**: both Primary and Extra role bolts. No exceptions.
- **Serving bolt**: birthing and serving overlap — bolt scales up while hovering above breaker. Bump input is ignored during birthing.
- **Orphaned bolts**: if the spawning source is despawned mid-birth, the bolt continues birthing normally. No parent tracking.

## Implementation

### New component
```rust
#[derive(Component)]
pub(crate) struct Birthing {
    pub timer: Timer,
    pub target_scale: Scale2D,
    pub stashed_layers: Option<CollisionLayers>,
}
```
- Lives in `shared/components/` — entity-type agnostic
- `timer`: caller-determined duration (0.3s for bolts, could differ for cells)
- `target_scale`: the entity's real scale, stashed so the lerp knows what to aim for
- `stashed_layers`: `Some` if the entity had `CollisionLayers` that were removed, `None` for entities without collision (e.g., a UI element with a pop-in)

### New system
- `tick_birthing` in `shared/systems/` (or wherever shared systems live) — single generic system for all entity types
  - Schedule: `FixedUpdate`, runs BEFORE `ActiveFilter`-gated systems
  - Queries `(Entity, &mut Birthing, &mut Scale2D)`
  - Ticks the `Birthing` timer
  - Lerps `Scale2D` from zero to `birthing.target_scale` using `timer.fraction()`
  - When timer finishes: sets `Scale2D` to exact `target_scale` (no float drift), re-inserts `stashed_layers` if `Some`, removes `Birthing` component
  - No per-domain systems needed — one system handles bolts, cells, breakers, anything with `Birthing`

### Filter change (critical — single point of change)
Update `bolt/filters.rs`:
```rust
// BEFORE
pub type ActiveFilter = (With<Bolt>, Without<BoltServing>);

// AFTER — Birthing is the shared component, not bolt-specific
pub type ActiveFilter = (With<Bolt>, Without<BoltServing>, Without<Birthing>);
```

This automatically excludes birthing bolts from ALL systems using `ActiveFilter`:
- `bolt_cell_collision`
- `bolt_wall_collision`
- `bolt_breaker_collision`
- `clamp_bolt_speed`
- `clamp_bolt_to_playfield`
- `clamp_bolt_angle`
- Any other system using `ActiveFilter`

**DO NOT add `Without<Birthing>` to individual system queries** — the `ActiveFilter` change handles it.

### Additional query changes (NOT covered by ActiveFilter)
These systems use `With<Bolt>` directly, not `ActiveFilter`. Evaluate each:

| System | Current filter | Needs `Without<Birthing>`? | Reason |
|--------|---------------|-------------------------------|--------|
| `tick_bolt_lifespan` | `With<Bolt>` | **YES** — lifespan must not tick during birthing | Add `Without<Birthing>` |
| `hover_bolt` | `ServingFilter` | **NO** — hovering during birthing is correct (birthing overlaps serving) | Position stays synced to breaker |
| `launch_bolt` | `ServingFilter` | **YES** — must not launch during birthing | Add `Without<Birthing>` |
| `sync_bolt_scale` | `With<Bolt>` | **YES** — birthing system controls scale | Add `Without<Birthing>` |
| `dispatch_bolt_effects` | `Added<BoltDefinitionRef>` | **NO** — effects should bind immediately | Effects bind during birthing, just don't fire |
| `apply_node_scale_to_bolt` | `With<Bolt>` | **NO** — runs on OnEnter, not per-tick | One-time setup is fine |
| `detect_nail_biter` | `Without<BoltServing>` | **NO** — reads position for highlight detection, harmless | Birthing bolts at zero scale won't trigger |
| `detect_close_save` | `Without<BoltServing>` | **NO** — same reasoning | |
| `gravity_well` | `BoltNotWell` | Handled by **ActiveFilter** if it uses it, otherwise **YES** | Check implementation |
| `tether_beam` | `With<Bolt>` | **YES** — birthing bolts should not join tether chains | Add `Without<Birthing>` |
| Effect target resolution (`ext.rs`) | `With<Bolt>` / `With<PrimaryBolt>` | **NO** — effects should bind to birthing bolts | They bind but don't fire until triggers |
| `chip_dispatch` | `With<Bolt>` | **NO** — chip effects should bind to all bolts | Same reasoning |
| `spawn_bolts` effect | `Without<ExtraBolt>` | **NO** — reads primary bolt's BoundEffects for inheritance | |

### Bolt builder — NO changes
The builder does NOT insert `Birthing`. Birthing is a gameplay concern, not a construction concern. Tests spawn bolts without birthing and shouldn't have to deal with it.

Birthing is added by:
- **AnimateIn system**: `OnEnter(NodeState::AnimateIn)` queries entities, stashes/removes `CollisionLayers`, inserts `Birthing`, sets `Scale2D` to zero
- **Effect spawn sites**: the effect's `fire()` function calls `spawn()`, gets back the `Entity`, then inserts `Birthing` and removes `CollisionLayers` itself (stashing into the `Birthing` component)

### Quadtree — remove CollisionLayers during birthing
The bolt builder's `spawn()` inserts `CollisionLayers` as normal (entity is correct at spawn). A system running on `OnEnter(NodeState::AnimateIn)` removes `CollisionLayers` from all entities with `Birthing` — this hides them from the quadtree during the animation phase. When `tick_birthing` completes (timer done), it re-inserts the entity's `CollisionLayers` and removes the `Birthing` component.

This means:
- `spawn()` produces a complete, correct entity (all components present)
- During birthing, the entity is invisible to the physics world (no `CollisionLayers` → quadtree can't match it)
- After birthing, the entity enters the physics world cleanly
- No future system querying `BOLT_LAYER` can accidentally find a birthing bolt

The entity's `CollisionLayers` value must be stashed before removal and restored exactly — not re-inserted from a hardcoded default. An effect or behavior could have modified the entity's layers before AnimateIn.

```rust
#[derive(Component)]
pub(crate) struct Birthing {
    pub timer: Timer,
    pub stashed_layers: CollisionLayers,
}
```

Both paths stash the value:
- **AnimateIn path**: `OnEnter(NodeState::AnimateIn)` reads the entity's current `CollisionLayers`, stores it in `Birthing { stashed_layers }`, then removes `CollisionLayers` from the entity
- **Mid-gameplay spawn path**: `spawn()` inserts `CollisionLayers` as normal, then immediately removes it and stores it in `Birthing { stashed_layers }` (or: `spawn()` creates the `Birthing` with the layers value and skips inserting `CollisionLayers` on the entity)

When `tick_birthing` completes: re-insert `stashed_layers` as the entity's `CollisionLayers`. Exact value preserved regardless of what modified it before birthing.

### What NOT to change
- **Do NOT remove `Aabb2D`** from birthing entities — it stays, only `CollisionLayers` is temporarily removed
- **Do NOT modify the bolt builder's component assembly** (position, velocity, spatial, etc.) — everything is set at spawn, birthing is purely a "not yet active" gate
- **Do NOT add new components for the scale target** if it can be derived from existing data (`BoltRadius` + `NodeScalingFactor`)

### Effect spawn sites that need Birthing
These effect `fire()` functions call `Bolt::builder()...spawn()` and must add `Birthing` + remove `CollisionLayers` after spawn:

| File | Effect | Notes |
|------|--------|-------|
| `effect/effects/spawn_bolts/effect.rs` | SpawnBolts | Cascade risk — this is the primary motivator |
| `effect/effects/mirror_protocol/effect.rs` | MirrorProtocol | Mirror bolt |
| `effect/effects/spawn_phantom/effect.rs` | SpawnPhantom | Phantom bolt |
| `effect/effects/chain_bolt/effect.rs` | ChainBolt | Chain mode bolt |
| `effect/effects/tether_beam/effect.rs` | TetherBeam | Tether anchor bolt |
| `effect/effects/speed_boost.rs` | SpeedBoost | Test-only bolt spawns (2 occurrences) — these are in tests, do NOT add Birthing |

Pattern at each site:
```rust
let entity = Bolt::builder()...spawn(&mut commands);
begin_birthing(world, entity, 0.3);
```

Extract a helper (in `shared/` or as a `Commands` extension):
```rust
fn begin_birthing(world: &mut World, entity: Entity, duration: f32) {
    let layers = world.get::<CollisionLayers>(entity).copied();
    let target_scale = world.get::<Scale2D>(entity).copied().unwrap_or_default();
    if layers.is_some() {
        world.entity_mut(entity).remove::<CollisionLayers>();
    }
    if let Some(mut scale) = world.entity_mut(entity).get_mut::<Scale2D>() {
        scale.x = 0.0;
        scale.y = 0.0;
    }
    world.entity_mut(entity).insert(Birthing {
        timer: Timer::from_seconds(duration, TimerMode::Once),
        target_scale,
        stashed_layers: layers,
    });
}
```

Single helper, works for any entity type, stashes whatever it finds.

## Implementation Checklist

1. **Create `Birthing` component** in `shared/components/` — `timer: Timer`, `target_scale: Scale2D`, `stashed_layers: Option<CollisionLayers>`
2. **Create `begin_birthing` helper** in `shared/` — stashes layers + scale, zeros scale, inserts `Birthing`
3. **Create `tick_birthing` system** in `shared/systems/` — single generic system for all entity types. Ticks timer, lerps scale, restores layers, removes component on completion.
4. **Update `ActiveFilter`** in `bolt/filters.rs` — add `Without<Birthing>`
5. **Update individual queries** per the table above — `tick_bolt_lifespan`, `launch_bolt`, `sync_bolt_scale`, `tether_beam`
6. **Create `OnEnter(NodeState::AnimateIn)` system** — calls `begin_birthing` on all bolts (and later cells/etc.)
7. **Update effect spawn sites** (5 production files) — call `begin_birthing` after `spawn()`
8. **Do NOT touch**: bolt builder, `build()`, `Aabb2D`, tests that spawn bolts directly

## Scope
- In: `Birthing` component, `tick_bolt_birthing` system, `ActiveFilter` update, individual query updates per table above, builder `spawn()` changes, scale-up animation
- Out: Graphics refactor (future), sound effects, easing curves (linear lerp for now)

Touches: bolt, effect domains

### Integration with NodeState::AnimateIn
The state lifecycle refactor (#1) adds `NodeState::AnimateIn` — cells slam onto the map, level intro plays. An `OnEnter(NodeState::AnimateIn)` system:
1. Queries all entities with `CollisionLayers` (or a broader filter like `With<Bolt>`, `With<Cell>`)
2. Removes `CollisionLayers` (stores value for re-insertion)
3. Inserts `Birthing(Timer)` + sets `Scale2D` to zero

This handles both cases:
- **First node**: primary bolt was just spawned by `setup_run` — `spawn()` inserted `CollisionLayers`, AnimateIn removes it and adds `Birthing`
- **Subsequent nodes**: primary bolt already exists (persists via `CleanupOnExit<RunState>`) — AnimateIn removes its `CollisionLayers` and adds `Birthing`, giving it the same pop-in animation on every node entry

When `tick_bolt_birthing` completes the timer, it re-inserts `CollisionLayers` and removes `Birthing`. The bolt enters the physics world and is ready for play.

Effect-spawned bolts during gameplay birth inline — `spawn()` inserts `Birthing`, and `tick_bolt_birthing` handles the lifecycle. No AnimateIn involvement.

## Dependencies
- Depends on: State lifecycle refactor (#1) — NodeState::AnimateIn is the primary bolt's birthing window
- Blocks: Nothing directly, but improves cascade effect quality

## Status
`ready`

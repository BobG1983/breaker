# Wall Construction Sites — Complete Inventory

Research date: 2026-04-02

---

## 1. Production Spawn Sites

### 1a. `spawn_walls` system — the canonical 3-wall spawn
**File:** `breaker-game/src/wall/systems/spawn_walls/system.rs:29-68`
**Triggered by:** `WallPlugin` in `OnEnter(GameState::Playing)`, chained before `dispatch_wall_effects`

Three separate `commands.spawn(...)` calls — one per wall — each with identical component sets:

**Left wall (line 29-40):**
```
Wall,
WallSize {},
Position2D(Vec2::new(playfield.left() - wall_ht, 0.0)),
Scale2D { x: wall_ht, y: half_height },
Aabb2D::new(Vec2::ZERO, Vec2::new(wall_ht, half_height)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
GameDrawLayer::Wall,
```
`Wall` `#[require]` auto-inserts: `Spatial2D`, `CleanupOnNodeExit`

**Right wall (line 43-55):**
```
Wall,
WallSize {},
Position2D(Vec2::new(playfield.right() + wall_ht, 0.0)),
Scale2D { x: wall_ht, y: half_height },
Aabb2D::new(Vec2::ZERO, Vec2::new(wall_ht, half_height)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
GameDrawLayer::Wall,
```
`Wall` `#[require]` auto-inserts: `Spatial2D`, `CleanupOnNodeExit`

**Ceiling (line 57-68):**
```
Wall,
WallSize {},
Position2D(Vec2::new(0.0, playfield.top() + wall_ht)),
Scale2D { x: half_width, y: wall_ht },
Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, wall_ht)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
GameDrawLayer::Wall,
```
`Wall` `#[require]` auto-inserts: `Spatial2D`, `CleanupOnNodeExit`

After all three spawns: `walls_spawned.write(WallsSpawned)` (line 70).

---

### 1b. `second_wind::fire` — effect-spawned bottom wall
**File:** `breaker-game/src/effect/effects/second_wind/system.rs:38-54`
**Triggered by:** `EffectKind::SecondWind` firing on any entity during gameplay

```
SecondWindWall,      // marker for single-use guard
Wall,
WallSize {},
Position2D(Vec2::new(0.0, bottom_y)),
Scale2D { x: half_width, y: wall_ht },
Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, wall_ht)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
CleanupOnNodeExit,   // explicit (not from #[require] — no Spatial2D here)
```

Note: `SecondWindWall` does NOT carry `GameDrawLayer::Wall` or `Spatial2D`. The `Wall` `#[require]` would normally auto-insert `Spatial2D` and `CleanupOnNodeExit`, but `CleanupOnNodeExit` is explicitly listed here, and `Spatial2D` is absent from the spawn (walls are invisible; second wind wall may be intentionally headless for the physics-only path).

Guard: `reverse()` at `system.rs:58-67` despawns all `SecondWindWall` entities via `world.despawn()`.

---

## 2. Test Spawn Sites

All test spawns are manual inline `.spawn(...)` calls in test helper functions. None use a builder.

### 2a. `bolt_wall_collision` test helper
**File:** `breaker-game/src/bolt/systems/bolt_wall_collision/tests/helpers.rs:100-119`
**Function:** `pub(super) fn spawn_wall(app, x, y, half_width, half_height) -> Entity`

```
Wall,
Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
Position2D(pos),
GlobalPosition2D(pos),
Spatial2D,
GameDrawLayer::Wall,
```

No `WallSize`. No `Scale2D`. Has `GlobalPosition2D` (not present in production spawn).

---

### 2b. `bolt_cell_collision` test helper
**File:** `breaker-game/src/bolt/systems/bolt_cell_collision/tests/helpers.rs:132-144`
**Function:** `pub(super) fn spawn_wall(app, x, y, half_width, half_height)`

```
Wall,
WallSize {},
Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
Position2D(pos),
GlobalPosition2D(pos),
Spatial2D,
GameDrawLayer::Wall,
```

Has `WallSize`. Has `GlobalPosition2D`. No `Scale2D`.

---

### 2c. `bolt_cell_collision/tests/aabb_collision.rs` — inline spawn
**File:** `breaker-game/src/bolt/systems/bolt_cell_collision/tests/aabb_collision.rs:136-145`
**Test:** `ccd_reads_wall_half_extents_from_aabb2d_not_wall_size`

```
Wall,
WallSize {},
Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
Position2D(pos),          // Vec2::new(200.0, 0.0)
GlobalPosition2D(pos),
Spatial2D,
```

No `GameDrawLayer`. No `Scale2D`. Uses deliberately mismatched `WallSize` vs `Aabb2D` to regression-test CCD.

---

### 2d. `cell_wall_collision` inline test helper
**File:** `breaker-game/src/cells/systems/cell_wall_collision.rs:126-138`
**Function:** `fn spawn_wall(app, pos, half_extents) -> Entity` (private to test module)

```
Wall,
Aabb2D::new(Vec2::ZERO, half_extents),
CollisionLayers::new(WALL_LAYER, CELL_LAYER),   // NOTE: mask is CELL_LAYER, not BOLT_LAYER
Position2D(pos),
GlobalPosition2D(pos),
Spatial2D,
GameDrawLayer::Wall,
```

No `WallSize`. No `Scale2D`. Mask is `CELL_LAYER` (not `BOLT_LAYER`) — tests cell-wall collision specifically.

---

### 2e. `breaker_wall_collision` inline test helper
**File:** `breaker-game/src/breaker/systems/breaker_wall_collision.rs:138-150`
**Function:** `fn spawn_wall(app, pos, half_extents) -> Entity` (private to test module)

```
Wall,
Aabb2D::new(Vec2::ZERO, half_extents),
CollisionLayers::new(WALL_LAYER, BREAKER_LAYER),   // NOTE: mask is BREAKER_LAYER
Position2D(pos),
GlobalPosition2D(pos),
Spatial2D,
GameDrawLayer::Wall,
```

No `WallSize`. No `Scale2D`. Mask is `BREAKER_LAYER` — tests breaker-wall collision specifically.

---

### 2f. `dispatch_wall_effects` inline test spawns
**File:** `breaker-game/src/wall/systems/dispatch_wall_effects.rs:57-60`, `:107`, `:57`
**Tests:** `no_spurious_inserts_on_wall_entities_without_effect_definitions`, `does_not_modify_existing_bound_effects_on_wall_entity`

```
Wall                           // bare marker only — no physics, no position
Wall, pre_existing_effects     // Wall + BoundEffects (for pre-existing test)
```

---

### 2g. `wall::components` inline test spawn
**File:** `breaker-game/src/wall/components.rs:31`, `:72-78`

```
Wall                            // minimal — for require tests
Wall, Spatial2D, Position2D(...), Scale2D { ... }   // explicit override test
Wall, CollisionLayers::new(WALL_LAYER, BOLT_LAYER)  // collision layers test
```

---

### 2h. `dispatch_chip_effects` test helper
**File:** `breaker-game/src/chips/systems/dispatch_chip_effects/tests/helpers.rs:195-200`
**Function:** `pub(super) fn spawn_wall(app) -> Entity`

```
Wall,
BoundEffects::default(),
StagedEffects::default(),
```

No physics, no position. Exists purely for effect-dispatch testing.

---

### 2i. `dispatch_bolt_effects` test — Wall-targeted effects
**File:** `breaker-game/src/bolt/systems/dispatch_bolt_effects/tests/entity_targeting.rs:290-330`
**Tests:** `dispatch_pushes_wall_targeted_effects_to_all_wall_entities`, `dispatch_wall_targeted_with_zero_walls_no_panic`

Spawns via `app.world_mut().spawn((Wall, BoundEffects::default(), StagedEffects::default()))`.

---

### 2j. `ResolveOnCommand` target tests
**File:** `breaker-game/src/effect/triggers/evaluate/tests/on_resolution/resolve_all_targets.rs:219-391`
**Tests:** `resolve_on_command_resolves_all_walls_to_wall_entities`, `resolve_on_command_all_walls_with_single_wall`, `all_walls_with_context_entity_still_resolves_to_all_walls`

Spawns via `world.spawn((Wall, BoundEffects::default(), StagedEffects::default()))`.

---

### 2k. `ResolveOnCommand` singular target tests
**File:** `breaker-game/src/effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs`

Spawns `Wall` in same pattern as `resolve_all_targets.rs`.

---

### 2l. `dispatch_cell_effects` multi-entity regression test
**File:** `breaker-game/src/cells/systems/dispatch_cell_effects/tests/marker_and_components/multi_entity_dispatch.rs`

Uses `spawn_wall(app)` from helpers — delegates to the chips dispatch helper pattern.

---

### 2m. `dispatch_initial_effects` test helpers
**File:** `breaker-game/src/effect/commands/tests/dispatch_initial_effects_tests/helpers.rs:11`

Imports `Wall` for test fixture construction.

---

### 2n. Scenario runner entity tagging tests
**File:** `breaker-scenario-runner/src/lifecycle/tests/entity_tagging.rs:145`, `:159`, `:201-203`, `:289`

```
Wall     // bare marker only
```

Used by `tag_game_entities` tests to verify `ScenarioTagWall` is inserted.

---

### 2o. `second_wind::fire` tests
**File:** `breaker-game/src/effect/effects/second_wind/tests/fire_tests.rs`

Queries `With<SecondWindWall>` and `(With<SecondWindWall>, With<Wall>)` — no direct `Wall` spawn in these tests; validates `fire()` spawned the wall correctly.

---

### 2p. `effect::triggers::impact` context entity tests
**File:** `breaker-game/src/effect/triggers/impact/tests/context_entity_tests.rs`
**File:** `breaker-game/src/effect/triggers/impacted/tests/context_entity_tests.rs`

Import `Wall` for context entity assertions. Spawns vary — see those files.

---

## 3. Message Sites

### 3a. `WallsSpawned` — definition
**File:** `breaker-game/src/wall/messages.rs:9`
`pub(crate) struct WallsSpawned;`
Derives: `Message`, `Clone`, `Debug`

### 3b. `WallsSpawned` — sent
**File:** `breaker-game/src/wall/systems/spawn_walls/system.rs:70`
`walls_spawned.write(WallsSpawned)` — by `spawn_walls` system via `MessageWriter<WallsSpawned>`

### 3c. `WallsSpawned` — received
**File:** `breaker-game/src/run/node/systems/check_spawn_complete.rs:37`
`mut walls_reader: MessageReader<WallsSpawned>` — consumed by `check_spawn_complete` to gate `SpawnNodeComplete`

### 3d. `WallsSpawned` — registered in plugin
**File:** `breaker-game/src/wall/plugin.rs:21`
`app.add_message::<WallsSpawned>()`

### 3e. `WallsSpawned` — test message resource writes (simulate send)
**File:** `breaker-game/src/run/node/systems/check_spawn_complete.rs:86-87`, `:133-134`, `:151-152`, `:202-203`
`app.world_mut().resource_mut::<Messages<WallsSpawned>>().write(WallsSpawned)`

### 3f. `WallsSpawned` — test app registration in `spawn_walls` helper
**File:** `breaker-game/src/wall/systems/spawn_walls/tests/helpers.rs:11`
`app.add_message::<WallsSpawned>()`

### 3g. `WallsSpawned` — test import and assertion
**File:** `breaker-game/src/wall/systems/spawn_walls/tests/basic_spawn.rs:11`, `:59`
`app.world().resource::<Messages<WallsSpawned>>()` — asserts message was sent

---

## 4. Query Sites (Production `With<Wall>`)

### 4a. `bolt_wall_collision` system — `WallLookup` type alias
**File:** `breaker-game/src/bolt/systems/bolt_wall_collision/system.rs:26-27`
```rust
type WallLookup<'w, 's> =
    Query<'w, 's, (&'static Position2D, &'static Aabb2D), (With<Wall>, Without<Bolt>)>;
```
Reads `Position2D` and `Aabb2D` for overlap detection.

### 4b. `cell_wall_collision` system — `WallLookup` type alias
**File:** `breaker-game/src/cells/systems/cell_wall_collision.rs:21`
```rust
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;
```

### 4c. `breaker_wall_collision` system — `WallLookup` type alias
**File:** `breaker-game/src/breaker/systems/breaker_wall_collision.rs:22`
```rust
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;
```

### 4d. `dispatch_wall_effects` system
**File:** `breaker-game/src/wall/systems/dispatch_wall_effects.rs:14`
```rust
pub(crate) const fn dispatch_wall_effects(_commands: Commands, _walls: Query<Entity, With<Wall>>) {}
```
Currently a no-op.

### 4e. `dispatch_chip_effects` — `DispatchTargets` SystemParam
**File:** `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs:28`
```rust
walls: Query<'w, 's, Entity, With<Wall>>,
```
Used in `resolve_target_entities` for `Target::Wall | Target::AllWalls`.

### 4f. `dispatch_bolt_effects` system
**File:** `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs:32`
```rust
wall_query: Query<Entity, With<Wall>>,
```
Used in `Target::Wall | Target::AllWalls` resolution.

### 4g. `dispatch_cell_effects` system
**File:** `breaker-game/src/cells/systems/dispatch_cell_effects/system.rs:38`
```rust
wall_query: Query<Entity, With<Wall>>,
```
Used in `Target::Wall | Target::AllWalls` resolution.

### 4h. `effect::commands::ext` — `resolve_all` function
**File:** `breaker-game/src/effect/commands/ext.rs:356-358`
```rust
Target::AllWalls => {
    let mut query = world.query_filtered::<Entity, With<Wall>>();
    query.iter(world).collect()
}
```

### 4i. `second_wind::fire` guard check
**File:** `breaker-game/src/effect/effects/second_wind/system.rs:25-30`
```rust
world.query_filtered::<Entity, With<SecondWindWall>>().iter(world).count()
```
(Uses `SecondWindWall`, not `Wall`)

### 4j. `second_wind::reverse`
**File:** `breaker-game/src/effect/effects/second_wind/system.rs:59-63`
```rust
world.query_filtered::<Entity, With<SecondWindWall>>().iter(world).collect()
```

### 4k. `despawn_second_wind_on_contact` system
**File:** `breaker-game/src/effect/effects/second_wind/system.rs:80`
```rust
wall_query: Query<Entity, With<SecondWindWall>>,
```

---

## 5. Scenario Runner Sites

### 5a. `tag_game_entities` system — tags Wall entities
**File:** `breaker-scenario-runner/src/lifecycle/systems/entity_tagging.rs:8`, `:29`, `:51`
```rust
use breaker::wall::components::Wall;
wall_query: Query<Entity, (With<Wall>, Without<ScenarioTagWall>)>,
commands.entity(entity).insert(ScenarioTagWall);
```
Inserts `ScenarioTagWall` on every untagged `Wall` entity.

### 5b. `apply_pending_wall_effects` system
**File:** `breaker-scenario-runner/src/lifecycle/systems/pending_effects.rs:128-131`
```rust
wall_query: Query<Entity, With<ScenarioTagWall>>,
```
Reads `ScenarioTagWall` (not `Wall` directly) to apply deferred initial effects.

### 5c. `menu_bypass` — initial effects routing
**File:** `breaker-scenario-runner/src/lifecycle/systems/menu_bypass.rs:107-109`, `:130`
```rust
Target::Wall | Target::AllWalls => {
    wall_entries.extend(then.iter().cloned().map(|node| (String::new(), node)));
}
commands.insert_resource(PendingWallEffects(wall_entries));
```

### 5d. `MutationKind::SpawnExtraSecondWindWalls`
**File:** `breaker-scenario-runner/src/lifecycle/systems/frame_mutations/mutations.rs:140-143`
```rust
MutationKind::SpawnExtraSecondWindWalls(count) => {
    for _ in 0..*count {
        targets.commands.spawn(SecondWindWall);
    }
}
```
Spawns bare `SecondWindWall` marker only (no `Wall`, no physics). Used only by the `second_wind_wall_at_most_one` self-test scenario.

### 5e. `check_second_wind_wall_at_most_one` invariant checker
**File:** `breaker-scenario-runner/src/invariants/checkers/check_second_wind_wall_at_most_one.rs:2`, `:16`
```rust
use breaker::effect::effects::second_wind::SecondWindWall;
walls: Query<Entity, With<SecondWindWall>>,
```

### 5f. `ScenarioTagWall` type
**File:** `breaker-scenario-runner/src/invariants/types.rs:47`
```rust
#[derive(Component)]
pub struct ScenarioTagWall;
```

### 5g. `PendingWallEffects` resource
**File:** `breaker-scenario-runner/src/lifecycle/systems/types.rs:83`
```rust
pub struct PendingWallEffects(pub Vec<(String, EffectNode)>);
```

---

## 6. RON Data Sites

### 6a. `defaults.playfield.ron` — `wall_thickness` field
**File:** `breaker-game/assets/config/defaults.playfield.ron:6`
```ron
wall_thickness: 180.0,
```
Consumed by `PlayfieldConfig::wall_half_thickness()` at spawn time.

### 6b. `aftershock.chip.ron` — `Impacted(Wall)` trigger (3 tiers)
**File:** `breaker-game/assets/chips/standard/aftershock.chip.ron:8`, `:18`, `:28`
```ron
When(trigger: Impacted(Wall), then: [Do(Shockwave(...))])
```

### 6c. `ricochet_protocol.chip.ron` — `Impacted(Wall)` trigger
**File:** `breaker-game/assets/chips/standard/ricochet_protocol.chip.ron:8`
```ron
When(trigger: Impacted(Wall), then: [Until(trigger: Impacted(Cell), then: [...])])
```

### 6d. `breaker.example.ron` — documents `Wall`/`AllWalls` as valid targets
**File:** `breaker-game/assets/examples/breaker.example.ron:57`, `:60`
Comment-only documentation.

---

## 7. Scenario RON References

### 7a. `impacted_wall_speed.scenario.ron` — `Impacted(Wall)` trigger in initial_effects
**File:** `breaker-scenario-runner/scenarios/mechanic/impacted_wall_speed.scenario.ron:19`
```ron
When(trigger: Impacted(Wall), then: [Do(SpeedBoost(multiplier: 1.1))])
```

### 7b. `second_wind_single_use.scenario.ron` — `Do(SecondWind)` which spawns a Wall
**File:** `breaker-scenario-runner/scenarios/mechanic/second_wind_single_use.scenario.ron`
Indirectly exercises Wall entity construction via `SecondWind` effect.

### 7c. `cell_wall_proximity.scenario.ron` — exercises `CellImpactWall` path
**File:** `breaker-scenario-runner/scenarios/mechanic/cell_wall_proximity.scenario.ron`
No direct Wall spawn — exercises the geometry path.

### 7d. `breaker_wall_impact_chaos.scenario.ron` — exercises `BreakerImpactWall` path
**File:** `breaker-scenario-runner/scenarios/chaos/breaker_wall_impact_chaos.scenario.ron`

### 7e. `second_wind_wall_at_most_one.scenario.ron` — self-test with `SpawnExtraSecondWindWalls`
**File:** `breaker-scenario-runner/scenarios/self_tests/second_wind_wall_at_most_one.scenario.ron`
Uses `frame_mutations: [(frame: 30, mutation: SpawnExtraSecondWindWalls(2))]`.

---

## 8. Documentation References

- `docs/architecture/messages.md:27` — `WallsSpawned` message table entry
- `docs/architecture/messages.md:15,17,18` — `BoltImpactWall`, `BreakerImpactWall`, `CellImpactWall` message table entries
- `docs/architecture/ordering.md:36-37` — `BoltSystems::WallCollision` ordering entry
- `docs/architecture/plugins.md:78` — `WallPlugin` in plugin registration order
- `docs/architecture/plugins.md:98` — `BoltImpactWall`, `BreakerImpactWall` in effect domain message list
- `docs/architecture/effects/core_types.md:53,70,71,183` — `Wall`/`AllWalls` in `ImpactTarget` and `Target` enums
- `docs/architecture/effects/collisions.md:10,13,14` — collision pair table
- `docs/architecture/effects/targets.md:14,15,23,32,36` — target resolution rules for `Wall`/`AllWalls`
- `docs/architecture/effects/dispatch.md:28,32` — `Wall` is no-op at dispatch time (no entities yet)
- `docs/architecture/effects/trigger_systems.md:93-150` — `bridge_impact_bolt_wall`, `bridge_impacted_bolt_wall` pseudo-code
- `docs/architecture/effects/examples.md:41-53` — "Wall redirecting to bolt" example
- `docs/design/chip-catalog.md:220-256` — Aftershock and Ricochet Protocol chip designs
- `docs/design/effects/size_boost.md:18` — Wall: no-op for `SizeBoost`
- `docs/design/effects/attraction.md:9` — `AttractionType::Wall` variant
- `docs/design/triggers/impact.md:7` — `Impact(Wall)` trigger variant
- `docs/design/triggers/impacted.md:7` — `Impacted(Wall)` trigger variant
- `docs/design/graphics/gameplay-elements.md:95-101` — Walls visual design section
- `docs/design/graphics/catalog/entities.md:129-140` — Wall entity visual spec
- `docs/design/terminology/chips.md:40-41` — `Target::Wall`/`AllWalls`, `AttractionType::Wall`
- `docs/plan/done/phase-1/phase-1b-bolt.md:25` — "Wall domain extraction" in completed work

---

## Summary

### Production spawn sites: 2
1. `spawn_walls` system — 3 walls (left, right, ceiling) spawned via `commands.spawn(...)`
2. `second_wind::fire` — 1 wall spawned via `world.spawn(...)` with `SecondWindWall` marker

### Test spawn sites: 15 locations across 10+ files
All are bare inline `world.spawn(...)` or `app.world_mut().spawn(...)` calls. Component sets vary significantly:
- `bolt_wall_collision` helper: no `WallSize`, no `Scale2D`, adds `GlobalPosition2D`
- `bolt_cell_collision` helper: has `WallSize`, no `Scale2D`, adds `GlobalPosition2D`
- `cell_wall_collision` helper: no `WallSize`, no `Scale2D`, `CollisionLayers` mask is `CELL_LAYER`
- `breaker_wall_collision` helper: no `WallSize`, no `Scale2D`, `CollisionLayers` mask is `BREAKER_LAYER`
- Effect dispatch tests: bare `Wall` + effect components only, no physics

### Message sites: 7 (WallsSpawned)
- 1 send site (production): `spawn_walls` system
- 1 receive site (production): `check_spawn_complete` system
- 1 plugin registration
- 4 test manipulation sites

### Query sites (production With<Wall>): 9
- 3 collision detection systems: `bolt_wall_collision`, `cell_wall_collision`, `breaker_wall_collision`
- 4 effect dispatch systems: `dispatch_chip_effects`, `dispatch_bolt_effects`, `dispatch_cell_effects`, `effect::commands::ext::resolve_all`
- 1 no-op system: `dispatch_wall_effects`
- (SecondWind uses `With<SecondWindWall>` — subset of Wall)

### Scenario runner sites: 7
- `tag_game_entities` (queries `With<Wall>`, tags with `ScenarioTagWall`)
- `apply_pending_wall_effects` (queries `With<ScenarioTagWall>`)
- `menu_bypass` (routes `Target::Wall | Target::AllWalls` to `PendingWallEffects`)
- `MutationKind::SpawnExtraSecondWindWalls` (spawns bare `SecondWindWall` only)
- `check_second_wind_wall_at_most_one` (queries `With<SecondWindWall>`)
- `ScenarioTagWall` component type definition
- `PendingWallEffects` resource type definition

### Key observations for builder pattern work

1. **Component set inconsistency:** Production `spawn_walls` always includes `WallSize + Scale2D + Aabb2D + CollisionLayers + GameDrawLayer`. Test helpers omit different subsets (no `WallSize`, no `Scale2D`, different `CollisionLayers` masks, added `GlobalPosition2D`). A builder would need to model these variations explicitly.

2. **SecondWind wall diverges from canonical set:** Missing `GameDrawLayer::Wall` and `Spatial2D` (though `Wall` `#[require]` would auto-insert `Spatial2D`). `CleanupOnNodeExit` is explicit rather than relying on `#[require]`.

3. **`WallSize` is currently empty `{}`** — it's a marker that signals "this entity is a physics wall" for legacy documentation purposes. It does not carry data that the `Aabb2D` doesn't already hold.

4. **No test spawns `Wall` + `WallSize` + `Scale2D` + `Aabb2D` + `CollisionLayers` + `GameDrawLayer` together** (the full production set). Test helpers always reduce the set — meaning test isolation is intentional but drift is possible.

5. **`WallsSpawned` has no readers outside `check_spawn_complete`** — low coupling on the message side. Safe to keep as-is or extend.

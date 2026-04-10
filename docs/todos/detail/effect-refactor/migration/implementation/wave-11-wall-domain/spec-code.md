## Implementation Spec: Walls — Death Pipeline Migration

### Domain
`src/walls/`

### Prerequisites

The following must be complete before this wave:

- **Wave 2 (scaffold)**: `GameEntity` trait with `impl GameEntity for Wall {}`, `Hp`, `KilledBy`, `Dead` components, `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity` messages, convenience constructors (`KillYourself::new()`, `Destroyed::new()`)
- **Wave 7 (death pipeline)**: `apply_damage::<Wall>`, `detect_wall_deaths`, `process_despawn_requests` systems, `DeathPipelineSystems` system set with `ApplyDamage` and `DetectDeaths` variants
- **Wave 5 (triggers)**: Trigger bridge infrastructure including `on_destroyed::<Wall>` bridge
- **Wave 6 (effects)**: Shield and Second Wind effect `fire()` functions that spawn wall entities

### Failing Tests
- `src/walls/systems/handle_wall_kill/tests.rs` -- tests for the new wall kill handler (count TBD from test spec)

Note: `apply_damage::<Wall>` and `detect_wall_deaths` were implemented in Wave 7 with their own tests. This wave does NOT re-test those systems. This wave tests the wall kill handler, the shield/second-wind spawn updates, and end-to-end flows.

---

### What to Implement

#### 1. Create `handle_wall_kill` system (NEW)
- **File**: `src/walls/systems/handle_wall_kill/system.rs` (directory module with tests)
- **Description**: Wall domain kill handler. Reads `KillYourself<Wall>` messages and performs domain-specific death logic.
- **Signature**:
```rust
pub fn handle_wall_kill(
    mut messages: MessageReader<KillYourself<Wall>>,
    mut commands: Commands,
    query: Query<&Position2D, (With<Wall>, Without<Dead>)>,
    killer_query: Query<&Position2D>,
    mut despawn_messages: MessageWriter<DespawnEntity>,
    mut destroyed_messages: MessageWriter<Destroyed<Wall>>,
)
```
- **Behavior for each `KillYourself<Wall>` message**:
  1. Look up the victim entity in the `query`. If entity not found (already despawned, or already `Dead`), skip. Log a debug warning in dev builds.
  2. Insert `Dead` marker component on the victim entity via `commands.entity(victim).insert(Dead)`. This prevents double-processing by `apply_damage::<Wall>` and `detect_wall_deaths` via their `Without<Dead>` filters in subsequent frames.
  3. Remove the victim entity's `Aabb2D` component via `commands.entity(victim).remove::<Aabb2D>()`. The quadtree from `rantzsoft_spatial2d` auto-deregisters when the `Aabb2D` component is removed. The entity is dead and should not participate in collision detection.
  4. Extract `victim_pos` (Vec2) from the victim's `Position2D` component: `position.0`.
  5. Determine `killer_pos`: if `msg.killer` is `Some(killer_entity)`, try to look up that entity's `Position2D` in the `killer_query` to get the killer position as `Some(position.0)`. If the killer entity no longer exists (query returns `Err`), use `None`.
  6. Send `Destroyed<Wall>` message with `victim: msg.victim`, `killer: msg.killer`, `victim_pos`, `killer_pos`.
  7. Send `DespawnEntity { entity: msg.victim }` message. The entity will be despawned in PostFixedUpdate by `process_despawn_requests`.
- **Does NOT**:
  - Despawn the entity directly. The entity must survive through trigger evaluation.
  - Modify Hp or KilledBy.
  - Deal damage.
  - Evaluate triggers.
  - Require `Hp` on the victim -- `Fire(Die)` can send `KillYourself<Wall>` for entities without Hp.
  - Update node completion tracking (walls are not tracked for node clearing -- unlike cells, walls have no `RequiredToClear` equivalent).
- **Dedup strategy**: `Without<Dead>` on the victim query is sufficient. Each entity only gets one `KillYourself<Wall>` per frame because `detect_wall_deaths` queries `With<Wall>` (distinct from other entity types) and `Dead` is inserted by the kill handler, visible next frame. No `HashSet` needed.

#### 2. Update shield effect `fire()` to add `Hp` and `KilledBy`
- **File**: `src/effect/effects/shield/` (wherever the shield wall entity is spawned in the `fire()` function)
- **What changes**: When the shield effect's `fire()` function spawns a `ShieldWall` entity, add `Wall`, `Hp`, and `KilledBy` components alongside existing components. The `Wall` component is required for `Fire(Die)` to classify the entity type and for `handle_wall_kill`'s `With<Wall>` query filter.
- **Hp construction**: `Hp { current: 1.0, starting: 1.0, max: None }` -- one-shot wall. Shield walls primarily expire via timer (`tick_shield_duration`) or reflection cost depletion, but adding Hp makes them eligible for the damage path as well.
- **KilledBy construction**: `KilledBy::default()` (dealer is None initially -- set by `apply_damage::<Wall>` on the killing blow, if the wall dies via damage).
- **Note**: The shield timer-expiry system (`tick_shield_duration`) still uses its existing path for now. Only the `Fire(Die)` and `DamageDealt<Wall>` paths go through the kill handler.

#### 3. Update second-wind effect `fire()` to add `Hp` and `KilledBy`
- **File**: `src/effect/effects/second_wind/` (wherever the second-wind wall entity is spawned in the `fire()` function)
- **What changes**: When the second-wind effect's `fire()` function spawns a `SecondWindWall` entity, add `Wall`, `Hp`, and `KilledBy` components alongside existing components. The `Wall` component is required for `Fire(Die)` to classify the entity type and for `handle_wall_kill`'s `With<Wall>` query filter.
- **Hp construction**: `Hp { current: 1.0, starting: 1.0, max: None }` -- one-shot wall. Second-wind walls die via `Fire(Die)` after their first bounce.
- **KilledBy construction**: `KilledBy::default()`.
- **Note**: The second-wind bounce handler still uses its existing path for now. Only the `Fire(Die)` and `DamageDealt<Wall>` paths go through the kill handler.

#### 4. Permanent walls: no changes
- Permanent walls (side walls, top wall) are spawned by the wall spawn system without `Hp` or `KilledBy`.
- No modifications needed. Permanent walls have no `Hp` component, which means they are not queryable by `apply_damage::<Wall>` or `detect_wall_deaths`. They cannot be damaged or killed through the damage path.
- However, they CAN still be killed via `Fire(Die)`, which sends `KillYourself<Wall>` directly, bypassing the damage/Hp path. The kill handler processes `KillYourself<Wall>` regardless of whether the entity has Hp. But there is no game mechanic that does this for permanent walls.

---

### Patterns to Follow

- **Kill handler pattern (from design docs)**: The death pipeline design specifies that domain kill handlers follow a consistent pattern: read `KillYourself<T>` messages, insert `Dead`, perform domain-specific cleanup, extract positions while entity is alive, send `Destroyed<T>`, send `DespawnEntity`. The wall handler is simpler than the cell handler because:
  - Walls have no `RequiredToClear` tracking (cells have it; walls do not).
  - Walls have no visual damage feedback system (cells have `cell_damage_visual`; walls do not need one in this wave).
  - The core pattern is identical: read `KillYourself<T>` -> insert `Dead` -> remove `Aabb2D` (spatial deregistration) -> extract positions -> send `Destroyed<T>` -> send `DespawnEntity`.
- **Message reader pattern**: Use `MessageReader<KillYourself<Wall>>` and iterate with `messages.read()`.
- **Query pattern**: Use `With<Wall>` and `Without<Dead>` filters consistently. The kill handler queries `Without<Dead>` because a wall should not be killed twice.
- **Spatial deregistration**: Use `commands.entity(victim).remove::<Aabb2D>()`. The `rantzsoft_spatial2d` quadtree auto-deregisters entities when their `Aabb2D` component is removed. Do NOT use `ResMut<SpatialIndex>` directly.
- **`Position2D` to `Vec2` conversion**: Use `position.0` to extract the Vec2 from `Position2D(Vec2)`.
- **Killed entity position extraction**: The `killer_query` must be a separate, unfiltered `Query<&Position2D>` (not restricted to `With<Wall>`) because the killer can be any entity type (Bolt, Breaker, Cell, or even another Wall).

---

### RON Data
- No new RON fields needed. Wall Hp values are hardcoded per wall type (1.0 for one-shot walls). Shield wall and Second Wind wall configs already exist in effect RON. No new data files.

---

### Schedule

#### `handle_wall_kill`
- **Schedule**: `FixedUpdate`
- **After**: `DeathPipelineSystems::DetectDeaths` -- kill handlers consume `KillYourself<T>` messages produced by death detection systems. Also runs after `Fire(Die)` sends `KillYourself<Wall>` directly.
- **Before**: `EffectSystems::Bridge` -- the `Destroyed<Wall>` message must be available for `on_destroyed::<Wall>` to dispatch triggers. Per the system-set-ordering doc, domain kill handlers run between DetectDeaths and the effect system's death bridges.
- **Full frame ordering context**:
  ```
  collision systems produce DamageDealt<Wall>
      -> ApplyDamage set (apply_damage::<Wall> decrements Hp, sets KilledBy)
      -> DetectDeaths set (detect_wall_deaths sends KillYourself<Wall> when Hp <= 0)
      -> handle_wall_kill (this system: inserts Dead, removes Aabb2D, sends Destroyed<Wall> + DespawnEntity)
      -> EffectSystems::Bridge (on_destroyed::<Wall> dispatches Died/Killed/DeathOccurred triggers)
      -> PostFixedUpdate: process_despawn_requests (despawns entity)
  ```

---

### Wiring Requirements

#### `src/walls/plugin.rs`
1. **Add** registration of `handle_wall_kill` system in `FixedUpdate`, ordered after `DeathPipelineSystems::DetectDeaths` and before `EffectSystems::Bridge`.
2. **Add** `Destroyed<Wall>` message registration: `app.add_message::<Destroyed<Wall>>();` -- if not already registered by the death pipeline plugin or a prior wave. (Check if `Destroyed<Wall>` registration already exists. If the death pipeline plugin or Wave 7 registered it, do not duplicate.)

#### `src/walls/systems/mod.rs`
1. **Add** `pub(crate) mod handle_wall_kill;` and re-export the system function.

#### Effect-domain wall spawners
These changes ensure that effect-spawned walls (ShieldWall, SecondWindWall) include `Hp` and `KilledBy` so they can participate in the death pipeline:

1. `src/effect/effects/shield/` (wherever the shield wall is spawned in the `fire()` function) -- when spawning the `ShieldWall` entity, add `Hp { current: 1.0, starting: 1.0, max: None }` and `KilledBy::default()`.
2. `src/effect/effects/second_wind/` (wherever the second-wind wall is spawned in the `fire()` function) -- when spawning the `SecondWindWall` entity, add `Hp { current: 1.0, starting: 1.0, max: None }` and `KilledBy::default()`.
3. `src/walls/` -- if there is a regular wall spawn system that spawns destructible walls (other than shield/second-wind), update it to add `Hp` and `KilledBy`. Permanent walls do NOT get these components.

---

### Constraints

#### Do NOT modify
- `src/shared/systems/apply_damage.rs` -- generic system already implemented in Wave 7
- `src/walls/systems/detect_wall_deaths.rs` -- already implemented in Wave 7
- `src/shared/systems/process_despawn_requests.rs` -- already implemented in Wave 7
- `src/shared/components/` -- Hp, KilledBy, Dead already defined
- `src/shared/messages/` -- DamageDealt, KillYourself, Destroyed, DespawnEntity already defined
- `src/effect/triggers/death/` -- on_destroyed::<Wall> bridge already implemented in an earlier wave
- Any other domain (bolt, cells, breaker) -- this wave only touches the wall domain and effect wall spawners

#### Do NOT add
- New message types -- use the existing shared death pipeline messages
- New component types -- use the existing shared Hp, KilledBy, Dead
- New system sets -- use the existing `DeathPipelineSystems` sets
- Visual damage feedback system for walls -- walls do not need a `Changed<Hp>` visual system like cells. Shield walls have their own visual treatment handled by the shield effect's tick system. This is out of scope.
- Invulnerability or shield logic in the kill handler -- the wall kill handler always confirms the kill. If a wall receives `KillYourself<Wall>`, it dies. There are no wall-level invulnerability checks.

---

### File Layout for New System

#### `handle_wall_kill`
```
src/walls/systems/handle_wall_kill/
    mod.rs          -- pub(crate) use system::*; mod system; #[cfg(test)] mod tests;
    system.rs       -- handle_wall_kill function
    tests.rs        -- all test code
```

---

### Prerequisite Types (Must Already Exist Before This Wave)

All of these were created in Wave 2 (scaffold) or Wave 7 (death pipeline). The writer-code for this wave does NOT create these; it uses them.

| Type | Kind | Location | Description |
|------|------|----------|-------------|
| `GameEntity` | Trait | `src/shared/` | Marker trait with impls for `Bolt`, `Cell`, `Wall`, `Breaker` |
| `Hp` | Component | `src/shared/components/` | `{ current: f32, starting: f32, max: Option<f32> }` |
| `KilledBy` | Component | `src/shared/components/` | `{ dealer: Option<Entity> }` |
| `Dead` | Component | `src/shared/components/` | Marker component for dead entities |
| `KillYourself<Wall>` | Message | `src/shared/messages/` | `{ victim: Entity, killer: Option<Entity>, _marker: PhantomData<Wall> }` |
| `Destroyed<Wall>` | Message | `src/shared/messages/` | `{ victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<Wall> }` |
| `DespawnEntity` | Message | `src/shared/messages/` | `{ entity: Entity }` |
| `DamageDealt<Wall>` | Message | `src/shared/messages/` | `{ dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<Wall> }` |
| `Wall` | Component | `src/walls/` | Entity marker component |
| `DeathPipelineSystems` | SystemSet | `src/shared/sets/` | Enum with `ApplyDamage`, `DetectDeaths` variants |
| `DeathDetectionData` | QueryData | `src/shared/queries/` | Read-only query data for death detection |
| `DamageTargetData` | QueryData | `src/shared/queries/` | Mutable query data for apply_damage |
| `Position2D` | Component | `rantzsoft_spatial2d` | 2D position: `Position2D(Vec2)` |
| `Aabb2D` | Component | `rantzsoft_spatial2d` | Axis-aligned bounding box; removing it auto-deregisters from quadtree |

---

### End-to-End Flow Summary

The full wall damage-to-death flow after this wave:

1. Bolt collision system detects wall collision, sends `BoltImpactWall` (for effect triggers) AND optionally `DamageDealt<Wall>` (for destructible walls -- only if the collision system is updated to send damage for walls. Currently `BoltImpactWall` is sent but damage is not sent for walls. Destructible walls die through `Fire(Die)`, not through `DamageDealt<Wall>`.)
2. `apply_damage::<Wall>` (Wave 7) reads `DamageDealt<Wall>`, decrements `Hp`, sets `KilledBy` on killing blow. Walls without `Hp` (permanent walls) are unaffected. Walls with `Dead` are skipped.
3. `detect_wall_deaths` (Wave 7) reads walls with `Hp <= 0`, sends `KillYourself<Wall>` with victim and killer.
4. **`handle_wall_kill` (this wave)** reads `KillYourself<Wall>`:
   - Inserts `Dead` on victim
   - Removes `Aabb2D` (spatial deregistration)
   - Extracts positions from `Position2D`
   - Sends `Destroyed<Wall>` with victim, killer, positions
   - Sends `DespawnEntity`
5. `on_destroyed::<Wall>` (effect bridge, earlier wave) reads `Destroyed<Wall>`, dispatches Died/Killed/DeathOccurred triggers.
6. `process_despawn_requests` (PostFixedUpdate, Wave 7) despawns the entity.

#### Shield wall death flow:
1. Shield effect fires, spawns `ShieldWall` entity with `Hp(1.0)`, `KilledBy::default()`, `ShieldDuration`, `ShieldReflectionCost`, `ShieldOwner`.
2. `tick_shield_duration` decrements duration over time and on bolt reflections.
3. When duration reaches 0 or below, the shield tick system triggers wall death via `Fire(Die)`.
4. `Fire(Die)` sends `KillYourself<Wall>` with killer = None (environmental death, timer expiry).
5. `handle_wall_kill` processes the kill as described above.

#### Second Wind wall death flow:
1. Second Wind effect fires, spawns `SecondWindWall` entity with `Hp(1.0)`, `KilledBy::default()`, `SecondWindOwner`.
2. Bolt bounces off the wall. The wall's bound effects include `When(Impacted(Bolt), Fire(Die(DieConfig())))`.
3. `Fire(Die)` sends `KillYourself<Wall>` with killer = None.
4. `handle_wall_kill` processes the kill as described above.

#### Permanent wall:
- Permanent walls have no `Hp` component. They are never matched by `apply_damage::<Wall>` or `detect_wall_deaths`. They cannot be killed through the damage path. They CAN still be killed via `Fire(Die)`, but there is no game mechanic that does this for permanent walls. Permanent walls are never modified by this wave.

---

### Testing Note

Tests should be written to the files listed in "Failing Tests" above (the `tests.rs` within the `handle_wall_kill/` directory module).

Each test should:
1. Create a minimal Bevy `App` with `MinimalPlugins`
2. Register the message types needed (`app.add_message::<KillYourself<Wall>>()`, `app.add_message::<Destroyed<Wall>>()`, `app.add_message::<DespawnEntity>()`)
3. Spawn wall entities with the required components (`Wall`, `Position2D`, `Hp`, `KilledBy`, and optionally `Dead`)
4. Send `KillYourself<Wall>` messages via `MessageWriter`
5. Run the `handle_wall_kill` system
6. Assert: `Dead` component was inserted, `Aabb2D` was removed, `Destroyed<Wall>` message was sent with correct fields, `DespawnEntity` message was sent

Key test behaviors (aligned with test spec):
- **Wall kill handler basic**: `KillYourself<Wall>` -> inserts `Dead`, removes `Aabb2D`, sends `Destroyed<Wall>` + `DespawnEntity`
- **Fire(Die) path**: Wall without Hp killed via `KillYourself<Wall>` -> kill handler works (no Hp required)
- **Nonexistent victim**: `KillYourself<Wall>` for despawned entity -> no panic, gracefully skip
- **Destructible wall e2e**: Wall with `Hp(1.0)`, receives `DamageDealt<Wall>`, full pipeline to despawn (multi-system integration test)
- **Permanent wall not affected**: Wall without `Hp` -- not matched by `apply_damage::<Wall>` or `detect_wall_deaths`. No `KillYourself<Wall>` sent from detect path.
- **Shield wall through pipeline**: Effect-spawned wall with `Hp` works through death pipeline
- **Already-dead wall skipped**: Wall with `Dead` component is not processed by kill handler (`Without<Dead>` filter)
- **Killer position extraction**: When killer entity exists, `killer_pos` is `Some(Vec2)`; when killer is gone, `killer_pos` is `None`
- **Environmental kill (no killer)**: `KillYourself<Wall>` with `killer: None` -> `Destroyed<Wall>` with `killer: None`, `killer_pos: None`

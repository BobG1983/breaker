## Implementation Spec: Cells — Death Pipeline Migration

### Domain
`src/cells/`

### Prerequisites

The following waves must be complete before wave 9 implementation:

- **Wave 2 (scaffold)**: All shared types exist — `Hp`, `KilledBy`, `Dead`, `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity`, `GameEntity` trait, `Cell` impl of `GameEntity`
- **Wave 7 (death pipeline)**: `apply_damage::<Cell>`, `detect_cell_deaths`, `process_despawn_requests` are implemented. `DamageTargetData` and `DeathDetectionData` query types exist. `DeathPipelineSystems::ApplyDamage` and `DeathPipelineSystems::DetectDeaths` system sets exist. `Aabb2D` component exists in `rantzsoft_spatial2d` and `maintain_quadtree` handles `RemovedComponents<Aabb2D>` for auto-deregistration.
- **Wave 5 (triggers)**: `on_destroyed::<Cell>` effect bridge exists (consumes `Destroyed<Cell>`)
- **`Position2D`**: The `Position2D` component from `rantzsoft_spatial2d` is used for entity positions (not `Transform`). Access the inner `Vec2` via `position.0`.

### Failing Tests
- `src/cells/systems/handle_cell_kill/tests.rs` — tests for the new cell kill handler (count TBD from test spec)
- `src/cells/systems/cell_damage_visual/tests.rs` — tests for damage visual feedback system (count TBD from test spec)
- `src/bolt/systems/bolt_cell_collision/tests.rs` — updated tests for DamageDealt<Cell> emission (count TBD from test spec)

Note: `apply_damage::<Cell>` and `detect_cell_deaths` were implemented in Wave 7 with their own tests. This wave does NOT re-test those systems. This wave tests the cell kill handler, the collision system update, the visual feedback system, and end-to-end flows.

---

### What to Implement

#### 1. Remove `handle_cell_hit` system
- **File to delete**: `src/cells/systems/handle_cell_hit/` (entire directory module)
- **What it did**: Read `DamageCell` messages, decremented `CellHealth`, applied visual feedback (material color update), sent `RequestCellDestroyed` when health reached zero. All of this is replaced by the death pipeline.
- **Replaced by**: `apply_damage::<Cell>` (Wave 7) + `detect_cell_deaths` (Wave 7) + `handle_cell_kill` (this wave) + `cell_damage_visual` (this wave)
- **Plugin deregistration**: Remove the system from `CellsPlugin::build()` in `src/cells/plugin.rs`. Remove the `DamageCell` message registration if no other consumers remain.

#### 2. Remove `cleanup_cell` system
- **File to delete**: `src/cells/systems/cleanup_cell.rs`
- **What it did**: Read `RequestCellDestroyed`, removed entity from spatial index, updated `RequiredToClear` tracking, sent `CellDestroyedAt`, despawned entity.
- **Replaced by**: `handle_cell_kill` (this wave) + `process_despawn_requests` (Wave 7)
- **Plugin deregistration**: Remove the system from `CellsPlugin::build()` in `src/cells/plugin.rs`. Remove old message registrations (`RequestCellDestroyed`, `CellDestroyedAt`) if no other consumers remain.

#### 3. Create `handle_cell_kill` system (NEW)
- **File**: `src/cells/systems/handle_cell_kill/system.rs` (directory module with tests)
- **Description**: Cell domain kill handler. Reads `KillYourself<Cell>` messages and performs domain-specific death logic.
- **Signature**: `fn handle_cell_kill(messages: MessageReader<KillYourself<Cell>>, mut commands: Commands, query: Query<(&Position2D, Has<RequiredToClear>), (With<Cell>, Without<Dead>)>, killer_query: Query<&Position2D>, /* MessageWriter params as needed */)`
- **Dedup**: The `Without<Dead>` query filter is sufficient. If the same entity appears in two `KillYourself<Cell>` messages in one frame, the first processes it and inserts `Dead`. The query lookup for the second message fails (entity no longer matches `Without<Dead>`), so it is skipped. No `HashSet<Entity>` needed.
- **Behavior for each `KillYourself<Cell>` message**:
  1. Look up the victim entity in the query. If entity not found or already `Dead`, skip (log debug warning in dev builds).
  2. Insert `Dead` marker component on the victim entity via `commands.entity(victim).insert(Dead)`. This prevents double-processing by `apply_damage::<Cell>` and `detect_cell_deaths` via their `Without<Dead>` filters.
  3. Remove the `Aabb2D` component from the victim entity via `commands.entity(victim).remove::<Aabb2D>()`. The `maintain_quadtree` system in `rantzsoft_spatial2d` watches `RemovedComponents<Aabb2D>` and auto-deregisters the entity from the quadtree. Do NOT directly manipulate `SpatialIndex` / `ResMut<SpatialIndex>`.
  4. If the victim entity `Has<RequiredToClear>`, the entity still has this component when `Destroyed<Cell>` is sent. Downstream consumers (run-domain node completion tracking) read `RequiredToClear` from the still-alive entity. The kill handler does NOT need to track this separately.
  5. Extract `victim_pos` (Vec2) from the victim's `Position2D` component while the entity is still alive: `position.0` (a `Vec2`).
  6. Determine `killer_pos`: if `msg.killer` is `Some(killer_entity)`, try to read that entity's `Position2D` via `killer_query.get(killer_entity)` to get the killer position as `Some(position.0)`. If the killer entity no longer exists, use `None`.
  7. Send `Destroyed<Cell>` message with `victim: msg.victim`, `killer: msg.killer`, `victim_pos`, `killer_pos`.
  8. Send `DespawnEntity { entity: msg.victim }` message. The entity will be despawned in PostFixedUpdate by `process_despawn_requests`.
- **Does NOT**: despawn the entity directly, modify Hp, modify KilledBy, deal damage, evaluate triggers. The entity must survive through trigger evaluation (which happens when `on_destroyed::<Cell>` reads the `Destroyed<Cell>` message in `EffectSystems::Bridge`).

#### 4. Update `bolt_cell_collision` to send `DamageDealt<Cell>`
- **File**: `src/bolt/systems/bolt_cell_collision/system.rs` (or wherever the collision system lives)
- **What changes**: Instead of sending `DamageCell { cell, damage, source_chip }`, send `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: damage_amount, source_chip, _marker: PhantomData }`.
- **The `dealer` field**: Set to `Some(bolt_entity)` — the bolt that hit the cell. This is used by `apply_damage::<Cell>` to populate `KilledBy.dealer` on the killing blow, which is then used by `on_destroyed::<Cell>` to dispatch `Trigger::Killed(Cell)` on the killer entity.
- **The `amount` field**: Use the same damage value that was previously sent in `DamageCell.damage`. This comes from the bolt's damage stat (base damage, possibly modified by damage boost effects).
- **The `source_chip` field**: Carry forward the same chip attribution that was previously sent in `DamageCell.source_chip`.
- **Also**: Continue sending `BoltImpactCell { cell, bolt }` — the impact message is separate from the damage message and is consumed by the effect trigger bridges. Do NOT remove or modify impact message sending.

#### 5. Add `Hp` and `KilledBy` to cell builder
- **File**: Wherever cells are spawned (cell builder / spawn system in `src/cells/` or `src/run/node/`)
- **What changes**: When spawning a cell entity, add `Hp` and `KilledBy` components alongside existing components.
- **Hp construction**: `Hp { current: cell_health_value, starting: cell_health_value, max: None }` where `cell_health_value` is the value that was previously used for `CellHealth::new(value)`. The starting value equals current at spawn time. No max — cells have no healing mechanic.
- **KilledBy construction**: `KilledBy::default()` (dealer is None initially — set by `apply_damage` on the killing blow).
- **Remove**: `CellHealth` component from the spawn. `Hp` replaces `CellHealth` entirely.

#### 6. Create `cell_damage_visual` system (NEW)
- **File**: `src/cells/systems/cell_damage_visual/system.rs` (directory module with tests)
- **Description**: Visual feedback system for cell damage. Reads `Hp` changes and updates material/color to reflect damage level. Extracted from the visual feedback portion of the old `handle_cell_hit`.
- **Signature**: `fn cell_damage_visual(query: Query<(&Hp, &MeshMaterial2d<ColorMaterial>), (With<Cell>, Changed<Hp>, Without<Dead>)>, mut materials: ResMut<Assets<ColorMaterial>>)`
- **Behavior**: For each cell with a changed `Hp`:
  1. Calculate health fraction: `hp.current / hp.starting`. Clamp to 0.0..=1.0. If `hp.starting == 0.0`, treat the fraction as 0.0 to avoid division by zero.
  2. Look up the cell's `ColorMaterial` via `materials.get_mut(&mesh_material.0)` and update its `color` field based on the health fraction. Damaged cells should look visually distinct from full-health cells.
  3. The exact visual treatment should match what `handle_cell_hit` was doing — interpolate color from full color to a damaged color based on health fraction. The cell's `MeshMaterial2d<ColorMaterial>` handle references a `ColorMaterial` asset whose color is modified in place.
- **Does NOT**: Apply damage, detect deaths, despawn entities, or interact with the death pipeline in any way. This is purely a visual read of the current Hp state.
- **Note on `Changed<Hp>` filter**: This filter ensures the system only runs on entities whose `Hp` component was mutated this frame. Since `apply_damage::<Cell>` mutates `Hp`, this system will automatically trigger after damage is applied. The `Without<Dead>` filter skips entities that are already dead (no point updating visuals for dead entities about to be despawned).

---

### Patterns to Follow

- **Kill handler pattern**: The cell kill handler follows the same pattern that all domain kill handlers will follow (bolt in Wave 10, wall in Wave 11, breaker in Wave 12). It is the first one implemented, so it establishes the pattern. Key elements: read `KillYourself<T>`, insert `Dead`, perform domain cleanup, extract positions while entity is alive, send `Destroyed<T>`, send `DespawnEntity`.
- **Message reader pattern**: Use `MessageReader<KillYourself<Cell>>` to iterate over messages. Follow existing message reading patterns in the codebase (e.g., how collision systems read collision messages).
- **Query pattern**: Use `With<Cell>` and `Without<Dead>` filters consistently. The kill handler queries `Without<Dead>` because a cell should not be killed twice.
- **Spatial deregistration**: Remove `Aabb2D` from the entity via `commands.entity(victim).remove::<Aabb2D>()`. The `maintain_quadtree` system in `rantzsoft_spatial2d` watches `RemovedComponents<Aabb2D>` and auto-deregisters the entity from the quadtree. Do NOT directly access `SpatialIndex` or `ResMut<SpatialIndex>`.
- **`Changed<Hp>` for visual feedback**: Bevy's change detection is the idiomatic way to react to component mutations. The visual system uses `Changed<Hp>` rather than reading damage messages directly.

---

### RON Data
- No new RON fields needed. Cell health values already exist in cell type RON definitions. The spawn code reads from the same data — only the component type changes from `CellHealth` to `Hp`.

---

### Schedule

#### `handle_cell_kill`
- **Schedule**: `FixedUpdate`
- **After**: `DeathPipelineSystems::DetectDeaths` — kill handlers consume `KillYourself<T>` messages produced by death detection
- **Before**: `EffectSystems::Bridge` — the `Destroyed<Cell>` message must be available for `on_destroyed::<Cell>` to dispatch triggers. However, per the system-set-ordering doc, the domain kill handlers run after DetectDeaths and before the effect system's death bridges process `Destroyed<T>`.
- **Note on ordering**: The full frame ordering is: collision systems produce `DamageDealt<Cell>` -> `ApplyDamage` set -> `DetectDeaths` set -> domain kill handlers -> `EffectSystems::Bridge` reads `Destroyed<Cell>` -> PostFixedUpdate despawn. The kill handler sits between DetectDeaths and Bridge.

#### `cell_damage_visual`
- **Schedule**: `Update` (not `FixedUpdate`)
- **Rationale**: Visual feedback is a rendering concern. It reads `Changed<Hp>` which persists across the FixedUpdate→Update boundary. Running in Update means visuals update at display framerate, not physics framerate. If the project convention is to run visual systems in FixedUpdate alongside gameplay, use FixedUpdate instead — but check existing visual/fx system patterns.
- **No ordering constraints**: This system is read-only on Hp (no mutation), writes only to `ColorMaterial` assets via `ResMut<Assets<ColorMaterial>>`, and has no ordering dependencies with the death pipeline.

---

### Wiring Requirements

#### `src/cells/plugin.rs`
1. **Remove** registration of `handle_cell_hit` system
2. **Remove** registration of `cleanup_cell` system
3. **Remove** `DamageCell` message registration (if `app.register_message::<DamageCell>()` exists and no other consumers remain)
4. **Remove** `RequestCellDestroyed` message registration (if it exists and no other consumers remain)
5. **Remove** `CellDestroyedAt` message registration (if it exists and no other consumers remain)
6. **Add** registration of `handle_cell_kill` system in `FixedUpdate`, ordered after `DeathPipelineSystems::DetectDeaths`
7. **Add** registration of `cell_damage_visual` system in `Update` (or `FixedUpdate` — see schedule note above)

#### `src/cells/systems/mod.rs`
1. **Remove** `pub(crate) mod handle_cell_hit;` and its re-exports
2. **Remove** `pub(crate) mod cleanup_cell;` and its re-exports
3. **Add** `pub(crate) mod handle_cell_kill;` and re-export
4. **Add** `pub(crate) mod cell_damage_visual;` and re-export

#### `src/cells/messages.rs`
1. **Remove** `DamageCell` message type definition (replaced by `DamageDealt<Cell>` in shared)
2. **Remove** `RequestCellDestroyed` message type definition (replaced by `KillYourself<Cell>` in shared)
3. **Remove** `CellDestroyedAt` message type definition (replaced by `Destroyed<Cell>` in shared)
4. If `messages.rs` becomes empty, remove the file and its `mod` declaration

#### `src/cells/components.rs` (or `components/types.rs`)
1. **Remove** `CellHealth` component type definition (replaced by `Hp` in shared)
2. Update any references to `CellHealth` within the cells domain to use `Hp`

#### `src/bolt/systems/bolt_cell_collision/system.rs`
1. **Change** `DamageCell` send to `DamageDealt<Cell>` send
2. Update imports accordingly

#### Consumers of old messages to update
- Any system that was reading `DamageCell` must now read `DamageDealt<Cell>` (primarily `handle_cell_hit` which is being removed, but check if effects also read it)
- Any system that was reading `RequestCellDestroyed` must now read `KillYourself<Cell>` (primarily `cleanup_cell` which is being removed, but check effect bridge)
- Any system that was reading `CellDestroyedAt` must now read `Destroyed<Cell>`:
  - `src/run/node/systems/track_node_completion.rs` (or equivalent) — reads `CellDestroyedAt { was_required_to_clear }` for node completion tracking. Must be updated to read `Destroyed<Cell>` and check `RequiredToClear` on the victim entity (still alive at this point).
  - `src/effect/triggers/cell_destroyed.rs` — triggers `Trigger::CellDestroyed` globally. Must be updated to read `Destroyed<Cell>` instead. (This may already be handled if Wave 5 migrated triggers.)

---

### Constraints

#### Do NOT modify
- `src/shared/systems/apply_damage.rs` — generic system already implemented in Wave 7
- `src/cells/systems/detect_cell_deaths.rs` — already implemented in Wave 7
- `src/shared/systems/process_despawn_requests.rs` — already implemented in Wave 7
- `src/shared/components/` — Hp, KilledBy, Dead already defined
- `src/shared/messages/` — DamageDealt, KillYourself, Destroyed, DespawnEntity already defined
- `src/effect/triggers/death/` — on_destroyed::<Cell> bridge already implemented in Wave 5
- Any other domain (bolt, wall, breaker) except the specific `bolt_cell_collision` update

#### Do NOT add
- New message types — use the existing shared death pipeline messages
- New component types — use the existing shared Hp, KilledBy, Dead
- New system sets — use the existing `DeathPipelineSystems` sets
- Shield / invulnerability / second-wind logic to the kill handler — that is a future concern. The cell kill handler always confirms the kill. Cells do not have invulnerability.

---

### File Layout for New Systems

#### `handle_cell_kill`
```
src/cells/systems/handle_cell_kill/
    mod.rs          — pub(crate) use system::*; mod system; #[cfg(test)] mod tests;
    system.rs       — handle_cell_kill function
    tests.rs        — all test code
```

#### `cell_damage_visual`
```
src/cells/systems/cell_damage_visual/
    mod.rs          — pub(crate) use system::*; mod system; #[cfg(test)] mod tests;
    system.rs       — cell_damage_visual function
    tests.rs        — all test code
```

---

### End-to-End Flow Summary

The full cell damage-to-death flow after this wave:

1. `bolt_cell_collision` detects collision, sends `BoltImpactCell` (for triggers) AND `DamageDealt<Cell>` (for damage)
2. `apply_damage::<Cell>` reads `DamageDealt<Cell>`, decrements `Hp`, sets `KilledBy` on killing blow. Skips `Locked` cells. Skips `Dead` cells.
3. `cell_damage_visual` (visual system) reads `Changed<Hp>`, updates cell color/tint based on health fraction
4. `detect_cell_deaths` reads cells with `Hp <= 0`, sends `KillYourself<Cell>` with victim and killer
5. `handle_cell_kill` reads `KillYourself<Cell>`:
   - Inserts `Dead` on victim
   - Removes `Aabb2D` (triggers quadtree auto-deregistration)
   - Extracts positions from `Position2D` components
   - Sends `Destroyed<Cell>` with victim, killer, positions
   - Sends `DespawnEntity`
6. `on_destroyed::<Cell>` (effect bridge, Wave 5) reads `Destroyed<Cell>`, dispatches Died/Killed/DeathOccurred triggers
7. Node completion tracking reads `Destroyed<Cell>` and checks `RequiredToClear` on victim entity
8. `process_despawn_requests` (PostFixedUpdate) despawns the entity

Locked cells: `DamageDealt<Cell>` messages for locked cells are silently dropped by `apply_damage::<Cell>` (Wave 7 implementation, `Without<Locked>` filter). No Hp change, no death, no visual update.

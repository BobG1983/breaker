## Test Spec: Wall Domain — Unified Death Pipeline Migration

### Domain
src/walls/

### Overview

Migrate the wall domain to the unified death pipeline. This involves:
1. A wall kill handler system that processes `KillYourself<Wall>` messages
2. Adding `Hp` and `KilledBy` components to destructible wall builders
3. Permanent walls remain unaffected (no Hp, not queryable by damage/death systems)
4. Shield walls and second-wind walls (effect-spawned, with Hp) work through the death pipeline

---

### Behavior

#### Wall Kill Handler (`handle_wall_kill`)

The wall kill handler reads `KillYourself<Wall>` messages and performs domain-specific cleanup before confirming the kill. It inserts `Dead`, removes the wall from the spatial index, sends `Destroyed<Wall>` with position data, and sends `DespawnEntity`.

1. **Wall kill handler inserts Dead marker on victim**
   - Given: A wall entity with `Wall` component, `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, at position `Vec2::new(0.0, 300.0)`. A `KillYourself<Wall> { victim: wall_entity, killer: Some(bolt_entity) }` message is pending.
   - When: `handle_wall_kill` runs
   - Then: The wall entity has the `Dead` component inserted
   - Edge case: Wall entity that already has `Dead` component -- `KillYourself<Wall>` message is silently ignored (no double-processing, no panic)

2. **Wall kill handler sends Destroyed<Wall> with correct positions**
   - Given: A wall entity with `Wall` component at position `Vec2::new(-200.0, 300.0)`. A killer bolt entity at position `Vec2::new(-180.0, 280.0)`. A `KillYourself<Wall> { victim: wall_entity, killer: Some(bolt_entity) }` message is pending.
   - When: `handle_wall_kill` runs
   - Then: A `Destroyed<Wall>` message is sent with `victim: wall_entity`, `killer: Some(bolt_entity)`, `victim_pos: Vec2::new(-200.0, 300.0)`, `killer_pos: Some(Vec2::new(-180.0, 280.0))`
   - Edge case: Killer entity has been despawned before the kill handler runs -- `Destroyed<Wall>` is still sent with `killer: Some(bolt_entity)` and `killer_pos: None`

3. **Wall kill handler sends DespawnEntity for the victim**
   - Given: A wall entity with `Wall` component. A `KillYourself<Wall> { victim: wall_entity, killer: None }` message is pending.
   - When: `handle_wall_kill` runs
   - Then: A `DespawnEntity { entity: wall_entity }` message is sent
   - Edge case: Environmental kill (killer is None) -- `DespawnEntity` is still sent, `Destroyed<Wall>` has `killer: None` and `killer_pos: None`

4. **Wall kill handler removes wall from spatial index**
   - Given: A wall entity with `Wall` component registered in the spatial index (has `SpatialEntity` or equivalent spatial component). A `KillYourself<Wall> { victim: wall_entity, killer: Some(bolt_entity) }` message is pending.
   - When: `handle_wall_kill` runs
   - Then: The wall entity is removed from the spatial index (spatial component removed or entity deregistered) so collision queries no longer find it
   - Edge case: Wall entity has no spatial index entry (already removed) -- handler does not panic, continues normally

5. **Wall kill handler processes multiple kills in one frame**
   - Given: Two wall entities (wall_a at `Vec2::new(-200.0, 300.0)` and wall_b at `Vec2::new(200.0, 300.0)`), each with `Wall` component and Hp. Two `KillYourself<Wall>` messages, one per wall, both with `killer: Some(bolt_entity)`.
   - When: `handle_wall_kill` runs
   - Then: Both wall entities get `Dead` inserted. Two `Destroyed<Wall>` messages sent (one per wall with correct victim_pos). Two `DespawnEntity` messages sent.
   - Edge case: Same bolt kills both walls in one frame -- both `Destroyed<Wall>` messages correctly attribute the same killer entity

---

#### Destructible Wall Builder Hp/KilledBy

Destructible walls (walls that can be damaged and destroyed) must spawn with `Hp` and `KilledBy` components so the unified death pipeline can process them.

6. **Destructible wall spawns with Hp(1) and KilledBy(default)**
   - Given: A destructible wall definition (one-hit wall, e.g., a shield wall or second-wind wall)
   - When: The wall builder spawns the entity
   - Then: The entity has `Hp { current: 1.0, starting: 1.0, max: None }` and `KilledBy { dealer: None }` components
   - Edge case: Hp starting value matches current exactly at spawn time

7. **Destructible wall with higher Hp spawns correctly**
   - Given: A destructible wall definition with Hp(3) (multi-hit destructible wall)
   - When: The wall builder spawns the entity
   - Then: The entity has `Hp { current: 3.0, starting: 3.0, max: None }` and `KilledBy { dealer: None }` components
   - Edge case: Hp max is None -- no upper bound on healing for walls (matches Hp convention)

---

#### Permanent Wall: Unaffected by Death Pipeline

Permanent walls (side walls, top wall) have no Hp and are not matched by `apply_damage<Wall>` or `detect_wall_deaths`.

8. **Permanent wall has no Hp component**
   - Given: A permanent wall definition (side wall, top wall)
   - When: The wall builder spawns the entity
   - Then: The entity has `Wall` component but does NOT have `Hp` or `KilledBy` components
   - Edge case: Permanent wall is never matched by `Query<..., With<Wall>, Without<Dead>>` that also requires `&Hp` -- the missing `Hp` component excludes it from the query

9. **DamageDealt<Wall> targeting a permanent wall is silently dropped**
   - Given: A permanent wall entity with `Wall` component and no `Hp` component. A `DamageDealt<Wall> { target: permanent_wall_entity, amount: 10.0, dealer: Some(bolt_entity), source_chip: None }` message is pending.
   - When: `apply_damage::<Wall>` runs
   - Then: Nothing happens -- the permanent wall is not queryable (no `Hp` component) so the message is effectively dropped. No panic, no error, no log.
   - Edge case: Multiple `DamageDealt<Wall>` messages targeting the same permanent wall in one frame -- all silently dropped

10. **detect_wall_deaths does not process permanent walls**
    - Given: A permanent wall entity with `Wall` component, no `Hp` component, no `KilledBy` component
    - When: `detect_wall_deaths` runs
    - Then: No `KillYourself<Wall>` message is sent for the permanent wall. The system's query requires `&Hp` which excludes the permanent wall.
    - Edge case: Frame where destructible walls die but permanent walls exist -- only destructible walls produce `KillYourself<Wall>`, permanent walls are untouched

---

#### Shield Wall: Effect-Spawned Wall Through Death Pipeline

Shield walls are spawned by the Shield effect with `Hp(1)` (conceptually one-shot for the death pipeline path when destroyed by damage, though their primary expiry is timer-based via `tick_shield_duration`). When a shield wall is killed through the death pipeline (e.g., by a `Fire(Die)` effect), it flows through `KillYourself<Wall>` like any destructible wall.

11. **Shield wall with Hp works through apply_damage**
    - Given: A shield wall entity with `Wall`, `ShieldWall`, `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, at position `Vec2::new(0.0, -280.0)`. A `DamageDealt<Wall> { target: shield_wall_entity, amount: 1.0, dealer: Some(bolt_entity), source_chip: None }` message is pending.
    - When: `apply_damage::<Wall>` runs
    - Then: `Hp.current` is decremented to `0.0`. `KilledBy.dealer` is set to `Some(bolt_entity)`.
    - Edge case: Shield wall takes fractional damage (0.5) from an effect -- Hp decrements from 1.0 to 0.5, KilledBy is NOT set (not a killing blow)

12. **Shield wall death detected after Hp reaches zero**
    - Given: A shield wall entity with `Wall`, `ShieldWall`, `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, without `Dead`
    - When: `detect_wall_deaths` runs
    - Then: A `KillYourself<Wall> { victim: shield_wall_entity, killer: Some(bolt_entity) }` message is sent
    - Edge case: Shield wall already has `Dead` -- `detect_wall_deaths` skips it (Without<Dead> filter)

13. **Shield wall kill handler completes the death pipeline**
    - Given: A shield wall entity with `Wall`, `ShieldWall`, at position `Vec2::new(0.0, -280.0)`. A `KillYourself<Wall> { victim: shield_wall_entity, killer: Some(bolt_entity) }` message is pending. Bolt entity at position `Vec2::new(10.0, -260.0)`.
    - When: `handle_wall_kill` runs
    - Then: `Dead` inserted on shield wall, `Destroyed<Wall> { victim: shield_wall_entity, killer: Some(bolt_entity), victim_pos: Vec2::new(0.0, -280.0), killer_pos: Some(Vec2::new(10.0, -260.0)) }` sent, `DespawnEntity { entity: shield_wall_entity }` sent
    - Edge case: Shield wall expires via timer before kill handler processes KillYourself -- if entity is already despawned, handler should not panic (use defensive entity access)

---

#### Destructible Wall End-to-End: Full Pipeline

14. **Destructible wall e2e: Hp(1), DamageDealt<Wall>, full pipeline to despawn**
    - Given: A destructible wall entity with `Wall`, `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, at position `Vec2::new(100.0, 300.0)`. A bolt entity at position `Vec2::new(100.0, 280.0)`.
    - When: `DamageDealt<Wall> { target: wall_entity, amount: 1.0, dealer: Some(bolt_entity), source_chip: None }` is sent
    - Then (after apply_damage): `Hp.current == 0.0`, `KilledBy.dealer == Some(bolt_entity)`
    - Then (after detect_wall_deaths): `KillYourself<Wall> { victim: wall_entity, killer: Some(bolt_entity) }` sent
    - Then (after handle_wall_kill): `Dead` inserted, wall removed from spatial index, `Destroyed<Wall>` sent with `victim_pos: Vec2::new(100.0, 300.0)` and `killer_pos: Some(Vec2::new(100.0, 280.0))`, `DespawnEntity` sent
    - Then (after process_despawn_requests): wall entity is despawned from the world
    - Edge case: Bolt despawns between damage and despawn processing -- pipeline still completes, killer_pos may be None in Destroyed<Wall> if bolt is gone by kill handler time

15. **Destructible wall e2e: Hp(3), three damage messages, dies on third**
    - Given: A destructible wall entity with `Wall`, `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, at position `Vec2::new(-100.0, 300.0)`. Bolt entities bolt_a, bolt_b, bolt_c.
    - When: Three `DamageDealt<Wall>` messages are sent in sequence across three frames:
      - Frame 1: `{ target: wall_entity, amount: 1.0, dealer: Some(bolt_a) }` -- Hp goes to 2.0
      - Frame 2: `{ target: wall_entity, amount: 1.0, dealer: Some(bolt_b) }` -- Hp goes to 1.0
      - Frame 3: `{ target: wall_entity, amount: 1.0, dealer: Some(bolt_c) }` -- Hp goes to 0.0, killing blow
    - Then (frame 3): `KilledBy.dealer == Some(bolt_c)` (third bolt gets the kill credit). `detect_wall_deaths` sends `KillYourself<Wall>`. Kill handler completes the pipeline with `killer: Some(bolt_c)`.
    - Edge case: Two damage messages in the same frame total enough to kill -- only the first message that crosses zero sets KilledBy (first kill wins)

16. **Dead prevents double-processing of already-dead wall**
    - Given: A destructible wall entity with `Wall`, `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, and `Dead` component already inserted. A `DamageDealt<Wall> { target: wall_entity, amount: 1.0, dealer: Some(other_bolt) }` message is pending.
    - When: `apply_damage::<Wall>` runs, then `detect_wall_deaths` runs
    - Then: Neither system processes this wall entity -- the `Without<Dead>` filter excludes it from both queries. No additional `KillYourself<Wall>` message is sent.
    - Edge case: Dead wall that also receives a `KillYourself<Wall>` message (duplicate kill request) -- the kill handler should check for Dead and skip the entity (idempotent)

---

### Types

All types below are defined by the unified death pipeline and should already exist from earlier waves. No new types are created by this wave.

- `Hp { current: f32, starting: f32, max: Option<f32> }` -- `#[derive(Component, Debug, Clone)]` -- unified health component. From `src/shared/components/`.
- `KilledBy { dealer: Option<Entity> }` -- `#[derive(Component, Default, Debug)]` -- kill attribution. From `src/shared/components/`.
- `Dead` -- `#[derive(Component)]` -- death marker. From `src/shared/components/`.
- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` -- `#[derive(Message, Clone, Debug)]` -- generic damage message. From `src/shared/messages.rs`.
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` -- `#[derive(Message, Clone, Debug)]` -- death request. From `src/shared/messages.rs`.
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` -- `#[derive(Message, Clone, Debug)]` -- death confirmed. From `src/shared/messages.rs`.
- `DespawnEntity { entity: Entity }` -- `#[derive(Message, Clone, Debug)]` -- deferred despawn. From `src/shared/messages.rs`.
- `GameEntity` -- trait bound, `Wall` implements it. From `src/shared/`.
- `Wall` -- `#[derive(Component)]` -- entity marker. From `src/walls/components.rs`.
- `ShieldWall` -- `#[derive(Component)]` -- marker for shield wall entities. From `src/effect/effects/shield/`.
- `SecondWindWall` -- `#[derive(Component)]` -- marker for second-wind wall entities. From `src/effect/effects/second_wind/`.

### Messages

No new message types. All messages used (`DamageDealt<Wall>`, `KillYourself<Wall>`, `Destroyed<Wall>`, `DespawnEntity`) are generic instantiations of existing pipeline types created in Wave 7.

### Reference Files

- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/apply-damage-wall.md` -- behavioral spec for `apply_damage::<Wall>`
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/detect-wall-deaths.md` -- behavioral spec for `detect_wall_deaths`
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-sets.md` -- system set ordering for the death pipeline
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` -- full frame ordering
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/death/on_destroyed_wall.md` -- death bridge (downstream consumer of Destroyed<Wall>)

For test patterns, follow the test structure established in Wave 7 (death pipeline systems) and Wave 9 (cell domain migration) as those are the closest analogues:
- Wave 7 tests for `apply_damage::<T>` and `detect_*_deaths` -- follow the same pattern for wall-specific tests
- Wave 9 tests for the cell kill handler -- follow the same pattern for the wall kill handler

### Scenario Coverage

- New invariants: none -- `ShieldWallAtMostOne` and `SecondWindWallAtMostOne` already exist and cover shield/second-wind wall lifecycle. `NoEntityLeaks` covers despawn correctness. `AabbMatchesEntityDimensions` covers spatial index consistency.
- New scenarios: none -- existing scenarios that include shield walls and destructible walls exercise the death pipeline path. The migration is behavioral-preserving; existing scenario coverage validates the pipeline produces identical observable behavior.
- Self-test scenarios: none -- all relevant invariant kinds already have self-tests
- Layout updates: none -- existing layouts already include wall configurations. No new wall types are introduced.

### Constraints

- Tests for the wall kill handler go in: `src/walls/systems/handle_wall_kill.rs` (new file -- system + tests, or directory module if tests exceed threshold)
- Tests for destructible wall builder Hp/KilledBy go in: the existing wall builder test file (wherever the wall builder is defined in `src/walls/`)
- Tests for permanent wall exclusion go alongside the wall kill handler tests or builder tests as appropriate
- Tests for shield wall / second-wind wall through-pipeline go in the wall kill handler test file (they exercise the same kill handler)
- Do NOT test: `apply_damage::<Wall>` or `detect_wall_deaths` in isolation -- those are generic pipeline systems tested in Wave 7. This wave tests the wall-domain-specific kill handler and builder changes.
- Do NOT test: `on_destroyed::<Wall>` (death bridge to effect triggers) -- that is an effect domain concern tested in the trigger bridge wave.
- Do NOT test: `tick_shield_duration` or shield timer expiry -- that is the shield effect's concern, not the death pipeline.
- Do NOT test: bolt-wall collision producing `DamageDealt<Wall>` -- that is the collision system's concern, tested in its own wave.
- Do NOT modify: any existing test files outside `src/walls/`
- The e2e tests (behaviors 14-16) are integration tests that compose the full pipeline: they set up a minimal app with `apply_damage::<Wall>`, `detect_wall_deaths`, `handle_wall_kill`, and `process_despawn_requests` to verify the chain from damage to despawn.

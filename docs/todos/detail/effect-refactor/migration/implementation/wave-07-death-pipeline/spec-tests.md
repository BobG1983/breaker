## Test Spec: Shared/Cells/Bolt/Walls/Breaker — Death Pipeline Systems

This spec covers the core death pipeline: `apply_damage<T>`, `detect_*_deaths`, `process_despawn_requests`, and `Dead` marker component integration. Domain kill handlers (wave 9-12), trigger bridges, and effects are out of scope.

### Prerequisites

This wave assumes all types and systems from waves 2-6 already exist:

- **Wave 2 (Scaffold)**: All death pipeline types are created as stubs — `GameEntity` trait, `Hp`, `KilledBy`, `Dead`, `Locked`, `DamageDealt<T>`, `KillYourself<T>`, `DespawnEntity`, `Destroyed<T>`, `DamageTargetData`, `DeathDetectionData`, `DeathPipelineSystems` system set enum. Entity marker components (`Bolt`, `Cell`, `Wall`, `Breaker`) already exist from the original codebase. Plugin wiring stubs for `DeathPipelinePlugin` are in place.
- **Wave 3 (RON Assets)**: RON asset migration complete. No direct dependency for this wave but the build must compile.
- **Wave 4 (Functions)**: `EffectStack`, walking algorithm, dispatch, command extensions, passive effects — all implemented and tested.
- **Wave 5 (Triggers)**: All trigger bridge systems implemented and tested. `EffectSystems::Tick` system set exists and is configured.
- **Wave 6 (Effects)**: All 30 effects implemented and tested. Tick systems and condition evaluators operational. Effects that send `DamageDealt<T>` messages (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam) are operational — though their messages are not yet consumed until this wave implements `apply_damage`.

All types listed in the Types section below were created in wave 2. This wave implements the **systems** that use them, not the types themselves.

---

### Part A: apply_damage Generic System

#### Domain
`src/shared/systems/apply_damage.rs` — one generic system, monomorphized per GameEntity.

#### Behavior

1. **apply_damage<Cell> decrements Hp by damage amount**
   - Given: Cell entity with `Hp { current: 30.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` is `20.0`. `KilledBy.dealer` remains `None` (not a killing blow).
   - Edge case: `DamageDealt` with `amount: 0.0` — Hp remains `30.0`, no KilledBy set.

2. **apply_damage<Cell> sets KilledBy on killing blow**
   - Given: Cell entity with `Hp { current: 5.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` is `-5.0`. `KilledBy.dealer` is `Some(bolt_entity)`.
   - Edge case: Damage exactly equals remaining Hp (`amount: 5.0`) — `Hp.current` is `0.0`, `KilledBy.dealer` is `Some(bolt_entity)`.

3. **apply_damage<Cell> first kill wins — KilledBy not overwritten**
   - Given: Cell entity with `Hp { current: 10.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`. Two `DamageDealt<Cell>` messages pending: first `{ dealer: Some(bolt_a), target: cell_entity, amount: 10.0, source_chip: None }`, second `{ dealer: Some(bolt_b), target: cell_entity, amount: 5.0, source_chip: None }`.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` is `-5.0`. `KilledBy.dealer` is `Some(bolt_a)` (first message killed it; second message's dealer is NOT written).
   - Edge case: First message is the killing blow, second message is overkill on an already-dead entity (`amount: 15.0` and `amount: 20.0` with `Hp.current: 10.0`). Messages are processed sequentially: first message brings Hp from 10.0 to -5.0 (killing blow, sets KilledBy to bolt_a). Second message brings Hp from -5.0 to -25.0 — hp_before is already negative so this is NOT a killing blow, KilledBy is NOT overwritten. Result: `KilledBy.dealer` is `Some(bolt_a)`.

4. **apply_damage<Cell> processes multiple damage messages in one frame**
   - Given: Cell entity with `Hp { current: 30.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`. Three `DamageDealt<Cell>` messages for same target: amounts `5.0`, `10.0`, `8.0`.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` is `7.0` (30.0 - 5.0 - 10.0 - 8.0). `KilledBy.dealer` remains `None` (no killing blow).

5. **apply_damage<Cell> skips Locked entities**
   - Given: Cell entity with `Hp { current: 30.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component, AND `Locked` component. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` remains `30.0`. `KilledBy.dealer` remains `None`. The damage is silently dropped.

6. **apply_damage<Cell> skips Dead entities**
   - Given: Cell entity with `Hp { current: -5.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(old_killer) }`, `Cell` component, AND `Dead` component. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Cell>` runs
   - Then: Cell's `Hp.current` remains `-5.0`. `KilledBy.dealer` remains `Some(old_killer)`. The damage is silently dropped.

7. **apply_damage<Cell> skips messages for non-existent targets**
   - Given: No Cell entity exists matching the target in the `DamageDealt<Cell>` message. `DamageDealt<Cell> { dealer: Some(bolt_entity), target: Entity::from_raw(999), amount: 10.0, source_chip: None }`.
   - When: `apply_damage::<Cell>` runs
   - Then: No panic. No entity is modified.

8. **apply_damage<Bolt> decrements Hp and sets KilledBy on killing blow**
   - Given: Bolt entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component. A `DamageDealt<Bolt> { dealer: None, target: bolt_entity, amount: 1.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Bolt>` runs
   - Then: Bolt's `Hp.current` is `0.0`. `KilledBy.dealer` is `None` (environmental death — dealer was None).
   - Edge case: Bolt with no Hp component — not queryable, system has no effect.

9. **apply_damage<Bolt> skips Dead bolts**
   - Given: Bolt entity with `Hp { current: -1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component, AND `Dead` component. A `DamageDealt<Bolt> { dealer: None, target: bolt_entity, amount: 1.0, source_chip: None }` message is pending.
   - When: `apply_damage::<Bolt>` runs
   - Then: Bolt's `Hp.current` remains `-1.0`. Damage silently dropped.

10. **apply_damage<Wall> decrements Hp and sets KilledBy on killing blow**
    - Given: Wall entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Wall` component. A `DamageDealt<Wall> { dealer: Some(bolt_entity), target: wall_entity, amount: 1.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Wall>` runs
    - Then: Wall's `Hp.current` is `0.0`. `KilledBy.dealer` is `Some(bolt_entity)`.
    - Edge case: Wall without Hp component (permanent wall) — not queryable, system has no effect.

11. **apply_damage<Wall> skips Dead walls**
    - Given: Wall entity with `Hp { current: -1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Wall` component, AND `Dead` component. A `DamageDealt<Wall> { dealer: Some(bolt_entity), target: wall_entity, amount: 1.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Wall>` runs
    - Then: Wall's `Hp.current` remains `-1.0`. Damage silently dropped.

12. **apply_damage<Breaker> decrements Hp (life lost)**
    - Given: Breaker entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component. A `DamageDealt<Breaker> { dealer: None, target: breaker_entity, amount: 1.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Breaker>` runs
    - Then: Breaker's `Hp.current` is `2.0`. `KilledBy.dealer` remains `None` (not a killing blow).

13. **apply_damage<Breaker> sets KilledBy on killing blow (last life)**
    - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component. A `DamageDealt<Breaker> { dealer: None, target: breaker_entity, amount: 1.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Breaker>` runs
    - Then: Breaker's `Hp.current` is `0.0`. `KilledBy.dealer` is `None` (environmental death — bolt loss is unattributed).

14. **apply_damage<Breaker> skips Dead breakers**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component, AND `Dead` component. A `DamageDealt<Breaker> { dealer: None, target: breaker_entity, amount: 1.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Breaker>` runs
    - Then: Breaker's `Hp.current` remains `0.0`. Damage silently dropped.

15. **apply_damage does NOT despawn, does NOT send KillYourself**
    - Given: Cell entity with `Hp { current: 5.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending (killing blow).
    - When: `apply_damage::<Cell>` runs
    - Then: The cell entity still exists in the world (not despawned). No `KillYourself<Cell>` message is produced. Only Hp and KilledBy are modified.

16. **apply_damage does NOT modify Hp.starting or Hp.max**
    - Given: Cell entity with `Hp { current: 30.0, starting: 30.0, max: Some(50.0) }`, `KilledBy { dealer: None }`. A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Cell>` runs
    - Then: `Hp.current` is `20.0`. `Hp.starting` remains `30.0`. `Hp.max` remains `Some(50.0)`.

17. **apply_damage does NOT set KilledBy when Hp is already at 0.0 before damage**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component (NO `Dead` component). A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 5.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Cell>` runs
    - Then: `Hp.current` is `-5.0`. `KilledBy.dealer` remains `None`. The killing blow check requires `hp_before > 0.0` — since `hp_before` was `0.0` (not positive), this is NOT a killing blow and KilledBy is not set.
    - Edge case: `Hp.current` is `-3.0` before damage (already negative, no Dead yet) — `hp_before` is negative, damage brings it to `-8.0`, still NOT a killing blow, KilledBy not set.

---

### Part B: detect_cell_deaths

#### Domain
`src/cells/systems/detect_cell_deaths.rs`

**Scope note on RequiredToClear**: The design doc for `detect_cell_deaths` includes `Has<RequiredToClear>` in the query for downstream use. However, `detect_cell_deaths` in this wave does NOT read `Has<RequiredToClear>`. That is deferred to wave 9 where the cell domain kill handler reads it from the still-alive entity (the entity survives through the kill handler before being despawned in PostFixedUpdate). Tests in this wave do not set up or assert on `RequiredToClear`.

#### Behavior

18. **detect_cell_deaths sends KillYourself<Cell> when Hp <= 0**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Cell` component.
    - When: `detect_cell_deaths` runs
    - Then: A `KillYourself<Cell> { victim: cell_entity, killer: Some(bolt_entity) }` message is sent.
    - Edge case: `Hp.current` is `-5.0` (overkill) — still sends `KillYourself<Cell>`.

19. **detect_cell_deaths does NOT send KillYourself when Hp > 0**
    - Given: Cell entity with `Hp { current: 10.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component.
    - When: `detect_cell_deaths` runs
    - Then: No `KillYourself<Cell>` message is sent.

20. **detect_cell_deaths skips Dead cells**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Cell` component, AND `Dead` component.
    - When: `detect_cell_deaths` runs
    - Then: No `KillYourself<Cell>` message is sent (entity already dead, filtered out by `Without<Dead>`).

21. **detect_cell_deaths does NOT insert Dead component**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Cell` component.
    - When: `detect_cell_deaths` runs
    - Then: The cell entity does NOT have a `Dead` component after the system runs. (Domain kill handler inserts Dead, not this system.)

22. **detect_cell_deaths does NOT despawn the entity**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Cell` component.
    - When: `detect_cell_deaths` runs
    - Then: The cell entity still exists in the world.

23. **detect_cell_deaths sends KillYourself with killer=None for environmental kills**
    - Given: Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component.
    - When: `detect_cell_deaths` runs
    - Then: A `KillYourself<Cell> { victim: cell_entity, killer: None }` message is sent.

24. **detect_cell_deaths processes multiple dead cells in one frame**
    - Given: Cell A with `Hp { current: 0.0, ... }`, `KilledBy { dealer: Some(bolt_a) }`, `Cell`. Cell B with `Hp { current: -3.0, ... }`, `KilledBy { dealer: Some(bolt_b) }`, `Cell`. Cell C with `Hp { current: 15.0, ... }`, `KilledBy { dealer: None }`, `Cell`.
    - When: `detect_cell_deaths` runs
    - Then: `KillYourself<Cell>` sent for Cell A (killer: Some(bolt_a)) and Cell B (killer: Some(bolt_b)). No message for Cell C (still alive).

25. **detect_cell_deaths ignores Cell entity without Hp component**
    - Given: Cell entity with `Cell` component but NO `Hp` component and NO `KilledBy` component (e.g., a special non-damageable cell type). No `Dead` component.
    - When: `detect_cell_deaths` runs
    - Then: No panic. No `KillYourself<Cell>` message sent. The entity is not matched by the query (which requires `Hp` and `KilledBy`).

---

### Part C: detect_bolt_deaths

#### Domain
`src/bolt/systems/detect_bolt_deaths.rs`

#### Behavior

26. **detect_bolt_deaths sends KillYourself<Bolt> when Hp <= 0**
    - Given: Bolt entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component.
    - When: `detect_bolt_deaths` runs
    - Then: A `KillYourself<Bolt> { victim: bolt_entity, killer: None }` message is sent.
    - Edge case: `Hp.current` is `-1.0` — still sends `KillYourself<Bolt>`.

27. **detect_bolt_deaths does NOT send KillYourself when Hp > 0**
    - Given: Bolt entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component.
    - When: `detect_bolt_deaths` runs
    - Then: No `KillYourself<Bolt>` message is sent.

28. **detect_bolt_deaths skips Dead bolts**
    - Given: Bolt entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component, AND `Dead` component.
    - When: `detect_bolt_deaths` runs
    - Then: No `KillYourself<Bolt>` message is sent.

29. **detect_bolt_deaths does NOT insert Dead or despawn**
    - Given: Bolt entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Bolt` component.
    - When: `detect_bolt_deaths` runs
    - Then: Bolt entity still exists, no `Dead` component inserted.

30. **detect_bolt_deaths ignores Bolt entity without Hp component**
    - Given: Bolt entity with `Bolt` component but NO `Hp` component and NO `KilledBy` component. No `Dead` component.
    - When: `detect_bolt_deaths` runs
    - Then: No panic. No `KillYourself<Bolt>` message sent. The entity is not matched by the query (which requires `Hp` and `KilledBy`).

---

### Part D: detect_wall_deaths

#### Domain
`src/walls/systems/detect_wall_deaths.rs`

#### Behavior

31. **detect_wall_deaths sends KillYourself<Wall> when Hp <= 0**
    - Given: Wall entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Wall` component.
    - When: `detect_wall_deaths` runs
    - Then: A `KillYourself<Wall> { victim: wall_entity, killer: Some(bolt_entity) }` message is sent.

32. **detect_wall_deaths does NOT send KillYourself when Hp > 0**
    - Given: Wall entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Wall` component.
    - When: `detect_wall_deaths` runs
    - Then: No `KillYourself<Wall>` message is sent.

33. **detect_wall_deaths skips Dead walls**
    - Given: Wall entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Wall` component, AND `Dead` component.
    - When: `detect_wall_deaths` runs
    - Then: No `KillYourself<Wall>` message is sent.

34. **detect_wall_deaths does NOT insert Dead or despawn**
    - Given: Wall entity with `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Wall` component.
    - When: `detect_wall_deaths` runs
    - Then: Wall entity still exists, no `Dead` component inserted.

35. **detect_wall_deaths ignores Wall entity without Hp component (permanent wall)**
    - Given: Wall entity with `Wall` component but NO `Hp` component and NO `KilledBy` component (permanent, indestructible wall). No `Dead` component.
    - When: `detect_wall_deaths` runs
    - Then: No panic. No `KillYourself<Wall>` message sent. The entity is not matched by the query (which requires `Hp` and `KilledBy`).

---

### Part E: detect_breaker_deaths

#### Domain
`src/breaker/systems/detect_breaker_deaths.rs`

#### Behavior

36. **detect_breaker_deaths sends KillYourself<Breaker> when Hp <= 0**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component.
    - When: `detect_breaker_deaths` runs
    - Then: A `KillYourself<Breaker> { victim: breaker_entity, killer: None }` message is sent.

37. **detect_breaker_deaths does NOT send KillYourself when Hp > 0**
    - Given: Breaker entity with `Hp { current: 2.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component.
    - When: `detect_breaker_deaths` runs
    - Then: No `KillYourself<Breaker>` message is sent.

38. **detect_breaker_deaths skips Dead breakers**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component, AND `Dead` component.
    - When: `detect_breaker_deaths` runs
    - Then: No `KillYourself<Breaker>` message is sent.

39. **detect_breaker_deaths does NOT insert Dead or despawn**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, `Breaker` component.
    - When: `detect_breaker_deaths` runs
    - Then: Breaker entity still exists, no `Dead` component inserted.

40. **detect_breaker_deaths ignores Breaker entity without Hp component (infinite lives)**
    - Given: Breaker entity with `Breaker` component but NO `Hp` component and NO `KilledBy` component (infinite lives breaker — `LivesCount(None)` mapped to no Hp component). No `Dead` component.
    - When: `detect_breaker_deaths` runs
    - Then: No panic. No `KillYourself<Breaker>` message sent. The entity is not matched by the query (which requires `Hp` and `KilledBy`).

---

### Part F: process_despawn_requests

#### Domain
`src/shared/systems/process_despawn_requests.rs`

#### Behavior

41. **process_despawn_requests despawns entities from DespawnEntity messages**
    - Given: Entity A exists in the world. A `DespawnEntity { entity: entity_a }` message is pending.
    - When: `process_despawn_requests` runs (and commands are applied)
    - Then: Entity A no longer exists in the world.

42. **process_despawn_requests handles multiple DespawnEntity messages**
    - Given: Entity A and Entity B exist in the world. Two `DespawnEntity` messages pending: one for entity_a, one for entity_b.
    - When: `process_despawn_requests` runs (and commands are applied)
    - Then: Neither Entity A nor Entity B exist in the world.

43. **process_despawn_requests uses try_despawn (not try_despawn_recursive) — no panic on already-despawned entity**
    - Given: Entity A does NOT exist in the world (already despawned or never existed). A `DespawnEntity { entity: entity_a }` message is pending.
    - When: `process_despawn_requests` runs (and commands are applied)
    - Then: No panic. System completes normally.

44. **process_despawn_requests handles duplicate DespawnEntity for same entity**
    - Given: Entity A exists in the world. Two `DespawnEntity { entity: entity_a }` messages are pending.
    - When: `process_despawn_requests` runs (and commands are applied)
    - Then: Entity A no longer exists. No panic from the second despawn (try_despawn).

45. **process_despawn_requests with no messages is a no-op**
    - Given: Entity A exists in the world. No `DespawnEntity` messages pending.
    - When: `process_despawn_requests` runs
    - Then: Entity A still exists. Nothing happens.

---

### Part G: Dead Component Integration (Cross-System)

#### Domain
`src/shared/` (component) tested via cross-system integration

#### Behavior

46. **Dead prevents apply_damage from processing further damage**
    - Given: Cell entity with `Hp { current: -5.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_a) }`, `Cell`, `Dead`. A `DamageDealt<Cell> { dealer: Some(bolt_b), target: cell_entity, amount: 10.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Cell>` runs
    - Then: `Hp.current` remains `-5.0`. `KilledBy.dealer` remains `Some(bolt_a)`.

47. **Dead prevents detect_cell_deaths from re-sending KillYourself**
    - Given: Cell entity with `Hp { current: -5.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_a) }`, `Cell`, `Dead`.
    - When: `detect_cell_deaths` runs
    - Then: No `KillYourself<Cell>` message is sent.

48. **Entity without Dead IS processed by apply_damage (positive check)**
    - Given: Cell entity with `Hp { current: 20.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`, `Cell` (NO `Dead` component). A `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 5.0, source_chip: None }` message is pending.
    - When: `apply_damage::<Cell>` runs
    - Then: `Hp.current` is `15.0`.

49. **Entity without Dead IS processed by detect_cell_deaths (positive check)**
    - Given: Cell entity with `Hp { current: -5.0, starting: 30.0, max: None }`, `KilledBy { dealer: Some(bolt_entity) }`, `Cell` (NO `Dead` component).
    - When: `detect_cell_deaths` runs
    - Then: `KillYourself<Cell>` IS sent for this entity.

---

### Types

All types below were created in wave 2 (scaffold). They are listed here for reference — this wave implements systems that use them, not the types themselves:

- `Hp { current: f32, starting: f32, max: Option<f32> }` — `#[derive(Component, Debug, Clone)]`. Unified health. Location: `src/shared/components/hp.rs` (directory module under `src/shared/components/`).
- `KilledBy { dealer: Option<Entity> }` — `#[derive(Component, Default, Debug)]`. Kill attribution. Location: `src/shared/components/killed_by.rs` (directory module under `src/shared/components/`).
- `Dead` — `#[derive(Component)]`. Death marker. Location: `src/shared/components/dead.rs` (directory module under `src/shared/components/`).
- `GameEntity` — marker trait: `trait GameEntity: Component {}` with impls for `Bolt`, `Cell`, `Wall`, `Breaker`. Location: `src/shared/traits.rs`.
- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]`. Location: `src/shared/messages.rs`.
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]`. Location: `src/shared/messages.rs`.
- `DespawnEntity { entity: Entity }` — `#[derive(Message, Clone, Debug)]`. Location: `src/shared/messages.rs`.
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]`. NOT tested in this wave (domain kill handlers send it, wave 9-12).
- `DamageTargetData` — `#[derive(QueryData)] #[query_data(mutable)]` with fields `hp: &'static mut Hp`, `killed_by: &'static mut KilledBy`. Location: `src/shared/queries.rs`.
- `DeathDetectionData` — `#[derive(QueryData)]` (read-only) with fields `entity: Entity`, `killed_by: &'static KilledBy`, `hp: &'static Hp`. Location: `src/shared/queries.rs`.
- `DeathPipelineSystems` — `#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]` enum with variants `ApplyDamage`, `DetectDeaths`. Location: `src/shared/sets.rs`.

### Messages

- `DamageDealt<Cell> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` — sent by collision systems and effects, consumed by `apply_damage::<Cell>`.
- `DamageDealt<Bolt> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` — sent by future bolt-damage mechanics, consumed by `apply_damage::<Bolt>`.
- `DamageDealt<Wall> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` — sent by collision systems, consumed by `apply_damage::<Wall>`.
- `DamageDealt<Breaker> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` — sent by LoseLife effect, consumed by `apply_damage::<Breaker>`.
- `KillYourself<Cell> { victim: Entity, killer: Option<Entity> }` — sent by `detect_cell_deaths`, consumed by domain kill handler (wave 9).
- `KillYourself<Bolt> { victim: Entity, killer: Option<Entity> }` — sent by `detect_bolt_deaths`, consumed by domain kill handler (wave 10).
- `KillYourself<Wall> { victim: Entity, killer: Option<Entity> }` — sent by `detect_wall_deaths`, consumed by domain kill handler (wave 11).
- `KillYourself<Breaker> { victim: Entity, killer: Option<Entity> }` — sent by `detect_breaker_deaths`, consumed by domain kill handler (wave 12).
- `DespawnEntity { entity: Entity }` — sent by domain kill handlers (wave 9-12), consumed by `process_despawn_requests`.

### Reference Files

- `docs/todos/detail/unified-death-pipeline/rust-types/` — all type definitions (authoritative)
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/` — all system specs (authoritative)
- `docs/todos/detail/unified-death-pipeline/migration/query-data-to-create/` — QueryData specs (authoritative)
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-sets.md` — set ordering (authoritative)

### Scenario Coverage

- New invariants: none — death pipeline correctness is validated through unit tests. Scenario invariants for entity lifecycle (like `NoEntityLeaks`) already exist and will catch despawn bugs.
- New scenarios: none — existing chaos scenarios exercise cell damage and death paths. Once domain kill handlers (wave 9-12) are in place, scenarios will exercise the full chain end-to-end.
- Self-test scenarios: none needed for this wave.
- Layout updates: none.

### Constraints

- Tests for `apply_damage::<T>` go in: `src/shared/systems/apply_damage.rs` (system + tests in same file, or split per file-splitting rules if needed)
- Tests for `detect_cell_deaths` go in: `src/cells/systems/detect_cell_deaths.rs`
- Tests for `detect_bolt_deaths` go in: `src/bolt/systems/detect_bolt_deaths.rs`
- Tests for `detect_wall_deaths` go in: `src/walls/systems/detect_wall_deaths.rs`
- Tests for `detect_breaker_deaths` go in: `src/breaker/systems/detect_breaker_deaths.rs`
- Tests for `process_despawn_requests` go in: `src/shared/systems/process_despawn_requests.rs`
- Tests for Dead integration can go in `src/shared/systems/apply_damage.rs` (behaviors 46-49 are tested there and in the relevant detect_*_deaths file)
- **Changed\<Hp\> is deliberately omitted**: detect systems do NOT use `Changed<Hp>` — they check all non-Dead entities with `Hp <= 0` each frame. `Without<Dead>` is the only filter beyond the entity marker (`With<Cell>`, `With<Bolt>`, etc.). This is intentional: `Changed<Hp>` would miss entities whose Hp was set to zero before they entered the world (e.g., spawned dead), and the cost of scanning all entities with Hp is negligible given the entity counts in this game.
- **process_despawn_requests uses `try_despawn` not `try_despawn_recursive`**: Death pipeline entities have no children requiring recursive despawn. The system calls `commands.entity(msg.entity).try_despawn()`, not `try_despawn_recursive()`.
- Do NOT test: domain kill handlers, trigger bridges, Destroyed<T> dispatch, death animations, visual feedback, node completion tracking
- Do NOT test: scheduling/ordering — ordering is wiring, tested by the plugin, not by unit tests
- Do NOT reference: existing `src/` code. This is a clean-room implementation. All types and systems are new.

### Test Pattern Notes

Each test should:
1. Create a minimal Bevy `App` with `MinimalPlugins`
2. Register the message types needed (`app.add_message::<DamageDealt<Cell>>()`, etc.)
3. Spawn entities with the required components
4. Send messages via `MessageWriter`
5. Run the system under test
6. Assert component values and/or read messages via `MessageReader`

For `apply_damage::<T>` tests: spawn the entity, send `DamageDealt<T>` via `MessageWriter`, run the system, then query Hp and KilledBy to verify.

For `detect_*_deaths` tests: spawn the entity with Hp already at or below zero and KilledBy already set, run the system, then read `KillYourself<T>` via `MessageReader` to verify.

For `process_despawn_requests` tests: spawn an entity, send `DespawnEntity` via `MessageWriter`, run the system, apply commands, then verify the entity no longer exists via `world.get_entity(entity)`.

For `Dead` integration tests: spawn entity WITH `Dead` component, verify the system skips it. These are covered inline in the apply_damage and detect_*_deaths test files — no separate test file needed.

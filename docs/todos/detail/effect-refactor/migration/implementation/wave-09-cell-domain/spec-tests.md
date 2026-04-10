## Test Spec: Cell Domain — Unified Death Pipeline Migration

### Domain
src/cells/

### Prerequisites

The following waves must be complete before wave 9 tests can compile and run:

- **Wave 2 (scaffold)**: All shared types exist — `Hp`, `KilledBy`, `Dead`, `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity`, `GameEntity` trait, `Cell` impl of `GameEntity`
- **Wave 7 (death pipeline)**: `apply_damage::<Cell>`, `detect_cell_deaths`, `process_despawn_requests` are implemented and tested. `DamageTargetData` and `DeathDetectionData` query types exist in `src/shared/queries.rs`. The `DeathPipelineSystems::ApplyDamage` and `DeathPipelineSystems::DetectDeaths` system sets exist.
- **Wave 5 (triggers)**: `on_destroyed::<Cell>` effect bridge exists (consumes `Destroyed<Cell>`)

### Overview

Wave 9 migrates the cell domain from the old damage/cleanup model (`handle_cell_hit` + `cleanup_cell`) to the unified death pipeline. The old model combined damage application, visual feedback, death detection, spatial index removal, and node completion tracking into two monolithic systems. The new model splits these into discrete pipeline stages: `apply_damage::<Cell>` (shared, generic), `detect_cell_deaths` (cell-specific), a cell domain kill handler (cell-specific), and a separate damage visual feedback system. The `bolt_cell_collision` system is updated to send `DamageDealt<Cell>` instead of `DamageCell`. Cell entities gain `Hp` and `KilledBy` components.

This spec covers the cell domain's participation in the death pipeline. It does NOT cover `apply_damage::<Cell>` or `detect_cell_deaths` directly — those are tested in wave 07 (death pipeline systems). This spec covers: the cell kill handler, `bolt_cell_collision` sending `DamageDealt<Cell>`, cell builder producing Hp/KilledBy, Locked cell interaction with the pipeline, RequiredToClear tracking through the kill handler, cell damage visual system, and end-to-end cell death flows.

---

### Section A: Cell Kill Handler

The cell kill handler is a new system in the cell domain. It reads `KillYourself<Cell>` messages and performs cell-specific death logic: inserts `Dead`, removes the cell from the spatial index, updates RequiredToClear tracking, sends `Destroyed<Cell>`, and sends `DespawnEntity`.

File: `src/cells/systems/handle_cell_kill.rs`

#### Behavior

1. **Kill handler inserts Dead on the victim entity**
   - Given: A Cell entity (entity_id=E1) at position (100.0, 200.0) with `Hp { current: 0.0, starting: 30.0, max: None }` and `KilledBy { dealer: Some(bolt_entity) }`
   - When: `KillYourself<Cell> { victim: E1, killer: Some(bolt_entity) }` is processed by the cell kill handler
   - Then: Entity E1 has the `Dead` component inserted
   - Edge case: Entity E1 already has `Dead` (was killed by Die effect in same frame) — the kill handler should skip it entirely (no double-processing). Verify no `Destroyed<Cell>` or `DespawnEntity` is sent for an already-Dead entity.

2. **Kill handler removes cell from spatial index**
   - Given: A Cell entity E1 at position (100.0, 200.0) registered in the spatial index (quadtree)
   - When: `KillYourself<Cell> { victim: E1, killer: Some(bolt_entity) }` is processed
   - Then: Entity E1 is removed from the spatial index. Subsequent spatial queries at (100.0, 200.0) do not return E1.
   - Edge case: Entity E1 is not in the spatial index (already removed by another system) — the kill handler should not panic, should proceed with remaining cleanup.

3. **Kill handler sends Destroyed<Cell> with correct positions**
   - Given: A Cell entity E1 at position (100.0, 200.0) and a Bolt entity (the killer) at position (80.0, 180.0)
   - When: `KillYourself<Cell> { victim: E1, killer: Some(bolt_entity) }` is processed
   - Then: `Destroyed<Cell>` is sent with `victim: E1`, `killer: Some(bolt_entity)`, `victim_pos: Vec2(100.0, 200.0)`, `killer_pos: Some(Vec2(80.0, 180.0))`
   - Edge case: Killer entity does not exist (was despawned, e.g., bolt lost in same frame) — `killer_pos` should be `None`. The `Destroyed<Cell>` should still be sent with `killer: Some(bolt_entity)` and `killer_pos: None`.

4. **Kill handler sends Destroyed<Cell> with environmental killer (None)**
   - Given: A Cell entity E1 at position (100.0, 200.0), killed by an environmental source (e.g., Die effect)
   - When: `KillYourself<Cell> { victim: E1, killer: None }` is processed
   - Then: `Destroyed<Cell>` is sent with `victim: E1`, `killer: None`, `victim_pos: Vec2(100.0, 200.0)`, `killer_pos: None`
   - Edge case: No edge case beyond the None path covered above.

5. **Kill handler sends DespawnEntity**
   - Given: A Cell entity E1
   - When: `KillYourself<Cell> { victim: E1, killer: Some(bolt_entity) }` is processed
   - Then: `DespawnEntity { entity: E1 }` is sent
   - Edge case: Multiple `KillYourself<Cell>` messages for the same entity in one frame — only the first should be processed (second is skipped because `Dead` was already inserted by the first).

6. **Kill handler updates RequiredToClear tracking for required cells**
   - Given: A Cell entity E1 with `RequiredToClear` component, and a resource or message tracking remaining required cells for node completion
   - When: `KillYourself<Cell> { victim: E1, killer: Some(bolt_entity) }` is processed
   - Then: The `Destroyed<Cell>` message carries enough information (or the RequiredToClear status is readable on the entity before despawn) for downstream node completion tracking to know a required cell was destroyed. Specifically: the `Destroyed<Cell>` message should be sent while the entity still has `RequiredToClear`, so downstream consumers can query `Has<RequiredToClear>` on the victim entity, OR the was_required_to_clear info is included in the message. (Note: The existing `CellDestroyedAt` carried `was_required_to_clear: bool`. The `Destroyed<Cell>` type does not have this field. The kill handler must ensure this information is not lost.)
   - Edge case: Cell entity does NOT have `RequiredToClear` — downstream tracking should not count it.

7. **Kill handler processes multiple KillYourself<Cell> messages in one frame**
   - Given: Three Cell entities E1 (pos: 100.0, 200.0), E2 (pos: 150.0, 200.0), E3 (pos: 200.0, 200.0), each with `Hp { current: 0.0, ... }`. A single Bolt entity B1 at (80.0, 180.0) killed all three.
   - When: Three `KillYourself<Cell>` messages arrive in one frame: `{ victim: E1, killer: Some(B1) }`, `{ victim: E2, killer: Some(B1) }`, `{ victim: E3, killer: Some(B1) }`
   - Then: All three get `Dead` inserted. Three `Destroyed<Cell>` messages are sent. Three `DespawnEntity` messages are sent. All three removed from spatial index.
   - Edge case: Same entity appears in two KillYourself messages (duplicate) — second message is skipped because Dead was already inserted by the first.

---

### Section B: bolt_cell_collision Sends DamageDealt<Cell>

The `bolt_cell_collision` system is updated to send `DamageDealt<Cell>` instead of the old `DamageCell` message. The collision detection logic itself does not change — only the output message type changes.

File: `src/bolt/systems/bolt_cell_collision.rs` (or wherever bolt-cell collision lives)

#### Behavior

8. **bolt_cell_collision sends DamageDealt<Cell> on collision**
   - Given: A Bolt entity B1 at position (100.0, 195.0) with velocity (0.0, 400.0) moving toward a Cell entity C1 at position (100.0, 200.0) with `Hp { current: 30.0, starting: 30.0, max: None }`
   - When: Bolt B1 collides with Cell C1
   - Then: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, source_chip: None }` is sent. (The damage amount comes from the bolt's current damage value — use a concrete test value of 10.0.)
   - Edge case: Bolt has a source chip (e.g., from a damage boost effect) — `source_chip` should be `Some("Surge".to_string())` or whatever the chip name is, propagated from the bolt's `EffectSourceChip` component if present.

9. **bolt_cell_collision sends DamageDealt<Cell> even for Locked cells**
   - Given: A Bolt entity B1 at position (100.0, 195.0) and a Cell entity C1 at position (100.0, 200.0) with the `Locked` component and `Hp { current: 30.0, starting: 30.0, max: None }`
   - When: Bolt B1 collides with Cell C1
   - Then: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, source_chip: None }` IS sent. The collision system does not filter by Locked. `apply_damage::<Cell>` silently drops damage for Locked cells via its `Without<Locked>` filter (tested in wave 7).
   - Edge case: Locked cell also receives `BoltImpactCell` — both messages sent regardless of lock status.

10. **bolt_cell_collision still sends BoltImpactCell for effect bridge**
    - Given: A Bolt entity B1 colliding with Cell entity C1
    - When: Collision occurs
    - Then: Both `DamageDealt<Cell>` AND `BoltImpactCell { cell: C1, bolt: B1 }` are sent. The impact message is for the effect bridge (triggers), the damage message is for the death pipeline.
    - Edge case: Cell is Locked — both `BoltImpactCell` and `DamageDealt<Cell>` are still sent. The collision system does not filter by lock status. Damage is silently dropped downstream by `apply_damage::<Cell>` (wave 7).

---

### Section C: Cell Builder Produces Hp and KilledBy

Cell entities spawned by the cell builder must include `Hp` and `KilledBy` components, replacing the old `CellHealth`.

File: where cells are spawned (cell builder/spawner)

#### Behavior

11. **Cell spawns with Hp matching cell type HP**
    - Given: A cell type definition with health value 30.0
    - When: A Cell entity is spawned from this definition
    - Then: The entity has `Hp { current: 30.0, starting: 30.0, max: None }`
    - Edge case: Cell type with health value 1.0 (single-hit cell) — `Hp { current: 1.0, starting: 1.0, max: None }`

12. **Cell spawns with default KilledBy**
    - Given: Any cell type definition
    - When: A Cell entity is spawned
    - Then: The entity has `KilledBy { dealer: None }` (default)
    - Edge case: No edge case — KilledBy is always default at spawn time.

13. **Cell does NOT spawn with old CellHealth component**
    - Given: Any cell type definition
    - When: A Cell entity is spawned
    - Then: The entity does NOT have a `CellHealth` component. Only `Hp` and `KilledBy` are present for health tracking.
    - Edge case: Cell types that previously had special CellHealth values — verify they map correctly to Hp.

---

### Section D: Locked Cell Interaction with Death Pipeline

Locked cells are immune to damage. The `apply_damage::<Cell>` system (tested in wave 07) skips entities with the `Locked` component. This section tests the cell-domain-specific Locked interactions: that locked cells remain alive through the full pipeline, and that unlocking a cell makes it damageable.

File: `src/cells/systems/handle_cell_kill.rs` (kill handler tests) and/or integration test file

#### Behavior

14. **Locked cell is not killed even with DamageDealt messages**
    - Given: A Locked Cell entity C1 with `Hp { current: 30.0, starting: 30.0, max: None }` and the `Locked` component
    - When: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, source_chip: None }` is sent, then `apply_damage::<Cell>` runs, then `detect_cell_deaths` runs
    - Then: C1 still has `Hp { current: 30.0, ... }` (unchanged). No `KillYourself<Cell>` is sent for C1. No `Dead` component on C1.
    - Edge case: DamageDealt amount exceeds Hp (amount: 50.0, Hp: 30.0) — still skipped because Locked.

15. **Unlocked cell receives damage normally after lock removal**
    - Given: A Cell entity C1 that previously had `Locked` but has had the `Locked` component removed (lock targets destroyed), with `Hp { current: 30.0, starting: 30.0, max: None }`
    - When: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, source_chip: None }` is sent, then `apply_damage::<Cell>` runs
    - Then: C1 now has `Hp { current: 20.0, starting: 30.0, max: None }`
    - Edge case: Cell is unlocked and receives lethal damage in the same frame — should die normally.

---

### Section E: Damage Visual Feedback

The old `handle_cell_hit` combined damage application with visual updates (material color update based on remaining health). The new pipeline splits these: `apply_damage` handles HP, and a separate `cell_damage_visual` system handles the visual response. This system reads Hp changes and updates the cell's material color to reflect remaining health fraction.

File: `src/cells/systems/cell_damage_visual/tests.rs` (new directory module)

#### Behavior

16. **Cell material updates based on health fraction**
    - Given: A Cell entity with `Hp { current: 15.0, starting: 30.0, max: None }` (50% health)
    - When: The damage visual system runs
    - Then: The cell's visual representation reflects 50% health. The health fraction is `current / starting = 0.5`.
    - Edge case: `Hp { current: 30.0, starting: 30.0 }` (100% health, full color). `Hp { current: 1.0, starting: 30.0 }` (3.3% health, nearly dead appearance). `Hp { current: 0.0, starting: 30.0 }` (dead — should not be processed because entity will have `Dead` component by this point, or the visual system uses `Without<Dead>` filter). `Hp { current: 0.0, starting: 0.0 }` (starting=0 — health fraction is undefined; system should treat as 0.0 or skip to avoid division by zero).

17. **Damage visual skips Dead cells**
    - Given: A Cell entity with `Hp { current: 0.0, starting: 30.0, max: None }` and the `Dead` component
    - When: The damage visual system runs
    - Then: No visual update is applied. The `Without<Dead>` filter (or equivalent) prevents processing.
    - Edge case: Cell has Dead but nonzero Hp (killed by Die effect directly, Hp not at 0) — still skipped.

18. **Damage visual responds to Hp changes within a frame**
    - Given: A Cell entity starts the frame with `Hp { current: 30.0, starting: 30.0 }`, then `apply_damage` decrements to `Hp { current: 20.0, ... }`
    - When: The damage visual system runs (after apply_damage)
    - Then: The health fraction reflects the new value: `20.0 / 30.0 = 0.667`
    - Edge case: Multiple damage events in one frame reduce Hp from 30.0 to 5.0 — visual shows 5.0/30.0 = 0.167.

---

### Section F: End-to-End Cell Death Flow

These tests verify the complete pipeline from collision to despawn. They exercise the full chain: `DamageDealt<Cell>` -> `apply_damage::<Cell>` -> `detect_cell_deaths` -> `KillYourself<Cell>` -> cell kill handler -> `Destroyed<Cell>` + `DespawnEntity` -> `process_despawn_requests`. These are integration tests.

File: `src/cells/systems/handle_cell_kill.rs` (integration tests section) or a dedicated integration test file

#### Behavior

19. **Single-hit cell dies in one frame**
    - Given: A Cell entity C1 at position (100.0, 200.0) with `Hp { current: 10.0, starting: 10.0, max: None }`, `KilledBy { dealer: None }`, `RequiredToClear` component, registered in spatial index. A Bolt entity B1 at position (80.0, 180.0).
    - When: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, source_chip: None }` is sent. Then `apply_damage::<Cell>` runs (Hp goes to 0.0, KilledBy set to Some(B1)). Then `detect_cell_deaths` runs (sends `KillYourself<Cell>`). Then the cell kill handler runs (inserts Dead, removes from spatial, sends Destroyed<Cell>, sends DespawnEntity). Then `process_despawn_requests` runs.
    - Then: Entity C1 is despawned. `Destroyed<Cell>` was sent with `victim_pos: Vec2(100.0, 200.0)`, `killer: Some(B1)`. RequiredToClear tracking was updated.
    - Edge case: No edge case — this is the baseline happy path.

20. **Multi-hit cell survives partial damage, dies on final hit**
    - Given: A Cell entity C1 with `Hp { current: 30.0, starting: 30.0, max: None }`, `KilledBy { dealer: None }`. Bolt B1 deals 10.0 damage per hit.
    - When: Frame 1: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, ... }`. Pipeline runs. Hp becomes 20.0. No death.
    - When: Frame 2: Another `DamageDealt<Cell> { amount: 10.0, ... }`. Pipeline runs. Hp becomes 10.0. No death.
    - When: Frame 3: Another `DamageDealt<Cell> { amount: 10.0, ... }`. Pipeline runs. Hp becomes 0.0. `detect_cell_deaths` sends `KillYourself<Cell>`. Kill handler fires. Entity despawned after `process_despawn_requests`.
    - Then: After frame 3, C1 is despawned. `Destroyed<Cell>` sent only in frame 3. KilledBy.dealer is Some(B1) (set during frame 3's apply_damage).
    - Edge case: Overkill — `DamageDealt { amount: 50.0 }` on a cell with Hp 30.0. Hp goes to -20.0. Cell still dies normally via the same pipeline (Hp <= 0 triggers death detection).

21. **Multiple damage sources in one frame — first killing blow wins attribution**
    - Given: A Cell entity C1 with `Hp { current: 15.0, starting: 30.0, max: None }`. Bolt B1 and shockwave effect (dealer: B2) both send damage in the same frame.
    - When: `DamageDealt<Cell> { dealer: Some(B1), target: C1, amount: 10.0, ... }` and `DamageDealt<Cell> { dealer: Some(B2), target: C1, amount: 10.0, ... }` are both sent in the same frame. `apply_damage::<Cell>` processes both.
    - Then: Hp ends at -5.0. `KilledBy.dealer` is `Some(B2)` because the second damage message is the one that crosses Hp from positive to <= 0. First DamageDealt: 15.0 - 10.0 = 5.0 (still positive, not the killing blow). Second DamageDealt: 5.0 - 10.0 = -5.0 (crosses zero, this IS the killing blow). `Destroyed<Cell>` has `killer: Some(B2)`.
    - Edge case: Both messages individually would kill (amount: 20.0 each, Hp: 15.0). First message: 15.0 - 20.0 = -5.0 (kills). KilledBy set to Some(B1). Second message: entity Hp already <= 0, KilledBy already set, do not overwrite. Final KilledBy.dealer = Some(B1).

22. **Cell death with no killer (environmental/Die effect)**
    - Given: A Cell entity C1 at position (100.0, 200.0) with `Hp { current: 30.0, ... }`
    - When: `KillYourself<Cell> { victim: C1, killer: None }` is sent directly (bypassing damage/detection, as from a Die effect)
    - Then: Kill handler inserts Dead, removes from spatial index, sends `Destroyed<Cell> { victim: C1, killer: None, victim_pos: Vec2(100.0, 200.0), killer_pos: None }`, sends `DespawnEntity { entity: C1 }`.
    - Edge case: Die effect sends KillYourself while cell also has pending DamageDealt in the same frame — cell is killed by the direct KillYourself. The subsequent apply_damage/detect_cell_deaths skip it because Dead is already inserted.

23. **Dead cell is skipped by subsequent pipeline stages**
    - Given: Cell entity C1 has `Dead` component (killed this frame by kill handler). C1 still exists (not yet despawned — despawn happens in PostFixedUpdate).
    - When: A `DamageDealt<Cell> { target: C1, amount: 10.0, ... }` arrives (from an effect that fired after the kill)
    - Then: `apply_damage::<Cell>` skips C1 (Without<Dead> filter). No Hp change. No additional KillYourself sent. No second Destroyed<Cell>.
    - Edge case: detect_cell_deaths also skips C1 — Without<Dead> filter prevents re-detection.

---

### Section G: RequiredToClear Tracking Through Kill Handler

The old `CellDestroyedAt` message carried `was_required_to_clear: bool`. The new `Destroyed<Cell>` does not have this field. The kill handler or a downstream system must preserve this information for node completion tracking.

File: `src/cells/systems/handle_cell_kill.rs` or `src/run/node/systems/track_node_completion.rs`

#### Behavior

24. **Required cell sends Destroyed<Cell> while RequiredToClear is still present**
    - Given: A Cell entity C1 with `RequiredToClear` component
    - When: C1 is killed via the full pipeline (KillYourself<Cell> -> kill handler -> Destroyed<Cell>)
    - Then: `Destroyed<Cell>` is sent while the victim entity C1 still has the `RequiredToClear` component. Downstream consumers (run-domain node completion tracking) can query `Has<RequiredToClear>` on the victim entity from the `Destroyed<Cell>` message.
    - Edge case: No edge case — the entity is still alive when `Destroyed<Cell>` is sent, so all components are still present.
    - Note: Whether "required cells remaining decrements" is the run-domain's responsibility, not the cell domain's. The cell domain's contract is that `Destroyed<Cell>` is sent before despawn, while the entity's components are intact.

25. **Non-required cell sends Destroyed<Cell> without RequiredToClear**
    - Given: A Cell entity C1 WITHOUT `RequiredToClear` component
    - When: C1 is killed via the full pipeline
    - Then: `Destroyed<Cell>` is sent. The victim entity C1 does NOT have `RequiredToClear`, so downstream consumers querying `Has<RequiredToClear>` on the victim entity will see `false`.
    - Edge case: No edge case — this is the complement of behavior 24.
    - Note: The run-domain node completion tracker decides how to handle required vs non-required. The cell kill handler treats both the same.

---

### Types (new types needed)

- `Hp { current: f32, starting: f32, max: Option<f32> }` — Component, derives: Component, Debug, Clone. Unified health for all damageable entities. Lives in `src/shared/components.rs`. (Created in wave 02 scaffold, used here.)

- `KilledBy { dealer: Option<Entity> }` — Component, derives: Component, Default, Debug. Set on killing blow by apply_damage. Lives in `src/shared/components.rs`. (Created in wave 02 scaffold, used here.)

- `Dead` — Component, derives: Component. Marker inserted by domain kill handlers. Lives in `src/shared/components.rs`. (Created in wave 02 scaffold, used here.)

- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` — Message, derives: Message, Clone, Debug. Generic damage message. Lives in `src/shared/messages.rs`. (Created in wave 02 scaffold, used here.)

- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` — Message, derives: Message, Clone, Debug. Death request message. Lives in `src/shared/messages.rs`. (Created in wave 02 scaffold, used here.)

- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` — Message, derives: Message, Clone, Debug. Death confirmed message. Lives in `src/shared/messages.rs`. (Created in wave 02 scaffold, used here.)

- `DespawnEntity { entity: Entity }` — Message, derives: Message, Clone, Debug. Deferred despawn. Lives in `src/shared/messages.rs`. (Created in wave 02 scaffold, used here.)

- `GameEntity` — Trait, bound: `Component`. Marker trait for death pipeline generics. Lives in `src/shared/traits.rs`. `impl GameEntity for Cell {}`. (Created in wave 02 scaffold.)

- `DamageTargetData { hp: &mut Hp, killed_by: &mut KilledBy }` — QueryData, derives: QueryData with `#[query_data(mutable)]`. Lives in `src/shared/queries.rs`. (Created in wave 07.)

- `DeathDetectionData { entity: Entity, killed_by: &KilledBy, hp: &Hp }` — QueryData, derives: QueryData (read-only). Lives in `src/shared/queries.rs`. (Created in wave 07.)

All types above are created in earlier waves (02, 07). Wave 09 does not create new shared types — it creates the cell kill handler system and modifies cell-domain systems to use these types.

### Messages (changes to existing messages)

- `DamageCell` — REMOVED. Replaced by `DamageDealt<Cell>`.
- `RequestCellDestroyed` — REMOVED. Replaced by `KillYourself<Cell>`.
- `CellDestroyedAt` — REMOVED. Replaced by `Destroyed<Cell>`.

The cell kill handler SENDS: `Destroyed<Cell>`, `DespawnEntity`.
The cell kill handler READS: `KillYourself<Cell>`.
The `bolt_cell_collision` system SENDS: `DamageDealt<Cell>` (replacing `DamageCell`).

### Reference Files

These are the wave 09 design docs that define the behaviors being tested:
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/apply-damage-cell.md` — apply_damage::<Cell> behavior (tested in wave 07, referenced here for integration)
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/detect-cell-deaths.md` — detect_cell_deaths behavior (tested in wave 07, referenced here for integration)
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-remove.md` — handle_cell_hit and cleanup_cell being replaced
- `docs/todos/detail/unified-death-pipeline/rust-types/` — all type definitions
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` — ordering constraints

### Scenario Coverage

- New invariants: `CellHpNonNegative` — optional new invariant checking that no living cell (Without<Dead>) has Hp.current < 0.0 at end of frame. This catches apply_damage bugs where Hp goes negative without triggering death detection. However, overkill IS expected (Hp can go below 0.0 from a single large hit), so this invariant may not be appropriate. Consider `CellDeathConsistency` instead: every cell with Hp <= 0 AND Without<Dead> should have a KillYourself<Cell> pending (hard to check per-frame). Recommendation: rely on existing invariants (`NoEntityLeaks`, `BoltInBounds`) plus the integration tests in this spec. No new invariant needed unless testing reveals a gap.
- New scenarios: `scenarios/mechanic/cell_death_pipeline.scenario.ron` — A node layout with a mix of single-hit (Hp: 10) and multi-hit (Hp: 30) cells, plus at least one Locked cell. Chaos input. Verifies no entity leaks and all required cells are eventually cleared.
- Self-test scenarios: none — no new InvariantKind added.
- Layout updates: Existing layouts already have Locked cells and RequiredToClear cells. No layout changes needed, but the cell type definitions must be updated to produce Hp instead of CellHealth.

### Constraints

- Tests go in:
  - `src/cells/systems/handle_cell_kill.rs` — cell kill handler system + unit tests (behaviors 1-7)
  - `src/bolt/systems/bolt_cell_collision.rs` — updated collision tests (behaviors 8-10), within existing test module
  - Cell builder/spawner test file — Hp/KilledBy spawn tests (behaviors 11-13)
  - `src/cells/systems/cell_damage_visual/tests.rs` — damage visual system tests (behaviors 16-18)
  - Integration test file (within cells domain) — end-to-end tests (behaviors 19-25)
- Do NOT test: `apply_damage::<Cell>` or `detect_cell_deaths` in isolation — those are wave 07 tests
- Do NOT test: Effect system internals (trigger bridges, effect firing)
- Do NOT test: Other domain kill handlers (bolt, wall, breaker)
- Do NOT test: `process_despawn_requests` in isolation — that is wave 07
- Do NOT modify: `src/shared/systems/apply_damage.rs`, `src/shared/systems/process_despawn_requests.rs` — those are wave 07 code
- Locked cell filtering in `apply_damage::<Cell>` is tested in wave 07 (the `Without<Locked>` filter). Wave 09 tests verify the Locked cell path end-to-end (behavior 14-15) to confirm the full pipeline respects locks.


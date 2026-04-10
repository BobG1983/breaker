# Test Spec: Effect Domain -- Wave 5: Trigger Bridge Systems

## Domain
`src/effect/triggers/`

---

## Section A: Bump Bridge Systems (10 systems)

All bump bridges live in `src/effect/triggers/bump/bridges.rs`. All run in FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))` (except `on_no_bump_occurred` which runs after `BoltSystems::BreakerCollision`).

### A1: Global Bump Bridges (6 systems)

#### A1.1 on_bump_occurred

1. **on_bump_occurred fires for Perfect grade BumpPerformed**
   - Given: Entity A with `BoundEffects` containing a tree gated on `Trigger::BumpOccurred`; Entity B with `StagedEffects` containing a tree gated on `Trigger::BumpOccurred`; a `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message in the queue
   - When: `on_bump_occurred` runs
   - Then: `walk_effects` is called for Entity A with trigger `Trigger::BumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`; same for Entity B
   - Edge case: Entity with `BoundEffects` but no `StagedEffects` -- still walked

2. **on_bump_occurred fires for Early grade BumpPerformed**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; a `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_bump_occurred` runs
   - Then: `walk_effects` is called for Entity A with `Trigger::BumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
   - Edge case: `bolt: None` -- context carries `bolt: None`, walk still happens

3. **on_bump_occurred fires for Late grade BumpPerformed**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; a `BumpPerformed { grade: BumpGrade::Late, bolt: None, breaker: breaker_entity }` message
   - When: `on_bump_occurred` runs
   - Then: `walk_effects` is called with `TriggerContext::Bump { bolt: None, breaker: breaker_entity }`
   - Edge case: Multiple BumpPerformed messages in one frame -- each produces a separate walk sweep

4. **on_bump_occurred does NOT fire when no BumpPerformed messages exist**
   - Given: Entity A with `BoundEffects` and `StagedEffects` containing `Trigger::BumpOccurred`; no messages in queue
   - When: `on_bump_occurred` runs
   - Then: `walk_effects` is never called
   - Edge case: Zero entities with BoundEffects/StagedEffects -- system is a no-op

#### A1.2 on_perfect_bump_occurred

5. **on_perfect_bump_occurred fires only for Perfect grade**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_perfect_bump_occurred` runs
   - Then: `walk_effects` called on Entity A with `Trigger::PerfectBumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
   - Edge case: `bolt: None` -- context still carries `bolt: None`

6. **on_perfect_bump_occurred skips Early grade**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_perfect_bump_occurred` runs
   - Then: `walk_effects` is NOT called
   - Edge case: Late grade also skipped

7. **on_perfect_bump_occurred skips Late grade**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Late, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_perfect_bump_occurred` runs
   - Then: `walk_effects` is NOT called
   - Edge case: Multiple messages with mixed grades -- only Perfect ones produce walks

#### A1.3 on_early_bump_occurred

8. **on_early_bump_occurred fires only for Early grade**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_early_bump_occurred` runs
   - Then: `walk_effects` called with `Trigger::EarlyBumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
   - Edge case: Perfect and Late grades are skipped

9. **on_early_bump_occurred skips Perfect grade**
   - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
   - When: `on_early_bump_occurred` runs
   - Then: `walk_effects` is NOT called
   - Edge case: No messages at all -- system is a no-op

#### A1.4 on_late_bump_occurred

10. **on_late_bump_occurred fires only for Late grade**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Late, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_late_bump_occurred` runs
    - Then: `walk_effects` called with `Trigger::LateBumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
    - Edge case: Perfect and Early grades are skipped

11. **on_late_bump_occurred skips Early grade**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_late_bump_occurred` runs
    - Then: `walk_effects` is NOT called
    - Edge case: No messages at all -- system is a no-op

#### A1.5 on_bump_whiff_occurred

12. **on_bump_whiff_occurred fires for BumpWhiffed message**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; a `BumpWhiffed` unit message in queue
    - When: `on_bump_whiff_occurred` runs
    - Then: `walk_effects` called on Entity A with `Trigger::BumpWhiffOccurred` and context `TriggerContext::None`
    - Edge case: Multiple BumpWhiffed messages -- each triggers a separate walk sweep

13. **on_bump_whiff_occurred context is TriggerContext::None**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; a `BumpWhiffed` message
    - When: `on_bump_whiff_occurred` runs
    - Then: context passed to walk_effects is exactly `TriggerContext::None` (no bolt, no breaker -- whiff has no participants)
    - Edge case: No BumpWhiffed messages -- system is a no-op

#### A1.6 on_no_bump_occurred

14. **on_no_bump_occurred fires when BoltImpactBreaker has BumpStatus::Inactive**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; `BoltImpactBreaker { bolt: bolt_entity, breaker: breaker_entity, bump_status: BumpStatus::Inactive }` message
    - When: `on_no_bump_occurred` runs
    - Then: `walk_effects` called on Entity A with `Trigger::NoBumpOccurred` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
    - Edge case: Multiple BoltImpactBreaker messages with Inactive -- each triggers a walk sweep

15. **on_no_bump_occurred skips BoltImpactBreaker with BumpStatus::Active**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; `BoltImpactBreaker { bolt: bolt_entity, breaker: breaker_entity, bump_status: BumpStatus::Active }` message
    - When: `on_no_bump_occurred` runs
    - Then: `walk_effects` is NOT called
    - Edge case: Mix of Active and Inactive in same frame -- only Inactive ones produce walks

### A2: Local Bump Bridges (4 systems)

Local bridges walk only the bolt and breaker entities from the message, NOT all entities.

#### A2.1 on_bumped

16. **on_bumped walks bolt and breaker for Perfect grade**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; breaker_entity with `BoundEffects` and `StagedEffects`; other_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_bumped` runs
    - Then: `walk_effects` called on bolt_entity with `Trigger::Bumped` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`; `walk_effects` called on breaker_entity with same trigger and context; `walk_effects` NOT called on other_entity
    - Edge case: Early grade also fires (any successful bump)

17. **on_bumped walks bolt and breaker for Early grade**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; breaker_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_bumped` runs
    - Then: `walk_effects` called on both bolt_entity and breaker_entity with `Trigger::Bumped`
    - Edge case: Late grade also fires

18. **on_bumped skips bolt walk when bolt is None, still walks breaker**
    - Given: breaker_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Perfect, bolt: None, breaker: breaker_entity }` message
    - When: `on_bumped` runs
    - Then: `walk_effects` called on breaker_entity with `Trigger::Bumped` and context `TriggerContext::Bump { bolt: None, breaker: breaker_entity }`; NO walk_effects call for a bolt entity
    - Edge case: bolt entity exists in world but is not referenced in message -- not walked

#### A2.2 on_perfect_bumped

19. **on_perfect_bumped walks bolt and breaker for Perfect grade only**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; breaker_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_perfect_bumped` runs
    - Then: `walk_effects` called on bolt_entity with `Trigger::PerfectBumped` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`; `walk_effects` called on breaker_entity with same trigger and context
    - Edge case: bolt is None -- only breaker is walked

20. **on_perfect_bumped skips Early grade entirely**
    - Given: bolt_entity and breaker_entity with effects; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_perfect_bumped` runs
    - Then: `walk_effects` NOT called on either entity
    - Edge case: Late grade also skipped

#### A2.3 on_early_bumped

21. **on_early_bumped walks bolt and breaker for Early grade only**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; breaker_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_early_bumped` runs
    - Then: `walk_effects` called on both entities with `Trigger::EarlyBumped` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
    - Edge case: bolt is None -- only breaker walked

22. **on_early_bumped skips Perfect and Late grades**
    - Given: entities with effects; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_early_bumped` runs
    - Then: `walk_effects` NOT called
    - Edge case: Late grade message also produces no walk

#### A2.4 on_late_bumped

23. **on_late_bumped walks bolt and breaker for Late grade only**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; breaker_entity with `BoundEffects` and `StagedEffects`; `BumpPerformed { grade: BumpGrade::Late, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_late_bumped` runs
    - Then: `walk_effects` called on both entities with `Trigger::LateBumped` and context `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`
    - Edge case: bolt is None -- only breaker walked

24. **on_late_bumped skips Perfect and Early grades**
    - Given: entities with effects; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `on_late_bumped` runs
    - Then: `walk_effects` NOT called
    - Edge case: Early grade also produces no walk

---

## Section B: Impact Bridge Systems (12 systems: 6 impacted + 6 impact_occurred)

All impact bridges live in `src/effect/triggers/impact/bridges.rs`. All run in FixedUpdate, in `EffectSystems::Bridge`, with `run_if(in_state(NodeState::Playing))`.

### B1: Local Impact Bridges (on_impacted -- 6 sub-systems)

Each sub-system reads one collision message type and walks the two participant entities.

#### B1.1 on_impacted_bolt_cell

25. **on_impacted_bolt_cell walks bolt with Impacted(Cell) and cell with Impacted(Bolt)**
    - Given: bolt_entity with `BoundEffects` and `StagedEffects`; cell_entity with `BoundEffects` and `StagedEffects`; other_entity with `BoundEffects` and `StagedEffects`; `BoltImpactCell { bolt: bolt_entity, cell: cell_entity }` message
    - When: `on_impacted_bolt_cell` runs
    - Then: `walk_effects` called on bolt_entity with `Trigger::Impacted(EntityKind::Cell)` and context `TriggerContext::Impact { impactor: bolt_entity, impactee: cell_entity }`; `walk_effects` called on cell_entity with `Trigger::Impacted(EntityKind::Bolt)` and same context; other_entity NOT walked
    - Edge case: bolt_entity has no BoundEffects/StagedEffects -- only cell_entity is walked (and vice versa)

26. **on_impacted_bolt_cell with no messages is a no-op**
    - Given: entities with BoundEffects/StagedEffects; no `BoltImpactCell` messages
    - When: `on_impacted_bolt_cell` runs
    - Then: `walk_effects` NOT called
    - Edge case: Messages of other collision types present -- still no-op for this bridge

#### B1.2 on_impacted_bolt_wall

27. **on_impacted_bolt_wall walks bolt with Impacted(Wall) and wall with Impacted(Bolt)**
    - Given: bolt_entity with effects; wall_entity with effects; `BoltImpactWall { bolt: bolt_entity, wall: wall_entity }` message
    - When: `on_impacted_bolt_wall` runs
    - Then: bolt_entity walked with `Trigger::Impacted(EntityKind::Wall)` and context `TriggerContext::Impact { impactor: bolt_entity, impactee: wall_entity }`; wall_entity walked with `Trigger::Impacted(EntityKind::Bolt)` and same context
    - Edge case: Multiple BoltImpactWall messages in one frame -- each produces separate walks for both participants

#### B1.3 on_impacted_bolt_breaker

28. **on_impacted_bolt_breaker walks bolt with Impacted(Breaker) and breaker with Impacted(Bolt)**
    - Given: bolt_entity with effects; breaker_entity with effects; `BoltImpactBreaker { bolt: bolt_entity, breaker: breaker_entity, bump_status: BumpStatus::Active }` message
    - When: `on_impacted_bolt_breaker` runs
    - Then: bolt_entity walked with `Trigger::Impacted(EntityKind::Breaker)` and context `TriggerContext::Impact { impactor: bolt_entity, impactee: breaker_entity }`; breaker_entity walked with `Trigger::Impacted(EntityKind::Bolt)` and same context
    - Edge case: bump_status is irrelevant to impact bridge -- walks happen regardless of Active/Inactive

#### B1.4 on_impacted_breaker_cell

29. **on_impacted_breaker_cell walks breaker with Impacted(Cell) and cell with Impacted(Breaker)**
    - Given: breaker_entity with effects; cell_entity with effects; `BreakerImpactCell { breaker: breaker_entity, cell: cell_entity }` message
    - When: `on_impacted_breaker_cell` runs
    - Then: breaker_entity walked with `Trigger::Impacted(EntityKind::Cell)` and context `TriggerContext::Impact { impactor: breaker_entity, impactee: cell_entity }`; cell_entity walked with `Trigger::Impacted(EntityKind::Breaker)` and same context
    - Edge case: Cell has effects but breaker does not -- only cell walked

#### B1.5 on_impacted_breaker_wall

30. **on_impacted_breaker_wall walks breaker with Impacted(Wall) and wall with Impacted(Breaker)**
    - Given: breaker_entity with effects; wall_entity with effects; `BreakerImpactWall { breaker: breaker_entity, wall: wall_entity }` message
    - When: `on_impacted_breaker_wall` runs
    - Then: breaker_entity walked with `Trigger::Impacted(EntityKind::Wall)` and context `TriggerContext::Impact { impactor: breaker_entity, impactee: wall_entity }`; wall_entity walked with `Trigger::Impacted(EntityKind::Breaker)` and same context
    - Edge case: No messages -- no-op

#### B1.6 on_impacted_cell_wall

31. **on_impacted_cell_wall walks cell with Impacted(Wall) and wall with Impacted(Cell)**
    - Given: cell_entity with effects; wall_entity with effects; `CellImpactWall { cell: cell_entity, wall: wall_entity }` message
    - When: `on_impacted_cell_wall` runs
    - Then: cell_entity walked with `Trigger::Impacted(EntityKind::Wall)` and context `TriggerContext::Impact { impactor: cell_entity, impactee: wall_entity }`; wall_entity walked with `Trigger::Impacted(EntityKind::Cell)` and same context
    - Edge case: No messages -- no-op

### B2: Global Impact Bridges (on_impact_occurred -- 6 sub-systems)

Each sub-system reads one collision message type and walks ALL entities with BoundEffects/StagedEffects, twice per message (once per participant kind).

#### B2.1 on_impact_occurred_bolt_cell

32. **on_impact_occurred_bolt_cell sweeps all entities with ImpactOccurred(Cell) then ImpactOccurred(Bolt)**
    - Given: Entity A, Entity B, Entity C all with `BoundEffects` and `StagedEffects`; `BoltImpactCell { bolt: bolt_entity, cell: cell_entity }` message
    - When: `on_impact_occurred_bolt_cell` runs
    - Then: All three entities walked with `Trigger::ImpactOccurred(EntityKind::Cell)` and context `TriggerContext::Impact { impactor: bolt_entity, impactee: cell_entity }`; then all three entities walked again with `Trigger::ImpactOccurred(EntityKind::Bolt)` and same context
    - Edge case: Zero entities with effects -- system is a no-op despite message present

33. **on_impact_occurred_bolt_cell produces two global sweeps per message**
    - Given: Entity A with effects; `BoltImpactCell { bolt: bolt_entity, cell: cell_entity }` message
    - When: `on_impact_occurred_bolt_cell` runs
    - Then: Entity A is walked twice -- once with `ImpactOccurred(Cell)`, once with `ImpactOccurred(Bolt)`, both with same context
    - Edge case: Two BoltImpactCell messages in one frame -- four total sweeps (2 per message)

#### B2.2 on_impact_occurred_bolt_wall

34. **on_impact_occurred_bolt_wall sweeps all entities with ImpactOccurred(Wall) then ImpactOccurred(Bolt)**
    - Given: Entity A with effects; `BoltImpactWall { bolt: bolt_entity, wall: wall_entity }` message
    - When: `on_impact_occurred_bolt_wall` runs
    - Then: Entity A walked with `ImpactOccurred(Wall)` and context `Impact { impactor: bolt_entity, impactee: wall_entity }`; then with `ImpactOccurred(Bolt)` and same context
    - Edge case: No messages -- no-op

#### B2.3 on_impact_occurred_bolt_breaker

35. **on_impact_occurred_bolt_breaker sweeps all entities with ImpactOccurred(Breaker) then ImpactOccurred(Bolt)**
    - Given: Entity A with effects; `BoltImpactBreaker { bolt: bolt_entity, breaker: breaker_entity, bump_status: BumpStatus::Active }` message
    - When: `on_impact_occurred_bolt_breaker` runs
    - Then: Entity A walked with `ImpactOccurred(Breaker)` and context `Impact { impactor: bolt_entity, impactee: breaker_entity }`; then with `ImpactOccurred(Bolt)` and same context
    - Edge case: bump_status is irrelevant -- walks happen regardless

#### B2.4 on_impact_occurred_breaker_cell

36. **on_impact_occurred_breaker_cell sweeps all entities with ImpactOccurred(Cell) then ImpactOccurred(Breaker)**
    - Given: Entity A with effects; `BreakerImpactCell { breaker: breaker_entity, cell: cell_entity }` message
    - When: `on_impact_occurred_breaker_cell` runs
    - Then: Entity A walked with `ImpactOccurred(Cell)` and context `Impact { impactor: breaker_entity, impactee: cell_entity }`; then with `ImpactOccurred(Breaker)` and same context
    - Edge case: No messages -- no-op

#### B2.5 on_impact_occurred_breaker_wall

37. **on_impact_occurred_breaker_wall sweeps all entities with ImpactOccurred(Wall) then ImpactOccurred(Breaker)**
    - Given: Entity A with effects; `BreakerImpactWall { breaker: breaker_entity, wall: wall_entity }` message
    - When: `on_impact_occurred_breaker_wall` runs
    - Then: Entity A walked with `ImpactOccurred(Wall)` and context `Impact { impactor: breaker_entity, impactee: wall_entity }`; then with `ImpactOccurred(Breaker)` and same context
    - Edge case: No messages -- no-op

#### B2.6 on_impact_occurred_cell_wall

38. **on_impact_occurred_cell_wall sweeps all entities with ImpactOccurred(Wall) then ImpactOccurred(Cell)**
    - Given: Entity A with effects; `CellImpactWall { cell: cell_entity, wall: wall_entity }` message
    - When: `on_impact_occurred_cell_wall` runs
    - Then: Entity A walked with `ImpactOccurred(Wall)` and context `Impact { impactor: cell_entity, impactee: wall_entity }`; then with `ImpactOccurred(Cell)` and same context
    - Edge case: No messages -- no-op

---

## Section C: Death Bridge Systems (4 monomorphized instances of a generic system)

All death bridges are generic: `on_destroyed::<T>` where T: GameEntity. The system lives in `src/effect/triggers/death/bridges.rs`. All run in FixedUpdate, in `EffectSystems::Bridge`, after domain kill handlers.

Each death dispatches three triggers in order: Died (local, victim) -> Killed(EntityKind) (local, killer) -> DeathOccurred(EntityKind) (global, all entities).

#### C1 on_destroyed::\<Cell\>

39. **on_destroyed_cell dispatches Died on victim, Killed(Cell) on killer, DeathOccurred(Cell) globally**
    - Given: victim_cell_entity with `BoundEffects` and `StagedEffects`; killer_bolt_entity with `Bolt` component, `BoundEffects` and `StagedEffects`; bystander_entity with `BoundEffects` and `StagedEffects`; `Destroyed<Cell> { victim: victim_cell_entity, killer: Some(killer_bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)) }` message
    - When: `on_destroyed::<Cell>` runs
    - Then: (1) `walk_effects` called on victim_cell_entity with `Trigger::Died` and context `TriggerContext::Death { victim: victim_cell_entity, killer: Some(killer_bolt_entity) }`; (2) `walk_effects` called on killer_bolt_entity with `Trigger::Killed(EntityKind::Cell)` and same context; (3) `walk_effects` called on victim_cell_entity, killer_bolt_entity, AND bystander_entity with `Trigger::DeathOccurred(EntityKind::Cell)` and same context
    - Edge case: Victim has no BoundEffects -- Died walk skipped, Killed and DeathOccurred still fire

40. **on_destroyed_cell with killer=None skips Killed trigger**
    - Given: victim_cell_entity with `BoundEffects` and `StagedEffects`; bystander_entity with effects; `Destroyed<Cell> { victim: victim_cell_entity, killer: None, victim_pos: Vec2::new(100.0, 200.0), killer_pos: None }` message
    - When: `on_destroyed::<Cell>` runs
    - Then: (1) `walk_effects` called on victim_cell_entity with `Trigger::Died`; (2) NO walk_effects call with `Trigger::Killed(...)` on any entity; (3) `walk_effects` called globally with `Trigger::DeathOccurred(EntityKind::Cell)` and context `TriggerContext::Death { victim: victim_cell_entity, killer: None }`
    - Edge case: Environmental cell death (e.g., cell fell off screen) -- Killed skipped but Died and DeathOccurred still fire

41. **on_destroyed_cell with killer entity already despawned skips Killed trigger**
    - Given: victim_cell_entity with effects; killer_entity specified in message but no longer exists in world; `Destroyed<Cell> { victim: victim_cell_entity, killer: Some(killer_entity), victim_pos: Vec2::ZERO, killer_pos: None }` message
    - When: `on_destroyed::<Cell>` runs
    - Then: (1) Died fires on victim; (2) Killed(Cell) is skipped (killer entity gone, debug warning emitted); (3) DeathOccurred(Cell) fires globally
    - Edge case: Killer despawned between message send and bridge execution -- safe skip with warning

#### C2 on_destroyed::\<Bolt\>

42. **on_destroyed_bolt dispatches all three triggers with killer present**
    - Given: victim_bolt_entity with effects; killer_cell_entity with `Cell` component and effects; bystander with effects; `Destroyed<Bolt> { victim: victim_bolt_entity, killer: Some(killer_cell_entity), victim_pos: Vec2::new(0.0, -400.0), killer_pos: Some(Vec2::new(0.0, 100.0)) }` message
    - When: `on_destroyed::<Bolt>` runs
    - Then: (1) Died on victim_bolt_entity with context `Death { victim: victim_bolt_entity, killer: Some(killer_cell_entity) }`; (2) Killed(Bolt) on killer_cell_entity with same context; (3) DeathOccurred(Bolt) globally
    - Edge case: Killer entity has `Wall` component instead of `Cell` -- EntityKind in Killed should still be `Bolt` (the victim kind), not the killer kind

43. **on_destroyed_bolt with killer=None (environmental death) skips Killed**
    - Given: victim_bolt_entity with effects; `Destroyed<Bolt> { victim: victim_bolt_entity, killer: None, victim_pos: Vec2::new(0.0, -500.0), killer_pos: None }` message
    - When: `on_destroyed::<Bolt>` runs
    - Then: (1) Died on victim_bolt_entity; (2) Killed skipped; (3) DeathOccurred(Bolt) globally
    - Edge case: Most bolt deaths are environmental (fell off bottom) -- this is the common case

#### C3 on_destroyed::\<Wall\>

44. **on_destroyed_wall dispatches all three triggers**
    - Given: victim_wall_entity with effects; killer_bolt_entity with `Bolt` component and effects; bystander with effects; `Destroyed<Wall> { victim: victim_wall_entity, killer: Some(killer_bolt_entity), victim_pos: Vec2::new(-300.0, 0.0), killer_pos: Some(Vec2::new(-100.0, 50.0)) }` message
    - When: `on_destroyed::<Wall>` runs
    - Then: (1) Died on victim; (2) Killed(Wall) on killer; (3) DeathOccurred(Wall) globally
    - Edge case: Wall destruction with no killer -- Killed skipped

45. **on_destroyed_wall with killer=None skips Killed**
    - Given: victim_wall_entity with effects; `Destroyed<Wall> { victim: victim_wall_entity, killer: None, victim_pos: Vec2::ZERO, killer_pos: None }` message
    - When: `on_destroyed::<Wall>` runs
    - Then: Died on victim; Killed skipped; DeathOccurred(Wall) globally
    - Edge case: Wall expired due to timer -- environmental death

#### C4 on_destroyed::\<Breaker\>

46. **on_destroyed_breaker dispatches all three triggers with killer present**
    - Given: victim_breaker_entity with effects; killer_entity with effects; `Destroyed<Breaker> { victim: victim_breaker_entity, killer: Some(killer_entity), victim_pos: Vec2::new(0.0, -350.0), killer_pos: Some(Vec2::new(0.0, 100.0)) }` message
    - When: `on_destroyed::<Breaker>` runs
    - Then: (1) Died on victim; (2) Killed(Breaker) on killer; (3) DeathOccurred(Breaker) globally
    - Edge case: Breaker deaths are typically environmental -- this case with a killer is uncommon

47. **on_destroyed_breaker with killer=None (all lives lost) skips Killed**
    - Given: victim_breaker_entity with effects; `Destroyed<Breaker> { victim: victim_breaker_entity, killer: None, victim_pos: Vec2::new(0.0, -350.0), killer_pos: None }` message
    - When: `on_destroyed::<Breaker>` runs
    - Then: Died on victim; Killed skipped; DeathOccurred(Breaker) globally
    - Edge case: This is the typical breaker death case

---

## Section D: Bolt Lost Bridge

Lives in `src/effect/triggers/bolt_lost/bridges.rs`. Runs in FixedUpdate, after `BoltSystems::BoltLost`.

48. **on_bolt_lost_occurred walks all entities with BoltLostOccurred trigger**
    - Given: Entity A with `BoundEffects` containing tree gated on `Trigger::BoltLostOccurred`; Entity B with `StagedEffects` containing tree gated on `Trigger::BoltLostOccurred`; Entity C with effects but no BoltLostOccurred trigger; `BoltLost { bolt: bolt_entity, breaker: breaker_entity }` message
    - When: `on_bolt_lost_occurred` runs
    - Then: `walk_effects` called on Entity A and Entity B with `Trigger::BoltLostOccurred` and context `TriggerContext::BoltLost { bolt: bolt_entity, breaker: breaker_entity }`; Entity C still walked (bridge walks all entities, trigger matching is the walker's job)
    - Edge case: Multiple BoltLost messages in one frame -- each produces a separate walk sweep

49. **on_bolt_lost_occurred with no BoltLost messages is a no-op**
    - Given: Entities with effects; no `BoltLost` messages
    - When: `on_bolt_lost_occurred` runs
    - Then: `walk_effects` NOT called
    - Edge case: Zero entities with effects -- also no-op

50. **on_bolt_lost_occurred context carries both bolt and breaker**
    - Given: Entity A with effects; `BoltLost { bolt: Entity::from_raw(42), breaker: Entity::from_raw(7) }` message
    - When: `on_bolt_lost_occurred` runs
    - Then: Context is exactly `TriggerContext::BoltLost { bolt: Entity::from_raw(42), breaker: Entity::from_raw(7) }`
    - Edge case: bolt entity may already be in the process of being destroyed -- bridge fires before despawn

---

## Section E: Node Bridge Systems (3 systems)

### E1: on_node_start_occurred

Lives in `src/effect/triggers/node/bridges.rs`. Runs on `OnEnter(NodeState::Playing)` -- NOT FixedUpdate.

51. **on_node_start_occurred walks all entities with BoundEffects/StagedEffects on node start**
    - Given: Entity A with `BoundEffects` and `StagedEffects`; Entity B with `BoundEffects` and `StagedEffects`; world state transitions to `NodeState::Playing`
    - When: `on_node_start_occurred` runs (via OnEnter)
    - Then: `walk_effects` called on Entity A and Entity B with `Trigger::NodeStartOccurred` and context `TriggerContext::None`
    - Edge case: No entities with effects -- system is a no-op

52. **on_node_start_occurred context is TriggerContext::None**
    - Given: Entity A with effects; state transitions to Playing
    - When: `on_node_start_occurred` runs
    - Then: Context passed is exactly `TriggerContext::None` (no participants -- node events have no entity context)
    - Edge case: System fires exactly once per state transition, not every frame

### E2: on_node_end_occurred

Lives in `src/effect/triggers/node/bridges.rs`. Runs on `OnExit(NodeState::Playing)` -- NOT FixedUpdate.

53. **on_node_end_occurred walks all entities with BoundEffects/StagedEffects on node end**
    - Given: Entity A and Entity B with `BoundEffects` and `StagedEffects`; world state exits `NodeState::Playing`
    - When: `on_node_end_occurred` runs (via OnExit)
    - Then: `walk_effects` called on Entity A and Entity B with `Trigger::NodeEndOccurred` and context `TriggerContext::None`
    - Edge case: No entities with effects -- system is a no-op

54. **on_node_end_occurred fires once on exit, not every frame**
    - Given: Entity A with effects; state transitions from Playing to some other state
    - When: `on_node_end_occurred` runs
    - Then: `walk_effects` called exactly once per entity (not per frame -- OnExit fires once)
    - Edge case: Rapid state transitions (Playing -> ChipSelect -> Playing) -- fires on each exit

### E3: on_node_timer_threshold_occurred

Lives in `src/effect/triggers/node/bridges.rs`. Runs in FixedUpdate, after `check_node_timer_thresholds`.

55. **on_node_timer_threshold_occurred walks all entities with matching threshold trigger**
    - Given: Entity A with effects; `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.5) }` message
    - When: `on_node_timer_threshold_occurred` runs
    - Then: `walk_effects` called on Entity A with `Trigger::NodeTimerThresholdOccurred(0.5)` and context `TriggerContext::None`
    - Edge case: Entity's trigger set has `NodeTimerThresholdOccurred(0.75)` but message says 0.5 -- trigger matching is the walker's job, bridge walks all entities

56. **on_node_timer_threshold_occurred handles multiple thresholds crossed in one frame**
    - Given: Entity A with effects; two messages: `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.25) }` and `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.5) }`
    - When: `on_node_timer_threshold_occurred` runs
    - Then: Entity A walked twice -- once with `NodeTimerThresholdOccurred(0.25)`, once with `NodeTimerThresholdOccurred(0.5)`
    - Edge case: No threshold messages -- no-op

---

## Section F: Time Bridge System

### F1: on_time_expires

Lives in `src/effect/triggers/time/bridges.rs`. Runs in FixedUpdate, after `tick_effect_timers`.

57. **on_time_expires walks the specific entity referenced in the message**
    - Given: entity_a with `BoundEffects` and `StagedEffects` and `EffectTimers`; entity_b with effects; `EffectTimerExpired { entity: entity_a }` message
    - When: `on_time_expires` runs
    - Then: `walk_effects` called on entity_a (Self scope) with `Trigger::TimeExpires(original_duration)` and context `TriggerContext::None`; entity_b NOT walked
    - Edge case: Entity referenced in message no longer has EffectTimers (removed between tick and bridge) -- skip gracefully

58. **on_time_expires is Self-scoped, not global**
    - Given: entity_a with effects and EffectTimers containing `(OrderedFloat(0.0), OrderedFloat(5.0))`; entity_b with effects; `EffectTimerExpired { entity: entity_a }` message
    - When: `on_time_expires` runs
    - Then: Only entity_a is walked with `Trigger::TimeExpires(5.0)` -- the original_duration from the timer. entity_b is never walked.
    - Edge case: Multiple EffectTimerExpired messages for same entity -- each triggers a separate walk

59. **on_time_expires context is TriggerContext::None**
    - Given: entity_a with effects; `EffectTimerExpired { entity: entity_a }` message
    - When: `on_time_expires` runs
    - Then: Context is `TriggerContext::None`
    - Edge case: No messages -- no-op

---

## Section G: Game Systems (non-bridge systems that support triggers)

### G1: tick_effect_timers

Lives in `src/effect/triggers/time/tick_timers.rs`. Runs in FixedUpdate.

60. **tick_effect_timers decrements remaining time by dt**
    - Given: entity with `EffectTimers { timers: vec![(OrderedFloat(3.0), OrderedFloat(5.0))] }`; `Time<Fixed>` with delta of 1.0/60.0 (one frame at 60fps)
    - When: `tick_effect_timers` runs
    - Then: Timer remaining is approximately `3.0 - 1.0/60.0` = `OrderedFloat(2.9833...)`; original duration unchanged at `OrderedFloat(5.0)`
    - Edge case: dt is 0.0 -- timer remains unchanged

61. **tick_effect_timers sends EffectTimerExpired when timer reaches zero**
    - Given: entity with `EffectTimers { timers: vec![(OrderedFloat(0.01), OrderedFloat(5.0))] }`; `Time<Fixed>` with delta of 0.02
    - When: `tick_effect_timers` runs
    - Then: `EffectTimerExpired { entity }` message sent; timer entry removed from the vec
    - Edge case: Timer goes negative (0.01 - 0.02 = -0.01) -- still fires and removes

62. **tick_effect_timers removes EffectTimers component when vec is empty**
    - Given: entity with `EffectTimers { timers: vec![(OrderedFloat(0.001), OrderedFloat(5.0))] }`; dt of 0.016
    - When: `tick_effect_timers` runs
    - Then: Timer fires, entry removed, vec is empty, `EffectTimers` component removed from entity
    - Edge case: Entity had multiple timers but only one expired -- component remains with remaining timers

63. **tick_effect_timers handles multiple timers on one entity**
    - Given: entity with `EffectTimers { timers: vec![(OrderedFloat(1.0), OrderedFloat(3.0)), (OrderedFloat(0.005), OrderedFloat(10.0))] }`; dt of 0.016
    - When: `tick_effect_timers` runs
    - Then: First timer decremented to ~0.984; second timer expires (0.005 - 0.016 <= 0.0), sends `EffectTimerExpired { entity }`, second entry removed; component remains with one timer
    - Edge case: Both timers expire in same frame -- two EffectTimerExpired messages sent, both entries removed, component removed

64. **tick_effect_timers with no entities having EffectTimers is a no-op**
    - Given: No entities with `EffectTimers` component
    - When: `tick_effect_timers` runs
    - Then: Nothing happens; no messages sent
    - Edge case: Entities exist but none have EffectTimers -- still no-op

### G2: check_node_timer_thresholds

Lives in `src/effect/triggers/node/check_thresholds.rs`. Runs in FixedUpdate, after node timer tick.

65. **check_node_timer_thresholds sends message when ratio crosses threshold**
    - Given: Node timer ratio is 0.55 (55% elapsed); `NodeTimerThresholdRegistry { thresholds: vec![OrderedFloat(0.25), OrderedFloat(0.5), OrderedFloat(0.75)], fired: HashSet::from([OrderedFloat(0.25)]) }`
    - When: `check_node_timer_thresholds` runs
    - Then: `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.5) }` message sent; `OrderedFloat(0.5)` added to `fired`; 0.75 NOT fired (ratio 0.55 < 0.75); 0.25 NOT re-fired (already in fired set)
    - Edge case: Ratio exactly equals threshold (0.5 >= 0.5) -- fires

66. **check_node_timer_thresholds does not re-fire already fired thresholds**
    - Given: Node timer ratio is 0.8; `NodeTimerThresholdRegistry { thresholds: vec![OrderedFloat(0.5), OrderedFloat(0.75)], fired: HashSet::from([OrderedFloat(0.5), OrderedFloat(0.75)]) }`
    - When: `check_node_timer_thresholds` runs
    - Then: No messages sent -- both thresholds already in fired set
    - Edge case: All thresholds already fired -- no-op every frame until end

67. **check_node_timer_thresholds fires multiple thresholds in one frame**
    - Given: Node timer ratio jumped from 0.2 to 0.6 (large dt); `NodeTimerThresholdRegistry { thresholds: vec![OrderedFloat(0.25), OrderedFloat(0.5)], fired: HashSet::new() }`
    - When: `check_node_timer_thresholds` runs
    - Then: Two messages sent: `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.25) }` and `NodeTimerThresholdCrossed { ratio: OrderedFloat(0.5) }`; both added to fired set
    - Edge case: Timer penalty causes ratio to jump backwards past a threshold -- threshold was already fired so no duplicate fire

68. **check_node_timer_thresholds with empty thresholds is a no-op**
    - Given: Node timer ratio is 0.9; `NodeTimerThresholdRegistry { thresholds: vec![], fired: HashSet::new() }`
    - When: `check_node_timer_thresholds` runs
    - Then: No messages sent
    - Edge case: Registry exists but no trees use NodeTimerThresholdOccurred -- thresholds vec is empty

### G3: track_combo_streak

Lives in `src/effect/conditions/` (or `src/effect/triggers/` -- follows design doc location). Runs in FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`.

69. **track_combo_streak increments count on Perfect bump**
    - Given: `ComboStreak { count: 3 }` resource; `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 4 }`
    - Edge case: count was 0 -- becomes 1

70. **track_combo_streak resets count to zero on Early bump**
    - Given: `ComboStreak { count: 5 }` resource; `BumpPerformed { grade: BumpGrade::Early, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 0 }`
    - Edge case: count was already 0 -- stays 0

71. **track_combo_streak resets count to zero on Late bump**
    - Given: `ComboStreak { count: 2 }` resource; `BumpPerformed { grade: BumpGrade::Late, bolt: Some(bolt_entity), breaker: breaker_entity }` message
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 0 }`
    - Edge case: Multiple BumpPerformed messages in one frame -- process all in order

72. **track_combo_streak resets count on BumpWhiffed**
    - Given: `ComboStreak { count: 7 }` resource; `BumpWhiffed` message
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 0 }`
    - Edge case: Both BumpPerformed(Perfect) and BumpWhiffed in same frame -- final state depends on processing order

73. **track_combo_streak resets count on NoBump (BoltImpactBreaker with Inactive status)**
    - Given: `ComboStreak { count: 3 }` resource; `BoltImpactBreaker { bolt: bolt_entity, breaker: breaker_entity, bump_status: BumpStatus::Inactive }` message (indicating NoBump)
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 0 }`
    - Edge case: BoltImpactBreaker with Active status -- does NOT reset (that is a graded bump, handled by BumpPerformed)

74. **track_combo_streak with no messages does not change count**
    - Given: `ComboStreak { count: 4 }` resource; no BumpPerformed, BumpWhiffed, or NoBump-triggering messages
    - When: `track_combo_streak` runs
    - Then: `ComboStreak { count: 4 }` unchanged
    - Edge case: Resource was just initialized (count: 0) -- stays 0

75. **track_combo_streak persists across nodes (NOT reset on node start)**
    - Given: `ComboStreak { count: 5 }` resource; node state transitions from Playing -> ChipSelect -> Playing
    - When: No messages arrive between transitions
    - Then: `ComboStreak { count: 5 }` -- streak is preserved across nodes
    - Edge case: Player chains perfect bumps across 3 nodes -- count accumulates continuously

### G4: watch_spawn_registry

Lives in `src/effect/triggers/` (likely `src/effect/triggers/spawn/watch.rs` or similar). Runs in FixedUpdate, after entity spawning systems, in `EffectSystems::Bridge`.

76. **watch_spawn_registry stamps tree on newly spawned bolt**
    - Given: `SpawnStampRegistry` resource containing entry `("chip_a".to_string(), EntityKind::Bolt, tree_clone)`; new entity spawned with `Bolt` component this frame (detected via `Added<Bolt>`)
    - When: `watch_spawn_registry` runs
    - Then: The tree from the registry entry is cloned and stamped onto the new bolt entity (via stamp_effect or push_bound_effects)
    - Edge case: Bolt entity already has BoundEffects -- tree is appended, not replaced

77. **watch_spawn_registry stamps tree on newly spawned cell**
    - Given: `SpawnStampRegistry` resource containing entry `("chip_b".to_string(), EntityKind::Cell, tree_clone)`; new entity spawned with `Cell` component this frame (detected via `Added<Cell>`)
    - When: `watch_spawn_registry` runs
    - Then: Tree stamped onto new cell entity
    - Edge case: Multiple cells spawned in one frame -- each gets its own independent tree copy

78. **watch_spawn_registry stamps tree on newly spawned wall**
    - Given: `SpawnStampRegistry` resource containing entry `("chip_c".to_string(), EntityKind::Wall, tree_clone)`; new entity spawned with `Wall` component this frame (detected via `Added<Wall>`)
    - When: `watch_spawn_registry` runs
    - Then: Tree stamped onto new wall entity
    - Edge case: No wall entities spawned -- nothing stamped

79. **watch_spawn_registry stamps tree on newly spawned breaker**
    - Given: `SpawnStampRegistry` resource containing entry `("chip_d".to_string(), EntityKind::Breaker, tree_clone)`; new entity spawned with `Breaker` component this frame (detected via `Added<Breaker>`)
    - When: `watch_spawn_registry` runs
    - Then: Tree stamped onto new breaker entity
    - Edge case: Breaker already existed from previous node -- Added<Breaker> only fires on new spawns

80. **watch_spawn_registry ignores entities that do not match registered EntityKind**
    - Given: `SpawnStampRegistry` with entry `("chip_a".to_string(), EntityKind::Bolt, tree)` only; new Cell entity spawned
    - When: `watch_spawn_registry` runs
    - Then: Cell entity is NOT stamped -- EntityKind::Bolt does not match Cell
    - Edge case: Registry has entries for multiple kinds -- each kind matches independently

81. **watch_spawn_registry with empty registry is a no-op**
    - Given: `SpawnStampRegistry` with empty vec; new Bolt entity spawned
    - When: `watch_spawn_registry` runs
    - Then: Nothing happens -- no entries to match against
    - Edge case: Registry was populated but all entries were removed -- same no-op behavior

82. **watch_spawn_registry stamps multiple trees from different sources onto same entity**
    - Given: `SpawnStampRegistry` with two entries: `("chip_a", EntityKind::Bolt, tree_a)` and `("chip_b", EntityKind::Bolt, tree_b)`; new Bolt entity spawned
    - When: `watch_spawn_registry` runs
    - Then: Both tree_a and tree_b are stamped onto the new Bolt entity -- each as an independent copy
    - Edge case: Same source name with same kind -- both still stamp (deduplication is not the registry's job)

### G5: reset_node_timer_thresholds

Lives in `src/effect/triggers/node/` (reset system). Runs on `OnEnter(NodeState::Playing)`.

83. **reset_node_timer_thresholds clears fired set on node start**
    - Given: `NodeTimerThresholdRegistry { thresholds: vec![OrderedFloat(0.25), OrderedFloat(0.5), OrderedFloat(0.75)], fired: HashSet::from([OrderedFloat(0.25), OrderedFloat(0.5)]) }`; state transitions to `NodeState::Playing`
    - When: `reset_node_timer_thresholds` runs (via OnEnter)
    - Then: `fired` is empty: `HashSet::new()`; `thresholds` unchanged: still `[0.25, 0.5, 0.75]`
    - Edge case: fired was already empty -- no-op

84. **reset_node_timer_thresholds does NOT clear thresholds vec**
    - Given: `NodeTimerThresholdRegistry { thresholds: vec![OrderedFloat(0.5)], fired: HashSet::from([OrderedFloat(0.5)]) }`
    - When: `reset_node_timer_thresholds` runs
    - Then: `thresholds` remains `[OrderedFloat(0.5)]`; only `fired` is cleared
    - Edge case: thresholds may have changed between nodes if new trees were installed -- that is the installation system's job, not reset's

---

## Types

### Existing types consumed (from prior waves and other domains)

- `Trigger` enum -- all variants listed in Section definitions above. Source: `effect/core/types/definitions/enums.rs`
- `TriggerContext` enum -- `Bump { bolt: Option<Entity>, breaker: Entity }`, `Impact { impactor: Entity, impactee: Entity }`, `Death { victim: Entity, killer: Option<Entity> }`, `BoltLost { bolt: Entity, breaker: Entity }`, `None`. Source: `effect/core/`
- `BoundEffects` component -- effect trees bound to an entity. Source: `effect/core/`
- `StagedEffects` component -- staged effect trees on an entity. Source: `effect/core/`
- `EffectTimers` component -- `{ timers: Vec<(OrderedFloat<f32>, OrderedFloat<f32>)> }`. Source: `effect/triggers/time/`
- `EntityKind` enum -- `Cell, Bolt, Wall, Breaker, Any`. Source: `effect/core/`
- `BumpGrade` enum -- `Perfect, Early, Late`. Source: `breaker/messages.rs`
- `BumpStatus` enum -- `Active, Inactive`. Source: `bolt/` or `breaker/`
- `BumpPerformed { grade: BumpGrade, bolt: Option<Entity>, breaker: Entity }` message. Source: `breaker/messages.rs`
- `BumpWhiffed` unit message. Source: `breaker/messages.rs`
- `BoltImpactBreaker { bolt: Entity, breaker: Entity, bump_status: BumpStatus }` message. Source: `bolt/messages.rs`
- `BoltImpactCell { bolt: Entity, cell: Entity }` message. Source: `bolt/messages.rs`
- `BoltImpactWall { bolt: Entity, wall: Entity }` message. Source: `bolt/messages.rs`
- `BreakerImpactCell { breaker: Entity, cell: Entity }` message. Source: `breaker/messages.rs`
- `BreakerImpactWall { breaker: Entity, wall: Entity }` message. Source: `breaker/messages.rs`
- `CellImpactWall { cell: Entity, wall: Entity }` message. Source: `cells/messages.rs`
- `BoltLost { bolt: Entity, breaker: Entity }` message (migrated from unit struct). Source: `bolt/messages.rs`
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }` message. Source: unified death pipeline
- `GameEntity` trait -- implemented on `Bolt`, `Cell`, `Wall`, `Breaker`
- `Bolt`, `Cell`, `Wall`, `Breaker` marker components
- `NodeState::Playing` state

### New types for this wave

- `EffectTimerExpired { entity: Entity }` -- `#[derive(Message, Clone, Debug)]`. Sent by `tick_effect_timers`, consumed by `on_time_expires`. Source: `effect/triggers/time/messages.rs`
- `NodeTimerThresholdCrossed { ratio: OrderedFloat<f32> }` -- `#[derive(Message, Clone, Debug)]`. Sent by `check_node_timer_thresholds`, consumed by `on_node_timer_threshold_occurred`. Source: `effect/triggers/node/messages.rs`
- `NodeTimerThresholdRegistry { thresholds: Vec<OrderedFloat<f32>>, fired: HashSet<OrderedFloat<f32>> }` -- `#[derive(Resource, Default)]`. Source: `effect/triggers/node/resources.rs`
- `ComboStreak { count: u32 }` -- `#[derive(Resource, Default)]`. Source: `effect/conditions/` or `effect/triggers/`
- `SpawnStampRegistry(Vec<(String, EntityKind, Tree)>)` -- `#[derive(Resource, Default)]`. Source: `effect/`

### Messages

- `EffectTimerExpired { entity: Entity }` -- sent by `tick_effect_timers`, consumed by `on_time_expires`
- `NodeTimerThresholdCrossed { ratio: OrderedFloat<f32> }` -- sent by `check_node_timer_thresholds`, consumed by `on_node_timer_threshold_occurred`

---

## Reference Files

- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/bump/` -- all bridge specs
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/impact/` -- impact bridge specs
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/death/` -- death bridge specs
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/bolt-lost/` -- bolt lost bridge spec
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/node/` -- node bridge and check_thresholds specs
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/time/` -- time bridge and tick_timers specs
- `docs/todos/detail/effect-refactor/dispatching-triggers/dispatch-algorithm.md` -- context population table
- `docs/todos/detail/effect-refactor/rust-types/trigger-context.md` -- TriggerContext enum
- `docs/todos/detail/effect-refactor/storing-effects/spawn-stamp-registry.md` -- SpawnStampRegistry
- `docs/todos/detail/effect-refactor/rust-types/resources/combo-streak.md` -- ComboStreak resource
- `docs/todos/detail/effect-refactor/rust-types/resources/node-timer-threshold-registry.md` -- NodeTimerThresholdRegistry

---

## Test Strategy Notes

### How to test bridge systems

Bridge systems call `walk_effects(entity, trigger, context, bound, staged, commands)`. Testing the bridge itself means verifying:
1. The correct trigger variant is passed
2. The correct TriggerContext is constructed
3. The correct entities are walked (local vs global vs self scope)
4. Grade/status filtering works correctly (skip wrong grades)

The simplest approach: spawn entities with `BoundEffects`/`StagedEffects` that contain trees gated on the trigger variant being tested. After running the system, verify that the walk produced the expected commands (effects fired, or spy/tracking mechanism shows walks happened). If `walk_effects` is a function from wave 4, consider whether to mock it or use a real tree with a trivial `Fire(SomeTestEffect)` leaf.

### Testing walk invocations without mocking

Since bridges call `walk_effects` which may produce commands, the test can:
1. Install a `BoundEffects` with `When(TriggerUnderTest, Fire(SomeMarkerEffect))` tree
2. Run the bridge system with the right message
3. Assert that the marker effect's fire command was queued (or a component was added)

This tests the full bridge-to-walk path. If walk_effects is not yet implemented (wave 4 dependency), use a stub that records its calls.

### Test file organization

Tests should follow the module layout in `src/effect/triggers/`:
- `src/effect/triggers/bump/` -- bump bridge tests
- `src/effect/triggers/impact/` -- impact bridge tests
- `src/effect/triggers/death/` -- death bridge tests
- `src/effect/triggers/bolt_lost/` -- bolt lost bridge tests
- `src/effect/triggers/node/` -- node bridge tests + check_thresholds tests + reset tests
- `src/effect/triggers/time/` -- time bridge tests + tick_timers tests
- `src/effect/triggers/spawn/` or similar -- watch_spawn_registry tests
- `src/effect/conditions/` -- combo_streak tests

Each system file has its own `#[cfg(test)] mod tests` block or sibling `tests.rs`.

---

## Scenario Coverage

- New invariants: none -- trigger bridges are intermediary plumbing that does not introduce new invariants. The effects they dispatch are covered by existing invariants.
- New scenarios: none -- trigger bridges will be exercised by existing chaos scenarios once the full effect pipeline is wired (wave 6+). Dedicated trigger bridge scenarios are not useful until fire/reverse is implemented.
- Self-test scenarios: none
- Layout updates: none

---

## Constraints

- Tests go in: Test files parallel to source files within `src/effect/triggers/` and `src/effect/conditions/`
- Do NOT test: Effect fire/reverse logic (wave 6), death pipeline systems (wave 7), walk_effects internals (wave 4), Until reversal logic, tree installation
- Do NOT test: Rendering, VFX, audio
- Do NOT modify: Any domain files outside `src/effect/` except shared type stubs that are needed to compile tests
- Dependencies from prior waves: `walk_effects` function (wave 4), `BoundEffects`/`StagedEffects`/`TriggerContext`/`Trigger` types (waves 1-3), `Tree`/`RootNode` types (wave 2). These must exist as at least stubs before wave 5 tests can compile.

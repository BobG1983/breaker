# Test Spec: Effect Domain — Wave 4 Non-System Functions

## Prerequisites

This wave requires the Wave 2 scaffold types to exist before writer-tests or writer-code can begin. Specifically:
- `Tree`, `ScopedTree`, `Terminal`, `ScopedTerminal` enums
- `Trigger`, `TriggerContext`, `ParticipantTarget` enums and sub-enums
- `EffectType`, `ReversibleEffectType` enums
- `RouteType`, `Condition`, `EntityKind` enums
- `BoundEffects`, `BoundEntry`, `StagedEffects` components
- `EffectStack<T>` generic component
- All config structs (`SpeedBoostConfig`, `DamageBoostConfig`, etc.)
- `Fireable`, `Reversible`, `PassiveEffect` traits
- All command structs (`StampEffectCommand`, `FireEffectCommand`, `ReverseEffectCommand`, `RouteEffectCommand`, `StageEffectCommand`, `RemoveEffectCommand`)

These types are defined in the Wave 2 type specs. If they do not exist when this wave starts, tests will not compile.

## Domain
src/effect_v3/

---

## Section A: EffectStack<T>

### Behavior

1. **push appends a (source, config) entry to an empty stack**
   - Given: An empty `EffectStack<SpeedBoostConfig>` (entries: [])
   - When: `stack.push("chip_a".to_string(), SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` has 1 entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Edge case: Push to a stack that has no entries yet — must not panic, must produce length 1.

2. **push appends a second entry without replacing the first**
   - Given: An `EffectStack<DamageBoostConfig>` with one entry: `("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.0) })`
   - When: `stack.push("chip_b".to_string(), DamageBoostConfig { multiplier: OrderedFloat(1.25) })`
   - Then: `stack.entries` has 2 entries in order: `[("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.0) }), ("chip_b", DamageBoostConfig { multiplier: OrderedFloat(1.25) })]`
   - Edge case: Both entries have different sources — order is preserved as insertion order.

3. **push allows duplicate (source, config) entries**
   - Given: An `EffectStack<SpeedBoostConfig>` with one entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - When: `stack.push("chip_a".to_string(), SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` has 2 identical entries: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`
   - Edge case: Exact duplicate source + config pair. Both persist. Length is 2.

4. **remove finds and removes the first matching (source, config) entry**
   - Given: An `EffectStack<SpeedBoostConfig>` with entries: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_b", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })]`
   - When: `stack.remove("chip_a", &SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` has 1 entry: `[("chip_b", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })]`
   - Edge case: Only one entry matches — it is removed, the other remains.

5. **remove only removes the first of multiple identical entries**
   - Given: An `EffectStack<SpeedBoostConfig>` with entries: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`
   - When: `stack.remove("chip_a", &SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` has 1 entry: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`
   - Edge case: Two identical entries, only the first is removed. Second remains.

6. **remove with no matching entry does nothing**
   - Given: An `EffectStack<SpeedBoostConfig>` with one entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - When: `stack.remove("chip_b", &SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` is unchanged — still 1 entry: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`
   - Edge case: Source mismatch. No panic, no removal.

7. **remove with matching source but different config does nothing**
   - Given: An `EffectStack<SpeedBoostConfig>` with one entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - When: `stack.remove("chip_a", &SpeedBoostConfig { multiplier: OrderedFloat(2.0) })`
   - Then: `stack.entries` is unchanged — still 1 entry
   - Edge case: Config mismatch with correct source. Both source AND config must match.

8. **remove on an empty stack does nothing**
   - Given: An empty `EffectStack<SpeedBoostConfig>` (entries: [])
   - When: `stack.remove("chip_a", &SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
   - Then: `stack.entries` is still empty
   - Edge case: Removing from empty stack must not panic.

9. **aggregate on a multiplicative stack returns the product of all multiplier values**
   - Given: An `EffectStack<SpeedBoostConfig>` with entries: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_b", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })]`
   - When: `stack.aggregate()`
   - Then: Returns `3.0` (1.5 * 2.0)
   - Edge case: Two entries with different values — product, not sum.

10. **aggregate on an empty multiplicative stack returns 1.0 (identity)**
    - Given: An empty `EffectStack<SpeedBoostConfig>` (entries: [])
    - When: `stack.aggregate()`
    - Then: Returns `1.0`
    - Edge case: Empty stack returns multiplicative identity, not 0.0.

11. **aggregate on an additive stack returns the sum of all values**
    - Given: An `EffectStack<PiercingConfig>` with entries: `[("chip_a", PiercingConfig { charges: 2 }), ("chip_b", PiercingConfig { charges: 3 })]`
    - When: `stack.aggregate()`
    - Then: Returns `5.0` (2 + 3, as f32)
    - Edge case: Additive effects return sum, not product.

12. **aggregate on an empty additive stack returns 0.0 (identity)**
    - Given: An empty `EffectStack<PiercingConfig>` (entries: [])
    - When: `stack.aggregate()`
    - Then: Returns `0.0`
    - Edge case: Empty additive stack returns 0.0, not 1.0.

13. **aggregate with a single entry returns that entry's value**
    - Given: An `EffectStack<DamageBoostConfig>` with one entry: `("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.5) })`
    - When: `stack.aggregate()`
    - Then: Returns `2.5`
    - Edge case: Single multiplicative entry returns its value directly.

13a. **aggregate with three entries returns the product of all three**
     - Given: An `EffectStack<SpeedBoostConfig>` with entries: `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_b", SpeedBoostConfig { multiplier: OrderedFloat(2.0) }), ("chip_c", SpeedBoostConfig { multiplier: OrderedFloat(0.5) })]`
     - When: `stack.aggregate()`
     - Then: Returns `1.5` (1.5 * 2.0 * 0.5)
     - Edge case: Three-entry case verifies fold behavior beyond the two-entry base case.

---

## Section B: Walking Algorithm (walk_effects)

### Behavior

14. **walk_effects processes StagedEffects before BoundEffects**
    - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }]` and StagedEffects containing `[("chip_b", When(Bumped, Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))))]`. Trigger is `Trigger::Bumped`, context is `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`.
    - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
    - Then: The staged entry's inner `Fire(DamageBoost(...))` dispatches `fire_effect` BEFORE the bound entry's inner `Fire(SpeedBoost(...))`. Both entries produce fire_effect commands. Additionally, a `remove_effect(entity, RouteType::Staged, "chip_b", ...)` command is queued for the staged entry. The bound entry is NOT removed.
    - Edge case: Staged-first order prevents a single trigger from cascading through multiple stages in one pass.

15. **walk_effects skips staged entries whose trigger does not match**
    - Given: Entity with StagedEffects containing `[("chip_a", When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))]`. Trigger is `Trigger::Bumped`.
    - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
    - Then: No commands are queued. The staged entry is not consumed (no remove_effect command).
    - Edge case: Trigger::Bumped does not match Trigger::Impacted(Cell) — exact equality required.

16. **walk_effects consumes all matching staged entries, not just the first**
    - Given: Entity with StagedEffects containing `[("chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))), ("chip_b", When(Bumped, Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))))]`. Trigger is `Trigger::Bumped`.
    - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
    - Then: Both entries match. Two `fire_effect` commands are queued (one for SpeedBoost, one for DamageBoost). Two `remove_effect(entity, RouteType::Staged, ...)` commands are queued (one for each consumed entry).
    - Edge case: Multiple staged entries matching the same trigger all fire and all are consumed.

16a. **walk_effects with mixed staged entries fires only those whose trigger matches**
     - Given: Entity with StagedEffects containing `[("chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))), ("chip_b", When(Impacted(Cell), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })))), ("chip_c", When(Bumped, Fire(SizeBoost(SizeBoostConfig { multiplier: OrderedFloat(0.5) }))))]`. Trigger is `Trigger::Bumped`.
     - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
     - Then: chip_a and chip_c match Bumped. Two `fire_effect` commands are queued (SpeedBoost for chip_a, SizeBoost for chip_c). Two `remove_effect(entity, RouteType::Staged, ...)` commands are queued (for chip_a and chip_c). chip_b does not match (Impacted(Cell) != Bumped) — no fire_effect or remove_effect for chip_b.
     - Edge case: Non-matching staged entries are untouched while matching ones are consumed. Three entries, two match, one does not.

17. **walk_effects skips BoundEntry with condition_active == Some(false)**
    - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: During(NodeActive, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: Some(false) }]`. Trigger is `Trigger::Bumped`.
    - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
    - Then: No commands are queued. The During entry is skipped because `condition_active` is `Some(false)`.
    - Edge case: During entries are handled by `evaluate_conditions`, not trigger walking.

17a. **walk_effects skips BoundEntry with condition_active == Some(true)**
     - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: During(NodeActive, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: Some(true) }]`. Trigger is `Trigger::Bumped`.
     - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
     - Then: No commands are queued. The During entry is skipped because `condition_active` is `Some(true)`.
     - Edge case: Both Some(true) and Some(false) are skipped — the check is `Some(_)`, not a specific bool value.

18. **walk_effects does not remove bound entries on match (re-arms automatically)**
    - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }]`. Trigger is `Trigger::Bumped`.
    - When: `walk_effects(entity, &Trigger::Bumped, &context, &bound, &staged, &mut commands)` is called
    - Then: A `fire_effect` command is queued. No `remove_effect` command is queued for the bound entry.
    - Edge case: Bound When entries persist and re-arm — they are NOT consumed like staged entries.

### Trigger Matching

19. **Trigger::Bumped matches only Trigger::Bumped, not other bump variants**
    - Given: Trigger is `Trigger::Bumped`
    - When: Compared against `Trigger::PerfectBumped`
    - Then: No match (not equal)
    - Edge case: Each bump variant is distinct — Bumped, PerfectBumped, EarlyBumped, LateBumped are all different triggers.

20. **Trigger::Impacted(Cell) matches only Trigger::Impacted(Cell), not Trigger::Impacted(Bolt)**
    - Given: Trigger is `Trigger::Impacted(EntityKind::Cell)`
    - When: Compared against `Trigger::Impacted(EntityKind::Bolt)`
    - Then: No match (not equal)
    - Edge case: EntityKind parameter must match exactly.

21. **Trigger::TimeExpires(5.0) matches only Trigger::TimeExpires(5.0), not Trigger::TimeExpires(10.0)**
    - Given: Trigger is `Trigger::TimeExpires(5.0)`
    - When: Compared against `Trigger::TimeExpires(10.0)`
    - Then: No match (not equal)
    - Edge case: f32 values use OrderedFloat for Eq — 5.0 != 10.0 exactly.

22. **Trigger::DeathOccurred(Cell) does not match Trigger::DeathOccurred(Bolt)**
    - Given: Trigger is `Trigger::DeathOccurred(EntityKind::Cell)`
    - When: Compared against `Trigger::DeathOccurred(EntityKind::Bolt)`
    - Then: No match (not equal)
    - Edge case: EntityKind variants are distinct within the same trigger variant.

---

## Section C: Per-Node Evaluators

### When Node

23. **When matching trigger evaluates non-gate inner tree**
    - Given: Tree is `When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Trigger is `Trigger::Bumped`.
    - When: The When node is evaluated
    - Then: The inner Fire leaf is reached, producing a `fire_effect(entity, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command.
    - Edge case: Fire is not a trigger gate, so it is evaluated recursively (not armed).

24. **When non-matching trigger produces no commands**
    - Given: Tree is `When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Trigger is `Trigger::Impacted(EntityKind::Cell)`.
    - When: The When node is evaluated
    - Then: No commands are queued.
    - Edge case: Complete mismatch — stops immediately.

25. **When with inner trigger gate arms to StagedEffects instead of recursing**
    - Given: Tree is `When(Bumped, When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Trigger is `Trigger::Bumped`. Source is "chip_a".
    - When: The outer When matches Bumped
    - Then: A `stage_effect(entity, "chip_a", When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` command is queued. The inner When is NOT evaluated recursively.
    - Edge case: Inner When is a trigger gate — it is armed, not evaluated.

26. **When with inner Once gate arms to StagedEffects**
    - Given: Tree is `When(Bumped, Once(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Trigger is `Trigger::Bumped`. Source is "chip_a".
    - When: The outer When matches Bumped
    - Then: A `stage_effect(entity, "chip_a", Once(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` command is queued.
    - Edge case: Once is also a trigger gate — same arming behavior as inner When.

27. **When with inner Until gate arms to StagedEffects**
    - Given: Tree is `When(Bumped, Until(TimeExpires(5.0), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Trigger is `Trigger::Bumped`. Source is "chip_a".
    - When: The outer When matches Bumped
    - Then: A `stage_effect(entity, "chip_a", Until(TimeExpires(5.0), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` command is queued.
    - Edge case: Until is a trigger gate — it is armed, not evaluated.

28. **When "bumped twice" pattern: inner When with same trigger is armed, not immediately fired**
    - Given: Tree is `When(Bumped, When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Trigger is `Trigger::Bumped`. Source is "chip_a".
    - When: The outer When matches Bumped
    - Then: A `stage_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` command is queued. No `fire_effect` command is queued. The inner When does NOT fire on this same Bumped trigger.
    - Edge case: Inner trigger gate is ALWAYS armed, even if its trigger matches the current trigger. This implements "bumped twice" — first bump arms, second bump fires.

29. **When with inner Sequence evaluates recursively (Sequence is not a trigger gate)**
    - Given: Tree is `When(Bumped, Sequence([Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))]))`. Trigger is `Trigger::Bumped`.
    - When: The outer When matches Bumped
    - Then: The Sequence is evaluated recursively. Two `fire_effect` commands are queued: SpeedBoost then DamageBoost.
    - Edge case: Sequence is NOT a trigger gate — it is evaluated immediately.

30. **When stays in storage after matching (re-arms)**
    - Given: A BoundEntry with tree `When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a".
    - When: The When matches Trigger::Bumped
    - Then: No `remove_effect` command is produced for this When entry. It remains in BoundEffects.
    - Edge case: When is repeating — it never removes itself. Contrast with Once.

### Once Node

31. **Once matching trigger evaluates inner tree and queues removal**
    - Given: A BoundEntry with tree `Once(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a".
    - When: The Once matches Trigger::Bumped
    - Then: A `fire_effect` command is queued for SpeedBoost. Then a `remove_effect(entity, Bound, "chip_a", Once(...))` command is queued.
    - Edge case: The tree fires BEFORE the removal is queued — fire then cleanup.

32. **Once non-matching trigger produces no commands and no removal**
    - Given: A BoundEntry with tree `Once(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`.
    - When: The Once is checked against Trigger::Impacted(EntityKind::Cell)
    - Then: No commands of any kind are queued. The Once remains in BoundEffects.
    - Edge case: Once only removes itself on a match — it does not remove on mismatch.

33. **Once with inner trigger gate arms to StagedEffects (same arming rules as When)**
    - Given: A BoundEntry with tree `Once(Bumped, When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Source is "chip_a".
    - When: The Once matches Trigger::Bumped
    - Then: A `stage_effect(entity, "chip_a", When(Impacted(Cell), Fire(SpeedBoost(...))))` command is queued. Then a `remove_effect(entity, Bound, "chip_a", Once(...))` is queued.
    - Edge case: Once uses the same arming rules as When for inner trigger gates.

### Sequence Node

Note: `Fire(...)` inside `Sequence` is `Terminal::Fire(EffectType)`, not `Tree::Fire(EffectType)`. The Sequence node takes `Vec<Terminal>`. `Route(...)` inside Sequence is `Terminal::Route(RouteType, Box<Tree>)`.

34. **Sequence evaluates Fire terminals left to right**
    - Given: Tree is `Sequence([Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))])`. Source is "chip_a".
    - When: The Sequence is evaluated
    - Then: Two `fire_effect` commands are queued in order: first SpeedBoost(1.5), then DamageBoost(2.0).
    - Edge case: Order is guaranteed left-to-right. Earlier effects may affect later ones.

35. **Sequence evaluates Route terminals**
    - Given: Tree is `Sequence([Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })), Route(Bound, When(Bumped, Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))))])`. Source is "chip_a". Current target entity is entity_42.
    - When: The Sequence is evaluated
    - Then: A `fire_effect` command is queued for SpeedBoost. Then a `route_effect(entity_42, "chip_a", When(Bumped, Fire(DamageBoost(...))), RouteType::Bound)` command is queued.
    - Edge case: Route terminals install trees for later, not immediate fire.

36. **Sequence with a single terminal evaluates that terminal**
    - Given: Tree is `Sequence([Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))])`. Source is "chip_a".
    - When: The Sequence is evaluated
    - Then: One `fire_effect` command is queued for SpeedBoost.
    - Edge case: Single-element sequence is valid.

37. **Sequence with an empty list produces no commands**
    - Given: Tree is `Sequence([])`. Source is "chip_a".
    - When: The Sequence is evaluated
    - Then: No commands are queued.
    - Edge case: Empty sequence is a no-op.

### On Node

> **Clarification:** `Fire(...)` inside `On` is `Terminal::Fire(EffectType)`, not `Tree::Fire(EffectType)`. The On node evaluates a Terminal (Fire or Route), not a full Tree.

38. **On resolves Bump(Bolt) participant from TriggerContext::Bump**
    - Given: Tree is `On(Bump(Bolt), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(bolt_entity, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued. The target is bolt_entity, NOT the owner.
    - Edge case: On redirects the Fire to the resolved participant entity.

39. **On resolves Bump(Breaker) participant from TriggerContext::Bump**
    - Given: Tree is `On(Bump(Breaker), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(breaker_entity, ...)` command is queued.
    - Edge case: On resolves to the breaker participant.

40. **On with Bump(Bolt) skips when bolt is None**
    - Given: Tree is `On(Bump(Bolt), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Bump { bolt: None, breaker: breaker_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: No commands are queued. The On is skipped because the bolt participant does not exist.
    - Edge case: None bolt in NoBump/BumpWhiff contexts — On silently skips.

41. **On resolves Impact(Impactor) participant from TriggerContext::Impact**
    - Given: Tree is `On(Impact(Impactor), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })))`. Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(impactor_entity, ...)` command is queued.
    - Edge case: On resolves to the impactor participant.

42. **On resolves Impact(Impactee) participant from TriggerContext::Impact**
    - Given: Tree is `On(Impact(Impactee), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })))`. Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(impactee_entity, ...)` command is queued.
    - Edge case: On resolves to the impactee participant.

43. **On resolves Death(Victim) participant from TriggerContext::Death**
    - Given: Tree is `On(Death(Victim), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Death { victim: victim_entity, killer: Some(killer_entity) }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(victim_entity, ...)` command is queued.
    - Edge case: On resolves to the victim participant.

44. **On resolves Death(Killer) participant, skips when killer is None**
    - Given: Tree is `On(Death(Killer), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Death { victim: victim_entity, killer: None }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: No commands are queued. The On is skipped because the killer participant is None (environmental death).
    - Edge case: Environmental death has no killer — On(Death(Killer)) silently skips.

45. **On resolves BoltLost(Bolt) from TriggerContext::BoltLost**
    - Given: Tree is `On(BoltLost(Bolt), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::BoltLost { bolt: bolt_entity, breaker: breaker_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(bolt_entity, ...)` command is queued.
    - Edge case: On resolves to the bolt from the BoltLost context.

46. **On resolves BoltLost(Breaker) from TriggerContext::BoltLost**
    - Given: Tree is `On(BoltLost(Breaker), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::BoltLost { bolt: bolt_entity, breaker: breaker_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `fire_effect(breaker_entity, ...)` command is queued.
    - Edge case: On resolves to the breaker from the BoltLost context.

47. **On with mismatched context variant skips (Bump target with Impact context)**
    - Given: Tree is `On(Bump(Bolt), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: No commands are queued. The On is skipped because the participant target type (Bump) does not match the context variant (Impact).
    - Edge case: Mismatched context type is silently skipped, not panicked.

48. **On with TriggerContext::None skips**
    - Given: Tree is `On(Bump(Bolt), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Context is `TriggerContext::None`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: No commands are queued. The On is skipped.
    - Edge case: None context has no participants to resolve.

49. **On with Route terminal installs tree on participant entity**
    - Given: Tree is `On(Impact(Impactee), Route(Bound, When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))))`. Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`. Source is "chip_a".
    - When: The On node is evaluated
    - Then: A `route_effect(impactee_entity, "chip_a", When(Bumped, Fire(SpeedBoost(...))), RouteType::Bound)` command is queued. The route targets the participant, not the owner.
    - Edge case: Route inside On uses the resolved participant entity.

### Route Node

50. **Route queues a route_effect command with the current target entity**
    - Given: Tree is `Route(Bound, When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Current target entity is entity_42. Source is "chip_a".
    - When: The Route node is evaluated
    - Then: A `route_effect(entity_42, "chip_a", When(Bumped, Fire(SpeedBoost(...))), RouteType::Bound)` command is queued.
    - Edge case: Route does NOT evaluate the tree contents — it installs for later.

51. **Route with RouteType::Staged queues a staged route_effect command**
    - Given: Tree is `Route(Staged, When(Impacted(Cell), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))))`. Current target entity is entity_42. Source is "chip_a".
    - When: The Route node is evaluated
    - Then: A `route_effect(entity_42, "chip_a", When(Impacted(Cell), Fire(DamageBoost(...))), RouteType::Staged)` command is queued.
    - Edge case: RouteType::Staged installs to StagedEffects (one-shot).

### Fire Node

52. **Fire node queues a fire_effect command on the owner entity**
    - Given: Tree is `Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))`. Owner entity is entity_42. Source is "chip_a".
    - When: The Fire leaf is evaluated
    - Then: A `fire_effect(entity_42, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued.
    - Edge case: Fire is a leaf — no recursion, no storage.

---

## Section C.5: Scoped Tree Functions (apply_scoped_tree / reverse_scoped_tree)

These are non-system helper functions used by `evaluate_conditions` (During) and the walking algorithm (Until). They are called when a During condition transitions or when an Until is installed/reversed. They operate on `ScopedTree` variants, not `Tree` variants.

### apply_scoped_tree

52a. **apply_scoped_tree with ScopedTree::Fire queues fire_effect using reversible_to_effect_type**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `fire_effect(entity_42, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued. The `ReversibleEffectType::SpeedBoost(...)` is converted to `EffectType::SpeedBoost(...)` via `reversible_to_effect_type`.
     - Edge case: ScopedTree::Fire holds a ReversibleEffectType, not an EffectType. The conversion to EffectType is required before calling fire_effect.

52b. **apply_scoped_tree with ScopedTree::Sequence fires effects left to right**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Sequence(vec![ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ReversibleEffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })])`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: Two `fire_effect` commands are queued in order: first `EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`, then `EffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })`. Each ReversibleEffectType is converted via `reversible_to_effect_type`.
     - Edge case: Left-to-right order matches the order in the Vec. Earlier effects may affect later ones.

52c. **apply_scoped_tree with ScopedTree::When installs listener via stage_effect**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `stage_effect(entity_42, "chip_a", Tree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))))` command is queued. The inner When is installed as a listener in StagedEffects, NOT evaluated immediately.
     - Edge case: When inside a scoped context is armed for later matching, not fired now.

52d. **apply_scoped_tree with ScopedTree::On resolves participant and fires on that entity**
     - Given: Entity is entity_42 (the owner). ScopedTree is `ScopedTree::On(ParticipantTarget::Bump(BumpTarget::Bolt), ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a". Context is `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `fire_effect(bolt_entity, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued. The target is bolt_entity (the resolved participant), NOT entity_42 (the owner).
     - Edge case: On redirects the fire to the resolved participant. The ReversibleEffectType is converted to EffectType via `reversible_to_effect_type`.

52e. **apply_scoped_tree with ScopedTree::On and Route terminal installs tree on participant**
     - Given: Entity is entity_42 (the owner). ScopedTree is `ScopedTree::On(ParticipantTarget::Impact(ImpactTarget::Impactee), ScopedTerminal::Route(RouteType::Bound, Box::new(Tree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))))))`. Source is "chip_a". Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `route_effect(impactee_entity, "chip_a", Tree::When(Trigger::Bumped, ...), RouteType::Bound)` command is queued. Target is impactee_entity.
     - Edge case: ScopedTerminal::Route during apply installs the tree on the participant via route_effect.

52f. **apply_scoped_tree with ScopedTree::On skips when participant is None**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::On(ParticipantTarget::Bump(BumpTarget::Bolt), ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a". Context is `TriggerContext::Bump { bolt: None, breaker: breaker_entity }`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: No commands are queued. The On is skipped because the bolt participant is None.
     - Edge case: Same None-participant skip behavior as regular On node evaluation.

52g. **apply_scoped_tree with empty ScopedTree::Sequence produces no commands**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Sequence(vec![])`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `apply_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: No commands are queued.
     - Edge case: Empty scoped sequence is a no-op.

### reverse_scoped_tree

52h. **reverse_scoped_tree with ScopedTree::Fire queues reverse_effect**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `reverse_effect(entity_42, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued. Note: reverse_effect takes ReversibleEffectType directly (no conversion needed).
     - Edge case: reverse_scoped_tree uses ReversibleEffectType directly, unlike apply which converts to EffectType.

52i. **reverse_scoped_tree with ScopedTree::Sequence reverses in right-to-left order**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Sequence(vec![ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ReversibleEffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })])`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: Two `reverse_effect` commands are queued in REVERSE order: first `ReversibleEffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })`, then `ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`.
     - Edge case: Reversal order is right-to-left, opposite of apply order. This ensures effects are undone in the opposite order they were applied.

52j. **reverse_scoped_tree with ScopedTree::When removes listener via remove_effect**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `remove_effect(entity_42, RouteType::Staged, "chip_a", Tree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))))` command is queued. The When listener is removed from StagedEffects. Individual effects that already fired from past trigger matches are NOT reversed.
     - Edge case: Only the listener is removed, not the effects it produced in the past.

52k. **reverse_scoped_tree with ScopedTree::On resolves participant and reverses on that entity**
     - Given: Entity is entity_42 (the owner). ScopedTree is `ScopedTree::On(ParticipantTarget::Bump(BumpTarget::Bolt), ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a". Context is `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `reverse_effect(bolt_entity, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` command is queued. Target is bolt_entity (the resolved participant), NOT entity_42.
     - Edge case: On redirects the reversal to the resolved participant.

52l. **reverse_scoped_tree with ScopedTree::On and Route terminal removes tree from participant**
     - Given: Entity is entity_42 (the owner). ScopedTree is `ScopedTree::On(ParticipantTarget::Impact(ImpactTarget::Impactee), ScopedTerminal::Route(RouteType::Bound, Box::new(Tree::When(Trigger::Bumped, Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))))))`. Source is "chip_a". Context is `TriggerContext::Impact { impactor: impactor_entity, impactee: impactee_entity }`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: A `remove_effect(impactee_entity, RouteType::Bound, "chip_a", Tree::When(Trigger::Bumped, ...))` command is queued. The tree previously installed via route_effect during apply is now removed.
     - Edge case: ScopedTerminal::Route during reverse calls remove_effect (not route_effect) on the participant.

52m. **reverse_scoped_tree with ScopedTree::On skips when participant is None**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::On(ParticipantTarget::Bump(BumpTarget::Bolt), ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))`. Source is "chip_a". Context is `TriggerContext::Bump { bolt: None, breaker: breaker_entity }`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: No commands are queued. The On is skipped because the bolt participant is None.
     - Edge case: Same None-participant skip behavior as apply direction.

52n. **reverse_scoped_tree with empty ScopedTree::Sequence produces no commands**
     - Given: Entity is entity_42. ScopedTree is `ScopedTree::Sequence(vec![])`. Source is "chip_a". Context is `TriggerContext::None`.
     - When: `reverse_scoped_tree(entity_42, &scoped, "chip_a", &context, &mut commands)` is called
     - Then: No commands are queued.
     - Edge case: Empty scoped sequence reversal is a no-op.

---

## Section D: Command Extensions

### stamp_effect

53. **stamp_effect appends tree to BoundEffects on entity**
    - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
    - When: `stamp_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity's BoundEffects has 1 entry: `BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(...))), condition_active: None }`
    - Edge case: Basic stamp to existing empty containers.

54. **stamp_effect inserts BoundEffects and StagedEffects if absent**
    - Given: Entity with no BoundEffects or StagedEffects component.
    - When: `stamp_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity now has BoundEffects with 1 entry AND StagedEffects (empty). Both are inserted as a pair.
    - Edge case: Components are created on demand.

55. **stamp_effect allows duplicate source entries**
    - Given: Entity with BoundEffects containing 1 entry with source "chip_a" and tree When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))).
    - When: `stamp_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied again with the same source and tree
    - Then: Entity's BoundEffects has 2 entries, both with source "chip_a" and the same tree.
    - Edge case: stamp_effect always appends — no dedup check. Callers must avoid double-stamping.

56. **stamp_effect sets condition_active to Some(false) for During trees**
    - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
    - When: `stamp_effect(entity, "chip_a", During(NodeActive, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity's BoundEffects has 1 entry: `BoundEntry { source: "chip_a", tree: During(NodeActive, ...), condition_active: Some(false) }`
    - Edge case: During entries start with `condition_active: Some(false)`, not `None`.

56a. **stamp_effect sets condition_active to None for Once trees**
     - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
     - When: `stamp_effect(entity, "chip_a", Once(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
     - Then: Entity's BoundEffects has 1 entry: `BoundEntry { source: "chip_a", tree: Once(Bumped, Fire(SpeedBoost(...))), condition_active: None }`
     - Edge case: Only During trees get `condition_active: Some(false)`. Once, When, Until, and all other tree variants get `condition_active: None`.

57. **stamp_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `stamp_effect(entity, "chip_a", ...)` is applied
    - Then: No panic. No effect. World is unchanged.
    - Edge case: Graceful no-op on despawned/invalid entity.

### fire_effect

58. **fire_effect dispatches to the correct config.fire() for a passive effect**
    - Given: Entity exists in the world with `EffectStack<SpeedBoostConfig>::default()`.
    - When: `fire_effect(entity, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` is applied
    - Then: Entity's `EffectStack<SpeedBoostConfig>` now has 1 entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
    - Edge case: Verifies the dispatch match arm correctly delegates to config.fire().

59. **fire_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `fire_effect(entity, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` is applied
    - Then: No panic. No effect.
    - Edge case: Graceful no-op on despawned entity.

### reverse_effect

60. **reverse_effect dispatches to the correct config.reverse() for a passive effect**
    - Given: Entity exists in the world with `EffectStack<SpeedBoostConfig>` containing `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`.
    - When: `reverse_effect(entity, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` is applied
    - Then: Entity's `EffectStack<SpeedBoostConfig>` is now empty.
    - Edge case: Verifies the dispatch match arm correctly delegates to config.reverse().

61. **reverse_effect with no matching entry does nothing**
    - Given: Entity exists in the world with an empty `EffectStack<SpeedBoostConfig>`.
    - When: `reverse_effect(entity, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` is applied
    - Then: Stack remains empty. No panic.
    - Edge case: No-match reverse is a safe no-op.

62. **reverse_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `reverse_effect(entity, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_a")` is applied
    - Then: No panic. No effect.
    - Edge case: Graceful no-op on despawned entity.

### route_effect

63. **route_effect with RouteType::Bound appends to BoundEffects**
    - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
    - When: `route_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), RouteType::Bound)` is applied
    - Then: Entity's BoundEffects has 1 entry: `BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(...))), condition_active: None }`. StagedEffects is unchanged (empty).
    - Edge case: Bound route goes to BoundEffects only.

64. **route_effect with RouteType::Staged appends to StagedEffects**
    - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
    - When: `route_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), RouteType::Staged)` is applied
    - Then: Entity's StagedEffects has 1 entry: `("chip_a", When(Bumped, Fire(SpeedBoost(...))))`. BoundEffects is unchanged (empty).
    - Edge case: Staged route goes to StagedEffects only.

65. **route_effect inserts both BoundEffects and StagedEffects if absent**
    - Given: Entity with no BoundEffects or StagedEffects component.
    - When: `route_effect(entity, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), RouteType::Bound)` is applied
    - Then: Entity now has BoundEffects with 1 entry AND StagedEffects (empty).
    - Edge case: Pair insertion when neither component is present.

66. **route_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `route_effect(entity, ...)` is applied
    - Then: No panic. No effect.
    - Edge case: Graceful no-op.

66a. **route_effect with RouteType::Bound and During tree sets condition_active to Some(false)**
     - Given: Entity with `BoundEffects(vec![])` and `StagedEffects(vec![])`.
     - When: `route_effect(entity, "chip_a", During(NodeActive, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), RouteType::Bound)` is applied
     - Then: Entity's BoundEffects has 1 entry: `BoundEntry { source: "chip_a", tree: During(NodeActive, ...), condition_active: Some(false) }`. The condition_active field is `Some(false)`, not `None`.
     - Edge case: During trees routed to Bound always start with condition_active: Some(false), same as stamp_effect behavior #56.

### stage_effect

67. **stage_effect appends tree to StagedEffects**
    - Given: Entity with `StagedEffects(vec![])` and `BoundEffects(vec![])`.
    - When: `stage_effect(entity, "chip_a", When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity's StagedEffects has 1 entry: `("chip_a", When(Impacted(Cell), Fire(SpeedBoost(...))))`.
    - Edge case: Sugar for route_effect with RouteType::Staged.

68. **stage_effect inserts both components if absent**
    - Given: Entity with no BoundEffects or StagedEffects.
    - When: `stage_effect(entity, "chip_a", ...)` is applied
    - Then: Entity now has both BoundEffects (empty) and StagedEffects (1 entry).
    - Edge case: Pair insertion.

69. **stage_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `stage_effect(entity, ...)` is applied
    - Then: No panic. No effect.
    - Edge case: Graceful no-op.

### remove_effect

70. **remove_effect with RouteType::Bound removes matching entry from BoundEffects**
    - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }]`.
    - When: `remove_effect(entity, RouteType::Bound, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity's BoundEffects is now empty.
    - Edge case: Matching is by (source, tree) equality.

71. **remove_effect with RouteType::Staged removes matching entry from StagedEffects**
    - Given: Entity with StagedEffects containing `[("chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))]`.
    - When: `remove_effect(entity, RouteType::Staged, "chip_a", When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))` is applied
    - Then: Entity's StagedEffects is now empty.
    - Edge case: Matching is by (source, tree) equality.

72. **remove_effect removes only the first matching entry**
    - Given: Entity with BoundEffects containing 2 entries both with source "chip_a" and the same tree.
    - When: `remove_effect(entity, RouteType::Bound, "chip_a", <same tree>)` is applied
    - Then: Entity's BoundEffects has 1 entry remaining.
    - Edge case: Only the first match is removed — not all matches.

73. **remove_effect with no matching entry does nothing**
    - Given: Entity with BoundEffects containing `[BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(...))), condition_active: None }]`.
    - When: `remove_effect(entity, RouteType::Bound, "chip_b", When(Bumped, Fire(SpeedBoost(...))))` is applied (wrong source)
    - Then: BoundEffects is unchanged. No panic.
    - Edge case: Source mismatch — no removal.

74. **remove_effect on nonexistent entity does nothing**
    - Given: Entity does not exist in the world.
    - When: `remove_effect(entity, ...)` is applied
    - Then: No panic. No effect.
    - Edge case: Graceful no-op.

---

## Section E: Passive Effect Trait Implementations

### SpeedBoostConfig fire/reverse/aggregate

75. **SpeedBoostConfig.fire pushes to EffectStack**
    - Given: Entity exists with `EffectStack<SpeedBoostConfig>::default()` (empty).
    - When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
    - Then: Entity's `EffectStack<SpeedBoostConfig>` has 1 entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
    - Edge case: Fire on empty stack inserts the first entry.

76. **SpeedBoostConfig.fire inserts EffectStack if absent**
    - Given: Entity exists without `EffectStack<SpeedBoostConfig>`.
    - When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
    - Then: Entity now has `EffectStack<SpeedBoostConfig>` with 1 entry.
    - Edge case: Missing stack component is inserted with default before pushing.

77. **SpeedBoostConfig.reverse removes matching entry from EffectStack**
    - Given: Entity with `EffectStack<SpeedBoostConfig>` containing `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`.
    - When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
    - Then: Entity's `EffectStack<SpeedBoostConfig>` is now empty.
    - Edge case: Exact (source, config) match required.

78. **SpeedBoostConfig.reverse on missing EffectStack does nothing**
    - Given: Entity exists without `EffectStack<SpeedBoostConfig>`.
    - When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
    - Then: No panic. No effect.
    - Edge case: Missing stack is safe no-op.

79. **SpeedBoostConfig.aggregate returns product of multipliers**
    - Given: Entries `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_b", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })]`
    - When: `SpeedBoostConfig::aggregate(&entries)`
    - Then: Returns `3.0` (1.5 * 2.0)
    - Edge case: Multiplicative aggregation.

80. **SpeedBoostConfig.aggregate on empty returns 1.0**
    - Given: Empty entries `[]`
    - When: `SpeedBoostConfig::aggregate(&entries)`
    - Then: Returns `1.0`
    - Edge case: Multiplicative identity.

### DamageBoostConfig fire/reverse/aggregate

81. **DamageBoostConfig.fire pushes to EffectStack**
    - Given: Entity with empty `EffectStack<DamageBoostConfig>`.
    - When: `DamageBoostConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_a", world)`
    - Then: Stack has 1 entry: `("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.0) })`
    - Edge case: Standard passive fire pattern.

82. **DamageBoostConfig.reverse removes matching entry**
    - Given: Entity with `EffectStack<DamageBoostConfig>` containing `[("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.0) })]`.
    - When: `DamageBoostConfig { multiplier: OrderedFloat(2.0) }.reverse(entity, "chip_a", world)`
    - Then: Stack is empty.
    - Edge case: Standard passive reverse pattern.

83. **DamageBoostConfig.aggregate returns product**
    - Given: Entries `[("a", DamageBoostConfig { multiplier: OrderedFloat(1.25) }), ("b", DamageBoostConfig { multiplier: OrderedFloat(3.0) })]`
    - When: `DamageBoostConfig::aggregate(&entries)`
    - Then: Returns `3.75` (1.25 * 3.0)
    - Edge case: Multiplicative aggregation.

84. **DamageBoostConfig.aggregate on empty returns 1.0**
    - Given: Empty entries `[]`
    - When: `DamageBoostConfig::aggregate(&entries)`
    - Then: Returns `1.0`
    - Edge case: Multiplicative identity.

### SizeBoostConfig fire/reverse/aggregate

85. **SizeBoostConfig.fire pushes to EffectStack**
    - Given: Entity with empty `EffectStack<SizeBoostConfig>`.
    - When: `SizeBoostConfig { multiplier: OrderedFloat(0.5) }.fire(entity, "chip_a", world)`
    - Then: Stack has 1 entry: `("chip_a", SizeBoostConfig { multiplier: OrderedFloat(0.5) })`
    - Edge case: Multiplier below 1.0 (shrink).

86. **SizeBoostConfig.reverse removes matching entry**
    - Given: Entity with `EffectStack<SizeBoostConfig>` containing `[("chip_a", SizeBoostConfig { multiplier: OrderedFloat(0.5) })]`.
    - When: `SizeBoostConfig { multiplier: OrderedFloat(0.5) }.reverse(entity, "chip_a", world)`
    - Then: Stack is empty.
    - Edge case: Standard passive reverse.

87. **SizeBoostConfig.aggregate returns product**
    - Given: Entries `[("a", SizeBoostConfig { multiplier: OrderedFloat(2.0) }), ("b", SizeBoostConfig { multiplier: OrderedFloat(0.5) })]`
    - When: `SizeBoostConfig::aggregate(&entries)`
    - Then: Returns `1.0` (2.0 * 0.5)
    - Edge case: Grow + shrink cancels out.

88. **SizeBoostConfig.aggregate on empty returns 1.0**
    - Given: Empty entries `[]`
    - When: `SizeBoostConfig::aggregate(&entries)`
    - Then: Returns `1.0`
    - Edge case: Multiplicative identity.

### BumpForceConfig fire/reverse/aggregate

89. **BumpForceConfig.fire pushes to EffectStack**
    - Given: Entity with empty `EffectStack<BumpForceConfig>`.
    - When: `BumpForceConfig { multiplier: OrderedFloat(1.3) }.fire(entity, "chip_a", world)`
    - Then: Stack has 1 entry: `("chip_a", BumpForceConfig { multiplier: OrderedFloat(1.3) })`
    - Edge case: Standard passive fire.

90. **BumpForceConfig.reverse removes matching entry**
    - Given: Entity with `EffectStack<BumpForceConfig>` containing `[("chip_a", BumpForceConfig { multiplier: OrderedFloat(1.3) })]`.
    - When: `BumpForceConfig { multiplier: OrderedFloat(1.3) }.reverse(entity, "chip_a", world)`
    - Then: Stack is empty.
    - Edge case: Standard passive reverse.

91. **BumpForceConfig.aggregate returns product**
    - Given: Entries `[("a", BumpForceConfig { multiplier: OrderedFloat(1.3) }), ("b", BumpForceConfig { multiplier: OrderedFloat(1.5) })]`
    - When: `BumpForceConfig::aggregate(&entries)`
    - Then: Returns `1.95` (1.3 * 1.5)
    - Edge case: Multiplicative aggregation.

92. **BumpForceConfig.aggregate on empty returns 1.0**
    - Given: Empty entries `[]`
    - When: `BumpForceConfig::aggregate(&entries)`
    - Then: Returns `1.0`
    - Edge case: Multiplicative identity.

### QuickStopConfig fire/reverse/aggregate

93. **QuickStopConfig.fire pushes to EffectStack**
    - Given: Entity with empty `EffectStack<QuickStopConfig>`.
    - When: `QuickStopConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_a", world)`
    - Then: Stack has 1 entry: `("chip_a", QuickStopConfig { multiplier: OrderedFloat(2.0) })`
    - Edge case: Standard passive fire.

94. **QuickStopConfig.reverse removes matching entry**
    - Given: Entity with `EffectStack<QuickStopConfig>` containing `[("chip_a", QuickStopConfig { multiplier: OrderedFloat(2.0) })]`.
    - When: `QuickStopConfig { multiplier: OrderedFloat(2.0) }.reverse(entity, "chip_a", world)`
    - Then: Stack is empty.
    - Edge case: Standard passive reverse.

95. **QuickStopConfig.aggregate returns product**
    - Given: Entries `[("a", QuickStopConfig { multiplier: OrderedFloat(2.0) }), ("b", QuickStopConfig { multiplier: OrderedFloat(3.0) })]`
    - When: `QuickStopConfig::aggregate(&entries)`
    - Then: Returns `6.0` (2.0 * 3.0)
    - Edge case: Multiplicative aggregation.

96. **QuickStopConfig.aggregate on empty returns 1.0**
    - Given: Empty entries `[]`
    - When: `QuickStopConfig::aggregate(&entries)`
    - Then: Returns `1.0`
    - Edge case: Multiplicative identity.

### VulnerableConfig fire/reverse/aggregate

97. **VulnerableConfig.fire pushes to EffectStack**
    - Given: Entity with empty `EffectStack<VulnerableConfig>`.
    - When: `VulnerableConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
    - Then: Stack has 1 entry: `("chip_a", VulnerableConfig { multiplier: OrderedFloat(1.5) })`
    - Edge case: Standard passive fire.

98. **VulnerableConfig.reverse removes matching entry**
    - Given: Entity with `EffectStack<VulnerableConfig>` containing `[("chip_a", VulnerableConfig { multiplier: OrderedFloat(1.5) })]`.
    - When: `VulnerableConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
    - Then: Stack is empty.
    - Edge case: Standard passive reverse.

99. **VulnerableConfig.aggregate returns product**
    - Given: Entries `[("a", VulnerableConfig { multiplier: OrderedFloat(1.5) }), ("b", VulnerableConfig { multiplier: OrderedFloat(2.0) })]`
    - When: `VulnerableConfig::aggregate(&entries)`
    - Then: Returns `3.0` (1.5 * 2.0)
    - Edge case: Multiplicative aggregation.

100. **VulnerableConfig.aggregate on empty returns 1.0**
     - Given: Empty entries `[]`
     - When: `VulnerableConfig::aggregate(&entries)`
     - Then: Returns `1.0`
     - Edge case: Multiplicative identity.

### PiercingConfig fire/reverse/aggregate (Additive)

101. **PiercingConfig.fire pushes to EffectStack**
     - Given: Entity with empty `EffectStack<PiercingConfig>`.
     - When: `PiercingConfig { charges: 3 }.fire(entity, "chip_a", world)`
     - Then: Stack has 1 entry: `("chip_a", PiercingConfig { charges: 3 })`
     - Edge case: Standard passive fire.

102. **PiercingConfig.reverse removes matching entry**
     - Given: Entity with `EffectStack<PiercingConfig>` containing `[("chip_a", PiercingConfig { charges: 3 })]`.
     - When: `PiercingConfig { charges: 3 }.reverse(entity, "chip_a", world)`
     - Then: Stack is empty.
     - Edge case: Standard passive reverse.

103. **PiercingConfig.aggregate returns sum of charges**
     - Given: Entries `[("a", PiercingConfig { charges: 2 }), ("b", PiercingConfig { charges: 3 })]`
     - When: `PiercingConfig::aggregate(&entries)`
     - Then: Returns `5.0` (2 + 3, as f32)
     - Edge case: Additive aggregation, not multiplicative.

104. **PiercingConfig.aggregate on empty returns 0.0 (additive identity)**
     - Given: Empty entries `[]`
     - When: `PiercingConfig::aggregate(&entries)`
     - Then: Returns `0.0`
     - Edge case: Additive identity is 0.0, not 1.0.

### RampingDamageConfig fire/reverse/aggregate (Additive)

105. **RampingDamageConfig.fire pushes to EffectStack**
     - Given: Entity with empty `EffectStack<RampingDamageConfig>`.
     - When: `RampingDamageConfig { increment: OrderedFloat(0.5) }.fire(entity, "chip_a", world)`
     - Then: Stack has 1 entry: `("chip_a", RampingDamageConfig { increment: OrderedFloat(0.5) })`
     - Edge case: Standard passive fire.

106. **RampingDamageConfig.reverse removes matching entry**
     - Given: Entity with `EffectStack<RampingDamageConfig>` containing `[("chip_a", RampingDamageConfig { increment: OrderedFloat(0.5) })]`.
     - When: `RampingDamageConfig { increment: OrderedFloat(0.5) }.reverse(entity, "chip_a", world)`
     - Then: Stack is empty.
     - Edge case: Standard passive reverse.

107. **RampingDamageConfig.aggregate returns sum of increments**
     - Given: Entries `[("a", RampingDamageConfig { increment: OrderedFloat(0.5) }), ("b", RampingDamageConfig { increment: OrderedFloat(1.0) })]`
     - When: `RampingDamageConfig::aggregate(&entries)`
     - Then: Returns `1.5` (0.5 + 1.0)
     - Edge case: Additive aggregation.

108. **RampingDamageConfig.aggregate on empty returns 0.0**
     - Given: Empty entries `[]`
     - When: `RampingDamageConfig::aggregate(&entries)`
     - Then: Returns `0.0`
     - Edge case: Additive identity.

---

## Section F: Condition Evaluator Functions

### is_node_active

109. **is_node_active returns true when NodeState is Playing**
     - Given: World with `State<NodeState>` resource set to `NodeState::Playing`.
     - When: `is_node_active(&world)` is called
     - Then: Returns `true`
     - Edge case: Playing is the active state.

110. **is_node_active returns false when NodeState is not Playing**
     - Given: World with `State<NodeState>` resource set to a non-Playing state (e.g., `NodeState::Setup` or equivalent inactive state).
     - When: `is_node_active(&world)` is called
     - Then: Returns `false`
     - Edge case: Any state other than Playing returns false.

### is_shield_active

111. **is_shield_active returns true when at least one ShieldWall entity exists**
     - Given: World with 1 entity that has the `ShieldWall` component.
     - When: `is_shield_active(&mut world)` is called
     - Then: Returns `true`
     - Edge case: One shield wall is sufficient.

112. **is_shield_active returns false when no ShieldWall entities exist**
     - Given: World with no entities that have the `ShieldWall` component.
     - When: `is_shield_active(&mut world)` is called
     - Then: Returns `false`
     - Edge case: Zero shield walls returns false.

113. **is_shield_active returns true with multiple ShieldWall entities**
     - Given: World with 3 entities that each have the `ShieldWall` component.
     - When: `is_shield_active(&mut world)` is called
     - Then: Returns `true`
     - Edge case: Any non-zero count is true — does not distinguish between 1 and 3.

### is_combo_active

114. **is_combo_active returns true when combo count is at threshold**
     - Given: World with `ComboStreak { count: 5 }` resource inserted. Threshold is 5.
     - When: `is_combo_active(&mut world, 5)` is called
     - Then: Returns `true`
     - Edge case: Exactly at threshold returns true (>= comparison).

115. **is_combo_active returns true when combo count exceeds threshold**
     - Given: World with `ComboStreak { count: 8 }` resource inserted. Threshold is 5.
     - When: `is_combo_active(&mut world, 5)` is called
     - Then: Returns `true`
     - Edge case: Above threshold returns true.

116. **is_combo_active returns false when combo count is below threshold**
     - Given: World with `ComboStreak { count: 3 }` resource inserted. Threshold is 5.
     - When: `is_combo_active(&mut world, 5)` is called
     - Then: Returns `false`
     - Edge case: Below threshold returns false.

117. **is_combo_active returns false when combo count is zero**
     - Given: World with `ComboStreak { count: 0 }` resource inserted. Threshold is 1.
     - When: `is_combo_active(&mut world, 1)` is called
     - Then: Returns `false`
     - Edge case: Zero streak is always below any positive threshold.

---

## Section G: Fire/Reverse Dispatch (EffectType and ReversibleEffectType match arms)

### EffectType fire dispatch

118. **EffectType::SpeedBoost dispatches to SpeedBoostConfig.fire()**
     - Given: Entity exists. `EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`.
     - When: The fire dispatch match arm is executed with entity and source "chip_a"
     - Then: `SpeedBoostConfig.fire(entity, "chip_a", world)` is called, producing the effect described in behavior #75.
     - Edge case: Each variant must dispatch to its own config type.

119. **EffectType::DamageBoost dispatches to DamageBoostConfig.fire()**
     - Given: Entity exists. `EffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })`.
     - When: The fire dispatch is executed
     - Then: `DamageBoostConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

120. **EffectType::SizeBoost dispatches to SizeBoostConfig.fire()**
     - Given: Entity exists. `EffectType::SizeBoost(SizeBoostConfig { multiplier: OrderedFloat(0.8) })`.
     - When: The fire dispatch is executed
     - Then: `SizeBoostConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

121. **EffectType::BumpForce dispatches to BumpForceConfig.fire()**
     - Given: Entity exists. `EffectType::BumpForce(BumpForceConfig { multiplier: OrderedFloat(1.3) })`.
     - When: The fire dispatch is executed
     - Then: `BumpForceConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

122. **EffectType::QuickStop dispatches to QuickStopConfig.fire()**
     - Given: Entity exists. `EffectType::QuickStop(QuickStopConfig { multiplier: OrderedFloat(2.0) })`.
     - When: The fire dispatch is executed
     - Then: `QuickStopConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

123. **EffectType::Vulnerable dispatches to VulnerableConfig.fire()**
     - Given: Entity exists. `EffectType::Vulnerable(VulnerableConfig { multiplier: OrderedFloat(1.5) })`.
     - When: The fire dispatch is executed
     - Then: `VulnerableConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

124. **EffectType::Piercing dispatches to PiercingConfig.fire()**
     - Given: Entity exists. `EffectType::Piercing(PiercingConfig { charges: 3 })`.
     - When: The fire dispatch is executed
     - Then: `PiercingConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

125. **EffectType::RampingDamage dispatches to RampingDamageConfig.fire()**
     - Given: Entity exists. `EffectType::RampingDamage(RampingDamageConfig { increment: OrderedFloat(0.5) })`.
     - When: The fire dispatch is executed
     - Then: `RampingDamageConfig.fire(entity, "chip_a", world)` is called.
     - Edge case: Distinct variant, distinct config.

### ReversibleEffectType reverse dispatch

126. **ReversibleEffectType::SpeedBoost dispatches to SpeedBoostConfig.reverse()**
     - Given: Entity with `EffectStack<SpeedBoostConfig>` containing `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Reverse dispatch calls the correct config.reverse().

127. **ReversibleEffectType::DamageBoost dispatches to DamageBoostConfig.reverse()**
     - Given: Entity with `EffectStack<DamageBoostConfig>` containing `[("chip_a", DamageBoostConfig { multiplier: OrderedFloat(2.0) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

128. **ReversibleEffectType::SizeBoost dispatches to SizeBoostConfig.reverse()**
     - Given: Entity with `EffectStack<SizeBoostConfig>` containing `[("chip_a", SizeBoostConfig { multiplier: OrderedFloat(0.8) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::SizeBoost(SizeBoostConfig { multiplier: OrderedFloat(0.8) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

129. **ReversibleEffectType::BumpForce dispatches to BumpForceConfig.reverse()**
     - Given: Entity with `EffectStack<BumpForceConfig>` containing `[("chip_a", BumpForceConfig { multiplier: OrderedFloat(1.3) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::BumpForce(BumpForceConfig { multiplier: OrderedFloat(1.3) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

130. **ReversibleEffectType::QuickStop dispatches to QuickStopConfig.reverse()**
     - Given: Entity with `EffectStack<QuickStopConfig>` containing `[("chip_a", QuickStopConfig { multiplier: OrderedFloat(2.0) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::QuickStop(QuickStopConfig { multiplier: OrderedFloat(2.0) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

131. **ReversibleEffectType::Vulnerable dispatches to VulnerableConfig.reverse()**
     - Given: Entity with `EffectStack<VulnerableConfig>` containing `[("chip_a", VulnerableConfig { multiplier: OrderedFloat(1.5) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::Vulnerable(VulnerableConfig { multiplier: OrderedFloat(1.5) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

132. **ReversibleEffectType::Piercing dispatches to PiercingConfig.reverse()**
     - Given: Entity with `EffectStack<PiercingConfig>` containing `[("chip_a", PiercingConfig { charges: 3 })]`.
     - When: Reverse dispatch for `ReversibleEffectType::Piercing(PiercingConfig { charges: 3 })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

133. **ReversibleEffectType::RampingDamage dispatches to RampingDamageConfig.reverse()**
     - Given: Entity with `EffectStack<RampingDamageConfig>` containing `[("chip_a", RampingDamageConfig { increment: OrderedFloat(0.5) })]`.
     - When: Reverse dispatch for `ReversibleEffectType::RampingDamage(RampingDamageConfig { increment: OrderedFloat(0.5) })` with source "chip_a"
     - Then: Stack is now empty.
     - Edge case: Correct config.reverse() is called.

---

## Types

### Existing types used (defined in Wave 2/3 type specs)
- `Tree` enum — `Fire`, `When`, `Once`, `During`, `Until`, `Sequence`, `On` variants
- `ScopedTree` enum — `Fire`, `Sequence`, `When`, `On` variants
- `Terminal` enum — `Fire`, `Route` variants
- `ScopedTerminal` enum — `Fire`, `Route` variants
- `Trigger` enum — all variants with PartialEq, Eq
- `TriggerContext` enum — `Bump`, `Impact`, `Death`, `BoltLost`, `None` variants
- `ParticipantTarget` enum — `Bump(BumpTarget)`, `Impact(ImpactTarget)`, `Death(DeathTarget)`, `BoltLost(BoltLostTarget)` variants
- `BumpTarget` — `Bolt`, `Breaker`
- `ImpactTarget` — `Impactor`, `Impactee`
- `DeathTarget` — `Victim`, `Killer`
- `BoltLostTarget` — `Bolt`, `Breaker`
- `EffectType` enum — all 30 variants
- `ReversibleEffectType` enum — all 16 variants
- `RouteType` — `Bound`, `Staged`
- `Condition` — `NodeActive`, `ShieldActive`, `ComboActive(u32)`
- `EntityKind` — `Cell`, `Bolt`, `Wall`, `Breaker`, `Any`
- `ComboStreak { count: u32 }` — resource tracking consecutive perfect bump streak, read by `is_combo_active`
- `BoundEffects(Vec<BoundEntry>)` — component
- `BoundEntry { source: String, tree: Tree, condition_active: Option<bool> }` — struct
- `StagedEffects(Vec<(String, Tree)>)` — component
- `EffectStack<T: PassiveEffect>` — generic component with `entries: Vec<(String, T)>`

### Config structs (passive subset — each implements PassiveEffect)
- `SpeedBoostConfig { multiplier: OrderedFloat<f32> }` — `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `SizeBoostConfig { multiplier: OrderedFloat<f32> }` — same derives
- `DamageBoostConfig { multiplier: OrderedFloat<f32> }` — same derives
- `BumpForceConfig { multiplier: OrderedFloat<f32> }` — same derives
- `QuickStopConfig { multiplier: OrderedFloat<f32> }` — same derives
- `VulnerableConfig { multiplier: OrderedFloat<f32> }` — same derives
- `PiercingConfig { charges: u32 }` — same derives (additive aggregation)
- `RampingDamageConfig { increment: OrderedFloat<f32> }` — same derives (additive aggregation)

### Traits
- `Fireable { fn fire(&self, entity: Entity, source: &str, world: &mut World); fn register(app: &mut App) {} }`
- `Reversible: Fireable { fn reverse(&self, entity: Entity, source: &str, world: &mut World); }`
- `PassiveEffect: Fireable + Reversible + Sized + Clone + PartialEq + Eq { fn aggregate(entries: &[(String, Self)]) -> f32; }`

### Command structs
- `StampEffectCommand { entity: Entity, source: String, tree: Tree }` — appends to BoundEffects
- `FireEffectCommand { entity: Entity, effect: EffectType, source: String }` — dispatches config.fire()
- `ReverseEffectCommand { entity: Entity, effect: ReversibleEffectType, source: String }` — dispatches config.reverse()
- `RouteEffectCommand { entity: Entity, source: String, tree: Tree, route_type: RouteType }` — appends to Bound/Staged
- `StageEffectCommand { entity: Entity, source: String, tree: Tree }` — appends to StagedEffects
- `RemoveEffectCommand { entity: Entity, route_type: RouteType, source: String, tree: Tree }` — removes matching entry

---

## Reference Files
- `docs/todos/detail/effect-refactor/walking-effects/walking-algorithm.md` — walking algorithm definition
- `docs/todos/detail/effect-refactor/walking-effects/when.md` — When node behavior
- `docs/todos/detail/effect-refactor/walking-effects/once.md` — Once node behavior
- `docs/todos/detail/effect-refactor/walking-effects/until.md` — Until node behavior
- `docs/todos/detail/effect-refactor/walking-effects/sequence.md` — Sequence node behavior
- `docs/todos/detail/effect-refactor/walking-effects/on.md` — On node behavior
- `docs/todos/detail/effect-refactor/walking-effects/route.md` — Route node behavior
- `docs/todos/detail/effect-refactor/walking-effects/fire.md` — Fire node behavior
- `docs/todos/detail/effect-refactor/walking-effects/during.md` — During node scoped tree behavior (apply/reverse)
- `docs/todos/detail/effect-refactor/walking-effects/arming-effects.md` — Arming mechanism
- `docs/todos/detail/effect-refactor/rust-types/scoped-tree.md` — ScopedTree enum definition
- `docs/todos/detail/effect-refactor/rust-types/scoped-terminal.md` — ScopedTerminal enum definition
- `docs/todos/detail/effect-refactor/command-extensions/` — all 6 command extension specs
- `docs/todos/detail/effect-refactor/creating-effects/effect-api/passive-effect.md` — passive effect pattern
- `docs/todos/detail/effect-refactor/rust-types/effect-stacking/effect-stack.md` — EffectStack type
- `docs/todos/detail/effect-refactor/rust-types/effect-stacking/passive-effect.md` — PassiveEffect trait
- `docs/todos/detail/effect-refactor/storing-effects/bound-effects.md` — BoundEffects type
- `docs/todos/detail/effect-refactor/storing-effects/staged-effects.md` — StagedEffects type
- `docs/todos/detail/effect-refactor/evaluating-conditions/` — condition evaluator specs
- `docs/todos/detail/effect-refactor/rust-types/enums/trigger.md` — Trigger enum
- `docs/todos/detail/effect-refactor/rust-types/trigger-context.md` — TriggerContext enum

---

## Scenario Coverage
- New invariants: none — these are internal functions, not gameplay systems. Invariants are for runtime behavior.
- New scenarios: none — walk_effects, EffectStack, and command extensions are unit-testable. They don't produce observable gameplay state changes until wired to bridge systems (Wave 5+).
- Self-test scenarios: none — no new invariants.
- Layout updates: none.

---

## Constraints
- Tests go in:
  - `src/effect_v3/stacking/effect_stack.rs` (EffectStack tests)
  - `src/effect_v3/dispatch/fire_dispatch.rs` (fire dispatch tests for all EffectType variants)
  - `src/effect_v3/dispatch/reverse_dispatch.rs` (reverse dispatch tests for all ReversibleEffectType variants)
  - `src/effect_v3/` walking algorithm tests (new file for walk_effects function tests)
  - `src/effect_v3/` scoped tree tests (new file for apply_scoped_tree / reverse_scoped_tree function tests)
  - `src/effect_v3/commands.rs` or `src/effect_v3/commands/` (command extension tests)
  - `src/effect_v3/effects/<effect_name>/` per-effect passive fire/reverse/aggregate tests
  - `src/effect_v3/conditions/` (condition evaluator tests)
- Do NOT test: System functions (bridges, tick systems, evaluate_conditions system). Those are Wave 5+.
- Do NOT test: Non-passive effect fire/reverse implementations (Shockwave, Explode, etc.). Those spawn entities and are out of scope.
- Do NOT test: RON deserialization of Tree, EffectType, or config structs. That is a separate concern.
- Do NOT test: During node evaluation as part of walk_effects. During is handled by the evaluate_conditions system (out of scope), and walk_effects skips During entries (behavior #17 covers the skip). DO test `apply_scoped_tree` and `reverse_scoped_tree` (the building blocks that evaluate_conditions calls) — see Section C.5.
- Do NOT test: Until node installation/reversal lifecycle as part of walk_effects. Until entries in StagedEffects are consumed like any other staged entry. The Until-specific apply/reverse behavior happens when the Until is first encountered during walking from BoundEffects — testing the full Until lifecycle requires timer infrastructure (Wave 5+). Test the trigger matching and staged consumption only. DO test `apply_scoped_tree` and `reverse_scoped_tree` which Until also calls — see Section C.5.
- Do NOT test: The `evaluate_conditions` system itself (Wave 5+). Section C.5 tests the helper functions it depends on, not the system loop.

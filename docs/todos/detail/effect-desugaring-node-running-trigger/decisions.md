# Resolved Design Decisions

All 22 decisions resolved during interrogation. Referenced from the main overview.

## 1. Dispatch mechanics
**HashMap-indexed storage.** BoundEffects and StagedEffects both use `HashMap<Trigger, Vec<(SourceId, ValidTree)>>`. When a trigger fires, look up the key, get matching trees, walk them. No separate index — the storage IS the index. For local triggers, fire on both participant entities if they have matching BoundEffects/StagedEffects entries.

## 2. TriggerContext
**Typed per-trigger structs.** Each trigger concept has its own context struct with named fields. `BumpContext { bolt, breaker, source }`, `ImpactContext { impactor, impactee, source }`, `DeathContext { victim, killer, source }`, `BoltLostContext { bolt, breaker, source }`. Wrapped in `enum TriggerContext { Bump(BumpContext), Impact(ImpactContext), Death(DeathContext), BoltLost(BoltLostContext), None }`.

## 3. During/Until reversal
**During is first-class, not desugared.** During stays as `During(condition, inner)` in BoundEffects. A condition-monitoring system watches for NodeState changes and fires/reverses During entries directly. No synthetic triggers (NodeActiveStarted/NodeActiveEnded don't exist). Condition cycling is handled by the monitor system.

**Until desugars to Once.** Until fires immediately, then inserts `Once(trigger, Reverse(effect))` into BoundEffects. One-shot reversal that self-removes after firing. Uses real triggers (Died, TimeExpires, etc.).

**Sequence in scoped context** produces paired reversals: `During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))` — on condition start both fire, on condition end both reverse via `Sequence([Reverse(SpeedBoost), Reverse(DamageBoost)])`.

## 4. Once self-removal
**Remove inline during dispatch.** Dispatch uses `retain()` on the Vec — Once entries return false (removed), When entries return true (kept). Practically may need collect-then-remove due to ownership/borrow constraints, but same-frame semantics.

## 5. EveryBolt desugaring
**Desugar at load time.** `Route(EveryBolt, tree)` expands to: (1) stamp tree onto all existing bolts via ActiveBolts query, (2) register tree in `SpawnedRegistry` resource for future bolts. SpawnedRegistry is a global `Resource<SpawnedRegistry>` holding `HashMap<EntityType, Vec<(SourceId, ValidTree)>>`.

## 6. Source tracking
**Chip definition name (String).** `type SourceId = String`. BoundEffects entries are `(SourceId, ValidTree)` pairs. Reverse index `HashMap<SourceId, Vec<Trigger>>` enables fast removal on chip unequip. SpawnedRegistry also tracks SourceId for cleanup.

## 7. Kill attribution — propagated through effect chains
**KilledBy propagates from TriggerContext.** `KilledBy { dealer: Option<Entity> }`. The dealer is the originating bolt entity, propagated through effect chains:
- Bolt hits cell -> `dealer: Some(bolt)`
- Bolt's shockwave kills cell -> `dealer: Some(bolt)` (shockwave inherits from spawning bolt)
- Bolt's chain lightning kills cell -> `dealer: Some(bolt)` (arc inherits from source)
- Powder keg: bolt B kills cell -> cell explodes -> `dealer: Some(bolt_B)` (from DeathContext.killer, not the transferring bolt) -> explosion kills cell C -> `dealer: Some(bolt_B)`
- Environmental/timer hazard -> `dealer: None` -> Killed doesn't fire, Died + DeathOccurred still fire

`DeathContext { victim: Entity, killer: Option<Entity> }`. When killer is None, `Killed(Cell)` is skipped (no entity to fire on). `Died` always fires on victim. `DeathOccurred(Cell)` always fires globally.

**Unified damage message:** All damage sources send `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` — one Bevy message queue per victim type T. One `apply_damage<T>` system per victim type processes them, decrements HP, and sets `KilledBy` only on the killing blow (HP crosses from positive to zero). `trait GameEntity: Component {}` impl'd on Bolt, Cell, Wall, Breaker. No S (killer) generic — killer is `Option<Entity>`, type determined at runtime.

**Corner cases:**
- **Multi-source same frame**: Message processing order determines the killing blow. Deterministic (system ordering + message queue order).
- **Dealer despawns mid-chain**: Before firing Killed on the dealer, verify entity still exists. If despawned, skip Killed silently.

## 8. Bridge systems for Spawned
**4 standard systems in PostFixedUpdate** (not Bevy Observers). One per entity type: `bridge_bolt_added`, `bridge_cell_added`, `bridge_wall_added`, `bridge_breaker_added`. Each queries `Added<Bolt/Cell/Wall/Breaker>`, reads SpawnedRegistry for matching entries, stamps/transfers trees onto the new entity's BoundEffects/StagedEffects.

## 9. Build phasing
**Bottom-up: types -> builder -> loader -> dispatch -> damage -> swap.** See [implementation-waves.md](implementation-waves.md) for detailed wave breakdown.

## 10. Nested When in HashMap storage
**Arm into StagedEffects.** BoundEffects keys by outer trigger (PerfectBumped). When it fires, the inner tree `When(Impacted(Cell), Fire(Shockwave))` moves to StagedEffects under the `Impacted(Cell)` key. When Impacted fires, Shockwave executes and the entry is consumed. Next PerfectBumped re-arms from BoundEffects again.

## 11. Multiple effects from one trigger — Sequence node
**Sequence for ordered execution.** New tree node `Sequence([Fire(A), Fire(B)])` executes children in order. Use when one effect must apply before another. Independent effects stay as separate Route entries — Sequence is only for when order matters.

## 12. Transfer/Stamp tree ownership
**Detach on transfer.** Once transferred/stamped onto another entity, the tree has no link back to the source. Unequipping the chip removes the bolt's BoundEffects entries (stops future transfers) but doesn't touch entities that already received trees.

## 13. Chip loading -> Route processing
**Equip command processes Routes.** Same timing as today. The chip equip command reads each ValidDef, matches on RouteTarget, and stamps the tree into the target entity's BoundEffects with the chip's SourceId. EveryBolt desugars here: stamp existing + register in SpawnedRegistry.

## 14. During is first-class, not desugared
During stays as a first-class node in BoundEffects. A condition-monitoring system watches for NodeState changes and fires/reverses During entries directly. No synthetic triggers.

## 15. RON participant syntax — fully qualified
RON uses shared enum names: `On(BumpTarget::Bolt, ...)`, `On(ImpactTarget::Impactee, ...)`. RawParticipant wraps the shared enums. No flat names, no ambiguity.

## 16. Sequence in scoped context — paired reversals
`During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))` — on condition start both fire, on condition end the condition monitor reverses both.

## 17. BoundEffects storage for During
During entries keyed by condition (not trigger). BoundEffects has two maps: `conditions: HashMap<Condition, Vec<...>>` and `triggers: HashMap<Trigger, Vec<...>>`.

## 18. Dispatch ordering — StagedEffects first
StagedEffects walked BEFORE BoundEffects on each trigger dispatch. Prevents arm-and-consume in same call.

## 19. During + nested When lifecycle
During's inner When registered into `BoundEffects.triggers` on condition start with scope source (`"ChipName:During(NodeActive)"`). On condition end, scope source removes registration + armed StagedEffects entries.

## 20. Recursion depth limit
Depth counter on TriggerContext, MAX_DISPATCH_DEPTH = 10.

## 21. Trigger locality — bridge systems decide
No centralized dispatch. Each trigger has its own Bevy bridge system. All call shared `walk_effects` helper.

## 22. Stale participant references
Debug warning + skip when targeting a despawned entity.

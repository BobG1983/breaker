---
name: Bolt builder migration performance review
description: Exclusive spawn_bolt system, spawn_inner multiple inserts, BoltConfig clone, nested bundle tuples, test helper spawn cost, definition() allocs — all reviewed on feature/chip-evolution-ecosystem
type: project
---

## Review scope (feature/chip-evolution-ecosystem)

Files reviewed (second pass — builder now at `bolt/builder/core.rs`):
- `breaker-game/src/bolt/builder/core.rs` — typestate builder, spawn_inner, .definition() method
- `breaker-game/src/bolt/registry.rs` — BoltRegistry resource
- `breaker-game/src/bolt/definition.rs` — BoltDefinition RON asset
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — exclusive system
- `breaker-game/src/bolt/systems/launch_bolt.rs`
- `breaker-game/src/bolt/systems/reset_bolt/system.rs`
- `breaker-game/src/effect/effects/spawn_bolts/effect.rs`
- `rantzsoft_spatial2d/src/builder.rs`
- `breaker-game/src/bolt/resources.rs`

## Key findings

### spawn_bolt exclusive system (Critical)
`spawn_bolt` is registered as `fn(world: &mut World)` — an exclusive system in Bevy 0.18.
Exclusive systems cannot run in parallel with ANY other system in the same schedule; the entire
FixedUpdate (or OnEnter in this case) graph stalls until spawn_bolt completes.

**Actual impact**: spawn_bolt runs only on `OnEnter(GameState::Playing)` — once per run, not
per frame. This makes the scheduling cost zero at steady state. The concern is academic given
the trigger.

**Why it is still noteworthy**: spawn_bolt's exclusive access is used to clone the `BoltConfig`
resource, run a manual query, then call `Bolt::builder().spawn(world)`. Because it uses
`world.resource::<BoltConfig>().clone()` — a full struct clone (~200 bytes on the stack) — this
is technically an allocation path inside an exclusive system. But since it runs once, it is fine.

**Verdict**: Minor/Clean. The exclusive system pattern is justified because `World` direct access
is needed for `Builder::spawn(world)`. The single-fire trigger makes scheduling cost zero.

### spawn_inner multiple entity.insert() calls (the real archetype question)
`spawn_inner` does up to 7 sequential `entity.insert()` calls after the initial `world.spawn(core)`:
1. Role marker + cleanup (PrimaryBolt+CleanupOnRunEnd OR ExtraBolt+CleanupOnNodeExit) — always
2. BoltServing (if serving) — conditional
3. BoltConfigParams group (4 components) — conditional on config() being called
4. SpawnedByEvolution — conditional
5. BoltLifespan — conditional
6. BoundEffects — conditional

**How Bevy handles sequential inserts**: Each `EntityWorldMut::insert()` call on a live entity
is a component-move in Bevy 0.18. Each insert that adds new components causes the entity to
move to a new archetype (O(components) copy). So 7 sequential inserts = up to 7 archetype moves
per spawn.

**Actual impact**: spawn_bolt is one-shot (OnEnter). spawn_bolts effect fires episodically (rare).
At 1-4 bolts active simultaneously, the total archetype move cost is negligible — it happens
once per bolt per event, not per frame. This is NOT a per-frame cost.

**If spawn_bolts effect fired every frame at scale**: would be an issue. It doesn't.

**Verdict**: Moderate (for spawn_bolts fire loop). For primary bolt: Clean at current scale.
The correct fix if this ever matters: collect all optional components into a single `insert()` call
instead of sequential conditional inserts. Not worth doing now.

### BoltConfig clone in spawn_bolt and spawn_bolts effect
- `spawn_bolt/system.rs:42`: `world.resource::<BoltConfig>().clone()` — one-shot on OnEnter
- `spawn_bolts/effect.rs:31`: `world.resource::<BoltConfig>().clone()` — once per fire() call

`BoltConfig` has 11 f32/array fields (~60 bytes). The clone is stack-only, no heap allocation.
Both call sites clone once before the spawn loop and pass `&config` to the builder — not inside
the per-bolt loop. This is the correct pattern.

**Verdict**: Clean. Not an allocation issue. Noted as intentional and efficient.

### impl Bundle nested tuple return
`build_core` returns `(base_components, spatial_components, radius_components)` — nested tuples
of tuples. `build()` impls then wrap this in another tuple with role/motion markers.

In Bevy 0.18, `Bundle` is implemented for tuples up to 15 elements. Nested tuples are flattened
at compile time — no runtime overhead for nesting. The component set is assembled into a single
archetype insert with the correct component set determined at compile time via associated types.

`Spatial::builder().build()` returns flat component tuples (no heap). All bundle types here are
Copy or small structs. Zero allocation.

**Verdict**: Clean. Nested tuple bundles are idiomatic and zero-cost.

### Test helper spawn using builder
Tests call `Bolt::builder()...spawn(app.world_mut())` or `world.spawn(...)` with the full
component set. These are unit tests — isolated worlds, no scheduling, no archetype pressure.
The multi-insert pattern in spawn_inner is a one-time cost per test. With hundreds of test cases,
the cumulative archetype registration overhead is in the test runner's startup path, not per-test.

**Verdict**: Clean. Test performance is not a concern at this scale.

### spawn_bolts fire loop: per-iteration entity.insert(BoundEffects)
In `spawn_bolts/effect.rs`, after the builder spawns the bolt entity, there are two additional
conditional inserts inside the loop (lifespan and BoundEffects). These happen per spawned bolt,
but spawn_bolts is episodic (fire() called on specific triggers) and spawns at most a few bolts
per call. The BoundEffects insert requires a `.clone()` of the effects vec.

**Verdict**: Minor. Episodic, bounded count. Not a per-frame cost.

### .definition() method: String allocation at spawn (Minor)
`bolt/builder/core.rs:255` — `.definition()` stores `def.name.clone()` into `BoltDefinitionParams`,
moved into `BoltDefinitionRef(String)` as a component. Heap allocation at spawn time.
No production call site uses `.definition()` today — only tests. When the first production caller
arrives (chip evolution, bolt type selection), this fires once per bolt spawn on that path.
Not per-frame. Acceptable.

### BoltDefinitionRef(String) as component — archetype variant
Bolts created via `.definition()` get a different component set from `.config()` bolts:
- config-path: BoltSpawnOffsetY + BoltRespawnOffsetY + BoltRespawnAngleSpread + BoltInitialAngle
- definition-path: BoltBaseDamage + BoltDefinitionRef + BoltAngleSpread + BoltSpawnOffsetY
These are distinct archetypes. With 1-4 bolts active, this is not a concern.

### .spawned_by() String allocation (Minor)
`bolt/builder/core.rs:284` — `name.to_string()` inside `.spawned_by()`.
No production call site uses `.spawned_by()` today. When attribution tracking is added to
chain_bolt or spawn_bolts effects, this fires once per spawned bolt. One allocation, episodic.

### spawn_inner Vec allocation for effect_entries (Minor)
`bolt/builder/core.rs:414` — `Vec::new()` only when `has_explicit || has_inherited` is true.
One allocation per bolt spawn with effects. Episodic. Not per-frame.

### BoltRegistry: confirmed clean
`HashMap<String, BoltDefinition>` seeded once at asset load via SeedableRegistry.
Never iterated or queried per frame. BoltDefinition.effects Vec cloned during seed() only.
BoltDefinitionRef is inserted on definition-path bolts but not queried by any production system.

## Summary of archetypes introduced
- PrimaryBolt + config params: 1 archetype (stable, one entity)
- PrimaryBolt + BoltServing: separate archetype on node 0 only (stable after launch)
- ExtraBolt + lifespan: one archetype variant for ephemeral bolts
- ExtraBolt + BoundEffects: one more variant if inherited
- ExtraBolt via definition-path: distinct archetype (definition params vs config params)
Total new archetypes: 4-5 variants at most. Well within the acceptable range.

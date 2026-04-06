# Implementation Waves

Clean room implementation in `src/new_effect/`. The old `src/effect/` (277 files, ~44k lines) is untouched until Phase 6 (swap). Each wave builds on the previous. Within a wave, items can be built in parallel.

## Wave 1: Core Types (no dependencies)

All enums, structs, and trait definitions. No systems, no Bevy plugins. Pure Rust types + derives. Fully testable in isolation.

**All items in this wave can be built in parallel.**

### 1a. Trigger + Condition enums
New file: `new_effect/triggers/mod.rs`

```rust
// Old has 18 variants with different naming. New has Occurred suffix for globals,
// local/global distinction, ImpactTarget/KillTarget/DeathTarget params.
enum Trigger { PerfectBumped, EarlyBumped, ..., PerfectBumpOccurred, ..., TimeExpires(f32) }
enum Condition { NodeActive, ShieldActive, ComboActive(u32) }
enum EntityType { Bolt, Cell, Wall, Breaker }
```

Reference: [api-reference.md](api-reference.md) Local Triggers + Global Triggers tables

### 1b. Participant enums
New files: `new_effect/triggers/bump.rs`, `impact.rs`, `death.rs`, `bolt_lost.rs`

```rust
enum BumpTarget { Bolt, Breaker }
enum ImpactTarget { Impactor, Impactee }
enum DeathTarget { Victim, Killer }
enum BoltLostTarget { Bolt, Breaker }
```

Reference: [api-reference.md](api-reference.md) Participant Enums section

### 1c. RouteTarget + ParticipantTarget
New file: `new_effect/tree/targets.rs`

```rust
enum RouteTarget { Bolt, Breaker, Cell, Wall, ActiveBolts, EveryBolt, PrimaryBolts, ExtraBolts, ... }
enum ParticipantTarget { Bump(BumpTarget), Impact(ImpactTarget), Death(DeathTarget), BoltLost(BoltLostTarget) }
```

Reference: [builder-design.md](builder-design.md) RouteTarget + ParticipantTarget sections

### 1d. EffectType + ReversibleEffectType enums
New file: `new_effect/tree/effect_types.rs`

```rust
enum EffectType { SpeedBoost(f32), SizeBoost(f32), ..., LoseLife, Explode(ExplodeConfig), ... }
enum ReversibleEffectType { SpeedBoost(f32), SizeBoost(f32), ..., Shield(ShieldConfig) }
```

Reference: [builder-design.md](builder-design.md) Effect types section

### 1e. ValidTree + ValidDef + ValidTerminal
New file: `new_effect/tree/valid.rs`

```rust
struct ValidDef { route_target: RouteTarget, tree: ValidTree }
enum ValidTree { Fire(EffectType), Sequence(Vec<ValidTree>), When(...), Once(...), During(...), Until(...), Spawned(...), On(...) }
enum ValidScopedTree { ... }
enum ValidTerminal { Fire, Stamp, Transfer, Reverse }
enum ValidScopedTerminal { ... }
```

Reference: [builder-design.md](builder-design.md) Validated tree structure section

### 1f. RawTree + RawDef (RON serde types)
New file: `new_effect/tree/raw.rs`

```rust
struct RawDef { route: RawRouteTarget, tree: RawTree }
enum RawTree { Fire, Sequence, When, Once, During, Until, Spawned, On }
enum RawTerminal { Fire, Stamp, Transfer }
enum RawParticipant { BumpTarget(BumpTarget), ImpactTarget(ImpactTarget), ... }
```

Reference: [builder-design.md](builder-design.md) Raw Types section

### 1g. TriggerContext
New file: `new_effect/triggers/context.rs`

```rust
// New: typed per-concept structs wrapped in an enum
struct BumpContext { bolt: Entity, breaker: Entity, source: SourceId, depth: u32 }
struct ImpactContext { impactor: Entity, impactee: Entity, source: SourceId, depth: u32 }
struct DeathContext { victim: Entity, killer: Option<Entity>, source: SourceId, depth: u32 }
struct BoltLostContext { bolt: Entity, breaker: Entity, source: SourceId, depth: u32 }
enum TriggerContext { Bump(BumpContext), Impact(ImpactContext), Death(DeathContext), BoltLost(BoltLostContext), None { depth: u32 } }
```

Reference: [decisions.md](decisions.md) #2

### 1h. SourceId type + BoundEffects/StagedEffects/SpawnedRegistry structs
New file: `new_effect/storage/mod.rs`

```rust
// New: HashMap-indexed with conditions map and source reverse index
type SourceId = String;
struct BoundEntry { source: SourceId, tree: ValidTree }
enum BoundKey { Trigger(Trigger), Condition(Condition) }
struct BoundEffects {
    triggers: HashMap<Trigger, Vec<BoundEntry>>,
    conditions: HashMap<Condition, Vec<BoundEntry>>,
    sources: HashMap<SourceId, Vec<BoundKey>>,
}
struct StagedEntry { source: SourceId, tree: ValidTree }
struct StagedEffects { entries: HashMap<Trigger, Vec<StagedEntry>> }
struct SpawnedEntry { source: SourceId, tree: ValidTree }
struct SpawnedRegistry { entries: HashMap<EntityType, Vec<SpawnedEntry>> }
```

Reference: [storage-and-dispatch.md](storage-and-dispatch.md) BoundEffects/StagedEffects/SpawnedRegistry sections

### 1i. DamageMessage + KilledBy
New file: `new_effect/damage/mod.rs`

```rust
struct DamageMessage { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }

#[derive(Component, Default)]
struct KilledBy { dealer: Option<Entity> }
```

Reference: [death-pipeline.md](death-pipeline.md) Unified Damage Message section

### 1j. Death pipeline messages
New file: `new_effect/damage/messages.rs`

```rust
struct KillYourself<S: Component, T: Component> { victim: Entity, killer: Option<Entity> }
struct Destroyed<S: Component, T: Component> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }
```

Reference: [death-pipeline.md](death-pipeline.md) Messages section

### 1k. Effect + Reversible traits
New file: `new_effect/tree/traits.rs`

```rust
trait Effect { fn fire(&self, entity: Entity, source_chip: &str, context: &TriggerContext, world: &mut World); }
trait Reversible: Effect { fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World); }
```

Reference: [builder-design.md](builder-design.md) Traits section

### 1l. DespawnEntity message
New file: `shared/messages.rs` (or add to existing shared messages)

```rust
struct DespawnEntity { pub entity: Entity }
```

Cross-domain message — lives in `shared::messages` since any domain can request a despawn. Replaces both `PendingDespawn` component and all direct `.despawn()`/`.try_despawn()` calls.

---

## Wave 2: Builder (depends on Wave 1 types)

Typestate builder. Can be tested by constructing trees and asserting on the resulting ValidDef/ValidTree structure. No Bevy systems.

**All items in this wave can be built in parallel** (they compose Wave 1 types in different ways).

### 2a. EffectDef::route() + RouteContext
New file: `new_effect/builder/route.rs`

The definition-level entry point. Returns RouteContext which chains into inner tree building.

Reference: [builder-design.md](builder-design.md) RouteContext section, [examples.md](examples.md) all examples

### 2b. EffectTree entry points
New file: `new_effect/builder/tree.rs`

Inner tree builder for Transfer/Stamp payloads. `EffectTree::when()`, `EffectTree::fire()`, etc.

### 2c. TriggerChain + fire/sequence/on
New file: `new_effect/builder/chain.rs`

The `.when().when().fire()` chaining with AnyFire/ReversibleOnly constraint.

### 2d. DuringContext + UntilContext
New file: `new_effect/builder/scoped.rs`

Scoped builders with reversibility enforcement + nested When relaxation.

### 2e. TargetContext + SpawnedContext
New file: `new_effect/builder/targets.rs`

Terminal builders: `.fire()`, `.stamp()`, `.transfer()`, `.sequence()`.

---

## Wave 3: Loader (depends on Wave 1 types + Wave 2 builder)

RON deserialization -> Raw -> builder -> ValidDef. Round-trip tests.

**3a and 3b can be built in parallel.**

### 3a. RON -> Raw -> ValidDef loader
New file: `new_effect/loader/mod.rs`

```rust
fn load_def(raw: &RawDef) -> Result<ValidDef, EffectError>
fn load_tree(raw: &RawTree, trigger_ctx: Option<&Trigger>) -> Result<ValidTree, EffectError>
```

Test with all 55 migrated RON files: parse -> load -> verify structure.

Reference: [builder-design.md](builder-design.md) RON -> Valid (loader) section

### 3b. ValidDef -> Raw -> RON round-trip
New file: `new_effect/loader/round_trip.rs`

Serialize ValidDef back to RawDef back to RON. Test: load -> save -> load -> compare.

Reference: [builder-design.md](builder-design.md) Valid -> Raw section

---

## Wave 4: Dispatch + Storage (depends on Waves 1-3)

Bevy systems. This is where BoundEffects/StagedEffects get wired into the ECS.

**4a and 4b must be serial** (4b depends on 4a). **4c-4e can be parallel after 4a.**

### 4a. walk_effects helper (MUST BE FIRST)
New file: `new_effect/dispatch/walk.rs`

The shared tree-walking function all bridge systems call. StagedEffects first, then BoundEffects. Handles When (arm), Once (arm + remove), Fire (execute), Sequence (ordered), On (redirect + Stamp/Transfer).

```rust
fn walk_effects(trigger, context, entity, bound, staged, world)
```

Reference: [storage-and-dispatch.md](storage-and-dispatch.md) Shared Tree Walker section

### 4b. Condition monitor (depends on 4a)
New file: `new_effect/dispatch/conditions.rs`

Watches NodeState changes, fires/reverses During entries. Manages scope source registration for nested When.

Reference: [storage-and-dispatch.md](storage-and-dispatch.md) Condition monitor + During lifecycle sections

### 4c. Spawned bridge systems (parallel with 4b)
New file: `new_effect/dispatch/spawned.rs`

4 systems (bolt/cell/wall/breaker) in PostFixedUpdate. Query `Added<T>`, read SpawnedRegistry, stamp trees.

Reference: [storage-and-dispatch.md](storage-and-dispatch.md) Bridge systems section

### 4d. Route processing / equip command (parallel with 4b)
New file: `new_effect/dispatch/route.rs`

Processes ValidDef list at chip equip time. Resolves RouteTarget, stamps into BoundEffects, desugars EveryBolt.

```rust
fn process_routes(defs: &[ValidDef], entity_map: &RouteEntityMap, ...)
```

Reference: [decisions.md](decisions.md) #13

### 4e. Command extensions (parallel with 4b)
New file: `new_effect/commands.rs`

```rust
trait EffectCommandsExt {
    fn fire_effect(entity, effect, context);
    fn reverse_effect(entity, effect);
    fn stamp_effect(entity, source, tree);
    fn transfer_effect(entity, source, tree);
    fn remove_source(entity, source);
}
```

Reference: [storage-and-dispatch.md](storage-and-dispatch.md) Command Extensions section

---

## Wave 5: Damage + Death Pipeline (depends on Wave 4)

New unified damage/death systems. These run alongside the old systems during development (both compile, neither references the other). At swap time (Phase 6), the old systems are deleted and the new ones take over.

**5a must be first. 5b-5d can be parallel after 5a.**

### 5a. apply_damage system (MUST BE FIRST)
New file: `new_effect/damage/systems.rs`

```rust
fn apply_damage(mut messages: MessageReader<DamageMessage>, mut query: Query<(&mut CellHealth, &mut KilledBy)>)
```

Sets KilledBy only on killing blow.

Reference: [death-pipeline.md](death-pipeline.md) apply_damage section

### 5b. detect_deaths system (depends on 5a)
New file: `new_effect/damage/systems.rs`

```rust
fn detect_deaths(query: Query<(Entity, &KilledBy, &CellHealth), Changed<CellHealth>>)
// Sends KillYourself<S, T>
```

### 5c. bridge_destroyed system (parallel with 5b after 5a)
New file: `new_effect/dispatch/death_bridge.rs`

```rust
fn bridge_destroyed<S, T>(mut reader: MessageReader<Destroyed<S, T>>, ...)
// Fires: Died on victim, Killed on killer (if alive), DeathOccurred globally
```

Reference: [death-pipeline.md](death-pipeline.md) Death Chain section

### 5d. Centralized despawn system (parallel with 5b)
New file: `new_effect/damage/despawn.rs`

```rust
fn process_despawn_requests(mut reader: MessageReader<DespawnEntity>, mut commands: Commands) {
    for msg in reader.read() {
        commands.entity(msg.entity).try_despawn();
    }
}
```

Replaces the old `PendingDespawn` component approach. All entity despawns go through the `DespawnEntity` message — the death pipeline writes it after trigger evaluation completes, and domain systems write it instead of calling `.despawn()` directly. Runs in **PostFixedUpdate** so FixedUpdate systems that write the message don't lose messages at the schedule boundary, and all FixedUpdate systems have a chance to read the entity before it's gone. Uses `try_despawn` internally to handle already-cleaned-up entities gracefully (e.g., `CleanupOnExit` races).

Phase 6 sweep: convert all `.despawn()`/`.try_despawn()` calls across effect fire/reverse functions, bolt_lost, chain_lightning arc cleanup, tether beam, shockwave, explode, and any entity lifecycle system to write `DespawnEntity` instead.

---

## Wave 6: Effect Implementations (depends on Waves 1-4)

Port each effect's `fire()` and `reverse()` to use the new types and `DamageMessage`. These are mostly mechanical — same logic, new message types.

**All effect implementations can be built in parallel.** They don't depend on each other.

### Passive effects (trivial — insert/remove a component)
- `speed_boost.rs` — insert/remove speed multiplier
- `size_boost.rs` — insert/remove size multiplier
- `damage_boost.rs` — insert/remove `ActiveDamageBoosts` entry
- `bump_force.rs` — insert/remove bump force modifier
- `quick_stop.rs` — insert/remove quick stop config
- `flash_step.rs` — insert/remove flash step marker
- `piercing.rs` — insert/remove piercing count
- `vulnerable.rs` — insert/remove `ActiveVulnerability` entry
- `anchor.rs` — insert/remove anchor config
- `ramping_damage.rs` — insert/remove ramp state

Each effect implements `Effect` (and optionally `Reversible`) per [builder-design.md](builder-design.md) traits section.

### Damage-dealing effects (send DamageMessage, propagate dealer from TriggerContext)
- `shockwave/` — expanding circle area damage
- `explode/` — instant area burst
- `piercing_beam/` — beam rectangle
- `pulse/` — periodic ring
- `tether_beam/` — line segment between paired bolts
- `chain_lightning/` — sequential arc hops

### Spawn effects
- `spawn_bolts/` — spawn new bolts via `Bolt::builder()`
- `spawn_phantom/` — spawn phantom bolt
- `chain_bolt/` — spawn tethered bolt pair
- `mirror_protocol/` — spawn mirror bolt

### Complex effects
- `circuit_breaker/` — counter + reward
- `entropy_engine/` — random effect pool
- `random_effect/` — weighted random selection
- `second_wind/` — wall spawn on bolt loss
- `shield/` — timed shield wall
- `gravity_well/` — pull bolts toward point

### Death/utility effects
- `LoseLife` — decrement life pool
- `TimePenalty` — add time penalty
- `Die` — send KillYourself

---

## Wave 7: Trigger Bridge Systems (depends on Waves 4 + 6)

Each bridge system reads existing game messages (these don't change — they're in bolt/cells domains) and calls `walk_effects` on the right entities.

**All bridges can be built in parallel.**

### Bump bridges
New folder: `new_effect/dispatch/bridges/bump/`

```rust
// New: bridges per trigger, calling walk_effects on participant entities
// Read: existing BumpGraded message from bolt domain
```

Reference files to read (existing game messages that don't change):
- `bolt/messages.rs` — `BumpGraded`, `BoltLost`
- `bolt/systems/bolt_cell_collision/system.rs` — `BoltImpactCell`
- `bolt/messages.rs` — `BoltImpactWall`
- `cells/messages.rs` — `CellImpactWall`

### Impact bridges
New folder: `new_effect/dispatch/bridges/impact/`

```rust
// ~800 lines of bridge code. New: simpler bridges calling walk_effects.
```

### Death bridges
New file: `new_effect/dispatch/bridges/death.rs`

```rust
// New: single generic bridge_destroyed<S, T> (see Wave 5c)
```

### Other bridges
- `bridge_node_start` / `bridge_node_end` — NodeStartOccurred, NodeEndOccurred
- `bridge_node_timer` — NodeTimerThresholdOccurred
- `bridge_bolt_lost` — BoltLostOccurred

---

## Wave 8: Plugin + Wiring (depends on all above)

Wire everything into a NewEffectPlugin. Register systems, messages, set ordering.

**Serial — single task.**

New file: `new_effect/plugin.rs`

```rust
pub struct NewEffectPlugin;
impl Plugin for NewEffectPlugin { ... }
```

Register: all bridge systems, walk_effects dependencies, condition monitor, apply_damage, detect_deaths, despawn_pending, spawned bridges.

---

## Phase 6: Swap (depends on all waves complete + tested)

See [phase-6-swap-spec.md](phase-6-swap-spec.md) for the complete step-by-step spec.

## Phase 7: Documentation Update

After swap is verified, update architecture and design docs to reflect the new effect system.

**`docs/architecture/`:**
- How to add a new trigger (create participant enum variant, add bridge system, register in plugin)
- How to add a new effect (implement `Effect` trait, optionally `Reversible`, add to `EffectType` enum)
- How to add a new condition (add `Condition` variant, add monitor system)
- Updated message flow diagrams (DamageMessage -> apply_damage -> KilledBy -> KillYourself -> Destroyed -> bridge_destroyed)
- BoundEffects/StagedEffects storage shape and dispatch ordering
- Route/Stamp/Transfer semantics

**`docs/design/`:**
- Effect system vocabulary reference (Route, When, During, Until, Spawned, On, Fire, Stamp, Transfer, Sequence)
- Participant enum reference (BumpTarget, ImpactTarget, DeathTarget, BoltLostTarget)
- Condition reference (NodeActive, ShieldActive, ComboActive)
- Kill attribution chain documentation

1. Delete `src/effect/` (277 files, ~44k lines)
2. Rename `src/new_effect/` -> `src/effect/`
3. Copy migrated RON files to asset directories (55 files)
4. Update plugin registration in lib.rs
5. Update domain systems to use new message types (see swap spec for exact file list)
6. Sweep all `.despawn()`/`.try_despawn()` calls across domains to write `DespawnEntity` message instead (bolt_lost, chain_lightning, tether beam, shockwave, explode, entity lifecycle systems)
7. Verify: `cargo dcheck`, `cargo dclippy`, `cargo dtest`, `cargo scenario -- --all`

---

## Parallelism Summary

```
Wave 1 (all parallel):  1a | 1b | 1c | 1d | 1e | 1f | 1g | 1h | 1i | 1j | 1k | 1l
                              ↓
Wave 2 (all parallel):  2a | 2b | 2c | 2d | 2e
                              ↓
Wave 3 (parallel):      3a | 3b
                              ↓
Wave 4:                 4a (serial) → 4b | 4c | 4d | 4e (parallel after 4a)
                              ↓
Wave 5:                 5a (serial) → 5b | 5c | 5d (parallel after 5a)
                              ↓ (5 + 6 can overlap — effects don't depend on damage systems)
Wave 6 (all parallel):  all effect impls
                              ↓
Wave 7 (all parallel):  all bridge impls
                              ↓
Wave 8 (serial):        plugin wiring
                              ↓
Phase 6 (serial):       swap
```

Maximum parallelism: Waves 1, 2, 6, and 7 are fully parallel (12, 5, ~20, and ~10 tasks respectively). Waves 5 and 6 can overlap since effect implementations don't depend on the damage system (they just need to know the DamageMessage type from Wave 1).

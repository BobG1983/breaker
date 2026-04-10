## Implementation Spec: Bolt — Death Pipeline Migration

### Domain
src/bolt/

### Failing Tests
- `src/bolt/systems/tick_bolt_lifespan/tests.rs` -- tests for tick_bolt_lifespan sending KillYourself<Bolt> instead of RequestBoltDestroyed
- `src/bolt/systems/bolt_lost/tests.rs` -- tests for bolt_lost sending KillYourself<Bolt> (ALL bolt types) and updated BoltLost { bolt, breaker }, respawn distinction for baseline vs ExtraBolt
- `src/bolt/systems/handle_bolt_kill/tests.rs` -- tests for new bolt domain kill handler
- Bolt builder tests -- tests for Hp/KilledBy on spawned bolts

Exact test count will be determined by the test spec. Expect approximately 20-25 tests across all files.

---

### Prerequisites

The following shared types and systems must exist from previous waves (wave 2 scaffold + wave 7 death pipeline) before wave 10 implementation can proceed:

- `GameEntity` trait with `impl GameEntity for Bolt {}`
- `Hp`, `KilledBy`, `Dead` components (src/shared/components/)
- `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity` messages (src/shared/messages/)
- `DeathPipelineSystems` system set (src/shared/sets.rs)
- `apply_damage::<Bolt>` and `detect_bolt_deaths` systems (wave 7)
- `process_despawn_requests` system (src/shared/systems/)
- `GlobalPosition2D` from `rantzsoft_spatial2d`

---

### What to Implement

#### 1. Update `tick_bolt_lifespan` system
Modify the existing system to send `KillYourself<Bolt>` instead of `RequestBoltDestroyed` when a bolt's lifespan timer expires.

**Before:**
```rust
writer.write(RequestBoltDestroyed { bolt: entity })
```

**After:**
```rust
writer.write(KillYourself::<Bolt>::new(entity, None))
```

The system continues to:
- Tick `BoltLifespan` timers each frame using `time.delta()`
- Skip bolts with `Birthing` component (via `Without<Birthing>` filter -- timer does NOT tick at all for birthing bolts)
- Fire only on `just_finished()`
- Skip bolts with `Dead` component (add `Without<Dead>` filter)

The system does NOT:
- Insert `Dead` -- the domain kill handler does that
- Despawn the bolt -- `process_despawn_requests` does that

**Message writer change:** Replace `MessageWriter<RequestBoltDestroyed>` with `MessageWriter<KillYourself<Bolt>>` in the system parameters.

#### 2. Update `bolt_lost` system
Modify the existing system to send `KillYourself<Bolt>` instead of `RequestBoltDestroyed` for ALL lost bolts (no ExtraBolt distinction in the kill path). Update the `BoltLost` message to include `bolt` and `breaker` fields.

**All bolts treated the same through death pipeline**: `bolt_lost` sends `KillYourself<Bolt>` for ALL lost bolts unconditionally -- both baseline and ExtraBolt. ALL bolts die the same way. The difference is in respawning: baseline bolts (no `ExtraBolt` component) ALSO get respawned (a new bolt is spawned). Extra bolts just die (no respawn).

**Before:**
```rust
writer.write(RequestBoltDestroyed { bolt: entity });
bolt_lost_writer.write(BoltLost);
```

**After:**
```rust
// KillYourself fires for ALL bolt types -- no ExtraBolt guard
kill_writer.write(KillYourself::<Bolt>::new(entity, None));
// BoltLost fires for ALL bolt types
bolt_lost_writer.write(BoltLost { bolt: entity, breaker: breaker_entity });
// Respawn only for baseline bolts (no ExtraBolt component)
if !has_extra_bolt {
    // existing respawn logic (unchanged)
}
```

The system continues to:
- Detect bolts that leave the play area (below breaker or off-screen)
- Send `KillYourself<Bolt>` for ALL bolt types (enters death pipeline)
- Send `BoltLost` for the effect system's `on_bolt_lost_occurred` bridge (ALL bolt types)
- Respawn baseline bolts only (no `ExtraBolt` component) -- existing respawn logic is unchanged

The system must query the breaker entity to populate `BoltLost.breaker`. The breaker entity is already available in the system's existing queries or can be obtained via a `Query<Entity, With<Breaker>>` (there is exactly one breaker entity during gameplay).

The ExtraBolt distinction is ONLY for respawning -- `Has<ExtraBolt>` (or equivalent query) is checked to decide whether to respawn, NOT whether to send `KillYourself`.

Add `Without<Dead>` filter to the bolt query to skip bolts that are already dying.

Killer is `None` -- bolt loss is an environmental death with no dealer.

The system does NOT:
- Insert `Dead`
- Despawn the bolt

**SystemParam migration -- BoltLostWriters**: The existing system may use a `BoltLostWriters` SystemParam struct that bundles multiple MessageWriters. This struct must be updated:
- Replace `MessageWriter<RequestBoltDestroyed>` with `MessageWriter<KillYourself<Bolt>>`
- **Remove the old `BoltLostWriters` SystemParam struct entirely** if it existed only to bundle `RequestBoltDestroyed` + `BoltLost` writers. Replace with inline system parameters or a new SystemParam that reflects the new message types. Removing the struct is preferred for clarity.
- Note: `KillYourself<Bolt>` is sent for ALL bolt types (no ExtraBolt guard). The ExtraBolt check is only used to decide whether to respawn.

#### 3. Update `BoltLost` message type
Change from a unit struct to a struct with fields.

**Before:**
```rust
#[derive(Message)]
pub struct BoltLost;
```

**After:**
```rust
#[derive(Message, Clone, Debug)]
pub struct BoltLost {
    pub bolt: Entity,
    pub breaker: Entity,
}
```

This lives in `src/bolt/messages.rs`. All consumers of `BoltLost` must be updated to destructure the new fields. The primary consumer is `on_bolt_lost_occurred` in `src/effect/triggers/bolt_lost/bridges.rs` -- that system reads `msg.bolt` and `msg.breaker` to build its `TriggerContext::BoltLost { bolt, breaker }`.

Also update `spawn_bolt_lost_text` (if it consumes `BoltLost`) to accept the new struct shape.

**Test update**: If there is an existing test file for the `BoltLost` message (e.g., in `src/bolt/messages.rs` tests), update the test to construct `BoltLost { bolt: ..., breaker: ... }` instead of `BoltLost`.

#### 4. Remove `RequestBoltDestroyed` message type
Delete the `RequestBoltDestroyed` message from `src/bolt/messages.rs`. Remove its registration from `src/bolt/plugin.rs`. Remove all imports referencing it.

There are exactly two producers:
- `tick_bolt_lifespan` -- migrated in step 1
- `bolt_lost` -- migrated in step 2

There is one consumer:
- `cleanup_destroyed_bolts` -- removed in step 5

After migration, this type has zero references and can be deleted.

**Prelude update**: If `RequestBoltDestroyed` is re-exported in `src/prelude/messages.rs`, remove the re-export.

**Test helper update**: If `bolt_lost/tests/helpers.rs` contains a `CapturedRequestBoltDestroyed` or similar test helper for the old message type, remove it.

#### 5. Remove `cleanup_destroyed_bolts` system
Delete `src/bolt/systems/cleanup_destroyed_bolts.rs`. Remove its `mod` declaration from `src/bolt/systems/mod.rs`. Remove its registration from `src/bolt/plugin.rs`. Remove all imports.

This system is replaced by: the bolt domain kill handler (step 6) + `process_despawn_requests` (already exists from wave 7).

#### 6. Create bolt domain kill handler: `handle_bolt_kill`
New system at `src/bolt/systems/handle_bolt_kill.rs`.

**System signature:**
```rust
pub fn handle_bolt_kill(
    mut commands: Commands,
    mut reader: MessageReader<KillYourself<Bolt>>,
    mut destroyed_writer: MessageWriter<Destroyed<Bolt>>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
    query: Query<&GlobalPosition2D, (With<Bolt>, Without<Dead>)>,
    killer_query: Query<&GlobalPosition2D>,
) {
```

**Behavior:**
1. Read each `KillYourself<Bolt>` message.
2. For each message:
   a. Check if the victim entity exists in the query (which uses `Without<Dead>`). If not, skip (entity may have been despawned by another path, or already has `Dead` from a previous frame -- log a debug warning in debug builds).
   b. Insert `Dead` component on the victim entity via `commands.entity(victim).insert(Dead)`.
   c. Extract `victim_pos` from the victim's `GlobalPosition2D`.
   d. If `killer` is `Some(entity)`, extract `killer_pos` from the killer's `GlobalPosition2D` (using `killer_query`). If the killer entity no longer exists, set `killer_pos` to `None`.
   e. Remove the bolt from the spatial index. Use `commands.entity(victim).remove::<CollisionLayers>()` to exclude from future spatial queries, or follow the same pattern used by the cell domain kill handler from wave 9.
   f. Send `Destroyed<Bolt>` with `victim`, `killer`, `victim_pos`, `killer_pos`.
   g. Send `DespawnEntity { entity: victim }`.

**Deduplication**: `Without<Dead>` is sufficient -- no local `HashSet<Entity>` needed. Each entity receives at most one `KillYourself` per frame. `Dead` is inserted by the kill handler via commands and becomes visible next frame, preventing re-processing. The producers (`tick_bolt_lifespan` and `bolt_lost`) both use `Without<Dead>` filters in their queries, so a bolt that already has `Dead` will not be re-detected.

**Key constraints:**
- The entity must survive through this system. Only `DespawnEntity` triggers the actual despawn, which happens in PostFixedUpdate.
- The bolt entity is still alive when `Destroyed<Bolt>` is sent -- the effect system's death bridge needs to walk the entity's `BoundEffects`/`StagedEffects`.

#### 7. Add `Hp` and `KilledBy` to bolt builder
Update the bolt spawn function/builder to include `Hp` and `KilledBy` components on newly spawned bolt entities.

Bolt Hp value: `Hp { current: 1.0, starting: 1.0, max: None }`. Bolts have 1 HP -- they die from a single damage event (future mechanic). Most bolt deaths are environmental (lifespan expiry, falling off-screen) and bypass Hp entirely via direct `KillYourself<Bolt>`.

`KilledBy` should be inserted with `KilledBy::default()` (dealer: None).

The bolt builder is the function/system that spawns bolt entities (primary bolts, chain bolts, extra bolts, phantom bolts). All bolt entity spawning paths must include `Hp` and `KilledBy`. Identify all spawn paths:
- Primary bolt spawn (e.g., `spawn_bolt`)
- Chain bolt spawn (in `effect/effects/chain_bolt/`)
- Extra bolt spawn (Prism breaker)
- Phantom bolt spawn (in `effect/effects/phantom_bolt/` or `spawn_phantom/`)
- SpawnBolts effect (in `effect/effects/spawn_bolts/`)

For this wave, update only the bolt domain's own spawn function. The effect domain's spawn paths (chain bolt, spawn bolts, phantom) are the effect domain's responsibility and should already have been handled or will be handled separately.

#### 8. Register new message types in bolt plugin
In `src/bolt/plugin.rs`:
- Register `KillYourself<Bolt>` as a message (if not already registered by DeathPipelinePlugin or shared)
- Register `Destroyed<Bolt>` as a message (if not already registered)
- Remove registration of `RequestBoltDestroyed`
- Add `handle_bolt_kill` system to FixedUpdate

Note: `KillYourself<Bolt>`, `Destroyed<Bolt>`, and `DespawnEntity` may already be registered by `DeathPipelinePlugin` from wave 7. Check whether the death pipeline plugin registers these generic messages. If so, do not double-register. If the death pipeline plugin only registers the pipeline systems but not the messages, register them in `BoltPlugin`.

---

### Patterns to Follow

- Follow the pattern established by wave 9's cell domain kill handler (`src/cells/systems/handle_cell_kill.rs` or equivalent) for the bolt kill handler structure. The bolt handler is simpler (no `RequiredToClear` tracking, no `Locked` checks).
- Follow the existing `tick_bolt_lifespan` system structure for the message writer swap -- minimal change, just replace the message type.
- Follow the existing `bolt_lost` system structure -- add the breaker query, populate the new `BoltLost` fields, send `KillYourself` for ALL bolts, gate respawn on `Has<ExtraBolt>` (only baseline bolts respawn).
- For `Dead` insertion and spatial index removal, follow whatever pattern the cell domain kill handler established in wave 9.
- Use `GlobalPosition2D` (not `Position2D`) for extracting positions in the kill handler -- `GlobalPosition2D` is the resolved world-space position.
- `Without<Dead>` is sufficient for deduplication in the kill handler -- no local `HashSet<Entity>` needed.

---

### RON Data
No RON data changes needed for this wave. Bolt Hp is hardcoded at 1.0 in the builder. If bolt Hp needs to be configurable in the future, it would go in the bolt config RON -- but that is out of scope for this migration.

---

### Schedule

#### Modified systems (schedule unchanged)
- `tick_bolt_lifespan` -- stays in FixedUpdate, same schedule position. Only the message type changes. Add `Without<Dead>` filter.
- `bolt_lost` -- stays in FixedUpdate, same schedule position. Message type changes (RequestBoltDestroyed -> KillYourself<Bolt> for ALL bolts), `BoltLost` fields added. Retains existing `Without<Birthing>` filter if present. Add `Without<Dead>` filter. ExtraBolt distinction is ONLY for respawning (not for KillYourself).

#### New system
- `handle_bolt_kill` -- runs in **FixedUpdate**, after `DeathPipelineSystems::DetectDeaths`.

**Ordering constraints for `handle_bolt_kill`:**
- AFTER `DeathPipelineSystems::DetectDeaths` -- so that Hp-based deaths have sent `KillYourself<Bolt>`
- AFTER `tick_bolt_lifespan` -- so that lifespan-based `KillYourself<Bolt>` messages are available
- AFTER `bolt_lost` -- so that off-screen-based `KillYourself<Bolt>` messages are available

**Note on one-frame delay**: The `Destroyed<Bolt>` message sent by `handle_bolt_kill` will be consumed by `on_destroyed::<Bolt>` in the NEXT frame's `EffectSystems::Bridge` pass. There is NO ordering constraint "BEFORE EffectSystems::Bridge" -- the one-frame delay is acceptable and intentional per the death pipeline docs.

The exact frame ordering:
```
Game systems (tick_bolt_lifespan, bolt_lost, collision, etc.)
    |
EffectSystems::Bridge
    |
EffectSystems::Tick
    |
EffectSystems::Conditions
    |
DeathPipelineSystems::ApplyDamage
    |
DeathPipelineSystems::DetectDeaths
    |
Domain kill handlers (handle_bolt_kill, handle_cell_kill, etc.)
    |
PostFixedUpdate: process_despawn_requests
```

Register `handle_bolt_kill` in `BoltPlugin::build()` with:
```rust
app.add_systems(
    FixedUpdate,
    handle_bolt_kill.after(DeathPipelineSystems::DetectDeaths)
);
```

#### Removed system
- `cleanup_destroyed_bolts` -- remove from FixedUpdate registration in `BoltPlugin`.

---

### Shared Prerequisites

These types must already exist from previous waves (wave 2 scaffold + wave 7 death pipeline):

- `GameEntity` trait (src/shared/) -- with `impl GameEntity for Bolt {}`
- `Hp` component (src/shared/components/)
- `KilledBy` component (src/shared/components/)
- `Dead` component (src/shared/components/)
- `KillYourself<T>` message (src/shared/messages/)
- `Destroyed<T>` message (src/shared/messages/)
- `DespawnEntity` message (src/shared/messages/)
- `DamageDealt<T>` message (src/shared/messages/)
- `DeathPipelineSystems` system set (src/shared/)
- `apply_damage::<Bolt>` system (src/shared/systems/)
- `detect_bolt_deaths` system (src/bolt/systems/ or src/shared/systems/)
- `process_despawn_requests` system (src/shared/systems/)
- `DeathDetectionData` QueryData (src/shared/queries.rs)
- `DamageTargetData` QueryData (src/shared/queries.rs)

Additionally, `KillYourself<Bolt>` should have a convenience constructor:
```rust
impl<T: GameEntity> KillYourself<T> {
    pub fn new(victim: Entity, killer: Option<Entity>) -> Self {
        Self { victim, killer, _marker: PhantomData }
    }
}
```

And `Destroyed<Bolt>` should have:
```rust
impl<T: GameEntity> Destroyed<T> {
    pub fn new(victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>) -> Self {
        Self { victim, killer, victim_pos, killer_pos, _marker: PhantomData }
    }
}
```

These constructors should already exist from wave 2 scaffold.

---

### Wiring Requirements

#### `src/bolt/plugin.rs`
1. Remove `cleanup_destroyed_bolts` system registration
2. Remove `RequestBoltDestroyed` message registration (if messages are registered per-type)
3. Add `handle_bolt_kill` system in FixedUpdate with ordering `.after(DeathPipelineSystems::DetectDeaths)`
4. Import `DeathPipelineSystems` from shared

#### `src/bolt/systems/mod.rs`
1. Remove `pub(crate) mod cleanup_destroyed_bolts;` (or equivalent)
2. Remove `pub(crate) use cleanup_destroyed_bolts::*;` (or equivalent)
3. Add `pub(crate) mod handle_bolt_kill;`
4. Add `pub(crate) use handle_bolt_kill::*;`

#### `src/bolt/messages.rs`
1. Remove `RequestBoltDestroyed` struct entirely
2. Update `BoltLost` from unit struct to `BoltLost { bolt: Entity, breaker: Entity }`
3. Add `Entity` import if not present
4. Update any tests in this file to use the new `BoltLost` struct shape

#### `src/bolt/systems/bolt_lost/tests/helpers.rs`
1. Remove `CapturedRequestBoltDestroyed` test helper (or equivalent) if it exists
2. Update any test helpers that construct or assert on `BoltLost` to use the new struct fields

#### `src/prelude/messages.rs`
1. Remove `RequestBoltDestroyed` re-export if present
2. `BoltLost` re-export (if present) automatically reflects the updated struct shape

#### `src/bolt/mod.rs`
No changes needed if messages.rs is already re-exported.

#### Consumers of `BoltLost` (cross-domain)
The following files consume `BoltLost` and must be updated to handle the new struct fields:
- `src/effect/triggers/bolt_lost/bridges.rs` -- `on_bolt_lost_occurred`: should already expect `BoltLost { bolt, breaker }` per the effect-refactor design. If the bridge was stubbed in wave 2 with the new signature, no change needed. If it was stubbed with the old unit struct, update the destructuring.
- `src/bolt/systems/spawn_bolt_lost_text.rs` (or wherever the bolt-lost text spawning lives): update to accept the new struct. The text spawning only needs to know a bolt was lost -- the fields may be unused, but the destructuring must compile.

---

### Constraints

#### Do NOT modify
- `src/shared/` -- types and death pipeline systems are already implemented from waves 2 and 7. Do not change Hp, KilledBy, Dead, DamageDealt, KillYourself, Destroyed, DespawnEntity, apply_damage, detect_bolt_deaths, process_despawn_requests.
- `src/cells/` -- cell domain migration is wave 9 (already complete)
- `src/effect/` -- effect domain systems are from waves 4-6. The `on_bolt_lost_occurred` bridge may need a field destructuring update if it was stubbed with the old `BoltLost` shape, but do NOT change the bridge's logic.
- `src/breaker/` -- breaker domain migration is wave 12 (future)
- `src/run/` -- not in scope for this wave
- `src/walls/` -- wall domain migration is wave 11 (future)

#### Do NOT add
- Damage-to-bolt mechanics -- `apply_damage::<Bolt>` exists for future use; this wave does not add any system that sends `DamageDealt<Bolt>`
- Bolt invulnerability checks -- the kill handler confirms every kill (no shield/second-wind for bolts)
- Visual death effects for bolts -- VFX is not part of this migration
- Any changes to bolt collision systems (bolt-cell, bolt-wall, bolt-breaker) -- those are unchanged
- Any changes to bolt spawning logic beyond adding Hp/KilledBy components

---

### Implementation Order

Implement in this order to maintain compile-ability at each step:

1. **Update `BoltLost` message type** in messages.rs -- add fields. Fix all consumers that destructure it (compiler errors will guide you). Update any tests in messages.rs.
2. **Create `handle_bolt_kill`** system -- new file, new system. Wire in plugin.rs and systems/mod.rs.
3. **Update `tick_bolt_lifespan`** -- swap message writer from `RequestBoltDestroyed` to `KillYourself<Bolt>`. Add `Without<Dead>` filter.
4. **Update `bolt_lost`** -- swap message writer (send `KillYourself<Bolt>` for ALL bolts unconditionally), populate new `BoltLost` fields, add breaker query, add `Without<Dead>` filter. ExtraBolt distinction is ONLY for respawning: gate respawn on `!Has<ExtraBolt>` (only baseline bolts get respawned).
5. **Remove `RequestBoltDestroyed`** -- delete from messages.rs, remove from prelude/messages.rs if present, remove all imports (should be zero references at this point).
6. **Remove `cleanup_destroyed_bolts`** -- delete file, remove mod/use from systems/mod.rs, remove registration from plugin.rs.
7. **Clean up bolt_lost test helpers** -- remove `CapturedRequestBoltDestroyed` from bolt_lost/tests/helpers.rs if it exists.
8. **Add `Hp` and `KilledBy` to bolt builder** -- update the bolt spawn function.
9. **Verify** -- all tests pass, no clippy warnings, no dead code.

---

### Message Flow Summary (Post-Migration)

**Lifespan expiry path:**
```
tick_bolt_lifespan
  -> KillYourself<Bolt> { victim, killer: None }
    -> handle_bolt_kill
      -> insert Dead (deferred via commands, visible next frame)
      -> Destroyed<Bolt> { victim, killer: None, victim_pos, killer_pos: None }
      -> DespawnEntity { entity: victim }
        -> process_despawn_requests (PostFixedUpdate)
          -> entity despawned
```

**ExtraBolt lost (off-screen) path:**
```
bolt_lost (ExtraBolt detected off-screen)
  -> KillYourself<Bolt> { victim, killer: None }   (ALL bolt types)
  -> BoltLost { bolt, breaker }                     (ALL bolt types)
  -> NO respawn (ExtraBolt — just dies)
    -> handle_bolt_kill (processes KillYourself)
      -> insert Dead
      -> Destroyed<Bolt> { victim, killer: None, victim_pos, killer_pos: None }
      -> DespawnEntity { entity: victim }
    -> on_bolt_lost_occurred (processes BoltLost, evaluates triggers)
      -> effect tree walking (LoseLife, etc.)
```

**Baseline bolt lost (off-screen) path:**
```
bolt_lost (baseline bolt detected off-screen)
  -> KillYourself<Bolt> { victim, killer: None }   (ALL bolt types — baseline bolts are killed too)
  -> BoltLost { bolt, breaker }                     (effect triggers fire)
  -> respawn logic (baseline only — new bolt spawned)
    -> handle_bolt_kill (processes KillYourself for the OLD bolt)
      -> insert Dead
      -> Destroyed<Bolt> { victim, killer: None, victim_pos, killer_pos: None }
      -> DespawnEntity { entity: victim }
```

**Hp-based death path (future, uses wave 7 systems):**
```
DamageDealt<Bolt> (from some future source)
  -> apply_damage::<Bolt> (wave 7)
    -> decrement Hp, set KilledBy on killing blow
  -> detect_bolt_deaths (wave 7)
    -> KillYourself<Bolt> { victim, killer: Some(dealer) }
      -> handle_bolt_kill (same as above, but with killer attribution)
```

---

### Test Alignment Notes

Tests should verify:
1. `tick_bolt_lifespan` sends `KillYourself<Bolt>` (not `RequestBoltDestroyed`) with `killer: None`
2. `tick_bolt_lifespan` does NOT tick timer for bolts with `Birthing` (Without<Birthing> filter)
3. `tick_bolt_lifespan` skips bolts with `Dead` (Without<Dead> filter)
4. `bolt_lost` sends `KillYourself<Bolt>` with `killer: None` for ALL bolt types (both baseline and ExtraBolt)
5. `bolt_lost` sends `BoltLost { bolt, breaker }` for ALL bolt types (ExtraBolt and baseline)
6. `bolt_lost` respawns baseline bolts (no ExtraBolt) after sending KillYourself -- extra bolts are NOT respawned
7. `bolt_lost` skips bolts with `Dead` (Without<Dead> filter)
8. `handle_bolt_kill` inserts `Dead`, sends `Destroyed<Bolt>` with correct position data, sends `DespawnEntity`
9. `handle_bolt_kill` skips entities that already have `Dead` component (`Without<Dead>` filter -- no HashSet needed)
10. `handle_bolt_kill` handles missing victim entity gracefully (skip with debug log)
11. End-to-end: bolt lifespan expires -> full pipeline to despawn
12. End-to-end: ExtraBolt lost -> full pipeline to despawn, no respawn
13. End-to-end: baseline bolt lost -> KillYourself + BoltLost fires, old bolt killed, new bolt respawned
14. Bolt builder spawns entities with `Hp(1.0)` and `KilledBy(default)`

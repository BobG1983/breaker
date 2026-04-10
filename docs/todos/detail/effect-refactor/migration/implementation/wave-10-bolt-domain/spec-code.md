## Implementation Spec: Bolt — Death Pipeline Migration

### Domain
src/bolt/

### Failing Tests
- `src/bolt/systems/tick_bolt_lifespan/tests.rs` — tests for tick_bolt_lifespan sending KillYourself<Bolt> instead of RequestBoltDestroyed
- `src/bolt/systems/bolt_lost/tests.rs` — tests for bolt_lost sending KillYourself<Bolt> and updated BoltLost { bolt, breaker }
- `src/bolt/systems/handle_bolt_kill/tests.rs` — tests for new bolt domain kill handler
- `src/bolt/systems/detect_bolt_deaths/tests.rs` — tests for detect_bolt_deaths (Hp-based path)

Exact test count will be determined by the test spec. Expect approximately 12-18 tests across all files.

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
- Skip bolts with `Birthing` component (via `Without<Birthing>` filter)
- Fire only on `just_finished()`
- Skip bolts with `Dead` component (add `Without<Dead>` filter)

The system does NOT:
- Insert `Dead` — the domain kill handler does that
- Despawn the bolt — `process_despawn_requests` does that

**Message writer change:** Replace `MessageWriter<RequestBoltDestroyed>` with `MessageWriter<KillYourself<Bolt>>` in the system parameters.

#### 2. Update `bolt_lost` system
Modify the existing system to send `KillYourself<Bolt>` instead of `RequestBoltDestroyed`, and update the `BoltLost` message to include `bolt` and `breaker` fields.

**Before:**
```rust
writer.write(RequestBoltDestroyed { bolt: entity });
bolt_lost_writer.write(BoltLost);
```

**After:**
```rust
writer.write(KillYourself::<Bolt>::new(entity, None));
bolt_lost_writer.write(BoltLost { bolt: entity, breaker: breaker_entity });
```

The system continues to:
- Detect bolts that leave the play area (below breaker or off-screen)
- Send `BoltLost` for the effect system's `on_bolt_lost_occurred` bridge
- Handle respawn logic (unchanged)

The system must query the breaker entity to populate `BoltLost.breaker`. The breaker entity is already available in the system's existing queries or can be obtained via a `Query<Entity, With<Breaker>>` (there is exactly one breaker entity during gameplay).

Add `Without<Dead>` filter to the bolt query to skip bolts that are already dying.

Killer is `None` — bolt loss is an environmental death with no dealer.

The system does NOT:
- Insert `Dead`
- Despawn the bolt

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

This lives in `src/bolt/messages.rs`. All consumers of `BoltLost` must be updated to destructure the new fields. The primary consumer is `on_bolt_lost_occurred` in `src/effect/triggers/bolt_lost/bridges.rs` — that system reads `msg.bolt` and `msg.breaker` to build its `TriggerContext::BoltLost { bolt, breaker }`.

Also update `spawn_bolt_lost_text` (if it consumes `BoltLost`) to accept the new struct shape.

#### 4. Remove `RequestBoltDestroyed` message type
Delete the `RequestBoltDestroyed` message from `src/bolt/messages.rs`. Remove its registration from `src/bolt/plugin.rs`. Remove all imports referencing it.

There are exactly two producers:
- `tick_bolt_lifespan` — migrated in step 1
- `bolt_lost` — migrated in step 2

There is one consumer:
- `cleanup_destroyed_bolts` — removed in step 5

After migration, this type has zero references and can be deleted.

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
    query: Query<&GlobalPosition2D, With<Bolt>>,
    killer_query: Query<&GlobalPosition2D>,
) {
```

**Behavior:**
1. Read each `KillYourself<Bolt>` message.
2. For each message:
   a. Check if the victim entity exists in the query. If not, skip (entity may have been despawned by another path — log a debug warning in debug builds).
   b. Check if the victim already has the `Dead` component. If yes, skip (prevents double-processing).
   c. Insert `Dead` component on the victim entity via `commands.entity(victim).insert(Dead)`.
   d. Extract `victim_pos` from the victim's `GlobalPosition2D`.
   e. If `killer` is `Some(entity)`, extract `killer_pos` from the killer's `GlobalPosition2D` (using `killer_query`). If the killer entity no longer exists, set `killer_pos` to `None`.
   f. Remove the bolt from the spatial index. Use `commands.entity(victim).remove::<CollisionLayers>()` to exclude from future spatial queries, or follow the same pattern used by the cell domain kill handler from wave 9.
   g. Send `Destroyed<Bolt>` with `victim`, `killer`, `victim_pos`, `killer_pos`.
   h. Send `DespawnEntity { entity: victim }`.

**Key constraints:**
- The entity must survive through this system. Only `DespawnEntity` triggers the actual despawn, which happens in PostFixedUpdate.
- `Dead` insertion prevents other systems from double-processing this entity (via `Without<Dead>` filters).
- The bolt entity is still alive when `Destroyed<Bolt>` is sent — the effect system's death bridge needs to walk the entity's `BoundEffects`/`StagedEffects`.

#### 7. Add `Hp` and `KilledBy` to bolt builder
Update the bolt spawn function/builder to include `Hp` and `KilledBy` components on newly spawned bolt entities.

Bolt Hp value: `Hp { current: 1.0, starting: 1.0, max: None }`. Bolts have 1 HP — they die from a single damage event (future mechanic). Most bolt deaths are environmental (lifespan expiry, falling off-screen) and bypass Hp entirely via direct `KillYourself<Bolt>`.

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
- Follow the existing `tick_bolt_lifespan` system structure for the message writer swap — minimal change, just replace the message type.
- Follow the existing `bolt_lost` system structure — add the breaker query and populate the new `BoltLost` fields.
- For `Dead` insertion and spatial index removal, follow whatever pattern the cell domain kill handler established in wave 9.
- Use `GlobalPosition2D` (not `Position2D`) for extracting positions in the kill handler — `GlobalPosition2D` is the resolved world-space position.

---

### RON Data
No RON data changes needed for this wave. Bolt Hp is hardcoded at 1.0 in the builder. If bolt Hp needs to be configurable in the future, it would go in the bolt config RON — but that is out of scope for this migration.

---

### Schedule

#### Modified systems (schedule unchanged)
- `tick_bolt_lifespan` — stays in FixedUpdate, same schedule position. Only the message type changes.
- `bolt_lost` — stays in FixedUpdate, same schedule position. Only the message type and `BoltLost` fields change. Retains existing `Without<Birthing>` filter if present. Add `Without<Dead>` filter.

#### New system
- `handle_bolt_kill` — runs in **FixedUpdate**, after `DeathPipelineSystems::DetectDeaths`.

**Ordering constraints for `handle_bolt_kill`:**
- AFTER `DeathPipelineSystems::DetectDeaths` — so that Hp-based deaths have sent `KillYourself<Bolt>`
- AFTER `tick_bolt_lifespan` — so that lifespan-based `KillYourself<Bolt>` messages are available
- AFTER `bolt_lost` — so that off-screen-based `KillYourself<Bolt>` messages are available
- BEFORE `EffectSystems::Bridge` — so that `Destroyed<Bolt>` is available for the death bridge to read in the same frame (if the death bridge runs after kill handlers)

The exact ordering constraint depends on the frame ordering established in wave 7:
```
Game systems (tick_bolt_lifespan, bolt_lost, collision, etc.)
    ↓
EffectSystems::Bridge
    ↓
EffectSystems::Tick
    ↓
EffectSystems::Conditions
    ↓
DeathPipelineSystems::ApplyDamage
    ↓
DeathPipelineSystems::DetectDeaths
    ↓
Domain kill handlers (handle_bolt_kill, handle_cell_kill, etc.)
    ↓
PostFixedUpdate: process_despawn_requests
```

Based on the frame ordering doc, domain kill handlers run AFTER `DetectDeaths` and BEFORE the effect bridge reads `Destroyed<T>`. The `Destroyed<Bolt>` message sent by `handle_bolt_kill` will be consumed by `on_destroyed::<Bolt>` in the NEXT frame's `EffectSystems::Bridge` pass (one-frame delay is acceptable per the death pipeline docs).

Register `handle_bolt_kill` in `BoltPlugin::build()` with:
```rust
app.add_systems(
    FixedUpdate,
    handle_bolt_kill.after(DeathPipelineSystems::DetectDeaths)
);
```

#### Removed system
- `cleanup_destroyed_bolts` — remove from FixedUpdate registration in `BoltPlugin`.

---

### Shared Prerequisites

These types must already exist from previous waves (wave 2 scaffold + wave 7 death pipeline):

- `GameEntity` trait (src/shared/) — with `impl GameEntity for Bolt {}`
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

If `detect_bolt_deaths` is in `src/bolt/systems/`, it was implemented in wave 7. If it's generic and in shared, it was also wave 7. Either way, it exists before wave 10.

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
1. Remove `RequestBoltDestroyed` struct
2. Update `BoltLost` from unit struct to `BoltLost { bolt: Entity, breaker: Entity }`
3. Add `Entity` import if not present

#### `src/bolt/mod.rs`
No changes needed if messages.rs is already re-exported.

#### Consumers of `BoltLost` (cross-domain)
The following files consume `BoltLost` and must be updated to handle the new struct fields:
- `src/effect/triggers/bolt_lost/bridges.rs` — `on_bolt_lost_occurred`: should already expect `BoltLost { bolt, breaker }` per the effect-refactor design. If the bridge was stubbed in wave 2 with the new signature, no change needed. If it was stubbed with the old unit struct, update the destructuring.
- `src/bolt/systems/spawn_bolt_lost_text.rs` (or wherever the bolt-lost text spawning lives): update to accept the new struct. The text spawning only needs to know a bolt was lost — the fields may be unused, but the destructuring must compile.

#### Prelude
If `BoltLost` is in the prelude (check `src/prelude/messages.rs`), the prelude re-export automatically reflects the updated struct.

---

### Constraints

#### Do NOT modify
- `src/shared/` — types and death pipeline systems are already implemented from waves 2 and 7. Do not change Hp, KilledBy, Dead, DamageDealt, KillYourself, Destroyed, DespawnEntity, apply_damage, detect_bolt_deaths, process_despawn_requests.
- `src/cells/` — cell domain migration is wave 9 (already complete)
- `src/effect/` — effect domain systems are from waves 4-6. The `on_bolt_lost_occurred` bridge may need a field destructuring update if it was stubbed with the old `BoltLost` shape, but do NOT change the bridge's logic.
- `src/breaker/` — breaker domain migration is wave 12 (future)
- `src/run/` — not in scope for this wave
- `src/walls/` — wall domain migration is wave 11 (future)

#### Do NOT add
- Damage-to-bolt mechanics — `apply_damage::<Bolt>` exists for future use; this wave does not add any system that sends `DamageDealt<Bolt>`
- Bolt invulnerability checks — the kill handler confirms every kill (no shield/second-wind for bolts)
- Visual death effects for bolts — VFX is not part of this migration
- Any changes to bolt collision systems (bolt-cell, bolt-wall, bolt-breaker) — those are unchanged
- Any changes to bolt spawning logic beyond adding Hp/KilledBy components

---

### Implementation Order

Implement in this order to maintain compile-ability at each step:

1. **Update `BoltLost` message type** in messages.rs — add fields. Fix all consumers that destructure it (compiler errors will guide you).
2. **Create `handle_bolt_kill`** system — new file, new system. Wire in plugin.rs and systems/mod.rs.
3. **Update `tick_bolt_lifespan`** — swap message writer from `RequestBoltDestroyed` to `KillYourself<Bolt>`. Add `Without<Dead>` filter.
4. **Update `bolt_lost`** — swap message writer, populate new `BoltLost` fields, add breaker query, add `Without<Dead>` filter.
5. **Remove `RequestBoltDestroyed`** — delete from messages.rs, remove all imports (should be zero references at this point).
6. **Remove `cleanup_destroyed_bolts`** — delete file, remove mod/use from systems/mod.rs, remove registration from plugin.rs.
7. **Add `Hp` and `KilledBy` to bolt builder** — update the bolt spawn function.
8. **Verify** — all tests pass, no clippy warnings, no dead code.

---

### Message Flow Summary (Post-Migration)

**Lifespan expiry path:**
```
tick_bolt_lifespan
  → KillYourself<Bolt> { victim, killer: None }
    → handle_bolt_kill
      → insert Dead
      → Destroyed<Bolt> { victim, killer: None, victim_pos, killer_pos: None }
      → DespawnEntity { entity: victim }
        → process_despawn_requests (PostFixedUpdate)
          → entity despawned
```

**Bolt lost (off-screen) path:**
```
bolt_lost
  → KillYourself<Bolt> { victim, killer: None }
  → BoltLost { bolt, breaker }
    → handle_bolt_kill (processes KillYourself)
      → insert Dead
      → Destroyed<Bolt> { victim, killer: None, victim_pos, killer_pos: None }
      → DespawnEntity { entity: victim }
    → on_bolt_lost_occurred (processes BoltLost, evaluates triggers)
      → effect tree walking (LoseLife, etc.)
```

**Hp-based death path (future):**
```
DamageDealt<Bolt> (from some future source)
  → apply_damage::<Bolt>
    → decrement Hp, set KilledBy on killing blow
  → detect_bolt_deaths
    → KillYourself<Bolt> { victim, killer: Some(dealer) }
      → handle_bolt_kill (same as above, but with killer attribution)
```

---

### Test Alignment Notes

Tests should verify:
1. `tick_bolt_lifespan` sends `KillYourself<Bolt>` (not `RequestBoltDestroyed`) with `killer: None`
2. `bolt_lost` sends `KillYourself<Bolt>` with `killer: None` AND `BoltLost { bolt, breaker }` with correct entity values
3. `handle_bolt_kill` inserts `Dead`, sends `Destroyed<Bolt>` with correct position data, sends `DespawnEntity`
4. `handle_bolt_kill` skips entities that already have `Dead` (no double-processing)
5. `handle_bolt_kill` handles missing victim entity gracefully (skip with debug log)
6. End-to-end: bolt lifespan expires → full pipeline to despawn
7. End-to-end: bolt lost → full pipeline to despawn + effect trigger evaluation
8. Bolt builder spawns entities with `Hp(1.0)` and `KilledBy(default)`
9. `detect_bolt_deaths` sends `KillYourself<Bolt>` when Hp <= 0 (Hp-based path exists for future)

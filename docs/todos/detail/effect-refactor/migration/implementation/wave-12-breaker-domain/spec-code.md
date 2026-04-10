## Implementation Spec: Breaker Domain — Death Pipeline Migration

### Domain
src/breaker/

### Failing Tests
- `src/breaker/systems/handle_breaker_kill/tests.rs` — tests for the breaker kill handler system
- `src/breaker/systems/detect_breaker_deaths/tests.rs` — tests for breaker death detection (if domain-specific tests exist beyond the shared death pipeline wave 7 tests)

Test count will be determined by the test spec. Estimated: 8-12 tests covering builder Hp/KilledBy insertion, kill handler behavior, infinite-lives skip, Dead prevention, and e2e flow.

### What to Implement

#### 1. Breaker builder: conditional Hp/KilledBy insertion
- **Location**: Breaker builder/spawn system (the system or function that creates breaker entities from `BreakerDefinition`)
- **Behavior**: When `BreakerDefinition.life_pool` is `Some(n)`, insert `Hp { current: n as f32, starting: n as f32, max: None }` and `KilledBy::default()` on the breaker entity. When `life_pool` is `None`, do NOT insert `Hp` or `KilledBy` — the breaker has infinite lives and cannot die from damage.
- **Types used**: `Hp` (from `src/shared/components/`), `KilledBy` (from `src/shared/components/`), `BreakerDefinition` (from `src/breaker/definition.rs`)

#### 2. `detect_breaker_deaths` system
- **Location**: `src/breaker/systems/detect_breaker_deaths.rs` (new file)
- **Description**: Per-domain death detection system. Queries all breaker entities with `Hp` and `KilledBy`, filtered by `With<Breaker>` and `Without<Dead>`. For each entity whose `Hp.current <= 0.0`, sends `KillYourself<Breaker> { victim: entity, killer: killed_by.dealer }`.
- **Query**: `Query<DeathDetectionData, (With<Breaker>, Without<Dead>)>`
- **Output**: `MessageWriter<KillYourself<Breaker>>`
- **Does NOT**: Insert `Dead`. Does NOT despawn. Does NOT modify any components.

#### 3. `handle_breaker_kill` system (domain kill handler)
- **Location**: `src/breaker/systems/handle_breaker_kill.rs` (new file)
- **Description**: Breaker-specific kill handler. Reads `KillYourself<Breaker>` messages. For each message:
  1. Verify the entity exists and does NOT already have `Dead` component
  2. Insert `Dead` marker component on the breaker entity
  3. Look up the breaker's world position (for `Destroyed<Breaker>`)
  4. Look up the killer's world position if killer exists (for `Destroyed<Breaker>`)
  5. Send `Destroyed<Breaker> { victim, killer, victim_pos, killer_pos }`
  6. Send `RunLost` message to trigger game state transition (run end)
  7. Send `DespawnEntity { entity: victim }` for deferred cleanup
- **Input**: `MessageReader<KillYourself<Breaker>>`
- **Output**: `MessageWriter<Destroyed<Breaker>>`, `MessageWriter<RunLost>`, `MessageWriter<DespawnEntity>`
- **Query**: `Query<(Entity, &Transform), With<Breaker>>` for victim position, `Query<&Transform>` for killer position
- **Commands**: `Commands` for inserting `Dead` component
- **Critical**: Does NOT call `commands.entity(e).despawn()` directly. The `DespawnEntity` message is processed later by `process_despawn_requests` in PostFixedUpdate.
- **Critical**: Must check `Without<Dead>` or skip entities that already have `Dead` to prevent double-processing.

### Types Used (All Pre-existing from Earlier Waves)

These types are created in wave 2 (scaffold) and implemented in wave 7 (death pipeline). They must exist before wave 12 begins.

| Type | Location | Purpose |
|------|----------|---------|
| `Hp` | `src/shared/components/` | Health component with `current`, `starting`, `max` fields |
| `KilledBy` | `src/shared/components/` | Kill attribution: `dealer: Option<Entity>` |
| `Dead` | `src/shared/components/` | Marker component preventing double-processing |
| `GameEntity` | `src/shared/traits.rs` | Trait bound for `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>` |
| `DamageDealt<Breaker>` | `src/shared/messages.rs` | Damage message monomorphized for Breaker |
| `KillYourself<Breaker>` | `src/shared/messages.rs` | Death request message monomorphized for Breaker |
| `Destroyed<Breaker>` | `src/shared/messages.rs` | Death confirmed message monomorphized for Breaker |
| `DespawnEntity` | `src/shared/messages.rs` | Deferred despawn request |
| `DeathDetectionData` | `src/shared/queries.rs` | `QueryData` struct: `{ entity: Entity, killed_by: &KilledBy, hp: &Hp }` |
| `RunLost` | `src/state/run/` (existing) | Message consumed by `handle_run_lost` to transition game state |

### Patterns to Follow

- Follow the pattern established by other domain kill handlers created in waves 9-11:
  - `src/cells/systems/handle_cell_kill.rs` (wave 9) — cell domain kill handler
  - `src/bolt/systems/handle_bolt_kill.rs` (wave 10) — bolt domain kill handler
  - `src/walls/systems/handle_wall_kill.rs` (wave 11) — wall domain kill handler
- Follow the `detect_*_deaths` pattern from wave 7:
  - `src/cells/systems/detect_cell_deaths.rs` — same query structure, just for Breaker
- Breaker builder modifications should follow how cell and bolt builders conditionally add `Hp`/`KilledBy` in waves 9-10

#### Key differences from other domain kill handlers
- Other kill handlers (cell, bolt, wall) do NOT send `RunLost` — only the breaker kill handler does
- Other kill handlers may remove entities from spatial indices — the breaker kill handler does NOT need to (breaker cleanup is handled by state exit cleanup systems)
- The breaker kill handler's primary unique behavior is triggering game state transition via `RunLost`

### RON Data
No RON changes needed. The `life_pool` field already exists in `BreakerDefinition` RON files. The migration reads this existing field to determine whether to insert `Hp`/`KilledBy`.

Existing breaker RON values:
- `aegis.breaker.ron`: `life_pool: Some(3)` — gets `Hp { current: 3.0, starting: 3.0, max: None }` + `KilledBy::default()`
- `chrono.breaker.ron`: `life_pool: None` — no `Hp`, no `KilledBy`
- `prism.breaker.ron`: `life_pool: None` — no `Hp`, no `KilledBy`

### Schedule

#### `detect_breaker_deaths`
- **Schedule**: `FixedUpdate`
- **SystemSet**: `DeathPipelineSystems::DetectDeaths`
- **Ordering**: After `DeathPipelineSystems::ApplyDamage` (set-level ordering configured by `DeathPipelinePlugin`)
- **Registration**: In `DeathPipelinePlugin::build()` — this system is already registered as a stub in wave 2. Wave 12 implements the body.

#### `handle_breaker_kill`
- **Schedule**: `FixedUpdate`
- **Ordering**: After `DeathPipelineSystems::DetectDeaths` (domain kill handlers run after death detection)
- **Registration**: In `BreakerPlugin::build()` — this is a breaker domain system, not a death pipeline system. It reads `KillYourself<Breaker>` which is produced by `detect_breaker_deaths`.

#### Frame ordering context
```
EffectSystems::Bridge (LoseLife fires, sends DamageDealt<Breaker>)
    |
EffectSystems::Tick
    |
EffectSystems::Conditions
    |
DeathPipelineSystems::ApplyDamage (apply_damage::<Breaker> decrements Hp)
    |
DeathPipelineSystems::DetectDeaths (detect_breaker_deaths sends KillYourself<Breaker>)
    |
handle_breaker_kill (inserts Dead, sends Destroyed<Breaker> + RunLost + DespawnEntity)
    |
EffectSystems::Bridge (on_destroyed::<Breaker> dispatches Died/Killed/DeathOccurred)
    |
PostFixedUpdate: process_despawn_requests (despawns breaker entity)
```

Note: The `handle_breaker_kill` must run AFTER `DetectDeaths` but BEFORE the death bridges in `EffectSystems::Bridge` that consume `Destroyed<Breaker>`. Since `EffectSystems::Bridge` runs before `DeathPipelineSystems::ApplyDamage`, the `Destroyed<Breaker>` message sent this frame will be consumed by the death bridge next frame. This one-frame delay is by design and consistent with the cascade damage model described in `system-set-ordering.md`.

### Wiring Requirements

#### `src/breaker/plugin.rs`
- Register `handle_breaker_kill` system in `FixedUpdate`, after `DeathPipelineSystems::DetectDeaths`
- Example: `.add_systems(FixedUpdate, handle_breaker_kill.after(DeathPipelineSystems::DetectDeaths))`

#### `src/breaker/systems/mod.rs`
- Add `pub mod detect_breaker_deaths;` and `pub mod handle_breaker_kill;`
- Add corresponding `pub use` re-exports

#### `src/breaker/mod.rs`
- No changes needed if systems/mod.rs handles re-exports

#### Death pipeline plugin (already wired in wave 2)
- `detect_breaker_deaths` is already registered in `DeathPipelinePlugin` as part of the `DetectDeaths` set
- `apply_damage::<Breaker>` is already registered in `DeathPipelinePlugin` as part of the `ApplyDamage` set
- No additional death pipeline wiring needed in wave 12

### Breaker Builder Changes

The breaker builder (wherever breaker entities are spawned from `BreakerDefinition`) must be updated:

```
// Pseudocode for the conditional insertion:
if let Some(life_count) = definition.life_pool {
    // Finite lives — participates in death pipeline
    entity.insert(Hp {
        current: life_count as f32,
        starting: life_count as f32,
        max: None,
    });
    entity.insert(KilledBy::default());
} else {
    // Infinite lives — no Hp, cannot die from damage
    // Do NOT insert Hp or KilledBy
}
```

### E2E Flow: Breaker Life Loss

Complete pipeline for a finite-lives breaker (e.g., Aegis with `life_pool: Some(3)`):

1. Bolt falls below playfield → `BoltLostOccurred` trigger fires
2. Effect system walks Aegis breaker definition: `When(BoltLostOccurred, Fire(LoseLife(LoseLifeConfig())))`
3. `LoseLife` effect fires → sends `DamageDealt<Breaker> { target: breaker, amount: 1.0, dealer: None }`
4. `apply_damage::<Breaker>` reads `DamageDealt<Breaker>`, decrements `Hp.current` from 3.0 to 2.0
5. `detect_breaker_deaths` checks Hp: 2.0 > 0.0 → no action
6. Repeat steps 1-5 two more times: Hp goes 2.0 → 1.0 → 0.0
7. On Hp reaching 0.0: `apply_damage::<Breaker>` sets `KilledBy.dealer = None` (environmental death, no dealer)
8. `detect_breaker_deaths` detects Hp <= 0.0, sends `KillYourself<Breaker> { victim: breaker, killer: None }`
9. `handle_breaker_kill` receives `KillYourself<Breaker>`:
   - Inserts `Dead` on breaker
   - Sends `Destroyed<Breaker> { victim, killer: None, victim_pos, killer_pos: None }`
   - Sends `RunLost`
   - Sends `DespawnEntity { entity: breaker }`
10. `on_destroyed::<Breaker>` dispatches `Died` trigger on breaker, `DeathOccurred(Breaker)` globally (no `Killed` because killer is None)
11. `handle_run_lost` receives `RunLost`, transitions game state
12. `process_despawn_requests` despawns breaker entity in PostFixedUpdate

### E2E Flow: Infinite Lives Breaker

For breakers with `life_pool: None` (e.g., Chrono, Prism):

1. Bolt falls → `BoltLostOccurred` trigger fires
2. Effect system walks Chrono breaker definition: `When(BoltLostOccurred, Fire(TimePenalty(TimePenaltyConfig(seconds: 5.0))))`
3. `TimePenalty` effect fires → sends `ApplyTimePenalty { seconds: 5.0 }` (NOT `DamageDealt<Breaker>`)
4. No `Hp` on breaker → `apply_damage::<Breaker>` never matches → no death detection → no kill
5. Game continues with time penalty applied

For Chrono specifically, if it has a `LoseLife` chip effect: `LoseLife` fires → sends `DamageDealt<Breaker>` → `apply_damage::<Breaker>` query uses `With<Breaker>` + entity must have `Hp` component → no `Hp` on Chrono → query returns nothing → damage silently dropped.

### Constraints

#### Do NOT modify
- `src/shared/systems/apply_damage.rs` — generic system, already implemented in wave 7
- `src/shared/systems/process_despawn_requests.rs` — already implemented in wave 7
- `src/shared/components/` — `Hp`, `KilledBy`, `Dead` already exist from wave 2/7
- `src/shared/messages.rs` — `DamageDealt<Breaker>`, `KillYourself<Breaker>`, `Destroyed<Breaker>`, `DespawnEntity` already exist from wave 2
- `src/shared/queries.rs` — `DeathDetectionData`, `DamageTargetData` already exist from wave 7
- `src/effect/` — do not modify the effect domain
- `src/cells/` — do not modify the cell domain
- `src/bolt/` — do not modify the bolt domain
- `src/state/run/` — do not modify run state systems (they already consume `RunLost`)
- Death pipeline plugin wiring — already done in wave 2

#### Do NOT add
- New message types — use existing `KillYourself<Breaker>`, `Destroyed<Breaker>`, `RunLost`, `DespawnEntity`
- New component types — use existing `Hp`, `KilledBy`, `Dead`
- Invulnerability or second-wind checks in the kill handler — breaker kill handler is simple: if `KillYourself<Breaker>` arrives, the breaker dies. No rejection logic.
- Visual feedback systems — VFX for breaker death is out of scope for this wave
- Spatial index removal — breakers do not need spatial index cleanup in the kill handler (unlike cells/bolts)

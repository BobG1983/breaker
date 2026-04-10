## Implementation Spec: Breaker Domain — Death Pipeline Migration

### Domain
breaker-game/src/breaker/

### Prerequisites

Wave 12 has hard dependencies on these earlier waves:

- **Wave 2 (Scaffold)**: All shared death pipeline types must exist — `Hp`, `KilledBy`, `Dead`, `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `GameEntity` trait, `DeathDetectionData` query data, `DeathPipelineSystems` set enum. These live in `breaker-game/src/shared/`.
- **Wave 7 (Death Pipeline Core)**: `apply_damage::<Breaker>` and `detect_breaker_deaths` systems must be implemented, tested, and wired into `DeathPipelineSystems::ApplyDamage` and `DeathPipelineSystems::DetectDeaths` respectively. Wave 12 does NOT implement or modify these systems.

### Failing Tests
- `breaker-game/src/breaker/systems/handle_breaker_kill.rs` — tests for the breaker kill handler system (behaviors 14-21, plus e2e behaviors 26-28)
- `breaker-game/src/breaker/builder/tests/build_tests.rs` — tests for builder Hp/KilledBy insertion and LivesCount removal (behaviors 22-25)

Test count: approximately 15 tests covering kill handler behavior, builder Hp/KilledBy insertion, LivesCount removal, edge cases (stale victim, despawned killer), and e2e flow.

### What to Implement

#### 1. `handle_breaker_kill` system (new file)
- **Location**: `breaker-game/src/breaker/systems/handle_breaker_kill.rs` (new file)
- **Description**: Breaker-specific kill handler. Reads `KillYourself<Breaker>` messages. For each message:
  1. Look up the victim entity — if it doesn't exist or already has `Dead`, skip silently
  2. Insert `Dead` marker component on the breaker entity
  3. Look up the breaker's world position (for `Destroyed<Breaker>`)
  4. Look up the killer's world position if killer entity exists (for `Destroyed<Breaker>`) — if killer entity is gone, use `None`
  5. Send `Destroyed<Breaker> { victim, killer, victim_pos, killer_pos }`
  6. Send `RunLost` message to trigger game state transition (run end)
- **Input**: `MessageReader<KillYourself<Breaker>>`
- **Output**: `MessageWriter<Destroyed<Breaker>>`, `MessageWriter<RunLost>`
- **Query**: `Query<(Entity, &Position2D), (With<Breaker>, Without<Dead>)>` for victim lookup, `Query<&Position2D>` for killer position lookup
- **Commands**: `Commands` for inserting `Dead` component
- **Critical**: Does NOT send `DespawnEntity`. Does NOT call `commands.entity(e).despawn()`. The breaker entity persists through game-over. Cleanup happens via `CleanupOnExit<RunState>` when the run state exits.
- **Critical**: Must skip entities that already have `Dead` (query filter `Without<Dead>`) to prevent double-processing.
- **Critical**: Must handle stale victim entities gracefully (entity despawned between message send and handler execution) — skip silently.
- **Critical**: Must handle despawned killer entities gracefully — set `killer_pos: None` when the killer entity no longer exists in the world.

#### 2. Breaker builder: replace LivesCount with conditional Hp/KilledBy insertion
- **Location**: `breaker-game/src/breaker/builder/core/terminal.rs` — the `build_core()` function
- **Behavior**: When `LivesSetting::Count(n)`, insert `Hp { current: n as f32, starting: n as f32, max: None }` and `KilledBy::default()` on the breaker entity. When `LivesSetting::Unset` or `LivesSetting::Infinite`, do NOT insert `Hp` or `KilledBy` — the breaker has infinite lives and cannot die from damage.
- **Remove**: The `LivesCount` component insertion from `build_core()`. Line 190 currently reads `let lives_component = LivesCount(lives);` — this entire line and the `LivesCount` in the returned tuple must be replaced with conditional `Hp`/`KilledBy` insertion.
- **Types used**: `Hp` (from shared components), `KilledBy` (from shared components). Remove import of `LivesCount` from `crate::effect::effects::life_lost::LivesCount`.

#### 3. LoseLife effect: update fire() to send DamageDealt\<Breaker\>
- **Location**: `breaker-game/src/effect/effects/life_lost.rs`
- **Behavior**: Instead of directly mutating `LivesCount`, the `fire()` function should send `DamageDealt<Breaker> { dealer: None, target: entity, amount: 1.0, source_chip }`. The `reverse()` function needs corresponding update (send a healing message or directly increment Hp — design decision deferred to implementation, but the simplest approach is to directly increment Hp.current by 1.0 in reverse since there is no "HealDealt" message).
- **Remove**: Direct mutation of `LivesCount` and direct `RunLost` write from fire(). The death pipeline handles the RunLost transition now.
- **Note**: The `LivesCount` component type itself may remain in this file temporarily if other code references it, or it may be removed entirely if wave 12 is the only consumer. The writer-code should check for remaining references.

### Types Used (All Pre-existing from Earlier Waves)

These types are created in wave 2 (scaffold) and implemented in wave 7 (death pipeline). They must exist before wave 12 begins.

| Type | Location | Purpose |
|------|----------|---------|
| `Hp` | `breaker-game/src/shared/components/` | Health component with `current`, `starting`, `max` fields |
| `KilledBy` | `breaker-game/src/shared/components/` | Kill attribution: `dealer: Option<Entity>` |
| `Dead` | `breaker-game/src/shared/components/` | Marker component preventing double-processing |
| `GameEntity` | `breaker-game/src/shared/` | Trait bound for `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>` |
| `DamageDealt<Breaker>` | `breaker-game/src/shared/messages/` | Damage message monomorphized for Breaker |
| `KillYourself<Breaker>` | `breaker-game/src/shared/messages/` | Death request message monomorphized for Breaker |
| `Destroyed<Breaker>` | `breaker-game/src/shared/messages/` | Death confirmed message monomorphized for Breaker |
| `DeathDetectionData` | `breaker-game/src/shared/queries.rs` | `QueryData` struct: `{ entity: Entity, killed_by: &KilledBy, hp: &Hp }` |
| `RunLost` | `breaker-game/src/state/run/messages.rs` (existing) | Message consumed by `handle_run_lost` to transition game state |

### Patterns to Follow

- **Kill handler pattern**: Follow the pattern that will be established by other domain kill handlers (waves 9-11). If those waves are not yet complete, use this canonical pattern:
  - Read `KillYourself<T>` messages via `MessageReader`
  - Look up victim entity via query with `Without<Dead>` filter
  - Insert `Dead` via `Commands`
  - Look up positions for `Destroyed<T>` message
  - Send domain-specific messages (`RunLost` for breaker, nothing extra for cells/bolts)
  - Send `Destroyed<T>` message
- **Key differences from other domain kill handlers**:
  - Other kill handlers send `DespawnEntity` — the breaker kill handler does NOT
  - Other kill handlers may remove entities from spatial indices — the breaker kill handler does NOT (breaker cleanup is handled by state exit cleanup via `CleanupOnExit<RunState>`)
  - The breaker kill handler's primary unique behavior is triggering game state transition via `RunLost`
- **Builder pattern**: Follow how `build_core()` in `breaker-game/src/breaker/builder/core/terminal.rs` currently bundles components. The `Hp`/`KilledBy` insertion replaces the `LivesCount` insertion at the same location in the returned tuple.

### RON Data
No RON changes needed. The `life_pool` field already exists in `BreakerDefinition` RON files. The migration reads this existing field to determine whether to insert `Hp`/`KilledBy`.

Existing breaker RON values:
- `aegis.breaker.ron`: `life_pool: Some(3)` — gets `Hp { current: 3.0, starting: 3.0, max: None }` + `KilledBy::default()`
- `chrono.breaker.ron`: `life_pool: None` — no `Hp`, no `KilledBy`
- `prism.breaker.ron`: `life_pool: None` — no `Hp`, no `KilledBy`

### Schedule

#### `handle_breaker_kill`
- **Schedule**: `FixedUpdate`
- **Ordering**: After `DeathPipelineSystems::DetectDeaths` (domain kill handlers run after death detection)
- **Registration**: In `BreakerPlugin::build()` — this is a breaker domain system, not a death pipeline system. It reads `KillYourself<Breaker>` which is produced by `detect_breaker_deaths`.
- **Run condition**: `in_state(NodeState::Playing)` — consistent with other breaker FixedUpdate systems

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
handle_breaker_kill (inserts Dead, sends Destroyed<Breaker> + RunLost)
    |
EffectSystems::Bridge (on_destroyed::<Breaker> dispatches Died/Killed/DeathOccurred)
```

Note: The `handle_breaker_kill` must run AFTER `DetectDeaths` but BEFORE the death bridges in `EffectSystems::Bridge` that consume `Destroyed<Breaker>`. Since `EffectSystems::Bridge` runs before `DeathPipelineSystems::ApplyDamage`, the `Destroyed<Breaker>` message sent this frame will be consumed by the death bridge next frame. This one-frame delay is by design and consistent with the cascade damage model described in `system-set-ordering.md`.

### Wiring Requirements

#### `breaker-game/src/breaker/plugin.rs`
- Register `handle_breaker_kill` system in `FixedUpdate`, after `DeathPipelineSystems::DetectDeaths`
- Example: `.add_systems(FixedUpdate, handle_breaker_kill.after(DeathPipelineSystems::DetectDeaths).run_if(in_state(NodeState::Playing)))`
- Add `Destroyed<Breaker>` message registration: `app.add_message::<Destroyed<Breaker>>()`
- Import `handle_breaker_kill` from `systems` module

#### `breaker-game/src/breaker/systems/mod.rs`
- Add `mod handle_breaker_kill;`
- Add `pub(crate) use handle_breaker_kill::handle_breaker_kill;`

#### `breaker-game/src/breaker/builder/core/terminal.rs`
- **File to modify**: `breaker-game/src/breaker/builder/core/terminal.rs` — the `build_core()` function
- Remove `LivesCount` import from line 15: `use crate::effect::{EffectCommandsExt, effects::life_lost::LivesCount};` — change to `use crate::effect::EffectCommandsExt;`
- Add imports for `Hp` and `KilledBy` from shared components
- Replace the `lives_component` logic (lines 89-91 and 190) with conditional Hp/KilledBy insertion:
  - When `LivesSetting::Count(n)`: include `Hp { current: n as f32, starting: n as f32, max: None }` and `KilledBy::default()` in the bundle
  - When `LivesSetting::Unset` or `LivesSetting::Infinite`: do NOT include `Hp` or `KilledBy`
- Note: Since Bevy bundles are static tuples, conditional component insertion requires either using `Commands` after spawn or restructuring the bundle. The writer-code should use the same approach as the existing `lives` variable (lines 89-93) and spawn the optional components conditionally via a separate `commands.entity(id).insert(...)` call after the main spawn.

#### `breaker-game/src/effect/effects/life_lost.rs`
- **File to modify**: `breaker-game/src/effect/effects/life_lost.rs`
- Update `fire()` to send `DamageDealt<Breaker>` instead of directly mutating `LivesCount`
- Update `reverse()` to directly increment `Hp.current` by 1.0 (or send a reverse damage message if one exists)
- Remove direct `RunLost` write from `fire()` — the death pipeline handles this now
- The `LivesCount` type definition may be removed if no other code references it, or kept temporarily with a deprecation comment

#### Death pipeline plugin (already wired in wave 2)
- `detect_breaker_deaths` is already registered in `DeathPipelinePlugin` as part of the `DetectDeaths` set
- `apply_damage::<Breaker>` is already registered in `DeathPipelinePlugin` as part of the `ApplyDamage` set
- No additional death pipeline wiring needed in wave 12

### E2E Flow: Breaker Life Loss

Complete pipeline for a finite-lives breaker (e.g., Aegis with `life_pool: Some(3)`):

1. Bolt falls below playfield -> `BoltLostOccurred` trigger fires
2. Effect system walks Aegis breaker definition: `When(BoltLostOccurred, Fire(LoseLife(LoseLifeConfig())))`
3. `LoseLife` effect fires -> sends `DamageDealt<Breaker> { target: breaker, amount: 1.0, dealer: None }`
4. `apply_damage::<Breaker>` reads `DamageDealt<Breaker>`, decrements `Hp.current` from 3.0 to 2.0
5. `detect_breaker_deaths` checks Hp: 2.0 > 0.0 -> no action
6. Repeat steps 1-5 two more times: Hp goes 2.0 -> 1.0 -> 0.0
7. On Hp reaching 0.0: `apply_damage::<Breaker>` sets `KilledBy.dealer = None` (environmental death, no dealer)
8. `detect_breaker_deaths` detects Hp <= 0.0, sends `KillYourself<Breaker> { victim: breaker, killer: None }`
9. `handle_breaker_kill` receives `KillYourself<Breaker>`:
   - Inserts `Dead` on breaker
   - Sends `Destroyed<Breaker> { victim, killer: None, victim_pos, killer_pos: None }`
   - Sends `RunLost`
   - Does NOT send `DespawnEntity` — breaker persists via `CleanupOnExit<RunState>`
10. `on_destroyed::<Breaker>` dispatches `Died` trigger on breaker, `DeathOccurred(Breaker)` globally (no `Killed` because killer is None)
11. `handle_run_lost` receives `RunLost`, transitions game state
12. When `RunState` exits, `CleanupOnExit<RunState>` despawns the breaker entity

### E2E Flow: Infinite Lives Breaker

For breakers with `life_pool: None` (e.g., Chrono, Prism):

1. Bolt falls -> `BoltLostOccurred` trigger fires
2. Effect system walks Chrono breaker definition: `When(BoltLostOccurred, Fire(TimePenalty(TimePenaltyConfig(seconds: 5.0))))`
3. `TimePenalty` effect fires -> sends `ApplyTimePenalty { seconds: 5.0 }` (NOT `DamageDealt<Breaker>`)
4. No `Hp` on breaker -> `apply_damage::<Breaker>` never matches -> no death detection -> no kill
5. Game continues with time penalty applied

For Chrono specifically, if it has a `LoseLife` chip effect: `LoseLife` fires -> sends `DamageDealt<Breaker>` -> `apply_damage::<Breaker>` query uses `With<Breaker>` + entity must have `Hp` component -> no `Hp` on Chrono -> query returns nothing -> damage silently dropped.

### Constraints

#### Do NOT modify
- `breaker-game/src/shared/systems/apply_damage.rs` — generic system, already implemented in wave 7
- `breaker-game/src/shared/components/` — `Hp`, `KilledBy`, `Dead` already exist from wave 2/7
- `breaker-game/src/shared/messages/` — `DamageDealt<Breaker>`, `KillYourself<Breaker>`, `Destroyed<Breaker>` already exist from wave 2
- `breaker-game/src/shared/queries.rs` — `DeathDetectionData`, `DamageTargetData` already exist from wave 7
- `breaker-game/src/cells/` — do not modify the cell domain
- `breaker-game/src/bolt/` — do not modify the bolt domain
- `breaker-game/src/state/run/` — do not modify run state systems (they already consume `RunLost`)
- Death pipeline plugin wiring — already done in wave 2

#### Do NOT add
- `DespawnEntity` output to the breaker kill handler — breaker is NOT despawned by the kill handler
- New message types — use existing `KillYourself<Breaker>`, `Destroyed<Breaker>`, `RunLost`
- New component types — use existing `Hp`, `KilledBy`, `Dead`
- Invulnerability or second-wind checks in the kill handler — breaker kill handler is simple: if `KillYourself<Breaker>` arrives, the breaker dies. No rejection logic.
- Visual feedback systems — VFX for breaker death is out of scope for this wave
- Spatial index removal — breakers do not need spatial index cleanup in the kill handler (unlike cells/bolts)

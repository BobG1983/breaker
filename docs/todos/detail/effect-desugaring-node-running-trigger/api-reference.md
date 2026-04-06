# Effect System API Reference

The unified vocabulary for RON definitions and the programmatic builder. RON and builder use the same names.

## Wrappers

| Wrapper | RON | Builder | Fires | Reverses | Reversible req | Self-removes |
|---|---|---|---|---|---|---|
| `When` | `When(trigger, ...)` | `.when(trigger)` | Every trigger match | Never | No | No |
| `Once` | `Once(trigger, ...)` | `.once(trigger)` | First trigger match | Never | No | Yes |
| `During` | `During(condition, ...)` | `.during(condition)` | Condition start | Condition end | Direct fire: yes | No (condition can cycle) |
| `Until` | `Until(trigger, ...)` | `.until(trigger)` | Immediately | Trigger match | Yes | Yes |
| `Spawned` | `Spawned(EntityType, Fire(...))` | `.spawned(type).fire(...)` | Entity add | Never | No | No |

### Once
One-shot gate. Matches its trigger once, evaluates inner tree, removes itself from BoundEffects. Doesn't care about reversibility — it's just a gate. The inner tree handles scoping:
```ron
// Simple one-shot
Once(PerfectBumped, Fire(Explode(range: 50.0)))

// One-shot + scoped reversal
Once(PerfectBumped, Until(BoltLostOccurred, Fire(SpeedBoost(1.5))))

// Nested one-shots
Once(PerfectBumped, Once(Impacted(Cell), Fire(ChainBolt(tether_distance: 120.0))))
```

### During vs Until
Both involve reversible effects. `During` is state-scoped (fires on condition start, reverses on condition end, stays in BoundEffects because conditions can cycle). `Until` is event-scoped (fires immediately, reverses when trigger fires, self-removes because the trigger can only match once meaningfully).

| | Takes | Fires when | Reverses when | Stays in BoundEffects |
|---|---|---|---|---|
| `During` | Condition (state) | Condition becomes true | Condition becomes false | Yes (condition can cycle) |
| `Until` | Trigger (event) | Immediately | Trigger fires | No (self-removes after reversal) |

Both require `Reversible` effects for direct `Fire()`. Both relax to `AnyFire` when wrapping a nested `When` (reversal removes the trigger registration, not individual firings).

## Conditions (for `During`)

| Condition | Start | End | Notes |
|---|---|---|---|
| `NodeActive` | Node start | Node teardown | Ignores pause. Spans Playing + Paused. Most common. |
| `ShieldActive` | Any `ShieldWall` entity spawns | Last `ShieldWall` despawns | Global — true when any shield exists in the world. |
| `ComboActive(u32)` | Nth consecutive perfect bump | Non-perfect bump (streak breaks) | Uses existing `consecutive_perfect_bumps` counter. `ComboActive(3)` = fires on 3rd consecutive perfect. |

## Local Triggers (fire on specific participant entities)

| Name | Fires on | Participant enum | Notes |
|---|---|---|---|
| `PerfectBumped` | Both bolt and breaker | `BumpTarget` | |
| `EarlyBumped` | Both bolt and breaker | `BumpTarget` | |
| `LateBumped` | Both bolt and breaker | `BumpTarget` | |
| `Bumped` | Both bolt and breaker | `BumpTarget` | Any successful bump grade |
| `Impacted(ImpactTarget)` | Both entities in collision | `ImpactTarget` | |
| `Died` | Victim only | `DeathTarget` | "I died" |
| `Killed(KillTarget)` | Killer only | `DeathTarget` | **NEW** — "I killed X" |

## Global Triggers (fire on ALL entities with BoundEffects)

| Name | Participant enum | Replaces |
|---|---|---|
| `PerfectBumpOccurred` | `BumpTarget` | `PerfectBump` |
| `EarlyBumpOccurred` | `BumpTarget` | `EarlyBump` |
| `LateBumpOccurred` | `BumpTarget` | `LateBump` |
| `BumpOccurred` | `BumpTarget` | `Bump` |
| `BumpWhiffOccurred` | `BumpTarget` | `BumpWhiff` |
| `NoBumpOccurred` | `BumpTarget` | `NoBump` |
| `ImpactOccurred(ImpactTarget)` | `ImpactTarget` | `Impact(X)` |
| `DeathOccurred(DeathTarget)` | `DeathTarget` | `Death` + `CellDestroyed` |
| `BoltLostOccurred` | `BoltLostTarget` | `BoltLost` |
| `NodeStartOccurred` | (none) | `NodeStart` |
| `NodeEndOccurred` | (none) | `NodeEnd` |
| `NodeTimerThresholdOccurred(f32)` | (none) | `NodeTimerThreshold(f32)` |

## Participant Enums

Shared by concept, not per-trigger. Triggers sharing an enum live in the same source folder.

```rust
enum BumpTarget { Bolt, Breaker }          // triggers/bump/
enum ImpactTarget { Impactor, Impactee }   // triggers/impact/
enum DeathTarget { Victim, Killer }        // triggers/death/
enum BoltLostTarget { Bolt, Breaker }      // triggers/bolt_lost/
```

## Special Triggers

| Name | Type | Participants | Notes |
|---|---|---|---|
| `Spawned(EntityType)` | Bridge | (implicit target) | **NEW** — fires on `Added<Bolt/Cell/Wall/Breaker>` |
| `TimeExpires(f32)` | Self-consuming | (none) | Timer countdown, fires when zero |

## Stamp (chip/breaker definition routing)

| Name | RON | Builder | Purpose |
|---|---|---|---|
| `Route` | `Route(EntityTarget, tree)` | `.route(target)` | Route subtree to entity type at load time. Sets `This` for the subtree. Adds to BoundEffects (permanent). |

`Route` is a **chip/breaker definition wrapper** — it tells the loader which entity type to route the subtree onto. Required at root of every `effects: []` entry.

Entity targets for Route:

| Target | Meaning |
|---|---|
| `Bolt` | Route to bolt entities |
| `Breaker` | Route to breaker entities |
| `Cell` | Route to cell entities |
| `Wall` | Route to wall entities |
| `ActiveBolts` | Route to all existing bolt entities (point-in-time snapshot). Replaces `AllBolts`. |
| `EveryBolt` | All existing + future bolts. Desugars to `ActiveBolts` + `Spawned(Bolt)`. |
| `PrimaryBolts` | Entities with `PrimaryBolt` marker (plural — could be multiple). |
| `ExtraBolts` | Entities with `ExtraBolt` marker. |
| `ActiveCells` / `EveryCells` | Same pattern for cells. |
| `ActiveWalls` / `EveryWall` | Same pattern for walls. |
| `ActiveBreakers` / `EveryBreaker` | Same pattern for breakers. |

## On (runtime target redirect)

`On` is ONLY used inside a tree to redirect Fire/Stamp/Transfer to a **non-This** target:

| Target type | Example | When to use |
|---|---|---|
| `[ParticipantEnum]::[Variant]` | `On(BumpTarget::Bolt, ...)`, `On(ImpactTarget::Impactee, ...)` | Target a trigger event participant |

`On(This, ...)` is never needed — `Fire` implicitly targets `This`.

## Terminals

| Terminal | RON | Builder | Destination | When to use |
|---|---|---|---|---|
| `Fire` | `Fire(EffectType(...))` | `.fire(effect)` | Immediate | Execute an effect on `This` |
| `Stamp` | `Stamp(inner_tree)` | `.stamp(inner_tree)` | BoundEffects | Permanently add a tree to the `On` target — re-arms, survives multiple triggers |
| `Transfer` | `Transfer(inner_tree)` | `.transfer(inner_tree)` | StagedEffects | One-shot arm a tree on the `On` target — consumed when triggered |

## Rename Summary (current → new)

| Current | New |
|---|---|
| `PerfectBump` | `PerfectBumpOccurred` |
| `EarlyBump` | `EarlyBumpOccurred` |
| `LateBump` | `LateBumpOccurred` |
| `Bump` | `BumpOccurred` |
| `BumpWhiff` | `BumpWhiffOccurred` |
| `NoBump` | `NoBumpOccurred` |
| `Impact(X)` | `ImpactOccurred(X)` |
| `Death` | `DeathOccurred(DeathTarget)` |
| `CellDestroyed` | `DeathOccurred(Cell)` |
| `BoltLost` | `BoltLostOccurred` |
| `NodeStart` | `NodeStartOccurred` |
| `NodeEnd` | `NodeEndOccurred` |
| `NodeTimerThreshold(f32)` | `NodeTimerThresholdOccurred(f32)` |
| `AllBolts` | `ActiveBolts` |
| `AllCells` | `ActiveCells` |
| `AllWalls` | `ActiveWalls` |
| `On(target: Bolt, then: [...])` | `Route(Bolt, ...)` (chip-level routing) |
| `On(target: Breaker, then: [...])` | `Route(Breaker, ...)` (chip-level routing) |
| `On(target: AllBolts, then: [...])` | `Route(ActiveBolts, ...)` (chip-level routing) |
| `On(target: Cell, ...)` inside tree | `On(ImpactTarget::Impactee, ...)` (trigger participant) |
| `On(This, Fire(...))` | `Fire(...)` (Fire implicitly targets This) |
| `Do(...)` | `Fire(...)` |
| `Self` (target) | `This` (implicit via Stamp, rarely needed explicitly) |
| `then: [...]` | `Fire(...)` or nested wrapper |

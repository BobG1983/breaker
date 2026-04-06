# Effect System API Reference

The unified vocabulary for RON definitions and the programmatic builder. RON and builder use the same names.

## Wrappers

| Wrapper | RON | Builder | Semantics |
|---|---|---|---|
| `When` | `When(trigger, ...)` | `.when(trigger)` | One-shot: fires each time trigger matches. Can nest: `.when(A).when(B)`. |
| `During` | `During(condition, ...)` | `.during(condition)` | State-scoped: fires on condition start, reverses on condition end. Direct `Fire()` must be reversible. Nested `When` relaxes constraint. |
| `Until` | `Until(trigger, ...)` | `.until(trigger)` | Event-scoped: fires immediately, reverses when trigger fires. Direct `Fire()` must be reversible. Nested `When` relaxes constraint. |
| `Spawned` | `Spawned(EntityType, Fire(...))` | `.spawned(type).fire(...)` | Implicit target (the spawned entity). Fires on `Added<T>`. Does NOT fire retroactively. |

### During vs Until

| | Takes | Fires when | Reverses when | Example |
|---|---|---|---|---|
| `During` | Condition (state) | Condition becomes true | Condition becomes false | `During(NodeActive, ...)` |
| `Until` | Trigger (event) | Immediately | Trigger fires | `Until(Died, ...)` |

Both require `Reversible` effects for direct `Fire()`. Both relax to `AnyFire` when wrapping a nested `When` (reversal removes the trigger registration, not individual firings).

## Conditions (for `During`)

| Condition | Start | End | Notes |
|---|---|---|---|
| `NodeActive` | Node start | Node teardown | Ignores pause. Spans Playing + Paused. Most common. |
| `NodePlaying` | Enter `NodeState::Playing` | Exit `NodeState::Playing` | Respects pause — effect toggles on/off. Niche. |

## Local Triggers (fire on specific participant entities)

| Name | Fires on | Participants | Notes |
|---|---|---|---|
| `PerfectBumped` | Both bolt and breaker | `::Bolt`, `::Breaker` | |
| `EarlyBumped` | Both bolt and breaker | `::Bolt`, `::Breaker` | |
| `LateBumped` | Both bolt and breaker | `::Bolt`, `::Breaker` | |
| `Bumped` | Both bolt and breaker | `::Bolt`, `::Breaker` | Any successful bump grade |
| `Impacted(ImpactTarget)` | Both entities in collision | `::Impactor`, `::Target` | |
| `Died` | Victim only | `::Victim`, `::Killer` | "I died" |
| `Killed(KillTarget)` | Killer only | `::Killer`, `::Victim` | **NEW** — "I killed X" |

## Global Triggers (fire on ALL entities with BoundEffects)

| Name | Participants | Replaces |
|---|---|---|
| `PerfectBumpOccurred` | `::Bolt`, `::Breaker` | `PerfectBump` |
| `EarlyBumpOccurred` | `::Bolt`, `::Breaker` | `EarlyBump` |
| `LateBumpOccurred` | `::Bolt`, `::Breaker` | `LateBump` |
| `BumpOccurred` | `::Bolt`, `::Breaker` | `Bump` |
| `BumpWhiffOccurred` | `::Bolt`, `::Breaker` | `BumpWhiff` |
| `NoBumpOccurred` | `::Bolt`, `::Breaker` | `NoBump` |
| `ImpactOccurred(ImpactTarget)` | Depends on target: `::Bolt`+`::Cell`, etc. | `Impact(X)` |
| `DeathOccurred(DeathTarget)` | `::Entity`, `::Killer` | `Death` + `CellDestroyed` |
| `BoltLostOccurred` | `::Bolt`, `::Breaker` | `BoltLost` |
| `NodeStartOccurred` | (none) | `NodeStart` |
| `NodeEndOccurred` | (none) | `NodeEnd` |
| `NodeTimerThresholdOccurred(f32)` | (none) | `NodeTimerThreshold(f32)` |

## Special Triggers

| Name | Type | Participants | Notes |
|---|---|---|---|
| `Spawned(EntityType)` | Bridge | (implicit target) | **NEW** — fires on `Added<Bolt/Cell/Wall/Breaker>` |
| `TimeExpires(f32)` | Self-consuming | (none) | Timer countdown, fires when zero |

## Targets

| Name | Meaning |
|---|---|
| `This` | The entity BoundEffects lives on. Not a trigger participant — always the "owner." |
| `[Trigger]::[Participant]` | Named participant from trigger event (e.g., `PerfectBumped::Breaker`, `Died::Killer`) |
| `EveryBolt` | All existing + future bolts. Desugars to `ActiveBolts` + `Spawned(Bolt)`. |
| `ActiveBolts` | Existing bolt entities right now (point-in-time snapshot). Replaces `AllBolts`. |
| `PrimaryBolts` | Entities with `PrimaryBolt` marker (plural — could be multiple). |
| `ExtraBolts` | Entities with `ExtraBolt` marker. |
| `EveryCell` / `ActiveCells` | Same pattern for cells. |
| `EveryWall` / `ActiveWalls` | Same pattern for walls. |
| `EveryBreaker` / `ActiveBreakers` | Same pattern for breakers. |

## Terminals

| Terminal | RON | Builder | When to use |
|---|---|---|---|
| `Fire` | `Fire(EffectType(...))` | `.fire(effect)` | Execute an effect on the target |
| `Transfer` | `Transfer(inner_tree)` | `.transfer(inner_tree)` | Stamp an effect tree onto the target entity's BoundEffects |
| | `Transfer(target, inner_tree)` | `.transfer_to(target, inner_tree)` | Transfer to explicit target (override) |

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
| `Bolt` (target) | `This` or named participant |
| `Cell` (target) | `This` or named participant |
| `Wall` (target) | `This` or named participant |
| `Breaker` (target) | `This` or named participant |
| `Do(...)` | `Fire(...)` |
| `Self` (target) | `This` |
| `then: [...]` | `Fire(...)` or nested wrapper |

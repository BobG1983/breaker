# Trigger Reference

Every `Trigger` variant, its scope (global vs local), and what fires it. This is the authoritative table — the design docs in `design/triggers/` were the original source but used pre-refactor names. This table uses the actual `Trigger` enum variant names from `effect_v3/types/trigger.rs`.

**Scope conventions:**
- **Local** — fires only on the entities directly involved in the event. Bridge iterates specific entities from the message.
- **Global** — fires on every entity that has `BoundEffects` or `StagedEffects`. Bridge iterates the entire query.
- **Special** — handled by a dedicated system, not the standard bridge pattern.

## Bump Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `PerfectBumped` | Local (bolt + breaker) | Perfect-timed bump — fires on both participants | `TriggerContext::Bump { bolt, breaker }` |
| `EarlyBumped` | Local (bolt + breaker) | Early-timed bump | same |
| `LateBumped` | Local (bolt + breaker) | Late-timed bump | same |
| `Bumped` | Local (bolt + breaker) | Any non-whiff bump (early/late/perfect) | same |
| `PerfectBumpOccurred` | Global | A perfect bump happened somewhere | same (context available for On redirect) |
| `EarlyBumpOccurred` | Global | An early bump happened somewhere | same |
| `LateBumpOccurred` | Global | A late bump happened somewhere | same |
| `BumpOccurred` | Global | Any non-whiff bump happened somewhere | same |
| `BumpWhiffOccurred` | Global | Bump window expired without contact | `TriggerContext::Bump { bolt: None, breaker }` |
| `NoBumpOccurred` | Global | Bolt passed breaker without bump attempt | `TriggerContext::Bump { bolt: None, breaker }` |

Note: `BumpWhiffOccurred` and `NoBumpOccurred` carry `bolt: None` — there is no participating bolt, so `On(Bump(Bolt), ...)` resolves to `None` and the redirect is silently skipped.

## Impact Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `Impacted(EntityKind)` | Local (both participants) | "This entity collided with an entity of kind X" | `TriggerContext::Impact { impactor, impactee }` |
| `ImpactOccurred(EntityKind)` | Global | "A collision involving kind X happened somewhere" | same |

A single collision (e.g. `BoltImpactCell { bolt, cell }`) fires **four** triggers:

1. `Impacted(Cell)` — local on the bolt ("you collided with a cell")
2. `Impacted(Bolt)` — local on the cell ("you collided with a bolt")
3. `ImpactOccurred(Cell)` — global ("a collision with a cell happened")
4. `ImpactOccurred(Bolt)` — global ("a collision with a bolt happened")

## Death Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `Died` | Local (victim only) | "This entity died" | `TriggerContext::Death { victim, killer }` |
| `Killed(EntityKind)` | Local (killer only) | "This entity killed something of kind X" | same |
| `DeathOccurred(EntityKind)` | Global | "An entity of kind X died somewhere" | same |

`Death.killer` is `Option<Entity>` — environmental deaths (timer expiry, fall off screen) have `killer: None`. `Killed(EntityKind)` does not fire when the killer is `None`.

## Loss Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `BoltLostOccurred` | Global | A bolt fell off the bottom of the screen | `TriggerContext::BoltLost { bolt, breaker }` |

## Node Lifecycle Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `NodeStartOccurred` | Global | A new node has started | `TriggerContext::None` |
| `NodeEndOccurred` | Global | The current node has ended (cleared or failed) | `TriggerContext::None` |
| `NodeTimerThresholdOccurred(OrderedFloat<f32>)` | Global | Node timer ratio dropped below threshold (0.0–1.0) | `TriggerContext::None` |

These triggers have no participants — `On(...)` is not valid against them (resolution returns `None`).

## Timer Triggers

| Trigger variant | Scope | Description | Participants |
|---|---|---|---|
| `TimeExpires(OrderedFloat<f32>)` | Special (owner only) | Countdown reached zero on this entity | `TriggerContext::None` |

`TimeExpires` is fired by the time trigger category's timer-tick system (`effect_v3/triggers/time/`). It is not a regular bridge — instead of reading a game message, the timer system manages per-entity countdowns and fires the trigger when the countdown hits zero. See `until.md` for the common authoring pattern: `Until(TimeExpires(2.0), Fire(SpeedBoost(...)))`.

## Bridge → Trigger mapping

Each collision message type maps to specific bridge functions. The bridge functions live in `effect_v3/triggers/<category>/bridges/system.rs`:

| Source message | Bridge category | Triggers fired |
|---|---|---|
| `BumpPerformed` | `bump/` | `PerfectBumped`, `EarlyBumped`, `LateBumped`, `Bumped` (local) + `PerfectBumpOccurred`, `EarlyBumpOccurred`, `LateBumpOccurred`, `BumpOccurred` (global) |
| `BumpWhiffed` | `bump/` | `BumpWhiffOccurred` (global) |
| `NoBumpDetected` | `bump/` | `NoBumpOccurred` (global) |
| `BoltImpactCell` | `impact/` | `Impacted(Cell)` + `Impacted(Bolt)` (local) + `ImpactOccurred(Cell)` + `ImpactOccurred(Bolt)` (global) |
| `BoltImpactWall` | `impact/` | same pattern with `Wall`/`Bolt` |
| `BoltImpactBreaker` | `impact/` | same pattern with `Breaker`/`Bolt` |
| `Destroyed<Cell>` | `death/` | `Died` (local on victim) + `Killed(Cell)` (local on killer) + `DeathOccurred(Cell)` (global) |
| `Destroyed<Bolt>` | `death/` | same pattern with `Bolt` |
| `Destroyed<Wall>` | `death/` | same pattern with `Wall` |
| `BoltLost` | `bolt_lost/` | `BoltLostOccurred` (global) |
| node state transitions | `node/` | `NodeStartOccurred`, `NodeEndOccurred` (global) |
| node timer tick | `node/` | `NodeTimerThresholdOccurred(ratio)` (global) |
| per-entity timer tick | `time/` | `TimeExpires(seconds)` (special — owner only) |

## Design docs mapping

The `design/triggers/` files used pre-refactor names. Here's the mapping from design name to actual enum variant:

| Design name | Actual variant |
|---|---|
| `PerfectBump` | `PerfectBumpOccurred` |
| `PerfectBumped` | `PerfectBumped` (unchanged) |
| `EarlyBump` | `EarlyBumpOccurred` |
| `EarlyBumped` | `EarlyBumped` (unchanged) |
| `LateBump` | `LateBumpOccurred` |
| `LateBumped` | `LateBumped` (unchanged) |
| `Bump` | `BumpOccurred` |
| `Bumped` | `Bumped` (unchanged) |
| `BumpWhiff` | `BumpWhiffOccurred` |
| `NoBump` | `NoBumpOccurred` |
| `Impact(X)` | `ImpactOccurred(EntityKind)` |
| `Impacted(X)` | `Impacted(EntityKind)` (unchanged) |
| `Death` | `DeathOccurred(EntityKind)` |
| `Died` | `Died` (unchanged) |
| `CellDestroyed` | `DeathOccurred(EntityKind::Cell)` |
| `BoltLost` | `BoltLostOccurred` |
| `NodeStart` | `NodeStartOccurred` |
| `NodeEnd` | `NodeEndOccurred` |
| `NodeTimerThreshold(f32)` | `NodeTimerThresholdOccurred(OrderedFloat<f32>)` |
| `TimeExpires(f32)` | `TimeExpires(OrderedFloat<f32>)` |

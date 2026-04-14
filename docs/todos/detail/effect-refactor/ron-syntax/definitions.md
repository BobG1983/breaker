# Definitions

| Term | Definition |
|------|-----------|
| **RootNode** | Top-level entry in an `effects: []` list. Either Stamp(StampTarget, Tree) or Spawn(EntityKind, Tree). |
| **Tree** | Any node: Fire, When, Once, During, Until, Sequence, On. Can nest arbitrarily (subject to scoping rules). |
| **Scoped Tree** | A Tree inside During/Until. An immediate child Fire must be a reversible Effect. An immediate child Sequence must have all reversible children. A When child relaxes this rule (reversal removes the listener, not individual firings). |
| **Terminal** | A leaf operation: Fire(Effect) or Route(RouteType, Tree). Fire executes immediately. Route installs a tree on another entity. |
| **RouteType** | How Route installs a tree: Bound (permanent → BoundEffects) or Staged (one-shot → StagedEffects). |
| **Inner** | The Tree contained by a wrapper node — the thing inside When/Once/During/Until. |
| **Owner** | The entity whose BoundEffects/StagedEffects is being walked. Fire always executes on the Owner. On() redirects to a different entity (a Participant). |
| **StampTarget** | First argument to Stamp(). Which entity or entities receive the Tree. See [stamp-targets/](stamp-targets/index.md). |
| **Trigger** | A game event that gates evaluation. First argument to When, Once, and Until (e.g. `PerfectBumped`, `Impacted(Cell)`). See [triggers/](triggers/index.md). |
| **Effect** | The action that Fire executes on the Owner (e.g. `SpeedBoost(1.5)`, `Shockwave(ShockwaveConfig(...))`). See [effects/](effects/index.md). |
| **EntityKind** | Entity type filter used by Trigger parameters and Spawn (e.g. `Cell`, `Any`). See [entitykinds-list.md](entitykinds-list.md). |
| **Condition** | A state that gates During. Unlike a Trigger (one-time event), a Condition has a start and end and can cycle (e.g. `NodeActive`, `ComboActive(3)`). See [conditions/](conditions/index.md). |
| **Participant** | A named role in a Trigger event, used by On() to redirect a Terminal to a non-Owner entity (e.g. `Impact(Impactee)`, `Death(Killer)`). See [participants/](participants/index.md). |
| **Local** | Trigger scope. Fires only on the Participant entities involved in the event (e.g. the bolt and breaker that bumped). |
| **Global** | Trigger scope. Fires on all entities that have effect trees. |
| **Self** | Trigger scope. Fires only on the Owner entity. Used by TimeExpires. |
| **BoundEffects** | Permanent effect tree storage on an entity. Route(Bound) adds here. When/Once entries live here and re-arm on each match. |
| **StagedEffects** | One-shot effect tree storage on an entity. Route(Staged) adds here. Armed inner trees from When live here and are consumed when matched. |
| **DuringActive** | Runtime condition tracking component (`HashSet<String>`). Holds the source strings of During entries whose condition is currently true. Managed by `evaluate_conditions`. |
| **DuringInstallCommand** | Internal command that appends a During sub-tree to BoundEffects with an `#installed[0]` source suffix and an idempotency guard. Used by Shape A and Shape B nested conditions. |

## Nested Condition Shapes

The tree grammar supports four patterns for combining triggers and conditions:

### Shape A — `When(X, During(Cond, inner))`

A trigger gates the installation of a condition scope. The During is installed only when trigger X fires.

```
// Effect tree RON
When(PerfectBumped, During(NodeActive, Fire(SpeedBoost((mult: 1.5)))))
```

**Runtime behavior:**
1. On each `PerfectBumped` event, walker calls `DuringInstallCommand` which appends `During(NodeActive, Fire(SpeedBoost(...)))` to `BoundEffects` under source `{source}#installed[0]` (idempotent — re-fires if already installed do nothing).
2. `evaluate_conditions` picks up the new During entry next frame and manages its condition lifecycle.
3. While `NodeActive` is true: SpeedBoost is active. When node ends: SpeedBoost is reversed.

### Shape B — `Until(X, During(Cond, inner))`

A condition scope is active until a trigger fires. The During is installed immediately.

```
// Effect tree RON
Until(TimeExpires(30.0), During(NodeActive, Fire(DamageBoost((mult: 2.0)))))
```

**Runtime behavior:**
1. On install: `DuringInstallCommand` appends `During(NodeActive, Fire(DamageBoost(...)))` under source `{source}#installed[0]`. A 30-second timer is registered.
2. While `NodeActive` and within 30 seconds: DamageBoost is active.
3. When timer fires (or node ends before timer): DamageBoost is reversed and the During entry is removed.

### Shape C — `During(Cond, When(Trigger, Fire(reversible)))`

A condition gates whether a trigger listener is armed. Trigger effects that fire while the condition is active are bulk-reversed when the condition ends.

```
// Effect tree RON
During(ShieldActive, When(PerfectBumped, Fire(SpeedBoost((mult: 1.2)))))
```

**Runtime behavior:**
1. On condition-becomes-true (`ShieldActive`): `install_armed_entry()` appends the inner `When(PerfectBumped, Fire(SpeedBoost(...)))` to `BoundEffects` under source `{source}#armed[0]`.
2. While `ShieldActive`: each `PerfectBumped` fires SpeedBoost from that armed source.
3. On condition-becomes-false: the armed entry is removed and `reverse_all_by_source_dispatch` bulk-reverses all SpeedBoost stacks applied by `{source}#armed[0]`.

### Shape D — `During(Cond, On(Participant, Fire(reversible)))`

Same as Shape C but redirects the fire to a participant entity (e.g. the bolt, the cell).

```
// Effect tree RON (on breaker entity)
During(NodeActive, On(Impact(Impactee), Fire(Vulnerable((duration_secs: 2.0)))))
```

**Runtime behavior:**
1. On condition-becomes-true: `install_armed_entry()` appends the `On(Impact(Impactee), Fire(Vulnerable(...)))` entry under source `{source}#armed[0]`.
2. While condition is active: each impact fires `Vulnerable` on the impacted cell.
3. On condition-becomes-false: armed entry removed, all `Vulnerable` effects from `{source}#armed[0]` bulk-reversed via `reverse_all_by_source_dispatch`.

# Conditions (During Lifecycle)

Conditions are state-based (not event-based). They have a start and end. `During(condition, ...)` fires effects on condition start and reverses them on condition end. Conditions can cycle.

## Conditions

| Condition | Start | End |
|-----------|-------|-----|
| `NodeActive` | Enter Playing from non-Playing/non-Paused | Node teardown |
| `ShieldActive` | `Added<ShieldWall>` detected (first shield) | Last `ShieldWall` despawned |
| `ComboActive(u32)` | Nth consecutive perfect bump achieved | Non-perfect bump (streak breaks) |

## Condition Monitor Systems

One system per condition. Each watches for its specific state change and activates/deactivates During entries.

### `monitor_node_active`

Watches `NodeState` transitions. NodeActive spans both Playing and Paused states.

- **Start**: transition to Playing from non-Playing non-Paused state
- **End**: node teardown (exit from Playing/Paused entirely)

### `monitor_shield_active`

Watches `ShieldWall` entity existence.

- **Start**: `Added<ShieldWall>` detected AND no shield was active before
- **End**: `ShieldWall` removed AND no `ShieldWall` entities remain
- Uses `RemovedComponents<ShieldWall>` + existence query for edge detection

### `monitor_combo_active`

Watches consecutive perfect bump counter.

- **Start**: `consecutive_perfect_bumps` crosses N upward (for each `ComboActive(N)`)
- **End**: `consecutive_perfect_bumps` resets to 0 (non-perfect bump)
- Must track per-N threshold state to detect crossings

## During Activation/Deactivation

During entries live in `BoundEffects.conditions` permanently. Condition monitors call activate/deactivate on each transition. Monitors defer effect execution via `EffectCommandsExt` on `Commands` — condition activate and deactivate queue fire/reverse commands rather than executing immediately, ensuring consistent ordering with the rest of effect dispatch.

### Direct Fire: `During(NodeActive, Fire(SpeedBoost(1.5)))`

- **Activate**: `fire_effect(entity, SpeedBoost(1.5))`
- **Deactivate**: `reverse_effect(entity, SpeedBoost(1.5))`
- Condition cycles: re-fire on next activation, re-reverse on next deactivation

### Direct Sequence: `During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))`

- **Activate**: fire all children in order
- **Deactivate**: reverse all children in reverse order

### Nested When: `During(NodeActive, When(PerfectBumped, Fire(Explode)))`

- **Activate**: register `When(PerfectBumped, Fire(Explode))` into `BoundEffects.triggers` with scope source
- **While active**: normal trigger dispatch handles PerfectBumped → Fire(Explode)
- **Deactivate**: unregister from `BoundEffects.triggers` by scope source + clean up armed StagedEffects entries

## Scope Source

Derived from chip SourceId + condition: `format!("{source}:During({condition:?})")`

Enables targeted cleanup of just the During-registered entries without affecting other entries from the same chip.

## During vs Until

| | Takes | Fires when | Reverses when | Stays in BoundEffects |
|---|---|---|---|---|
| `During` | Condition (state) | Condition becomes true | Condition becomes false | Yes (can cycle) |
| `Until` | Trigger (event) | Immediately | Trigger fires | No (self-removes) |

Both require `Reversible` effects for direct `Fire`. Both relax to `AnyFire` when wrapping a nested `When`.

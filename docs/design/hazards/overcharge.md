# Hazard: Overcharge

## Game Design

Bolt gains speed per cell destroyed. A successful **Bump** (active breaker mechanic with a timing grade — see `docs/design/terminology/core.md`) resets the kill count and the accumulated speed. A passive rebound off the breaker does NOT reset — only a graded Bump does. "Sounds like a buff. Isn't." At high kill counts between bumps, the bolt becomes nearly uncontrollable. Player chooses between risky-but-efficient long kill streaks (skip the Bump, let the bolt rebound) and safe resets (time a Bump, accept the slower return).

**Per-kill multiplier**: `1.0 + base_frac + per_level_frac * (stacks - 1)`. Raised to the power of kills-this-cycle. Example: stack 3, 1 kill → `1.0 + 0.05 + 0.03 * 2 = 1.11`; 10 kills → `1.11^10 ≈ 2.84×`.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct OverchargeConfig {
    /// Base speed-multiplier increment per kill, as a fraction (0.05 = +5%).
    pub base_frac: f32,
    /// Per-stack additional increment per kill, as a fraction (0.03 = +3% per level beyond 1).
    pub per_level_frac: f32,
}
```

Populated from `HazardTuning::Overcharge`.

## Components

```rust
#[derive(Component, Debug, Default)]
pub(crate) struct OverchargeKillCount(pub u32);
```

Tracks kills-this-cycle per bolt. Lazily inserted on each bolt's first kill (inside `overcharge_count_kills`) — distinct from the `Added<Bolt>` attach pattern. Overcharge's count only matters once a bolt has scored, and most bolts never do. When a bolt despawns, its count goes with it — no cleanup system.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`) + its `KilledBy` attribution to resolve which bolt scored. `BumpPerformed` to reset.
**Writes**: Source-tagged `EffectStack<SpeedBoostConfig>` entry on the scoring bolt (source: `"hazard:overcharge"`). Value is `(1.0 + base_frac + per_level_frac * (stacks - 1))^kills`. Same reconciliation pattern as Haste — idempotent via `EffectStack::retain_by_source`. Reset on bump removes the entry.

## Systems

### `overcharge_count_kills`
- **Schedule**: `FixedUpdate`, `.after(DmgSystems::ApplyKill)` so kill attribution is resolved.
- **run_if**: `hazard_active(HazardKind::Overcharge)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `Destroyed<Cell>` + `KilledBy`. For each kill attributed to a bolt:
  1. Lazily insert `OverchargeKillCount(0)` on the bolt if absent.
  2. Increment the count.
  3. Reconcile the bolt's `EffectStack<SpeedBoostConfig>` entry `"hazard:overcharge"` with multiplier `(1.0 + base_frac + per_level_frac * (stack - 1))^count`.

### `overcharge_reset_on_bump`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::ProcessBump)` (or wherever `BumpPerformed` is emitted).
- **run_if**: `hazard_active(HazardKind::Overcharge)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. For each bumped bolt, sets `OverchargeKillCount = 0` and removes the `"hazard:overcharge"` source from its `EffectStack<SpeedBoostConfig>`.

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` from `rantzsoft_dmg` inside `overcharge_count_kills` — each kill attributed to a bolt (via `KilledBy`) increments the bolt's `OverchargeKillCount` (lazy insertion on first kill).
- **Not a damage emitter or mutator.** Overcharge does NOT participate in any `DeathPipelineSystems` set. After counting, it reconciles `EffectStack<SpeedBoostConfig>` on the bolt (source `"hazard:overcharge"`) — a speed multiplier, not a damage one. Reset on `BumpPerformed` clears the count.
- **Ordering**: `overcharge_count_kills` runs `.after(DmgSystems::ApplyKill)` so kill attribution is resolved.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

| Stack | Per-kill multiplier | After 3 kills | After 5 kills | After 10 kills |
|-------|---------------------|---------------|---------------|----------------|
| 1 | 1.05 | 1.16× | 1.28× | 1.63× |
| 2 | 1.08 | 1.26× | 1.47× | 2.16× |
| 3 | 1.11 | 1.37× | 1.69× | 2.84× |

Speed compounds multiplicatively per kill. Stack 3 with 10 kills = ~3× speed before the bump resets. Combined with Haste (constant multiplier), becomes extreme.

## Cross-Domain Dependencies
- **bolt**: Consumes `EffectStack<SpeedBoostConfig>` aggregate. Owns `Velocity2D` write.
- **damage crate (`rantzsoft_dmg`)**: Reads `Destroyed<Cell>` + `KilledBy` attribution.
- **breaker/bolt**: Reads `BumpPerformed`.
- **effect_v3**: Provides `EffectStack` + `SpeedBoostConfig` (survives post-TODO #0).

## Expected Behaviors (for test specs)

1. **Bolt gains speed on first kill at stack 1** — `base_frac: 0.05`, stack 1, 1 kill: `EffectStack<SpeedBoostConfig>` entry `"hazard:overcharge"` with value `1.05`; `OverchargeKillCount(1)`.
2. **Speed compounds on consecutive kills** — 2 kills at stack 1: entry value `1.05^2 = 1.1025`.
3. **Kill count resets on bump** — bolt with `OverchargeKillCount(5)`, `BumpPerformed`: count zeroed, `"hazard:overcharge"` source removed from `EffectStack<SpeedBoostConfig>`.
4. **Per-kill multiplier at stack 3** — `base_frac: 0.05`, `per_level_frac: 0.03`, stack 3, 1 kill: multiplier `1.11`.
5. **Lazy insertion** — bolt with no kills has no `OverchargeKillCount` component. On first kill it's inserted.
6. **No effect when inactive** — `hazard_active(Overcharge) = false`: no count updates, no `EffectStack` writes.
7. **Per-bolt attribution** — bolt A kills a cell: only bolt A gains speed; bolt B unaffected.

## Edge Cases
- **Overcharge + Haste**: both write `EffectStack<SpeedBoostConfig>` with different sources — aggregate multiplies them. Stack 2 Haste (1.30×) + Overcharge 10 kills at stack 2 (2.16×) = effective ~2.81× before kill-count reset.
- **Multi-bolt**: each bolt has its own `OverchargeKillCount`. Kill attribution via `KilledBy` ensures the scoring bolt gets the boost.
- **Bolt lost (falls off screen)**: bolt is despawned — count goes with it. New bolt spawns fresh.
- **Max speed clamp**: if the bolt domain clamps max speed, Overcharge hits the ceiling at extreme kill counts. Natural cap.
- **Zero kills in cycle**: no-op — `OverchargeKillCount` never inserted; no `EffectStack` entry.
- **Cleanup**: `OverchargeKillCount` on bolt entities — cleaned up on despawn. `OverchargeConfig` removed at run end. `"hazard:overcharge"` `EffectStack` entries cleared when the hazard deactivates.

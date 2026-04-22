# Hazard: Haste

## Game Design

Bolt moves faster. +20% base, +10% per stack, multiplicative with existing speed. Universal pressure multiplier — every other mechanic becomes harder when the bolt is faster. Visibly zips at high stacks.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct HasteConfig {
    pub base_percent: f32,       // 20.0
    pub per_level_percent: f32,  // 10.0
}
```

Populated from `HazardTuning::Haste`.

## Components
None.

## Messages
**Reads**: `Option<Res<ActiveHazards>>` for stack count (harness-safety — `Option<Res>` so the system is usable without `HazardPlugin`).
**Writes**: Source-tagged `EffectStack<SpeedBoostConfig>` entry on every Bolt (source: `"hazard:haste"`). The entry's multiplier is reconciled each FixedUpdate. Idempotent via `EffectStack::retain_by_source`. Haste's reconciliation mirrors Erosion's `SizeBoost` pattern — the canonical way for hazards to apply continuous modulation to Bolt components.

## Systems

### `haste_reconcile_speed`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Haste)` + `in_state(NodeState::Playing)`.
- **Behavior**: For every Bolt, computes `multiplier = 1.0 + (base_percent + per_level_percent * (stack - 1)) / 100.0`. Writes a single `EffectStack<SpeedBoostConfig>` entry with source `"hazard:haste"` and the multiplier. `retain_by_source` ensures exactly one entry per Bolt per source — re-running the system leaves stable state. The bolt's movement system aggregates the stack and applies it during integration.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Haste does not participate in any `DeathPipelineSystems` set.
- **Trigger**: FixedUpdate tick (`run_if = hazard_active(Haste) + in_state(NodeState::Playing)`) — reconciliation runs every frame regardless of damage events.
- **Writes**: `EffectStack<SpeedBoostConfig>` reconciled per-tick on every Bolt (source `"hazard:haste"`).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

Linear percentage, applied multiplicatively to existing speed: `multiplier = 1.0 + (base_percent + per_level_percent * (stack - 1)) / 100.0`.

| Stack | Percentage | Multiplier |
|-------|------------|------------|
| 1 | +20% | 1.20× |
| 2 | +30% | 1.30× |
| 3 | +40% | 1.40× |
| 5 | +60% | 1.60× |

Multiplicative with OTHER speed modifiers (chip `SpeedBoost`, Overcharge, protocol effects) — via the `EffectStack` aggregator, which composes multipliers across sources.

## Cross-Domain Dependencies
- **bolt**: Consumes `EffectStack<SpeedBoostConfig>` aggregate during movement integration. Owns `Velocity2D` write.
- **effect_v3 / rantzsoft_dmg**: Provides `EffectStack` + `SpeedBoostConfig` (survives post-TODO #1 — only `DamageBoostConfig` / `VulnerableConfig` retired).

## Expected Behaviors (for test specs)

1. **Multiplier at stack 1** — `base_percent: 20.0`, `per_level_percent: 10.0`, stack 1: entry value `1.20`.
2. **Multiplier at stack 3** — stack 3: entry value `1.40`.
3. **EffectStack entry reconciled each tick** — running the system twice leaves exactly one entry with source `"hazard:haste"`; no accumulation.
4. **Multiplicative with chip SpeedBoost** — haste entry 1.20 + chip entry 1.50: aggregate multiplier `1.80` (EffectStack composes multiplicatively).
5. **No entry when Haste is inactive** — `hazard_active(Haste) = false`: system skipped; no `"hazard:haste"` entry on bolts.
6. **Multiplier updates on stack change** — stack 1 (1.20×) → stack 2 (1.30×): next tick's reconciliation replaces the entry value.
7. **Harness-safety with absent `ActiveHazards`** — `Option<Res<ActiveHazards>>` is `None`: system returns without writing.

## Edge Cases
- **Haste + Overcharge**: both modulate Bolt speed via `EffectStack<SpeedBoostConfig>`. Sources differ (`"hazard:haste"` vs `"hazard:overcharge"`), so both entries coexist and aggregate multiplicatively. With Haste stack 2 (1.30×) + Overcharge adding 1.15× per kill, after 3 kills the bolt is at `1.30 * 1.15^3 ≈ 1.98×`.
- **Haste + Erosion**: faster bolt + smaller breaker — precision pressure.
- **Cleanup**: `HasteConfig` removed at run end. `"hazard:haste"` `EffectStack` entries cleared when the hazard deactivates.
- **Max speed bounds**: if the bolt domain clamps max speed, Haste reaches the ceiling and additional stacks have no further effect. Natural cap.
- **Multi-bolt**: reconciliation runs per Bolt entity — all bolts receive the same source-tagged multiplier.

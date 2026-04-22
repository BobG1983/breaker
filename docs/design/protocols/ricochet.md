# Protocol: Ricochet

## Category
`code-driven` (per TODO #6 — previously `effect-tree`; now dispatched from code).

## Game Design
You WANT to aim for walls, not cells.

After a wall bounce, the bolt deals multiplied damage (default 3x) until its next cell impact.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct RicochetConfig {
    pub damage_multiplier: f32,     // 3.0
}
```

Populated from `ProtocolTuning::Ricochet`. RON carries tuning only — no effect tree.

## Components
None owned by Ricochet. State (armed vs disarmed) is tracked by whether the bolt currently has a `"protocol:ricochet"`-sourced entry in its `DamageBoostStack::persistent`.

## Messages
**Reads**: `BoltImpactWall { bolt }` (bolt domain), `BoltImpactCell { bolt }` (bolt domain).
**Sends**: None. Effects applied via `commands.fire_effect(bolt, EffectType::DamageBoost(DamageBoostConfig { multiplier: damage_multiplier }), "protocol:ricochet")` on wall impact; reversed via `commands.reverse_effect(...)` on cell impact.

## Systems

### `ricochet_on_wall_impact`
- **Schedule**: `FixedUpdate`, `.after(bolt_wall_collision)`.
- **run_if**: `protocol_active(ProtocolKind::Ricochet)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BoltImpactWall`. For each impact: `commands.fire_effect(msg.bolt, EffectType::DamageBoost(DamageBoostConfig { multiplier: config.damage_multiplier }), "protocol:ricochet")`. Upsert semantics — refreshing an already-armed bolt doesn't stack multipliers.

### `ricochet_on_cell_impact`
- **Schedule**: `FixedUpdate`, `.after(bolt_cell_collision)`, `.before(DeathPipelineSystems::ApplyDamageBoosts)` — reverse must fire before the boost is applied to this same-tick damage? No — the design is "3x on the first cell hit after a wall bounce", so the boost aggregates THEN reverses:
  - If the reverse runs before ApplyDamageBoosts: the cell hit that triggered the cell impact does NOT get the boost. (Wrong.)
  - If the reverse runs after ApplyDamageBoosts: the cell hit gets the boost, then it clears for the next hit. (Correct.)
- So `ricochet_on_cell_impact` is ordered `.after(DeathPipelineSystems::ApplyDamageBoosts)`.
- **Behavior**: Reads `BoltImpactCell`. For each impact: `commands.reverse_effect(msg.bolt, ReversibleEffectType::DamageBoost(DamageBoostConfig { multiplier: config.damage_multiplier }), "protocol:ricochet")`.

### `ricochet_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: `commands.reverse_all_by_source(world, "protocol:ricochet")` to clear any armed state.

## Pipeline position (dmg crate)

- **Writes**: source-tagged `DamageBoostStack::persistent` entry on the bolt when armed; removed when disarmed.
- **Consumed in**: `DeathPipelineSystems::ApplyDamageBoosts` — aggregates `persistent` entries multiplicatively for every `DamageDealt<Cell>`.
- **Ordering on disarm**: reverse must run `.after(ApplyDamageBoosts)` so the triggering cell hit gets the boost before the stack entry is cleared.
- **No direct damage emission**.

## Cross-Domain Dependencies
- **bolt**: Reads `BoltImpactWall`, `BoltImpactCell`.
- **effect_v3**: Uses `fire_effect` / `reverse_effect`.
- **damage crate (`rantzsoft_dmg`)**: `ApplyDamageBoosts` consumes the persistent entry.

## Expected Behaviors (for test specs)

1. **3x damage on first cell hit after wall bounce** — bolt base damage 10, Ricochet active, bolt hits wall then cell: `DamageDealt<Cell>` emits 30 (aggregated with `DamageBoostStack::persistent` entry).
2. **Boost consumed after cell impact** — cell A impact gets 3x; cell B impact (no intervening wall) gets 1x.
3. **Wall bounce re-arms the cycle** — after cell hit disarms: next wall bounce re-fires the boost.
4. **Multiple wall bounces don't stack** — upsert semantics: a second wall bounce before a cell hit does NOT double the multiplier; the stack entry is replaced with the same-value source-tagged entry.
5. **Stacks multiplicatively with chip damage boosts** — existing chip `DamageBoost(1.5)` + Ricochet `DamageBoost(3.0)`: effective `10 * 1.5 * 3.0 = 45`.
6. **Each bolt tracked independently** — bolt A armed, bolt B not: A's cell hit gets 3x, B's gets 1x.
7. **Breaker bounce does not consume** — `BoltImpactBreaker` is not read; only wall bounce arms, only cell impact disarms.

## Edge Cases
- **Piercing bolt**: piercing counts as cell impact → first cell contacted consumes the boost, bolt continues through with normal damage.
- **Bolt-lost after wall bounce before cell hit**: stack entry dies with the bolt.
- **Wall bounce at node start**: Ricochet works immediately — no warmup.
- **Interaction with Deadline**: both multiply damage independently. Wall-bounced bolt during Deadline: `base * 3.0 (Ricochet) * 2.0 (Deadline) = 6x`.
- **Cell-type interactions**: the 3x modifies bolt damage, not cell vulnerability — applies equally to armored, shielded, etc.

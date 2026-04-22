# Protocol: Burnout

## Category
`custom-system`

## Game Design
You WANT to alternate frantic movement and deliberate stillness.

- Heat gauge fills while moving (4s to full), drains while stationary (2s to empty).
- Full heat: next bump deals mega-damage (default 4x) + fires a shockwave.
- Standing still for 1.5s: instant full drain + 2s breaker speed boost.
- Rhythm: move → build heat → stop → drain → speed boost → move → mega-bump.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct BurnoutConfig {
    pub fill_duration: f32,
    pub drain_duration: f32,
    pub still_threshold: f32,
    pub mega_damage_multiplier: f32,
    pub speed_boost_duration: f32,
    pub speed_boost_multiplier: f32,
}
```

Populated from `ProtocolTuning::Burnout`.

## Components
```rust
/// Breaker-side heat gauge.
#[derive(Component, Debug)]
pub(crate) struct BurnoutHeat {
    pub heat: f32,
    pub still_timer: f32,
    pub mega_bump_charged: bool,
}
```

Speed boost and mega-bump damage boost live in shared stacks:
- Speed boost: source-tagged entry in breaker's `EffectStack<SpeedBoostConfig>` (via `commands.fire_effect` / `commands.reverse_effect`, source `"protocol:burnout"`).
- Mega-bump damage: `DamageBoostStack::one_shots` entry on the bolt (Pattern B, consumed on next cell impact).

No bespoke `BurnoutSpeedBoost` or `BurnoutDamageBoost` components.

## Messages
**Reads**: `BumpPerformed` (breaker domain).
**Sends**: `ShockwaveEffect` fire via `commands.fire_effect(breaker, EffectType::Shockwave(...), "protocol:burnout")` on mega-bump. No direct `DamageDealt<Cell>` emission — mega-bump damage rides the bolt's next normal cell-impact through the `DamageBoostStack` aggregation.

## Systems

### `burnout_update_heat`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Burnout)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads breaker velocity. Moving (`velocity.length() > epsilon`): `heat += delta_secs / fill_duration`, clamp to 1.0, reset `still_timer = 0`. If `heat` reaches 1.0: `mega_bump_charged = true`. Stationary: `still_timer += delta_secs`, `heat -= delta_secs / drain_duration`, clamp to 0.0. If `still_timer >= still_threshold AND heat > 0.0`: instant drain (`heat = 0`), reset `still_timer`, `mega_bump_charged = false`, call `commands.fire_effect(breaker, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: speed_boost_multiplier }), "protocol:burnout")`.
- **Ordering**: `.after(BreakerSystems::Move)`, `.before(burnout_on_bump)`.

### `burnout_on_bump`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Burnout)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. If `mega_bump_charged`: increment the bolt's `DamageBoostStack::add_one_shot(config.mega_damage_multiplier)` (crate API, Pattern B), reset `mega_bump_charged = false` and `heat = 0`, call `commands.fire_effect(breaker, EffectType::Shockwave(...), "protocol:burnout")` for the visible shockwave.
- **Ordering**: `.after(BreakerSystems::GradeBump)`, `.before(DeathPipelineSystems::ApplyDamageBoosts)` — one-shot must be in the stack before the next damage-boost aggregation.

### `burnout_end_speed_boost`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Burnout)` + `in_state(NodeState::Playing)`.
- **Behavior**: Ticks a per-breaker timer for the active boost (inserted by `burnout_update_heat` alongside the `fire_effect` call). When expired: `commands.reverse_effect(breaker, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: speed_boost_multiplier }), "protocol:burnout")`.

### `burnout_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Removes `BurnoutHeat` from breakers. Reverses any outstanding `"protocol:burnout"` stack entries on breaker/bolts.

## Pipeline position (dmg crate)

- **Speed boost**: not in the death pipeline. Speed effect applied via breaker's `EffectStack<SpeedBoostConfig>` through `fire_effect` / `reverse_effect`. The breaker's movement system aggregates.
- **Mega-bump damage**: Pattern B via `DamageBoostStack::add_one_shot`. The `one_shots` entry is consumed in `DeathPipelineSystems::ApplyDamageBoosts` when the next `DamageDealt<Cell>` aggregates for that bolt.
- **Shockwave**: dispatched as a standard effect; the shockwave's own systems emit `DamageDealt<Cell>` in `DeathPipelineSystems::EmitDamage`.
- **No** direct `DamageDealt<T>` emission from Burnout itself.

## Cross-Domain Dependencies
- **breaker**: Reads `Velocity2D`. Breaker movement aggregates `EffectStack<SpeedBoostConfig>`.
- **bolt**: Writes `DamageBoostStack::one_shots` (via crate API).
- **effect_v3**: Uses `fire_effect` / `reverse_effect`. Shockwave effect owned by the effect domain.
- **damage crate (`rantzsoft_dmg`)**: `ApplyDamageBoosts` consumes the one-shot when the bolt's next cell impact aggregates.

## Expected Behaviors (for test specs)

1. **Heat fills while moving** — `fill_duration = 4.0`, moving at `|v| > 0` for 2.0s: `heat = 0.5`.
2. **Heat reaches full and charges mega-bump** — `heat = 0.9`, `fill_duration = 4.0`, moving for 0.5s: `heat = 1.0`, `mega_bump_charged = true`.
3. **Heat drains while stationary** — `heat = 1.0`, `drain_duration = 2.0`, stationary 1.0s: `heat = 0.5` (`mega_bump_charged` unchanged).
4. **Standing still triggers instant drain + speed-boost fire** — `heat = 0.8`, `still_threshold = 1.5`, stationary: at `still_timer = 1.5`, `heat = 0`, `mega_bump_charged = false`, `fire_effect(breaker, SpeedBoost(speed_boost_multiplier), "protocol:burnout")`.
5. **Mega-bump applies multiplied damage via stack** — `mega_bump_charged = true`, `mega_damage_multiplier = 4.0`, bolt base damage 10.0: Perfect bump pushes `DamageBoostStack::one_shots.push(4.0)`; next cell impact: `DamageDealt<Cell> { damage: 40.0 }` (emitted by bolt-cell collision as usual, multiplied by `ApplyDamageBoosts` consuming the one-shot).
6. **Speed boost reverses after duration** — after `speed_boost_duration`: `reverse_effect(breaker, SpeedBoost(speed_boost_multiplier), "protocol:burnout")` removes the stack entry.
7. **Mega-bump also fires shockwave** — `mega_bump_charged = true` + Perfect bump: `fire_effect(breaker, Shockwave(...), "protocol:burnout")` dispatched alongside the damage one-shot.

## Edge Cases
- Heat clamps at [0.0, 1.0].
- Still-threshold requires `heat > 0.0` — standing still on empty gauge does not fire speed boost.
- Still-threshold drain resets `mega_bump_charged` even if heat was full.
- Movement after still-threshold drain: `still_timer` resets on next movement frame; heat refills from 0.
- Multiple bolts: mega-bump one-shot applies to the specific bolt from the bump that consumed the charge. Other bolts unaffected.
- Speed-boost stacking: re-triggering while boost is active upserts (`fire_effect` replaces the source-tagged entry; no multiplicative stacking).
- Mega-bump consumed even if bolt is lost before hitting a cell — the one-shot disappears with the bolt entity.

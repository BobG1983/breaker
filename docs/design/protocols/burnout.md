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
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BurnoutConfig {
    pub fill_duration:               f32,
    pub drain_duration:              f32,
    pub still_threshold:             f32,
    pub full_heat_damage_multiplier: f32,
    pub speed_boost_duration:        f32,
    pub shockwave_base_range:        f32,
    pub shockwave_range_per_level:   f32,
    pub shockwave_stacks:            u32,
    pub shockwave_speed:             f32,
}
```

Populated from `ProtocolTuning::Burnout`. All shockwave parameters live in `BurnoutConfig` — moved here from hardcoded constants during the RON tuning sweep.

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

Speed boost lives in a shared stack:
- Speed boost: source-tagged entry in breaker's `EffectStack<SpeedBoostConfig>` (via `commands.fire_effect` / `commands.reverse_effect`, source `"protocol:burnout"`).

Mega-bump damage uses a bespoke per-bolt component:
- `BurnoutDamageBoost { multiplier: f32 }` — inserted on the bolt by `burnout_on_bump` on a mega-bump consume; consumed by `burnout_amplify_damage` on the bolt's next cell impact. This is a component (not a `DamageBoostStack` one-shot) so the consuming system can query specifically for it.

```rust
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct BurnoutDamageBoost {
    pub multiplier: f32,
}
```

## Messages
**Reads**: `BumpPerformed` (breaker domain).
**Sends**: Shockwave dispatch via `commands.fire_effect(breaker, EffectType::Shockwave(ShockwaveConfig { .. }), "protocol:burnout:shockwave")` on mega-bump. No direct `DamageDealt<Cell>` emission — mega-bump damage is handled by inserting `BurnoutDamageBoost` on the bolt, consumed by `burnout_amplify_damage` on the next cell impact.

## Systems

### `burnout_update_heat`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Burnout)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads breaker velocity. Moving (`velocity.length() > epsilon`): `heat += delta_secs / fill_duration`, clamp to 1.0, reset `still_timer = 0`. If `heat` reaches 1.0: `mega_bump_charged = true`. Stationary: `still_timer += delta_secs`, `heat -= delta_secs / drain_duration`, clamp to 0.0. If `still_timer >= still_threshold AND heat > 0.0`: instant drain (`heat = 0`), reset `still_timer`, `mega_bump_charged = false`, fire speed boost via `commands.fire_effect(breaker, EffectType::SpeedBoost(..), "protocol:burnout")`.
- **Ordering**: `.after(BreakerSystems::Move)`, `.before(burnout_on_bump)`.

### `burnout_on_bump`
- **Schedule**: `FixedUpdate`.
- **run_if**: in-body gate — drains `MessageReader` and returns when Burnout is inactive or `NodeState != Playing`.
- **Behavior**: Reads `BumpPerformed`. If `mega_bump_charged`: inserts `BurnoutDamageBoost { multiplier: config.full_heat_damage_multiplier }` on the bolt, resets breaker's `BurnoutHeat` (`heat = 0.0`, `still_timer = 0.0`, `mega_bump_charged = false`), fires shockwave via `commands.fire_effect(breaker, EffectType::Shockwave(ShockwaveConfig { base_range, range_per_level, stacks, speed }), "protocol:burnout:shockwave")`.
- **Ordering**: `.after(BreakerSystems::GradeBump)` — must read the bump grade before clearing charge.

### `burnout_tick_speed_boost`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Burnout)` + `in_state(NodeState::Playing)`.
- **Behavior**: Ticks a per-breaker timer for the active speed boost (inserted by `burnout_update_heat` alongside the `fire_effect` call). When expired: reverses the boost via `commands.reverse_effect(breaker, .., "protocol:burnout")`.

### `burnout_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Removes `BurnoutHeat` from breakers. Reverses any outstanding `"protocol:burnout"` stack entries on breaker/bolts.

## Pipeline position (dmg crate)

- **Speed boost**: not in the death pipeline. Speed effect applied via breaker's `EffectStack<SpeedBoostConfig>` through `fire_effect` / `reverse_effect`. The breaker's movement system aggregates.
- **Mega-bump damage**: `BurnoutDamageBoost` component on the bolt — consumed by `burnout_amplify_damage` (in `DmgSystems::MutateDamage`) when the next `DamageDealt<Cell>` arrives for that bolt.
- **Shockwave**: dispatched as a standard effect via `commands.fire_effect`; the shockwave's own systems emit `DamageDealt<Cell>` in `DmgSystems::EmitDamage`.
- **No** direct `DamageDealt<T>` emission from Burnout itself (shockwave emission is via effect_v3).

## Cross-Domain Dependencies
- **breaker**: Reads `Velocity2D`. Breaker movement aggregates `EffectStack<SpeedBoostConfig>`.
- **bolt**: Inserts `BurnoutDamageBoost` component on the bolt at mega-bump time.
- **effect_v3**: Uses `fire_effect` / `reverse_effect`. Shockwave effect owned by the effect domain.
- **damage crate (`rantzsoft_dmg`)**: `burnout_amplify_damage` runs in `DmgSystems::MutateDamage` and multiplies the bolt's next `DamageDealt<Cell>` by `BurnoutDamageBoost.multiplier`.

## Expected Behaviors (for test specs)

1. **Heat fills while moving** — `fill_duration = 4.0`, moving at `|v| > 0` for 2.0s: `heat = 0.5`.
2. **Heat reaches full and charges mega-bump** — `heat = 0.9`, `fill_duration = 4.0`, moving for 0.5s: `heat = 1.0`, `mega_bump_charged = true`.
3. **Heat drains while stationary** — `heat = 1.0`, `drain_duration = 2.0`, stationary 1.0s: `heat = 0.5` (`mega_bump_charged` unchanged).
4. **Standing still triggers instant drain + speed-boost fire** — `heat = 0.8`, `still_threshold = 1.5`, stationary: at `still_timer = 1.5`, `heat = 0`, `mega_bump_charged = false`, `fire_effect(breaker, SpeedBoost(..), "protocol:burnout")` emitted.
5. **Mega-bump applies multiplied damage via component** — `mega_bump_charged = true`, `full_heat_damage_multiplier = 4.0`, bolt base damage 10.0: bump inserts `BurnoutDamageBoost { multiplier: 4.0 }` on the bolt; `burnout_amplify_damage` (in `DmgSystems::MutateDamage`) multiplies the next `DamageDealt<Cell>` for that bolt by 4.0, yielding effective damage 40.0.
6. **Speed boost reverses after duration** — after `speed_boost_duration`: `reverse_effect(breaker, SpeedBoost(..), "protocol:burnout")` removes the stack entry.
7. **Mega-bump also fires shockwave** — `mega_bump_charged = true` + Perfect bump: `fire_effect(breaker, Shockwave(ShockwaveConfig { .. }), "protocol:burnout:shockwave")` dispatched alongside the `BurnoutDamageBoost` component insert.

## Edge Cases
- Heat clamps at [0.0, 1.0].
- Still-threshold requires `heat > 0.0` — standing still on empty gauge does not fire speed boost.
- Still-threshold drain resets `mega_bump_charged` even if heat was full.
- Movement after still-threshold drain: `still_timer` resets on next movement frame; heat refills from 0.
- Multiple bolts: mega-bump one-shot applies to the specific bolt from the bump that consumed the charge. Other bolts unaffected.
- Speed-boost stacking: re-triggering while boost is active upserts (`fire_effect` replaces the source-tagged entry; no multiplicative stacking).
- Mega-bump consumed even if bolt is lost before hitting a cell — `BurnoutDamageBoost` disappears with the bolt entity.

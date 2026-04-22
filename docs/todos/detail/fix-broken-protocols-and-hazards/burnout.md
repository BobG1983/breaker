# Burnout

Assumes: #1 (`rantzsoft_dmg`), #2 (`mutators/protocols/burnout/`), #7 (stamp dispatcher fixes protocol site). Pattern B (consume-on-use DamageBoost) is implemented via `DamageBoostStack::one_shots: Vec<f32>` in the crate — NOT via `EffectStack<DamageBoostConfig>` (that type no longer exists post-#1).

## What's broken

1. **Speed-boost half of the still-threshold trigger is inert.** `BurnoutSpeedBoost` is inserted on the breaker but NO movement system reads it. Design behavior 4 (speed boost while overheated) only half-lands — the damage side works, the movement side does nothing.
2. **`BurnoutSpeedBoost` uses a bespoke component + replace-on-insert for "reset on re-trigger."** Fragile, undocumented pattern. A second `insert(BurnoutSpeedBoost { multiplier: 1.5 })` only happens to replace the first because Bevy's insert semantics do that by default — it's not load-bearing intent.
3. **No end-to-end test covers mega-bump damage.** Pattern B (base damage + consume-on-use amplified bonus) is wired through the damage crate, but no integration test spins up a Burnout scenario with a known-HP cell and known-damage bolt and pins the final `Hp.current` value.

## Fix

### Delete `BurnoutSpeedBoost`

`mutators/protocols/burnout/components.rs` (or wherever the marker lives): delete the component type, its derives, and the `insert(BurnoutSpeedBoost { ... })` callsite. No movement system reads it today — the insertion has no consumer.

### Route speed boost through `EffectStack<SpeedBoostConfig>`

Replace the insertion site with:

```rust
commands.fire_effect(
    breaker_entity,
    EffectType::SpeedBoost(SpeedBoostConfig { multiplier: config.speed_multiplier }),
    SourceId::from("protocol:burnout"),
);
```

`EffectStack<SpeedBoostConfig>` already lives on the breaker and is already aggregated by `move_breaker` via `BreakerMovementData::speed_boosts` (existing pipeline — unchanged). `fire_effect` with source `"protocol:burnout"` upserts: a second fire replaces the prior entry with the new multiplier. No explicit "reset on re-trigger" branch.

### Route mega-bump damage amplification through `DamageBoostStack::one_shots`

Pattern B lives in the damage crate post-#1. `EffectStack<DamageBoostConfig>` is GONE — the crate's `DamageBoostStack { persistent, one_shots }` replaces it.

On mega-bump trigger (whatever condition currently fires `burnout_amplify_damage`), instead of emitting a direct `DamageDealt<Cell>`:

```rust
pub(crate) fn burnout_arm_mega_bump_on_bolt(
    mut query: Query<&mut DamageBoostStack, With<Bolt>>,
    config: Res<BurnoutConfig>,
    // ...trigger source (e.g., MessageReader<BumpPerformed> filtered to Perfect + overheated)...
) {
    for mut stack in &mut query {
        stack.add_one_shot(config.mega_damage_multiplier);
    }
}
```

The crate's `apply_damage_boosts::<Cell>` system aggregates `persistent.values().product()` * `one_shots.iter().product()` on the next `DamageDealt<Cell>` for that bolt, then clears `one_shots`. No source tag on one-shots (they're positionally consumed). Delete any direct `DamageDealt<Cell>` emissions from `burnout_amplify_damage` that exist purely to add the amplified bonus — base damage rides through `bolt_cell_collision` (which emits the crate's `DamageDealt<Cell>`), the amplified bonus rides through the stack aggregation. ONE damage event, multiplier applied once.

### End-of-boost reversal

For the speed boost, on the heat-drained/timer-expired trigger:

```rust
commands.reverse_effect(
    breaker_entity,
    ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: config.speed_multiplier }),
    SourceId::from("protocol:burnout"),
);
```

Multiplier must match the fired entry's multiplier for the source-tagged reverse dispatch to land on the right stack entry.

For the damage one-shot, no reversal is needed — it self-consumes on first cell impact via the stack's aggregate-and-clear behavior.

### Schedule

All Burnout systems run in `FixedUpdate`, `run_if = protocol_active(Burnout) + in_state(NodeState::Playing)`. `burnout_arm_mega_bump_on_bolt` runs `.before(DeathPipelineSystems::ApplyDamageBoosts)` so the one-shot is in the stack before the next damage aggregation tick.

## Tests

`mutators/protocols/burnout/tests/speed_boost.rs`:

1. **`still_threshold_fires_speed_boost_entry`** — dwell breaker under `still_threshold` for `still_for_secs`; tick; assert `EffectStack<SpeedBoostConfig>` on breaker contains exactly one `"protocol:burnout"` entry with multiplier `= config.speed_multiplier`.
2. **`move_breaker_applies_burnout_multiplier`** — same setup; tick; assert breaker velocity scaled by the multiplier.
3. **`end_trigger_reverses_entry`** — drain heat; tick; assert stack has no `"protocol:burnout"` entry; velocity returns to baseline.
4. **`refire_upserts_not_stacks`** — fire with multiplier 1.5, then fire again with 2.0; assert stack has exactly ONE `"protocol:burnout"` entry with multiplier 2.0.

`mutators/protocols/burnout/tests/mega_bump_damage.rs`:

5. **`mega_bump_applies_base_plus_amplified_damage`** — headless app with the damage crate + Burnout. `BurnoutConfig { speed_multiplier, mega_damage_multiplier: 2.0, .. }`. `BoltBaseDamage(10.0)`. `Hp { current: 100.0, max: 100.0, .. }` on cell. Arm Burnout (dwell past threshold); fire mega-bump trigger; drive bolt to impact cell; tick one `FixedUpdate`. Assert `cell.Hp.current == 100.0 - (10.0 * 2.0) == 80.0`. Assert the bolt's `DamageBoostStack.one_shots` is empty after the impact.
6. **`non_mega_bump_applies_base_only`** — same setup but do NOT arm Burnout. Fire non-Perfect bump. Assert `cell.Hp.current == 100.0 - 10.0 == 90.0`. Asserts `burnout_arm_mega_bump_on_bolt` is gated correctly.
7. **`multi_cell_shockwave_damage`** — if Burnout's shockwave targets multiple cells (verify in impl), spawn 3 cells in range; fire mega-bump; assert all 3 take the expected damage. Skip if Burnout's shockwave is single-target.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/burnout/components.rs` | DELETE `BurnoutSpeedBoost` component + all imports |
| `mutators/protocols/burnout/system.rs` | Replace `insert(BurnoutSpeedBoost { .. })` with `commands.fire_effect(...)` call; replace mega-bump direct damage emission with `stack.add_one_shot(...)`; add end-of-boost `reverse_effect(...)` |
| `mutators/protocols/burnout/register.rs` | Register `burnout_arm_mega_bump_on_bolt` with `.before(DeathPipelineSystems::ApplyDamageBoosts)` |
| `mutators/protocols/burnout/tests/speed_boost.rs` | NEW — tests 1-4 |
| `mutators/protocols/burnout/tests/mega_bump_damage.rs` | NEW — tests 5-7 |

## Out of scope

- File split (separate cleanup)
- Shockwave RON tuning (→ `ron-tuning-values.md`)
- Design doc updates for BurnoutSpeedBoost deletion (ride with the code change as a short inline doc update — not a separate sweep)

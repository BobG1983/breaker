# Erosion

Assumes: #1 (the crate keeps `EffectStack<SizeBoostConfig>` in effect_v3 — size-boost is NOT damage-related, so it does NOT migrate to the crate's stacks). Erosion lives at `mutators/hazards/erosion/` post-#2.

## What's broken

Visual and collision disagree on what Erosion shrinks:

- **Visual** — `sync_breaker_scale` (or equivalent) multiplies BOTH `Scale2D.x` and `Scale2D.y` by the `EffectStack<SizeBoostConfig>` aggregate. Breaker sprite shrinks on both axes.
- **Collision** — `bolt_breaker_collision/system.rs:294-297` applies the aggregate ONLY to `half_w`. Breaker's collision/catch zone shrinks horizontally only.

User decision: Erosion is WIDTH-ONLY. Visual must match collision. Breaker height stays at its base dimension on both systems.

## Fix

### Visual: stop shrinking height

`breaker/systems/sync_breaker_scale/system.rs`:

```rust
// before
let size_mult = size_boosts.aggregate();
scale.x = base_scale.x * size_mult;
scale.y = base_scale.y * size_mult;   // WRONG

// after
let size_mult = size_boosts.aggregate();
scale.x = base_scale.x * size_mult;
scale.y = base_scale.y;               // Height is not affected by SizeBoost.
```

### Collision: unchanged

`bolt_breaker_collision/system.rs:294-297` already applies the aggregate to `half_w` only. Keep it as-is. After this fix, both the visual and collision treat `SizeBoostConfig` as a width-only modifier — consistent.

### Design-doc update

`docs/design/hazards/erosion.md`:

Replace any "bump window height shrinks proportionally" or "shrinks the breaker" language with:

> Erosion shrinks only the breaker's WIDTH. Height stays at its base value. Both the visual (breaker sprite) and the collision box (catch zone) shrink horizontally; neither shrinks vertically. The `EffectStack<SizeBoostConfig>` aggregate is applied to the X dimension only.

Add to §Components or §Architecture Notes:

> `SizeBoostConfig` is currently width-only by convention. If a future hazard needs height modulation, split the config into `width_fraction` + `height_fraction` fields. Until then, SizeBoost is effectively a width-only knob; systems that read it apply it to `scale.x` / `half_w` and leave `scale.y` / `half_h` at baseline.

## Tests

`breaker/systems/sync_breaker_scale/tests/size_boost_width_only.rs`:

1. **`size_boost_scales_x_only`** — spawn breaker with baseline `Scale2D { x: 100.0, y: 20.0 }`; insert `EffectStack<SizeBoostConfig>` containing an entry with fraction `0.5`; tick; assert `Scale2D.x == 50.0`, `Scale2D.y == 20.0` (unchanged).
2. **`no_size_boost_leaves_baseline`** — no `EffectStack<SizeBoostConfig>` entries; tick; assert `Scale2D` equals baseline on both axes.
3. **`multiple_size_boosts_aggregate_on_x_only`** — stack contains two entries (e.g., `0.8` and `0.75`); tick; assert `Scale2D.x == base.x * 0.8 * 0.75`, `Scale2D.y == base.y`.

`bolt/systems/bolt_breaker_collision/tests/size_boost_affects_half_w_only.rs`:

4. **`collision_uses_base_half_height`** — breaker with baseline `Scale2D { x: 100.0, y: 20.0 }` + `SizeBoostConfig` aggregate `0.5`. Fire bolt at the position that would hit if `half_h` were scaled (`y_breaker + 5.0 * sign`) but miss if `half_h` is at base (`y_breaker + 10.0 * sign`). Assert NO collision (because `half_h` stays at base = 10, not 5).
5. **`collision_uses_scaled_half_width`** — same setup; fire bolt at horizontal position that would hit if `half_w` were at base (50) but miss at half (25). Assert NO collision (because `half_w` shrinks to 25).

### Tests to UPDATE

Any existing `sync_breaker_scale` test asserting `Scale2D.y` is scaled by SizeBoost — flip to assert height stays at base.

Any existing Erosion test asserting the breaker's height shrinks — flip the assertion.

## Code changes summary

| File | Change |
|------|--------|
| `breaker/systems/sync_breaker_scale/system.rs` | `scale.y = base_scale.y * size_mult` → `scale.y = base_scale.y` |
| `breaker/systems/sync_breaker_scale/tests/size_boost_width_only.rs` | NEW — tests 1-3 |
| `bolt/systems/bolt_breaker_collision/tests/size_boost_affects_half_w_only.rs` | NEW — tests 4-5 |
| `docs/design/hazards/erosion.md` | Design-doc updates per Fix section |

## Out of scope

- Splitting `SizeBoostConfig` into separate `width_fraction` / `height_fraction` fields — no caller needs it today. Noted in the design doc as the future path if a height-shrink hazard ever exists.
- Erosion RON tuning — `ron-tuning-values.md`.
- Auditing other `SizeBoostConfig` consumers for consistency — grep during impl; if any other system applies it to Y, that's a bug to flag separately.

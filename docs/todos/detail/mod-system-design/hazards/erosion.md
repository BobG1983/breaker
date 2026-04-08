# Hazard: Erosion

## Game Design

Breaker shrinks over time. Non-whiff bumps restore 25% of what was lost. Perfect bumps restore 50% of what was lost. Minimum width 35%. Also reduces bump window height proportionally. Shrink rate TBD (number that divides neatly into 100%/second). The player must keep bumping to maintain breaker size — idle time causes the breaker to melt. This creates a "stay active" pressure that punishes passive play and compounds with Haste (faster bolt + smaller target).

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct ErosionConfig {
    /// Rate at which the breaker shrinks, in fraction-of-full-width per second.
    /// E.g., 0.05 = 5% of full width lost per second.
    pub shrink_rate: f32,
    /// Minimum breaker width as a fraction of full width (0.35 = 35%).
    pub min_width_fraction: f32,
    /// Fraction of lost width restored on a non-whiff (Early/Late) bump.
    pub non_whiff_restore: f32,
    /// Fraction of lost width restored on a Perfect bump.
    pub perfect_restore: f32,
}
```

Extracted from `HazardTuning::Erosion { shrink_rate, min_width_fraction, non_whiff_restore, perfect_restore }` at activation time.

## Components

### `ErosionState`

```rust
/// Tracks the current erosion state for the breaker.
#[derive(Resource, Debug)]
pub(crate) struct ErosionState {
    /// Current breaker width as a fraction of full width (1.0 = full, 0.35 = minimum).
    pub width_fraction: f32,
}
```

Inserted alongside `ErosionConfig` at activation time, initialized to `1.0` (full width).

## Messages

**Reads**: `BumpPerformed` — existing message from the bolt/breaker domain. Contains bump grade (`Perfect`, `Early`, `Late`, `Whiff`). Used to determine restoration amount.
**Sends**: `ApplyBreakerShrink { amount: f32 }` — new message, owned by `breaker` domain. Positive values shrink, negative values grow (restore). The breaker domain applies the width change.

## Systems

### `erosion_shrink`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Erosion).and(in_state(NodeState::Playing))`
- **Ordering**: Before `erosion_restore` (shrink first, then restore from bumps in the same frame)
- **Behavior**: Each tick, compute shrink amount from `shrink_rate * stack * delta_secs`. Reduce `ErosionState.width_fraction` (clamped to `min_width_fraction`). Send `ApplyBreakerShrink { amount }` with the actual shrink applied (may be less than computed if at minimum).
- **Formula**: `shrink_per_tick = shrink_rate * stack * delta_secs`

### `erosion_restore`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Erosion).and(in_state(NodeState::Playing))`
- **Ordering**: After `erosion_shrink`
- **Behavior**: Reads `BumpPerformed` messages. For each bump:
  - `Perfect` grade: restore `perfect_restore` fraction of lost width
  - `Early` or `Late` grade: restore `non_whiff_restore` fraction of lost width
  - `Whiff` grade: no restoration
  - Lost width = `1.0 - width_fraction`. Restore amount = `lost * restore_fraction`.
  - Update `ErosionState.width_fraction` (clamped to 1.0).
  - Send `ApplyBreakerShrink` with negative amount (growth).

### Bump window height adjustment

The design specifies that bump window height shrinks proportionally with breaker width. This is communicated through the same `ApplyBreakerShrink` message — the breaker domain is responsible for scaling both the visual width AND the bump window height proportionally.

## Stacking Behavior

Linear shrink rate scaling: `effective_shrink_rate = shrink_rate * stack`

| Stack | Shrink rate multiplier | Notes |
|-------|----------------------|-------|
| 1 | 1x | Gradual shrink, easy to maintain with regular bumps |
| 2 | 2x | Noticeable erosion, must bump frequently to stay above minimum |
| 3 | 3x | Aggressive shrink, near-constant bumping required |

The restoration percentages (25% non-whiff, 50% perfect) do NOT change with stacking. Only the shrink rate increases. This means at higher stacks, restoration from bumps can't keep up — the breaker trends toward minimum width.

The minimum width (35%) is fixed regardless of stacks. The floor prevents the hazard from making the game unplayable — the breaker is always at least 35% of its full width.

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `breaker` | sends to | `ApplyBreakerShrink` — modifies breaker width (and proportionally, bump window height) |
| `bolt`/`breaker` | reads from | `BumpPerformed` — determines restoration amount based on bump grade |

Erosion never reads or writes breaker components directly. The breaker domain owns width and applies the changes.

## Expected Behaviors (for test specs)

1. **Breaker shrinks over time at stack 1**
   - Given: Erosion active at stack 1, `shrink_rate=0.05`, `width_fraction=1.0`, `delta_secs=1.0`
   - When: `erosion_shrink` runs
   - Then: `width_fraction` becomes 0.95, `ApplyBreakerShrink { amount: 0.05 }` sent

2. **Breaker shrinks faster at stack 3**
   - Given: Erosion active at stack 3, `shrink_rate=0.05`, `width_fraction=1.0`, `delta_secs=1.0`
   - When: `erosion_shrink` runs
   - Then: `width_fraction` becomes 0.85, `ApplyBreakerShrink { amount: 0.15 }` sent (0.05 * 3)

3. **Breaker does not shrink below minimum**
   - Given: Erosion active at stack 1, `min_width_fraction=0.35`, `width_fraction=0.36`, shrink would take it to 0.31
   - When: `erosion_shrink` runs
   - Then: `width_fraction` clamped to 0.35, only the actual delta sent as `ApplyBreakerShrink`

4. **Perfect bump restores 50% of lost width**
   - Given: Erosion active, `width_fraction=0.60`, `perfect_restore=0.50`
   - When: `BumpPerformed` with `Perfect` grade received
   - Then: Lost = 0.40, restore = 0.20, `width_fraction` becomes 0.80, `ApplyBreakerShrink { amount: -0.20 }` sent

5. **Non-whiff bump restores 25% of lost width**
   - Given: Erosion active, `width_fraction=0.60`, `non_whiff_restore=0.25`
   - When: `BumpPerformed` with `Early` grade received
   - Then: Lost = 0.40, restore = 0.10, `width_fraction` becomes 0.70, `ApplyBreakerShrink { amount: -0.10 }` sent

6. **Whiff bump restores nothing**
   - Given: Erosion active, `width_fraction=0.60`
   - When: `BumpPerformed` with `Whiff` grade received
   - Then: `width_fraction` unchanged at 0.60, no restore message sent

## Edge Cases

- **Erosion + Haste synergy**: Faster bolt + smaller breaker = dramatically harder catches. At Haste stack 2 + Erosion stack 2, the bolt is 30% faster and the breaker is aggressively shrinking — the player must be both fast and precise.
- **Erosion + Overcharge synergy**: Overcharge speeds up the bolt per kill within a bump cycle. Combined with a shrinking breaker, catching the accelerating bolt becomes a precision challenge.
- **Node-end cleanup**: `ErosionConfig` and `ErosionState` resources removed at run end. Breaker width resets naturally when the breaker entity is recreated for the next run.
- **Restoration above full width**: `width_fraction` is clamped to 1.0. Perfect bumps at near-full width restore very little (since lost width is small).
- **Multiple bumps per frame**: Each bump processes independently. Two Perfect bumps in the same frame each restore 50% of the lost width at the time they're processed (the second bump sees less lost width).
- **At minimum width**: When `width_fraction == min_width_fraction`, shrinking stops (no negative message sent). Restoration still works — a bump can bring the width back up, but it will immediately start eroding again.

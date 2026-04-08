# Hazard: Haste

## Game Design

Bolt speed increase (multiplicative with existing speed). 20%+10%/level. The bolt moves faster, giving the player less reaction time. This is a universal pressure multiplier — every other mechanic becomes harder when the bolt is faster. Stacking makes the bolt visibly zip around the arena.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct HasteConfig {
    /// Base speed multiplier percentage at stack 1.
    pub base_percent: f32,
    /// Additional speed multiplier percentage per stack beyond the first.
    pub per_level_percent: f32,
}
```

Extracted from `HazardTuning::Haste { base_percent, per_level_percent }` at activation time.

## Components

None. Haste applies a persistent speed multiplier to all bolts. No per-entity tracking needed — the multiplier is computed from config + stack count each time.

## Messages

**Reads**: `ActiveHazards` for stack count
**Sends**: Implementation depends on the effect system refactor (todo #2):
- **Preferred**: Use the new effect system's `During(HazardActive(HazardKind::Haste), Fire(SpeedBoost(multiplier)))` — an ambient effect not tied to a trigger. This lets the effect system manage the speed modification lifecycle.
- **Fallback**: `ApplyBoltSpeedMultiplier { multiplier: f32 }` — new message, owned by `bolt` domain. Sent each tick or once on activation/stack change.

The interface design notes that Haste (and Overcharge) "may be expressible through the effect system's `SpeedBoost` rather than new messages. This depends on whether the effect system supports 'ambient' effects not tied to a trigger." The final approach depends on the effect refactor outcome.

## Systems

### `haste_apply_speed` (fallback approach — if not using effect system)

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Haste).and(in_state(NodeState::Playing))`
- **Ordering**: Before bolt physics integration, after any base speed computation
- **Behavior**: Compute the speed multiplier from config + stack count. Apply as a multiplicative modifier to bolt speed. The exact mechanism depends on how the bolt domain exposes speed modification — either via message or via the effect system's `SpeedBoost`.
- **Formula**: `multiplier = 1.0 + (base_percent + per_level_percent * (stack - 1)) / 100.0`

Note: If using the effect system, this system may not exist as a standalone system. Instead, the speed boost would be installed as an effect at hazard activation time and updated when the stack count changes.

### `haste_update_on_stack_change` (if using effect system)

- **Schedule**: Runs when `HazardSelected { kind: Haste }` is received
- **Behavior**: Recalculate the speed multiplier for the new stack count and update the ambient `SpeedBoost` effect.

## Stacking Behavior

Linear percentage scaling, applied multiplicatively to existing bolt speed: `multiplier = 1.0 + (base_percent + per_level_percent * (stack - 1)) / 100.0`

| Stack | Percentage | Speed multiplier |
|-------|-----------|-----------------|
| 1 | +20% | 1.20x |
| 2 | +30% | 1.30x |
| 3 | +40% | 1.40x |

Each additional stack adds 10% to the speed boost. At stack 5, the bolt moves at 1.60x speed. This is multiplicative with OTHER speed modifiers (chips, Overcharge hazard, protocol effects).

## Cross-Domain Dependencies

| Domain | Direction | Message/Effect |
|--------|-----------|---------------|
| `bolt` | sends to | `SpeedBoost` effect or `ApplyBoltSpeedMultiplier` — increases bolt speed |

Haste never reads or writes bolt velocity directly. The bolt domain (or effect system) owns speed modification.

## Expected Behaviors (for test specs)

1. **Bolt speed increased at stack 1**
   - Given: Haste active at stack 1, `base_percent=20.0`, `per_level_percent=10.0`, bolt base speed 400.0
   - When: Haste speed modifier is applied
   - Then: Bolt effective speed is 480.0 (400.0 * 1.20)

2. **Bolt speed increased at stack 3**
   - Given: Haste active at stack 3, same config, bolt base speed 400.0
   - When: Haste speed modifier is applied
   - Then: Bolt effective speed is 560.0 (400.0 * 1.40)

3. **Multiplicative with other speed modifiers**
   - Given: Haste active at stack 1 (1.20x), chip speed boost active (1.50x), bolt base speed 400.0
   - When: All speed modifiers are applied
   - Then: Bolt effective speed is 720.0 (400.0 * 1.20 * 1.50) — modifiers multiply, not add

4. **No speed change when Haste is inactive**
   - Given: Haste not in `ActiveHazards`, bolt base speed 400.0
   - When: Bolt speed is computed
   - Then: Bolt effective speed is 400.0

5. **Speed modifier updates immediately on new stack**
   - Given: Haste active at stack 1 (1.20x), player selects Haste again
   - When: Stack count becomes 2
   - Then: Bolt effective speed updates to reflect 1.30x multiplier

## Edge Cases

- **Haste + Overcharge synergy**: Both increase bolt speed multiplicatively. Overcharge adds per-kill speed within a bump cycle. With Haste stack 2 (1.30x) and Overcharge adding 15% per kill, after 3 kills the bolt is at 1.30 * 1.15^3 = ~1.98x speed. This compounds dangerously.
- **Haste + Erosion synergy**: Faster bolt + smaller breaker = much harder to catch. The player must be precise with a narrower target.
- **Node-end cleanup**: `HasteConfig` resource removed at run end. If using effect system, the speed boost effect is cleaned up via `SourceId` removal.
- **Max speed bounds**: Bolt speed clamping (if it exists) still applies. Haste can push speed to the clamp ceiling, at which point additional stacks have no further effect. This is a natural cap.
- **Effect system dependency**: The preferred implementation path depends on the effect refactor (todo #2). The fallback message approach works without it but is less clean.

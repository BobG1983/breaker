# Hazard: Overcharge

## Game Design

Bolt gains speed per cell destroyed within a bump cycle, resets on bump. "Sounds like a buff. Isn't." At high kill counts within a single bump cycle, the bolt becomes nearly uncontrollable. The player must decide whether to aim for safe bumps (low kill count) or risk high-speed returns for efficiency.

A "bump cycle" is the period between consecutive breaker bumps -- from the moment the bolt leaves the breaker until it returns and bumps again.

**Stacking formula**: `5% + 3% * (stack - 1)` per kill, multiplicative with existing speed. Each kill within the cycle multiplies the bolt's current speed by `(1.0 + speed_per_kill / 100.0)`.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct OverchargeConfig {
    pub base_speed_per_kill: f32,  // 5.0 (percent)
    pub speed_per_level: f32,      // 3.0 (percent per additional stack)
}
```

## Components

```rust
/// Tracks kills within the current bump cycle for Overcharge speed scaling.
/// Attached to bolt entities when Overcharge is active.
#[derive(Component, Debug, Default)]
pub(crate) struct OverchargeKillCount(pub u32);
```

## Messages

**Reads**: `CellDestroyedAt` (to count kills), `BumpPerformed` (to reset kill count)
**Sends**: Speed modification -- either via effect system `SpeedBoost` or `ApplyBoltSpeedMultiplier` (implementation depends on whether the effect system supports ambient/persistent speed modifiers)

**Speed application**: Two possible patterns:
1. **Effect system `SpeedBoost`**: If the new effect system supports `During(HazardActive, ...)` scoping, Overcharge can fire a `SpeedBoost` effect per kill. This integrates cleanly with other speed effects (Haste hazard, chips).
2. **Direct `ApplyBoltSpeedMultiplier` message**: If the effect system doesn't support this pattern, send a message to the bolt domain. The bolt domain applies the multiplier to velocity magnitude.

Prefer option 1 if feasible after the effect refactor. Fall back to option 2.

## Systems

1. **`attach_overcharge_tracker`**
   - Schedule: When bolts are spawned
   - Run if: `hazard_active(HazardKind::Overcharge)`
   - Behavior: Insert `OverchargeKillCount(0)` on all bolt entities

2. **`overcharge_count_kills`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Overcharge)` AND `in_state(NodeState::Playing)`
   - Ordering: After cell death processing
   - Behavior:
     1. Read `CellDestroyedAt` messages
     2. For each kill, determine which bolt caused it (from the damage source tracking)
     3. Increment that bolt's `OverchargeKillCount`
     4. Compute `speed_per_kill = base_speed_per_kill + speed_per_level * (stack - 1)`
     5. Apply speed multiplier: `(1.0 + speed_per_kill / 100.0)` to the bolt's current speed
     6. Send speed modification (via effect system or message)

3. **`overcharge_reset_on_bump`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Overcharge)` AND `in_state(NodeState::Playing)`
   - Ordering: After bump processing
   - Behavior:
     1. Read `BumpPerformed` messages
     2. For each bumped bolt, reset `OverchargeKillCount` to 0
     3. Remove accumulated speed bonus (reset bolt speed to pre-Overcharge base)

## Stacking Behavior

| Stack | Speed per kill | After 3 kills | After 5 kills | After 10 kills |
|-------|---------------|---------------|---------------|----------------|
| 1     | 5%            | 1.16x         | 1.28x         | 1.63x          |
| 2     | 8%            | 1.26x         | 1.47x         | 2.16x          |
| 3     | 11%           | 1.37x         | 1.69x         | 2.84x          |

Speed is multiplicative per kill: `speed = base_speed * (1 + pct)^kills`. This compounds rapidly. At stack=3 with 10 kills in one cycle, the bolt is nearly 3x speed. Combined with Haste (which also increases speed), this becomes extreme.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell destruction events | `CellDestroyedAt` message (read) |
| `bolt`  | Modifies bolt speed | Effect system `SpeedBoost` or `ApplyBoltSpeedMultiplier` |
| `breaker` | Reads bump events for reset | `BumpPerformed` message (read) |

**Source tracking**: `CellDestroyedAt` (or `DamageDealt<Cell>`) must carry enough information to identify which bolt caused the kill. This is needed to attribute kills to specific bolts when multiple bolts are in play (from Fission protocol or multi-bolt chips).

## Expected Behaviors (for test specs)

1. **Bolt gains speed on first kill at stack=1**
   - Given: Bolt at base speed 400.0, `OverchargeKillCount(0)`, stack=1
   - When: Bolt destroys a cell
   - Then: Speed becomes 420.0 (400 * 1.05), `OverchargeKillCount(1)`

2. **Speed compounds multiplicatively on consecutive kills**
   - Given: Bolt at speed 420.0, `OverchargeKillCount(1)`, stack=1
   - When: Bolt destroys another cell
   - Then: Speed becomes 441.0 (420 * 1.05), `OverchargeKillCount(2)`

3. **Kill count resets on bump**
   - Given: Bolt with `OverchargeKillCount(5)`, accumulated speed bonus
   - When: `BumpPerformed` message for this bolt
   - Then: `OverchargeKillCount(0)`, bolt speed returns to base (pre-Overcharge)

4. **Speed per kill scales with stack count at stack=3**
   - Given: Bolt at base speed 400.0, stack=3 (speed_per_kill=11%)
   - When: Bolt destroys a cell
   - Then: Speed becomes 444.0 (400 * 1.11)

5. **System does not run when hazard is inactive**
   - Given: Overcharge not in `ActiveHazards`
   - When: Cell is destroyed
   - Then: No `OverchargeKillCount` components exist, no speed modification

## Edge Cases

- **Overcharge + Haste synergy**: Both increase bolt speed multiplicatively. Haste is a constant multiplier; Overcharge compounds per kill. At stack=2 of both, a 10-kill cycle produces `base * 1.2 (Haste) * 2.16 (Overcharge) = 2.59x`. The bolt domain should have a maximum speed cap to prevent physics breakage.
- **Multi-bolt scenarios**: Each bolt has its own `OverchargeKillCount`. Kills are attributed per-bolt. If bolt A destroys a cell, only bolt A's speed increases.
- **Bolt lost (falls off screen)**: If the bolt is lost without bumping, the kill count is effectively reset (bolt is despawned). A new bolt starts fresh with `OverchargeKillCount(0)`.
- **Speed reset mechanism**: When bump resets the kill count, the speed bonus must be cleanly removed. If using the effect system, this means removing the accumulated `SpeedBoost` effects. If using a direct multiplier, the bolt domain must track the Overcharge contribution separately from base speed.
- **Zero kills in cycle**: No speed change. The system is a no-op if no `CellDestroyedAt` messages reference this bolt.
- **Cleanup**: `OverchargeKillCount` is on bolt entities -- cleaned up when bolts despawn. `OverchargeConfig` removed at run end.

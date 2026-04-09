# LoseLife

## Config
No config fields.

**RON**: `LoseLife`

## Reversible: NO (side-effecting — can trigger RunLost)

## Target: Breaker (entity with `LivesCount`)

## Component Read
```rust
#[derive(Component)]
struct LivesCount(Option<u32>);
```
- `None` = infinite lives (fire is no-op)
- `Some(n)` = finite lives

## Fire
1. If `LivesCount(None)` (infinite): no-op
2. If `LivesCount(Some(0))` (already zero): no-op, stays at 0 (saturating)
3. If `LivesCount(Some(n))` where n > 0: decrement to n-1
4. If lives just reached zero (was positive, now zero): write `RunLost` message

## Reverse
1. If `LivesCount(None)`: no-op
2. If `LivesCount(Some(n))`: increment to n+1 (saturating at u32::MAX)
3. If no `LivesCount` component: no-op

## Messages Sent
- `RunLost` — written when lives reach zero (transition from positive to zero only, not when already zero)

## Notes
- The `RunLost` message triggers the run-end state transition
- Fire does NOT write `RunLost` if lives are already at 0 (no double-trigger)
- Technically has a reverse, but classified as non-reversible because the RunLost side effect can't be meaningfully undone

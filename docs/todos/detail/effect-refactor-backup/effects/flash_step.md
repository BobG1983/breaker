# FlashStep

## Config
No config fields — marker effect.

**RON**: `FlashStep`

## Reversible: YES

## Target: Breaker

## Component
```rust
#[derive(Component)]
struct FlashStepActive;
```
- Marker component — no data

## Fire
1. Guard: if entity despawned, return
2. If `FlashStepActive` already present, return (idempotent)
3. Insert `FlashStepActive` marker

## Reverse
1. Guard: if entity despawned, return
2. Remove `FlashStepActive` from entity

## Notes
- The dash system checks for `FlashStepActive` — when present, reverse-direction dash teleports instead of sliding
- No stacking — multiple fires are idempotent (only one marker)
- No runtime systems needed for the effect itself

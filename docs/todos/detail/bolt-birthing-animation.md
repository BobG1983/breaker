# Bolt Birthing Animation

## Summary
Effect-spawned bolts go through a "birthing" animation before becoming active, preventing single-frame bolt explosions from cascade effects.

## Context
Split Decision + ArcWelder (chain mode) can cascade: cell death → 2 bolts → new beams → more cell deaths → more bolts → all in one frame. This creates a single-frame bolt explosion that's visually jarring and gameplay-breaking. Birthing adds a brief delay before new bolts become active, spreading the cascade over time.

This is temporary until graphics refactoring, at which point the bolt will play a "spawning" animation that signals birthing is complete.

## Scope
- In: `BoltBirthing(Timer)` component, `tick_bolt_birthing` system in FixedUpdate, query changes to exclude birthing bolts from collision/chain/movement, scale-up VFX during birth period
- Out: Graphics refactor (future), sound effects

Touches: bolt, effect, fx domains

## Dependencies
- Depends on: spawn-bolt-setup-run-migration (needs spawn lifecycle clarity)
- Blocks: Nothing directly, but improves cascade effect quality

## Notes
Components:
- `BoltBirthing(Timer)` — countdown timer, removed when done
- All collision/chain/movement queries add `Without<BoltBirthing>`

During birthing:
- Not collidable (excluded from collision queries)
- Not chain-joinable (excluded from chain mode bolt queries)
- Not damage-dealing
- Visual: scale-up animation (pop-in effect)

Query changes needed: bolt_cell_collision, bolt_wall_collision, bolt_breaker_collision, maintain_tether_chain, fire_chain

## Status
`[NEEDS DETAIL]` — Missing: birthing timer duration, scale-up animation curve/easing, whether birthing applies to ALL effect-spawned bolts or only specific effects (e.g., does MirrorProtocol's mirror bolt birth?), what happens if a birthing bolt's parent is despawned mid-birth, interaction with bolt lifespan timers (does lifespan tick during birthing?)

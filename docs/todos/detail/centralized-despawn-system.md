# Centralized Entity Despawn System

## Summary
Centralize entity despawn into a DespawnEntity message + single despawn system — eliminates scattered .despawn()/.try_despawn() calls, gives a single place for future deferred cleanup or entity recycling.

## Context
During scenario testing (2026-04-06), multiple "Entity despawned" Bevy warnings appeared in visual mode because various systems called `.despawn()` on entities that were already cleaned up by CleanupOnNodeExit/CleanupOnRunEnd. Converting all calls to `.try_despawn()` is a band-aid — the real fix is to centralize despawn so:

1. Only one system ever calls despawn
2. That system can use try_despawn internally
3. Future: can strip visual/physics components first, queue for deferred cleanup, add grace periods for systems that still reference the entity

## Scope
- In: New `DespawnEntity { entity: Entity }` message
- In: Single `process_despawn_requests` system that reads the message and despawns
- In: Sweep all `.despawn()` / `.try_despawn()` calls to write the message instead
- In: Runs in a late schedule (PostUpdate or dedicated cleanup set) so all systems have a chance to read the entity first
- Out: Deferred cleanup / entity pooling (future enhancement, not now)
- Out: CleanupOnNodeExit / CleanupOnRunEnd markers (those are state-driven, separate mechanism)

## Dependencies
- Depends on: nothing
- Blocks: nothing (all callers already work, this is a structural improvement)

## Notes
- Main callers to sweep: effect fire/reverse functions, bolt_lost, chain_lightning arc cleanup, tether beam, shockwave, explode, any entity lifecycle system
- The message should live in `shared::messages` since it's cross-domain
- The system should run after all domain systems but before frame end

## Status
`ready`

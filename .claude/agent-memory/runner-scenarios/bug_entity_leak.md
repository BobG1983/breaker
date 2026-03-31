---
name: NoEntityLeaks fires in long chaos scenarios
description: Entity count grows to >2x baseline in long chaos runs. Bevy ECS "Entity despawned" command errors co-occur, suggesting effect entities (shockwaves, arcs, particles) are not being properly cleaned up in some race conditions.
type: project
---

`check_no_entity_leaks` fires in: `breaker_wall_impact_chaos` (x1 at frame 5880), `aegis_state_machine` (x10 at frames 4920..9240), `overclock_until_speed`, `quick_stop_dash_edges`, `surge_overclock`.

The checker fires when entity count > 2x the baseline count measured at `SpawnNodeComplete`. It checks every 120 frames.

Simultaneous with entity leaks, `breaker_cell_impact_chaos` shows multiple Bevy ECS warnings:
```
Entity despawned: The entity with ID 21v2 is invalid; its index now has generation 3.
```
These appear after cell destruction events, suggesting that commands are being applied to already-despawned entities. The cell destruction pathway (shockwave spawn, chain lightning arc spawn, etc.) may be spawning entities and then failing to despawn them correctly.

**Likely location:** `breaker-game/src/effect/effects/` — shockwave, chain_lightning, or similar FX spawn systems that don't properly despawn on bolt loss / node end. Also `breaker-game/src/chips/systems/dispatch_chip_effects/` — effect dispatch may be scheduling commands on dead entities.

**Note:** The entity leak and EffectiveSpeedConsistent bugs are independent — entity leaks appear in scenarios that do not exercise speed boost chips.

## Scenario Runner Memory

- [Universal failure: Chain Reaction name collision bug](bug_chain_reaction_collision.md) — Chain Reaction template overwritten by evolution of same name, causing WARN logs that fail every scenario
- [Universal failure: EffectiveSpeedConsistent / SizeBoostInRange state-gate mismatch](bug_effective_speed_state_gate.md) — recalculate_speed/size only runs in PlayingState::Active; fire()/reverse() can update Active* outside that state
- [Missing evolution: Railgun not in assets](bug_railgun_missing.md) — evolution_railgun scenario expects Railgun evolution chip that has no .evolution.ron file
- [Entity leak: gradually accumulating entities](bug_entity_leak.md) — NoEntityLeaks fires in long chaos scenarios; despawned-entity ECS errors co-occur
- [RESOLVED: Bridge/Recalculate/BoltLost scheduling cycle](bug_schedule_bridge_recalculate_cycle.md) — Fixed 2026-03-30 by removing Recalculate.after(Bridge) set ordering
- [TetherBeam standard mode bolt accumulation](bug_tether_beam_bolt_accumulation.md) — fire_standard() spawns 2 bolts per Bumped with no mid-node cleanup; BoltCountReasonable fires in tether_beam_stress

## Session History
See [ephemeral/](ephemeral/) — not committed.

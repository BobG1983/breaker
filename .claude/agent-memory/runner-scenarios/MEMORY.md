## Scenario Runner Memory

- [ChainArcCountReasonable self-test threshold too high](bug_chain_arc_self_test.md) — chain_lightning_arc_lifecycle and chain_lightning_chaos self-tests never trigger invariant; max_chain_arc_count=50 exceeds what current game generates per node cycle
- [RESOLVED: Chain Reaction name collision](bug_chain_reaction_collision.md) — evolution renamed to "Shock Chain"; name collision fixed on feature/chip-evolution-ecosystem
- [RESOLVED: EffectiveSpeedConsistent / SizeBoostInRange state-gate mismatch](bug_effective_speed_state_gate.md) — Effective* cache removal eliminated all recalculate_* systems and Effective* components; bug cannot recur
- [Missing evolution: Railgun not in assets](bug_railgun_missing.md) — evolution_railgun scenario expects Railgun evolution chip that has no .evolution.ron file
- [Entity leak: gradually accumulating entities](bug_entity_leak.md) — NoEntityLeaks fires in long chaos scenarios; despawned-entity ECS errors co-occur
- [RESOLVED: Bridge/Recalculate/BoltLost scheduling cycle](bug_schedule_bridge_recalculate_cycle.md) — Fixed 2026-03-30 by removing Recalculate.after(Bridge) set ordering
- [TetherBeam standard mode bolt accumulation](bug_tether_beam_bolt_accumulation.md) — fire_standard() spawns 2 bolts per Bumped with no mid-node cleanup; BoltCountReasonable fires in tether_beam_stress

## Session History
See [ephemeral/](ephemeral/) — not committed.

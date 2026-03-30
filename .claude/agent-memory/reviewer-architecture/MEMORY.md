# Architecture Reviewer Memory

## Patterns
- [Effect direct spawn pattern](pattern_effect_direct_spawn.md) — fire() functions spawn full entities directly rather than messaging owning domains
- [Dispatch pattern ownership](dispatch_pattern_ownership.md) — dispatch lives in entity domains; chip dispatch missing Once wrapper for All* desugaring
- [ShieldActive cross-domain write](shield_cross_domain_write.md) — bolt and cells domains authorized to mutate ShieldActive directly

## Known Gaps
- [Cleanup marker gaps](known_gap_cleanup_markers.md) — gravity_well entities lack CleanupOnNodeExit markers
- [Transform usage in effects](known_gap_transform_usage.md) — several effects read/write Transform instead of Position2D
- [Production logic in effects/mod.rs](known_gap_effects_mod_production_logic.md) — effective_range and spawn_extra_bolt violate routing-only rule
- [Velocity2D cross-domain writes](known_gap_velocity_cross_domain_write.md) — gravity_well and attraction write bolt Velocity2D without documented exception

## Session History
See [ephemeral/](ephemeral/) — not committed.

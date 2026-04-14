# Architecture Reviewer Memory

## Patterns
- [Effect direct spawn pattern](pattern_effect_direct_spawn.md) — fire() functions spawn full entities directly rather than messaging owning domains
- [Dispatch pattern ownership](dispatch_pattern_ownership.md) — dispatch lives in entity domains; chip dispatch missing Once wrapper for All* desugaring
- [ShieldActive cross-domain write — ELIMINATED](shield_cross_domain_write.md) — ShieldActive no longer exists; Shield is now a timed floor wall (ShieldWall) using normal bolt_wall_collision path
- [Bolt builder typestate migration](pattern_bolt_builder_migration.md) — Bolt::builder() with 6 dims (P,S,A,M,R,V); Visual added; BoltRadius aliased to BaseRadius
- [Breaker builder typestate migration](pattern_breaker_builder_migration.md) — Breaker::builder() with 7 dims (incl Role+Visual); fully wired; BreakerConfig eliminated
- [Cross-plugin set registration (phase set)](pattern_cross_plugin_set_registration.md) — RunPlugin registers handle_breaker_death into DeathPipelineSystems::HandleKill owned by DeathPipelinePlugin; first production instance

## Known Gaps
- [Cleanup marker status](known_gap_cleanup_markers.md) — all effect entities have CleanupOnNodeExit as of 2026-03-30; no open gaps
- [Transform usage in effects](known_gap_transform_usage.md) — all previously flagged Transform usages FIXED (full-verification-fixes); chain_lightning arc entities use Transform correctly (rendering objects)
- [Production logic in effects/mod.rs](known_gap_effects_mod_production_logic.md) — RESOLVED: extracted to fire_helpers.rs; mod.rs is routing-only; no open gaps
- [Velocity2D cross-domain writes](known_gap_velocity_cross_domain_write.md) — gravity_well and attraction write bolt Velocity2D without documented exception

## Session History
See [ephemeral/](ephemeral/) — not committed.

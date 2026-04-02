# Architecture Reviewer Memory

## Patterns
- [Effect direct spawn pattern](pattern_effect_direct_spawn.md) — fire() functions spawn full entities directly rather than messaging owning domains
- [Dispatch pattern ownership](dispatch_pattern_ownership.md) — dispatch lives in entity domains; chip dispatch missing Once wrapper for All* desugaring
- [ShieldActive cross-domain write](shield_cross_domain_write.md) — bolt and cells domains authorized to mutate ShieldActive directly
- [Bolt builder typestate migration](pattern_bolt_builder_migration.md) — Bolt::builder() replaces init_bolt_params and prepare_bolt_velocity; velocity clamping now inline
- [Breaker builder typestate migration](pattern_breaker_builder_migration.md) — Breaker::builder() with 7 dims (incl Role); old spawn chain still wired; visibility fix needed

## Known Gaps
- [Cleanup marker status](known_gap_cleanup_markers.md) — all effect entities have CleanupOnNodeExit as of 2026-03-30; no open gaps
- [Transform usage in effects](known_gap_transform_usage.md) — all previously flagged Transform usages FIXED (full-verification-fixes); chain_lightning arc entities use Transform correctly (rendering objects)
- [Production logic in effects/mod.rs](known_gap_effects_mod_production_logic.md) — RESOLVED: extracted to fire_helpers.rs; mod.rs is routing-only; no open gaps
- [Velocity2D cross-domain writes](known_gap_velocity_cross_domain_write.md) — gravity_well and attraction write bolt Velocity2D without documented exception

## Session History
See [ephemeral/](ephemeral/) — not committed.

# Architecture Reviewer Memory

## Patterns
- [Effect direct spawn pattern](pattern_effect_direct_spawn.md) — fire() functions spawn full entities directly rather than messaging owning domains
- [Dispatch pattern ownership](dispatch_pattern_ownership.md) — dispatch lives in entity domains; All* desugaring gap in chip dispatch
- [ShieldActive cross-domain write](shield_cross_domain_write.md) — bolt and cells domains authorized to mutate ShieldActive directly

## Known Gaps
- [Cleanup marker gaps](known_gap_cleanup_markers.md) — gravity_well entities lack CleanupOnNodeExit markers

## Session History
See [ephemeral/](ephemeral/) — not committed.

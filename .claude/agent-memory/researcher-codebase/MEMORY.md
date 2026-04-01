# Researcher Codebase Memory

- [effect-domain-inventory.md](effect-domain-inventory.md) — Complete inventory of all effect implementations and trigger bridges (all real as of feature/source-chip-shield-absorption); component names, test counts
- [effect-system-domain-map.md](effect-system-domain-map.md) — Complete trigger/effect enum inventory, evaluation flow, dispatch, collision message mapping
- [collision-system-map.md](collision-system-map.md) — All 7 collision systems: file paths, schedules, detection method (CCD vs AABB), messages fired per pairing
- [bolt-spawn-component-map.md](bolt-spawn-component-map.md) — Full bolt component inventory post-builder-migration: Bolt::builder() replaces init_bolt_params; BoltCollisionData replaces CollisionQueryBolt; ActiveDamageBoosts/ActivePiercings replace Effective* types
- [bolt-message-pattern-map.md](bolt-message-pattern-map.md) — Bolt domain message inventory (SpawnAdditionalBolt removed), real phantom bolt pattern, ExtraBolt/lifespan, DistanceConstraint wiring for ChainBolt
- [effect-trigger-design-inventory.md](effect-trigger-design-inventory.md) — Design-doc-sourced inventory of all triggers and effects with parameters, behaviors, and reversals
- [scenario-failure-trace.md](scenario-failure-trace.md) — Root causes for 5 systemic failures (feature/scenario-coverage); Failures 2 (EffectiveSpeedConsistent) and 3 (BoltSpeedInRange) RESOLVED by cache removal; chain arc lifecycle bug (Failure 4 subset) fixed in 48766c5; historical record
- [entity-leak-analysis.md](entity-leak-analysis.md) — Deep trace of NoEntityLeaks failures: 2 confirmed leaks (ChainBolt DistanceConstraint, SecondWindWall), warning source explanation, cleanup architecture
- [violation-log-output-trace.md](violation-log-output-trace.md) — Full ViolationLog→stdout trace; --verbose flag prints ViolationEntry.message with speed/bounds/entity; compact omits it
- [bolt-builder-pattern-reference.md](bolt-builder-pattern-reference.md) — Full structural trace of BoltBuilder typestate pattern: 5 dimensions, terminal impls, config() shortcut, build vs spawn, #[require] interaction, and Breaker mapping notes
- [breaker-spawn-inventory.md](breaker-spawn-inventory.md) — Complete Breaker spawn inventory: 5-layer initialization chain, all component sources, BreakerConfig/BreakerDefinition fields, every test helper pattern, node re-entry behavior
- [breaker-query-inventory.md](breaker-query-inventory.md) — Complete inventory of all Breaker entity queries: 8 type aliases in queries.rs, all 15 systems inside breaker/, 5 cross-domain systems (bolt/debug), 7 scenario runner systems, component groupings by category, and QueryData struct migration candidates

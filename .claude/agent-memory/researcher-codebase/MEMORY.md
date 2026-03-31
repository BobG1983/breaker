# Researcher Codebase Memory

- [effect-domain-inventory.md](effect-domain-inventory.md) — Complete inventory of all effect implementations and trigger bridges (all real as of feature/source-chip-shield-absorption); component names, test counts
- [effect-system-domain-map.md](effect-system-domain-map.md) — Complete trigger/effect enum inventory, evaluation flow, dispatch, collision message mapping
- [collision-system-map.md](collision-system-map.md) — All 7 collision systems: file paths, schedules, detection method (CCD vs AABB), messages fired per pairing
- [bolt-spawn-component-map.md](bolt-spawn-component-map.md) — Full bolt component inventory, CollisionLayers setup, CCD participation requirements
- [bolt-message-pattern-map.md](bolt-message-pattern-map.md) — Bolt domain message inventory (SpawnAdditionalBolt removed), real phantom bolt pattern, ExtraBolt/lifespan, DistanceConstraint wiring for ChainBolt
- [effect-trigger-design-inventory.md](effect-trigger-design-inventory.md) — Design-doc-sourced inventory of all triggers and effects with parameters, behaviors, and reversals
- [scenario-failure-trace.md](scenario-failure-trace.md) — Root causes + fixes for 5 systemic scenario failures (feature/scenario-coverage): warn log, EffectiveSpeedConsistent, BoltSpeedInRange, NoEntityLeaks, double-despawn
- [entity-leak-analysis.md](entity-leak-analysis.md) — Deep trace of NoEntityLeaks failures: 2 confirmed leaks (ChainBolt DistanceConstraint, SecondWindWall), warning source explanation, cleanup architecture
- [violation-log-output-trace.md](violation-log-output-trace.md) — Full ViolationLog→stdout trace; --verbose flag prints ViolationEntry.message with speed/bounds/entity; compact omits it

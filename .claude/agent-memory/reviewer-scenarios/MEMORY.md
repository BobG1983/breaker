# reviewer-scenarios Memory

- [Effect System Coverage Map](coverage_effect_system.md) — which effects/triggers have/lack scenario coverage, invariant gaps — updated Phase 4+5
- [Adversarial Quality Patterns](pattern_adversarial_quality.md) — techniques that find real bugs; anti-patterns to flag in scenario reviews
- [Bolt Builder Migration Coverage Map](coverage_bolt_builder_migration.md) — gaps from init_bolt_params deletion, spawn_extra_bolt removal, steering model change, BoltSpeedAccurate rename
- [Evolution Ecosystem Coverage Map](coverage_evolution_ecosystem.md) — gaps from chip-evolution-ecosystem branch: 4 new effects, 16 evolutions, recipe eligibility offering path
- [Breaker Builder Pattern Coverage Map](coverage_breaker_builder_pattern.md) — gaps from breaker-builder-pattern branch: spawn_or_reuse_breaker, effective_radius/size with ClampRange, node-scale+boost interaction, BreakerPositionClamped staleness

- [Bolt Birthing Animation Coverage Map](coverage_bolt_birthing_animation.md) — no scenario specifically tests birthing gating, AnimateIn duration, or bolt-lost respawn birthing; BoltBirthingInactive invariant missing
- [Pause Quit Fix Coverage Map](coverage_pause_quit_fix.md) — gaps for quit-from-pause fix; runner cannot inject into ButtonInput<KeyCode>, so no scenario exercises actual quit path
- [Scenario Runner Wiring Branch Coverage Map](coverage_scenario_runner_wiring.md) — new scenarios added on feature/scenario-runner-wiring: Prism/Aegis/Chrono baseline, CircuitBreaker/FlashStep/MirrorProtocol/Anchor/SplitDecision/NovaLance, node-scale+boost scenarios, multi-node reuse; remaining gaps noted

- [Toughness + HP Scaling Coverage Map](coverage_toughness_hp_scaling.md) — tier/boss HP scaling never exercised by scenarios; runner lacks ToughnessConfig injection; guardian_hp_fraction unit-tested but no scenario validates it

- [Effect System Refactor Coverage Map](coverage_effect_system_refactor.md) — feature/effect-system-refactor: nested arming ADEQUATE, Shape D On(Participant) MISSING, SpawnStampRegistry STRUCTURAL GAP, Lock/Invulnerable MISSING

## Session History
See [ephemeral/](ephemeral/) — not committed.

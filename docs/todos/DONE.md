# Done

- ~~Phase 5a: Rendering architecture~~ — Architecture docs written at docs/architecture/rendering/
- ~~Phase 5b: Design decisions~~ — DR-1 through DR-10 resolved, documented at docs/design/graphics/decisions-required.md
- ~~Breaker builder pattern~~ — Typestate builder with 7 dimensions, migrated breaker spawn + queries
- ~~Wall builder pattern~~ — Typestate builder with 2 dimensions (Side, Visual), migrated wall spawn sites
- ~~Shield timer cost per reflection~~ — 0.5s cost per reflection, duration tuned to 3.0s, ReflectionCost component + deduct_shield_on_reflection system
- ~~State lifecycle refactor~~ — Full state architecture overhaul with screen routing, transitions, and lifecycle management
- ~~Hide breaker/bolt during RunEnd/ChipSelect states~~ — Toggle Visibility on RunState::Node exit/entry
- ~~Extra bolts missing NodeScalingFactor on zoomed-out layouts~~ — Already fixed: apply_node_scale_to_late_bolts + sync_bolt_scale
- ~~Delete PhysicsFrozenDuringPause invariant~~ — Removed in commit 8ef028d7, replaced with unit tests
- ~~Fix detect_combo_king stale migration test~~ — Removed duplicate migration test, added HighlightTriggered assertion to dedup test
- ~~Bolt speed normalization after tether constraint~~ — Added normalize_bolt_speed_after_constraints system after enforce_distance_constraints
- ~~Phase 5p: Transitions & PlayingState~~ — Already delivered by rantzsoft_lifecycle crate (state routing, screen transitions, cleanup)
- ~~Cross-domain prelude, re-export modules & import cleanup~~ — breaker-game/src/prelude/ with 5 re-export modules, merged in refactor/cross-domain-prelude

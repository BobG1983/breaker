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
- ~~Phase 5p: Transitions & PlayingState~~ — Already delivered by rantzsoft_stateflow crate (state routing, screen transitions, cleanup)
- ~~Cross-domain prelude, re-export modules & import cleanup~~ — breaker-game/src/prelude/ with 5 re-export modules, merged in refactor/cross-domain-prelude
- ~~Scenario runner improvements~~ — Streaming subprocess pool, frame budget cuts (666K→386K), parse-once optimization, coverage report, unified log, --coverage/--fail-fast flags. 12min→1:22 runtime.
- ~~Rename rantzsoft_lifecycle → rantzsoft_stateflow~~ — Pure rename across 151 files, no behavior changes
- ~~Cell builder pattern~~ — `Cell::builder()` typestate builder (4 dimensions: Position, Dimensions, Health, Visual); `cells/behaviors/` folder restructure (locked, regen, guarded sub-packages); `CellBehavior` enum (Regen + Guarded variants); multi-char alias (`CellTypeAlias(String)`); guard cell redesign (3x3 ring, sliding guardians via `slide_guardian_cells`); `behaviors: Option<Vec<CellBehavior>>` + `effects: Option<Vec<RootEffect>>` in `CellTypeDefinition`; `locks: Option<LockMap>` in `NodeLayout`; architecture docs at `docs/architecture/builders/cell.md` + `docs/architecture/cell-behaviors.md`; example RON at `assets/examples/cell.example.ron`
- ~~Bolt birthing animation~~ — `Birthing` component in `shared/birthing.rs`, `begin_node_birthing` system on `OnEnter(NodeState::AnimateIn)`, `tick_birthing` in FixedUpdate; builder `.birthed()` method; `TransitionType::None` + quit teardown chain (`MenuState::Teardown` → `GameState::Teardown` → `AppState::Teardown` → `send_app_exit`)
- ~~Cell builder pattern + existing modifiers~~ — `Cell::builder()` typestate builder (4 dimensions), `cells/behaviors/` restructure, `CellBehavior` enum, guard cell redesign, RON migration

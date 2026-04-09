---
name: known_unsafe_blocks
description: Inventory of unsafe blocks in the workspace and their justification status
type: project
---

## Unsafe block inventory (as of 2026-03-28)

**Result: None found.**

The workspace lint configuration in `Cargo.toml` sets:
```
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"
undocumented_unsafe_blocks = "deny"
```

No unsafe blocks exist anywhere in `breaker-game/src/`. Verified by grep across all
changed files in the Phase 1 collision cleanup diff and full-source scan.

No FFI boundaries, no raw pointer manipulation, no proc macros with untrusted input.
No `build.rs` files in any crate.

Still confirmed after Phase 3 effect system + trigger bridge changes (2026-03-28).
Still confirmed after Phase 4+5 runtime effects changes (2026-03-28, feature/runtime-effects):
attraction, chain_bolt, explode, pulse, second_wind, shockwave, spawn_phantom — no unsafe.

Still confirmed after Phase 6 changes (2026-03-29, feature/source-chip-shield-absorption):
source_chip threading, EffectSourceChip component, shield charge absorption, chain lightning
arc-based rework — no unsafe. All mem::replace usage is safe Rust (not unsafe).

Still confirmed after refactor (2026-03-30, develop post-merge, c9964b7):
23 files split into directory modules — code-only structural moves. No unsafe introduced.
Grep for "unsafe" across all .rs files: no matches.

Still confirmed after feature/missing-unit-tests (2026-03-30):
Branch adds test code only (plus one pub(super) visibility widening). No unsafe blocks added.
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed after Wave 3 (2026-03-30, tether_beam chain mode + spawn_bolts inherit fix):
No unsafe blocks in tether_beam/effect.rs, spawn_bolts/effect.rs, or enums.rs.
World::query_filtered, world.get(), world.despawn(), world.insert_resource(),
world.remove_resource() — all safe Bevy World APIs.

Still confirmed after feature/scenario-coverage (2026-03-30):
New scenario runner checkers (check_aabb_matches_entity_dimensions.rs,
check_gravity_well_count_reasonable.rs, check_size_boost_in_range.rs), new frame mutation
helpers (apply_inject_mismatched_bolt_aabb, apply_spawn_extra_gravity_wells,
apply_inject_wrong_size_multiplier), and gravity_well visibility widening
(pub instead of pub(crate)) — no unsafe blocks anywhere. Grep confirmed zero matches.

Still confirmed after cache removal refactor (2026-03-30, commits d6d9b80 + 2bdb81b):
check_effective_speed_consistent.rs and check_size_boost_in_range.rs deleted. All
production code (prepare_bolt_velocity, bolt_breaker_collision, move_breaker, dash)
now reads ActiveSpeedBoosts.multiplier() / ActiveSizeBoosts.multiplier() on-demand.
EffectiveSpeedMultiplier and EffectiveSizeMultiplier types no longer exist in the codebase.
No unsafe blocks in any new or changed code. Grep confirmed zero matches.

Still confirmed after feature/chip-evolution-ecosystem (2026-03-31) — bolt builder migration:
New files: bolt/builder.rs (2700+ lines), rantzsoft_spatial2d/src/builder.rs (499 lines),
rantzsoft_spatial2d/src/queries.rs (38 lines). All are pure safe Rust — typestate
generics, Bevy Bundle impls, ECS QueryData derive. No unsafe anywhere. World access
in spawn_bolt/system.rs and effect systems (spawn_phantom, spawn_bolts, chain_bolt,
mirror_protocol, tether_beam) uses Bevy's safe World API exclusively.
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed after feature/chip-evolution-ecosystem (2026-04-01) — chip ecosystem + new effects:
New effect modules: anchor/effect.rs, circuit_breaker/effect.rs, mirror_protocol/effect.rs,
entropy_engine/effect.rs. All use Bevy's safe World API (world.get, world.get_mut,
world.entity_mut, world.resource_mut). No unsafe anywhere.
speed_boost.rs modified to add recalculate_velocity() using world.query::<SpatialData>() — safe.
Grep confirmed zero "unsafe" matches across all workspace .rs files.

Still confirmed after feature/breaker-builder-pattern (2026-04-02) — breaker builder migration:
New files: breaker/builder/core.rs (908 lines) — pure typestate Rust, no unsafe.
New system: spawn_bolt/system.rs uses remove_resource/insert_resource (Bevy World safe API).
DispatchInitialEffects command in effect/commands/ext.rs — pure safe Rust, Bevy World API only.
Grep confirmed zero "unsafe" matches across all workspace .rs files.

Still confirmed after wall builder feature (2026-04-02) — wall builder migration:
New files: wall/builder/core/{types,transitions,terminal}.rs — pure typestate Rust, no unsafe.
wall/definition.rs, wall/registry.rs, wall/components.rs, wall/plugin.rs,
wall/systems/spawn_walls/system.rs, effect/effects/second_wind/system.rs — no unsafe.
Grep confirmed zero "unsafe" matches across all wall domain .rs files.

Still confirmed after Shield refactor (2026-04-02, commit e887570) — shield.rs rewritten:
effect/effects/shield.rs uses only Bevy's safe World API (query_filtered, get_mut, resource,
resource_mut, spawn, despawn). No unsafe blocks. All .unwrap()/.expect() are in #[cfg(test)].
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed after refactor/state-folder-structure (2026-04-02, commit d2440054):
State module hierarchy restructure (screen/, run/, wall/, ui/ → state/; wall → walls rename).
Pure file moves + import path updates. Zero new "unsafe" keywords in any added line (grep
confirmed 0 matches across 1710 newly added lines). Workspace lint unsafe_code = "deny" remains.

Still confirmed for feature/wall-builder-pattern (2026-04-03) — rantzsoft_stateflow new crate:
New workspace member rantzsoft_stateflow/ added. Grepped all .rs files under
rantzsoft_stateflow/src/ for "unsafe": zero matches. The crate's Cargo.toml uses
`lints.workspace = true`, inheriting the workspace `unsafe_code = "deny"` lint.
No FFI, no raw pointers, no proc macros. No build.rs in this crate.

Still confirmed for feature/effect-placeholder-visuals (2026-04-06):
Changed files: handle_pause_input.rs, state/plugin.rs, shared/components.rs.
Grepped all four changed files for "unsafe": zero matches in all. Workspace lint
unsafe_code = "deny" remains in force. breaker-scenario-runner/Cargo.toml adds
rantzsoft_stateflow as workspace path dep; that crate's unsafe inventory is unchanged.

Still confirmed for refactor/add-cross-domain-prelude (2026-04-06, commit 9d6f8a18):
New files: prelude/mod.rs, prelude/components.rs, prelude/messages.rs, prelude/resources.rs,
prelude/states.rs. All five contain only use/pub(crate) re-export declarations. Grepped for
"unsafe": zero matches. Pure module wiring with no executable code paths. Workspace lint
unsafe_code = "deny" remains in force.

Still confirmed for feature/scenario-runner-wiring (2026-04-07):
New/changed files across breaker-scenario-runner/src/runner/ (app.rs, discovery.rs, execution.rs,
output_dir.rs, run_log.rs, streaming.rs, tiling.rs, output.rs, mod.rs) and main.rs.
Grepped for "unsafe" across all .rs files: zero matches. No FFI, no raw pointers, no build.rs.
Workspace lint unsafe_code = "deny" remains in force and is inherited by the crate via
`lints.workspace = true` in breaker-scenario-runner/Cargo.toml.

Still confirmed for feature/cell-builder-pattern / guarded behavior (2026-04-08):
New files: cells/behaviors/guarded/components.rs, cells/behaviors/guarded/systems/slide_guardian_cells.rs,
cells/builder/core/{types,transitions,terminal}.rs, cells/definition.rs (GuardedBehavior added).
Grepped all five new files for "unsafe": zero matches. No FFI, no raw pointers.
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed for Toughness + HP Scaling (2026-04-08, commit cd6fb019):
New/changed files include cells/resources/data.rs (ToughnessConfig), cells/definition/data.rs
(Toughness enum, GuardedBehavior), state/run/node/systems/spawn_cells_from_layout/system.rs
(HpContext, HpScale, ToughnessHpData, compute_hp), state/run/systems/advance_node.rs
(tier/position_in_tier tracking), state/run/resources/definitions.rs (NodeOutcome fields),
debug/hot_reload/systems/propagate_cell_type_changes.rs and propagate_node_layout_changes.rs.
Grep for "unsafe" across all changed files: zero matches.
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed for feature/bolt-birthing-animation (2026-04-08):
New/changed files: rantzsoft_stateflow/src/transition/types.rs (TransitionType::None variant),
rantzsoft_stateflow/src/transition/orchestration/system.rs (begin_transition + handle_transition_over),
rantzsoft_stateflow/src/transition/effects/post_process.rs, fade/effect.rs,
breaker-game/src/bolt/systems/begin_node_birthing.rs, tick_birthing.rs, shared/birthing.rs,
state/plugin/system.rs, state/plugin/tests.rs, state/menu/main/systems/handle_main_menu_input.rs,
breaker-scenario-runner/src/lifecycle/systems/frame_mutations/mutations.rs, app.rs.
Grepped all changed files for "unsafe": zero matches. Workspace lint unsafe_code = "deny" remains in force.

# System Moves

Every system in breaker-game and where it ends up after the restructure. Systems that stay in their current domain only get gate/import changes in their plugin.rs — no file moves.

## Key: Change Types

- **STAYS** — file stays in current domain, only plugin.rs gate changes
- **MOVES** — file physically moves to a new directory
- **DELETE** — system is removed entirely
- **NEW** — system doesn't exist yet, must be created

---

## Systems That STAY In Their Domain (gate changes only)

These are domain-specific runtime systems. They run during `NodeState::Playing` (or other active states). Only their plugin.rs registration changes from `PlayingState::Active` → `NodeState::Playing` (or similar gate swap).

### bolt/ (STAYS)

| System | Current Gate | New Gate | Notes |
|--------|-------------|----------|-------|
| `launch_bolt` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `hover_bolt` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `spawn_bolt_lost_text` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `dispatch_bolt_effects` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `bolt_cell_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `bolt_wall_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `bolt_breaker_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `clamp_bolt_to_playfield` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `bolt_lost` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `tick_bolt_lifespan` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `cleanup_destroyed_bolts` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `sync_bolt_scale` | Update, PlayingState::Active | Update, NodeState::Playing | |

### breaker/ (STAYS)

| System | Current Gate | New Gate | Notes |
|--------|-------------|----------|-------|
| `update_bump` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `move_breaker` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `update_breaker_state` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `grade_bump` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `perfect_bump_dash_cancel` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `spawn_bump_grade_text` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `spawn_whiff_text` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `trigger_bump_visual` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `breaker_cell_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `breaker_wall_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `animate_bump_visual` | Update, PlayingState::Active | Update, NodeState::Playing | |
| `animate_tilt_visual` | Update, PlayingState::Active | Update, NodeState::Playing | |
| `sync_breaker_scale` | Update, PlayingState::Active | Update, NodeState::Playing | |

### cells/ (STAYS)

| System | Current Gate | New Gate | Notes |
|--------|-------------|----------|-------|
| `handle_cell_hit` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `check_lock_release` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `tick_cell_regen` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `rotate_shield_cells` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `sync_orbit_cell_positions` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `cleanup_cell` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |
| `cell_wall_collision` | FixedUpdate, PlayingState::Active | FixedUpdate, NodeState::Playing | |

### chips/ (STAYS)

| System | Current Gate | New Gate | Notes |
|--------|-------------|----------|-------|
| `dispatch_chip_effects` | Update, GameState::ChipSelect | Update, ChipSelectState::Selecting | import changes for ChipSelected message path |

### effect/triggers/ (STAYS — 18 files, each changes 1 line in register())

| System(s) | Current Gate | New Gate |
|-----------|-------------|----------|
| `bridge_bump` | PlayingState::Active | NodeState::Playing |
| `bridge_perfect_bump` | PlayingState::Active | NodeState::Playing |
| `bridge_early_bump` | PlayingState::Active | NodeState::Playing |
| `bridge_late_bump` | PlayingState::Active | NodeState::Playing |
| `bridge_bump_whiff` | PlayingState::Active | NodeState::Playing |
| `bridge_no_bump` | none (unguarded) | no change |
| `bridge_bumped` | PlayingState::Active | NodeState::Playing |
| `bridge_perfect_bumped` | PlayingState::Active | NodeState::Playing |
| `bridge_early_bumped` | PlayingState::Active | NodeState::Playing |
| `bridge_late_bumped` | PlayingState::Active | NodeState::Playing |
| 7x `bridge_impact_*` | PlayingState::Active | NodeState::Playing |
| 7x `bridge_impacted_*` | PlayingState::Active | NodeState::Playing |
| `bridge_death` | PlayingState::Active | NodeState::Playing |
| `bridge_died` | PlayingState::Active | NodeState::Playing |
| `bridge_cell_destroyed` | PlayingState::Active | NodeState::Playing |
| `bridge_bolt_lost` | PlayingState::Active | NodeState::Playing |
| `bridge_node_end` | PlayingState::Active | NodeState::Playing |
| `tick_time_expires` | PlayingState::Active | NodeState::Playing |
| `desugar_until` | PlayingState::Active | NodeState::Playing |

**Bug fix (gate change, not file move):**
| System | Current Gate | New Gate | Bug Fixed |
|--------|-------------|----------|-----------|
| `bridge_node_start` | OnEnter(PlayingState::Active) | OnEnter(NodeState::Playing) | No longer fires on unpause |
| `reset_entropy_engine_on_node_start` | OnEnter(PlayingState::Active) | OnEnter(NodeState::Playing) | No longer fires on unpause |

### effect/effects/ (STAYS — 10 files, each changes 1 line in register())

| System(s) | Current Gate | New Gate |
|-----------|-------------|----------|
| `tick_gravity_well`, `apply_gravity_pull` | PlayingState::Active | NodeState::Playing |
| `tick_anchor` | PlayingState::Active | NodeState::Playing |
| `maintain_tether_chain`, `tick_tether_beam` | PlayingState::Active | NodeState::Playing |
| `tick_chain_lightning` | PlayingState::Active | NodeState::Playing |
| `tick_pulse` | PlayingState::Active | NodeState::Playing |
| `tick_shockwave`, `shockwave_collision` | PlayingState::Active | NodeState::Playing |
| `tick_piercing_beam` + variants | PlayingState::Active | NodeState::Playing |
| `tick_explode` | PlayingState::Active | NodeState::Playing |
| `apply_attraction` | PlayingState::Active | NodeState::Playing |
| `despawn_second_wind_on_contact` | PlayingState::Active | NodeState::Playing |
| tether_beam cleanup | OnExit(GameState::Playing) | OnEnter(NodeState::Teardown) |

### fx/ (STAYS — minus transition/ which moves)

| System | Current Gate | New Gate |
|--------|-------------|----------|
| `animate_fade_out` | PlayingState::Active | NodeState::Playing |
| `animate_punch_scale` | PlayingState::Active | NodeState::Playing |

### input/ (STAYS — no changes)

### audio/ (STAYS — stub, no changes)

### debug/ (STAYS)

| System | Current Gate | New Gate |
|--------|-------------|----------|
| `track_bump_result` | PlayingState::Active | NodeState::Playing |
| hot-reload propagation | GameState::Playing | NodeState::Playing |

---

## Systems That MOVE (file physically relocates)

### From bolt/ → state/run/node/ (setup systems, not about RUNNING the bolt)

| System | From | To | Reason |
|--------|------|----|--------|
| `apply_node_scale_to_bolt` | `bolt/systems/apply_node_scale_to_bolt.rs` | `state/run/node/systems/apply_node_scale_to_bolt.rs` | Node setup, not bolt runtime |
| `reset_bolt` | `bolt/systems/reset_bolt/` | `state/run/node/systems/reset_bolt/` | Node setup, not bolt runtime |

### From breaker/ → state/run/node/ (setup systems)

| System | From | To | Reason |
|--------|------|----|--------|
| `apply_node_scale_to_breaker` | `breaker/systems/apply_node_scale_to_breaker.rs` | `state/run/node/systems/apply_node_scale_to_breaker.rs` | Node setup, not breaker runtime |
| `reset_breaker` | `breaker/systems/spawn_breaker/system.rs` (reset_breaker fn) | `state/run/node/systems/reset_breaker.rs` | Node setup, not breaker runtime |

### From cells/ → state/run/node/ (setup system)

| System | From | To | Reason |
|--------|------|----|--------|
| `dispatch_cell_effects` | `cells/systems/dispatch_cell_effects.rs` | `state/run/node/systems/dispatch_cell_effects.rs` | Node setup, not cell runtime |

### From wall/ → state/run/node/ (setup systems)

| System | From | To | Reason |
|--------|------|----|--------|
| `spawn_walls` | `wall/systems/spawn_walls.rs` | `state/run/node/systems/spawn_walls.rs` | Node setup |
| `dispatch_wall_effects` | `wall/systems/dispatch_wall_effects.rs` | `state/run/node/systems/dispatch_wall_effects.rs` | Node setup |

### From run/ → state/run/ (entire domain absorbed)

| System | From | To |
|--------|------|----|
| `reset_run_state` | `run/systems/reset_run_state.rs` | `state/run/loading/systems/reset_run_state.rs` |
| `generate_node_sequence` | `run/systems/generate_node_sequence/` | `state/run/loading/systems/generate_node_sequence/` |
| `capture_run_seed` | `run/systems/capture_run_seed.rs` | `state/run/loading/systems/capture_run_seed.rs` |
| `advance_node` | `run/systems/advance_node.rs` | `state/run/systems/advance_node.rs` |
| `handle_node_cleared` | `run/systems/handle_node_cleared.rs` | `state/run/node/lifecycle/handle_node_cleared.rs` |
| `handle_timer_expired` | `run/systems/handle_timer_expired.rs` | `state/run/node/lifecycle/handle_timer_expired.rs` |
| `handle_run_lost` | `run/systems/handle_run_lost.rs` | `state/run/node/lifecycle/handle_run_lost.rs` |
| `complete_transition_out` | `run/systems/complete_transition_out.rs` | DELETE (replaced by state routing) |
| `reset_highlight_tracker` | `run/systems/reset_highlight_tracker.rs` | `state/run/node/lifecycle/reset_highlight_tracker.rs` |
| `spawn_highlight_text` | `run/systems/spawn_highlight_text/` | `state/run/node/lifecycle/spawn_highlight_text/` |
| `track_cells_destroyed` | `run/systems/track_cells_destroyed.rs` | `state/run/node/tracking/track_cells_destroyed.rs` |
| `track_bumps` | `run/systems/track_bumps.rs` (if exists) | `state/run/node/tracking/track_bumps.rs` |
| `track_bolts_lost` | `run/systems/track_bolts_lost.rs` | `state/run/node/tracking/track_bolts_lost.rs` |
| `track_time_elapsed` | `run/systems/track_time_elapsed.rs` | `state/run/node/tracking/track_time_elapsed.rs` |
| `track_evolution_damage` | `run/systems/track_evolution_damage.rs` | `state/run/node/tracking/track_evolution_damage.rs` |
| `track_node_cleared_stats` | `run/systems/track_node_cleared_stats/` | `state/run/node/tracking/track_node_cleared_stats/` |
| `detect_mass_destruction` | `run/highlights/systems/detect_mass_destruction.rs` | `state/run/node/highlights/detect_mass_destruction.rs` |
| `detect_close_save` | `run/highlights/systems/detect_close_save.rs` | `state/run/node/highlights/detect_close_save.rs` |
| `detect_combo_king` | `run/highlights/systems/detect_combo_king.rs` | `state/run/node/highlights/detect_combo_king.rs` |
| `detect_pinball_wizard` | `run/highlights/systems/detect_pinball_wizard.rs` | `state/run/node/highlights/detect_pinball_wizard.rs` |
| `detect_nail_biter` | `run/systems/detect_nail_biter.rs` | `state/run/node/highlights/detect_nail_biter.rs` |
| `track_chips_collected` | `run/systems/track_chips_collected.rs` | `state/run/chip_select/systems/track_chips_collected.rs` |
| `detect_first_evolution` | `run/systems/detect_first_evolution.rs` | `state/run/chip_select/systems/detect_first_evolution.rs` |
| `detect_most_powerful_evolution` | `run/systems/detect_most_powerful_evolution.rs` | `state/run/run_end/systems/detect_most_powerful_evolution.rs` |
| `select_highlights` | `run/systems/select_highlights/` | SPLIT — see system-changes.md |

### From run/node/ → state/run/node/ (entire subdomain absorbed)

| System | From | To |
|--------|------|----|
| `set_active_layout` | `run/node/systems/set_active_layout.rs` | `state/run/node/systems/set_active_layout.rs` |
| `spawn_cells_from_layout` | `run/node/systems/spawn_cells_from_layout/` | `state/run/node/systems/spawn_cells_from_layout/` |
| `init_clear_remaining` | `run/node/systems/init_clear_remaining.rs` | `state/run/node/systems/init_clear_remaining.rs` |
| `init_node_timer` | `run/node/systems/init_node_timer.rs` | `state/run/node/systems/init_node_timer.rs` |
| `check_spawn_complete` | `run/node/systems/check_spawn_complete.rs` | `state/run/node/systems/check_spawn_complete.rs` |
| `track_node_completion` | `run/node/systems/track_node_completion.rs` | `state/run/node/systems/track_node_completion.rs` |
| `tick_node_timer` | `run/node/systems/tick_node_timer.rs` | `state/run/node/systems/tick_node_timer.rs` |
| `apply_time_penalty` | `run/node/systems/apply_time_penalty.rs` | `state/run/node/systems/apply_time_penalty.rs` |
| `reverse_time_penalty` | `run/node/systems/reverse_time_penalty.rs` | `state/run/node/systems/reverse_time_penalty.rs` |

### From run/ resources/types → state/run/

| File | From | To |
|------|------|----|
| `run/resources/` | RunState, RunStats, DifficultyCurve, etc. | `state/run/resources/` |
| `run/components.rs` | | `state/run/components.rs` |
| `run/messages.rs` | RunLost, HighlightTriggered | `state/run/messages.rs` |
| `run/definition/` | TierDefinition, NodeType, HighlightConfig | `state/run/definition/` |
| `run/node/resources/` | NodeTimer, ClearRemainingCount, etc. | `state/run/node/resources/` |
| `run/node/definition/` | NodeLayout, NodeLayoutRegistry | `state/run/node/definition/` |
| `run/node/messages.rs` | NodeCleared, TimerExpired, etc. | `state/run/node/messages.rs` |
| `run/node/sets.rs` | NodeSystems | `state/run/node/sets.rs` |

### From screen/ → state/ (entire domain dissolved)

| File/Dir | From | To |
|----------|------|----|
| `screen/loading/` | | `state/app/loading/` |
| `screen/main_menu/` | | `state/menu/main/` |
| `screen/run_setup/` | | `state/menu/start_game/` |
| `screen/chip_select/` | | `state/run/chip_select/` |
| `screen/run_end/` | | `state/run/run_end/` |
| `screen/pause_menu/` | | `state/pause/` |
| `screen/systems/cleanup.rs` | | `state/cleanup.rs` |
| `screen/plugin.rs` | | `state/plugin.rs` (rewritten) |

### From ui/ → state/ (entire domain dissolved)

| File | From | To |
|------|------|----|
| `ui/systems/spawn_side_panels.rs` | | `state/run/node/hud/systems/spawn_side_panels.rs` |
| `ui/systems/spawn_timer_hud.rs` | | `state/run/node/hud/systems/spawn_timer_hud.rs` |
| `ui/systems/update_timer_display.rs` | | `state/run/node/hud/systems/update_timer_display.rs` |
| `ui/components.rs` | | `state/run/node/hud/components.rs` |
| `ui/resources.rs` | | `state/run/node/hud/resources.rs` |
| `ui/sets.rs` | | `state/run/node/hud/sets.rs` |
| `ui/messages.rs` (ChipSelected) | | `state/run/chip_select/messages.rs` |

### From fx/transition/ → state/transition/

| File | From | To |
|------|------|----|
| `fx/transition/system.rs` | | `state/transition/system.rs` |
| `fx/transition/tests.rs` | | `state/transition/tests.rs` |
| `fx/transition/mod.rs` | | `state/transition/mod.rs` |

### From shared/ → state/ or deleted

| File | From | To |
|------|------|----|
| `shared/game_state.rs` | | `state/types/game_state.rs` (rewritten, 4 variants) |
| `shared/playing_state.rs` | | DELETE |
| `shared/resources.rs` (RunSeed) | | `state/run/resources/` (run-scoped) |
| `shared/components.rs` CleanupOnNodeExit | | `state/cleanup.rs` (replaced by CleanupOnExit<NodeState>) |
| `shared/components.rs` CleanupOnRunEnd | | `state/cleanup.rs` (replaced by CleanupOnExit<RunState>) |

---

## Systems That Are DELETED

| System | File | Reason |
|--------|------|--------|
| `spawn_bolt` | `bolt/systems/spawn_bolt/` | Replaced by `setup_run` |
| `spawn_or_reuse_breaker` | `breaker/systems/spawn_breaker/system.rs` | Replaced by `setup_run` |
| `complete_transition_out` | `run/systems/complete_transition_out.rs` | State routing replaces this |

---

## Systems That Are NEW

| System | Location | Purpose |
|--------|----------|---------|
| `setup_run` | `state/run/systems/setup_run.rs` | OnExit(RunState::Setup): spawn breaker+bolt via builders with CleanupOnExit<RunState> |
| Pass-through routing systems | `state/routing.rs` | OnEnter for states that auto-advance (AnimateIn→Playing, Loading→next, etc.) |
| Teardown routing systems | `state/routing.rs` | OnEnter(Teardown): cleanup + determine next state |
| `toggle_pause` (rewritten) | `state/pause/systems/toggle_pause.rs` | Time<Virtual>::pause()/unpause() instead of PlayingState toggle |

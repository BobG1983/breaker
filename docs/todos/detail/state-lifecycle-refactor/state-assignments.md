# State Assignments

Every system, its current state/schedule, and the state it should run in after migration (Wave 4). This is the post-move, post-merge/split view — all files are in their final locations from `post-restructure-tree.md`.

---

## OnEnter Systems — What State They Set Up

These run once when entering a state. Grouped by target state.

### OnEnter(AppState::Loading)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `spawn_loading_screen` | OnEnter(GameState::Loading) | OnEnter(AppState::Loading) | state/app/loading/ |

### OnEnter(NodeState::Loading)

The big one — all node setup systems converge here. Currently they all run on `OnEnter(GameState::Playing)`.

| System | Current Schedule | New Schedule | Location | Notes |
|--------|-----------------|-------------|----------|-------|
| `set_active_layout` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .chain() |
| `spawn_cells_from_layout` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .in_set(NodeSystems::Spawn).chain() |
| `init_clear_remaining` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .chain() |
| `init_node_timer` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .in_set(NodeSystems::InitTimer).chain() |
| `spawn_walls` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .chain() with dispatch_wall_effects |
| `dispatch_wall_effects` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .chain() after spawn_walls |
| `dispatch_cell_effects` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .after(NodeSystems::Spawn) |
| `apply_node_scale_to_bolt` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .after(NodeSystems::Spawn) |
| `apply_node_scale_to_breaker` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .after(NodeSystems::Spawn) |
| `reset_bolt` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .after(BreakerSystems::Reset) .in_set(BoltSystems::Reset) |
| `reset_breaker` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/systems/ | .in_set(BreakerSystems::Reset) |
| `reset_highlight_tracker` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/lifecycle/ | per-node reset |
| `spawn_side_panels` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/hud/ | |
| `spawn_timer_hud` | OnEnter(GameState::Playing) | OnEnter(NodeState::Loading) | state/run/node/hud/ | .in_set(UiSystems::SpawnTimerHud) |

### OnEnter(NodeState::Playing)

| System | Current Schedule | New Schedule | Location | Notes |
|--------|-----------------|-------------|----------|-------|
| `bridge_node_start` | OnEnter(PlayingState::Active) | OnEnter(NodeState::Playing) | effect/triggers/node_start.rs | **BUG FIX** — no longer fires on unpause |
| `reset_entropy_engine_on_node_start` | OnEnter(PlayingState::Active) | OnEnter(NodeState::Playing) | effect/effects/entropy_engine/ | **BUG FIX** — no longer fires on unpause |

### OnEnter(RunState::Loading)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `reset_run_state` | OnExit(GameState::MainMenu) | OnEnter(RunState::Loading) | state/run/loading/ |
| `generate_node_sequence` | OnExit(GameState::MainMenu) | OnEnter(RunState::Loading) | state/run/loading/ |
| `capture_run_seed` | OnEnter(GameState::Playing) | OnEnter(RunState::Loading) | state/run/loading/ |

### OnEnter(RunState::Node)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `advance_node` | OnEnter(GameState::TransitionIn) | OnEnter(RunState::Node) | state/run/systems/ |

### OnExit(RunState::Setup)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `setup_run` | — (NEW) | OnExit(RunState::Setup) | state/run/systems/ |

### OnEnter(MenuState::Main)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `spawn_main_menu` | OnEnter(GameState::MainMenu) | OnEnter(MenuState::Main) | state/menu/main/ |

### OnEnter(MenuState::StartGame)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `spawn_run_setup` | OnEnter(GameState::RunSetup) | OnEnter(MenuState::StartGame) | state/menu/start_game/ |

### OnEnter(ChipSelectState::Selecting)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `generate_chip_offerings` | OnEnter(GameState::ChipSelect) | OnEnter(ChipSelectState::Selecting) | state/run/chip_select/ |
| `spawn_chip_select` | OnEnter(GameState::ChipSelect) | OnEnter(ChipSelectState::Selecting) | state/run/chip_select/ |

### OnEnter(RunEndState::Active)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `spawn_run_end_screen` | OnEnter(GameState::RunEnd) | OnEnter(RunEndState::Active) | state/run/run_end/ |
| `detect_most_powerful_evolution` | OnEnter(GameState::RunEnd) | OnEnter(RunEndState::Active) | state/run/run_end/ |
| `select_final_highlights` | — (SPLIT from select_highlights) | OnEnter(RunEndState::Active) | state/run/run_end/ |

### OnEnter(NodeState::Teardown)

| System | Current Schedule | New Schedule | Location |
|--------|-----------------|-------------|----------|
| `cleanup<CleanupOnExit<NodeState>>` | OnExit(GameState::Playing) | OnEnter(NodeState::Teardown) | state/routing.rs |
| tether_beam cleanup | OnExit(GameState::Playing) | OnEnter(NodeState::Teardown) | effect/effects/tether_beam/ |

### Teardown routing (determine next parent state)

| System | Schedule | Location | Decision |
|--------|----------|----------|----------|
| node_teardown_router | OnEnter(NodeState::Teardown) | state/routing.rs | Reads RunOutcome → RunState::ChipSelect or RunState::RunEnd |
| chip_select_teardown_router | OnEnter(ChipSelectState::Teardown) | state/routing.rs | Sets RunState::Node |
| run_end_teardown_router | OnEnter(RunEndState::Teardown) | state/routing.rs | Sets RunState::Teardown |
| run_teardown_router | OnEnter(RunState::Teardown) | state/routing.rs | Sets GameState::Menu |
| menu_teardown_router | OnEnter(MenuState::Teardown) | state/routing.rs | Sets GameState::Run |

---

## FixedUpdate Systems — NodeState::Playing

All of these currently run with `run_if(in_state(PlayingState::Active))` → change to `run_if(in_state(NodeState::Playing))`.

### From bolt/ (stays in bolt/)

| System | Ordering | Location |
|--------|----------|----------|
| `launch_bolt` | — | bolt/plugin.rs |
| `hover_bolt` | .after(BreakerSystems::Move) | bolt/plugin.rs |
| `spawn_bolt_lost_text` | — | bolt/plugin.rs |
| `dispatch_bolt_effects` | .before(EffectSystems::Bridge) | bolt/plugin.rs |
| `bolt_cell_collision` | .after(Physics*).after(BreakerSystems::Move).in_set(BoltSystems::CellCollision) | bolt/plugin.rs |
| `bolt_wall_collision` | .after(BoltSystems::CellCollision).in_set(BoltSystems::WallCollision) | bolt/plugin.rs |
| `bolt_breaker_collision` | .after(BoltSystems::CellCollision).in_set(BoltSystems::BreakerCollision) | bolt/plugin.rs |
| `clamp_bolt_to_playfield` | .after(bolt_breaker_collision) | bolt/plugin.rs |
| `bolt_lost` | .after(Physics*).after(clamp_bolt_to_playfield).in_set(BoltSystems::BoltLost) | bolt/plugin.rs |
| `tick_bolt_lifespan` | .before(BoltSystems::BoltLost) | bolt/plugin.rs |
| `cleanup_destroyed_bolts` | .after(EffectSystems::Bridge) | bolt/plugin.rs |

### From breaker/ (stays in breaker/)

| System | Ordering | Location |
|--------|----------|----------|
| `update_bump` | — | breaker/plugin.rs |
| `move_breaker` | .after(update_bump).in_set(BreakerSystems::Move) | breaker/plugin.rs |
| `update_breaker_state` | .after(move_breaker).in_set(BreakerSystems::UpdateState) | breaker/plugin.rs |
| `grade_bump` | .after(update_bump).after(BoltSystems::BreakerCollision).in_set(BreakerSystems::GradeBump) | breaker/plugin.rs |
| `perfect_bump_dash_cancel` | .after(grade_bump).before(BreakerSystems::UpdateState) | breaker/plugin.rs |
| `spawn_bump_grade_text` | .after(grade_bump).before(BreakerSystems::UpdateState) | breaker/plugin.rs |
| `spawn_whiff_text` | .after(grade_bump).before(BreakerSystems::UpdateState) | breaker/plugin.rs |
| `trigger_bump_visual` | .after(update_bump) | breaker/plugin.rs |
| `breaker_cell_collision` | .after(BreakerSystems::Move) | breaker/plugin.rs |
| `breaker_wall_collision` | .after(BreakerSystems::Move) | breaker/plugin.rs |

### From cells/ (stays in cells/)

| System | Ordering | Location |
|--------|----------|----------|
| `handle_cell_hit` | — | cells/plugin.rs |
| `check_lock_release` | .after(handle_cell_hit) | cells/plugin.rs |
| `tick_cell_regen` | — | cells/plugin.rs |
| `rotate_shield_cells` | — | cells/plugin.rs |
| `sync_orbit_cell_positions` | .after(rotate_shield_cells) | cells/plugin.rs |
| `cleanup_cell` | .after(EffectSystems::Bridge) | cells/plugin.rs |
| `cell_wall_collision` | — | cells/plugin.rs |

### From state/run/node/ (moved from run/)

| System | Ordering | Location |
|--------|----------|----------|
| `track_node_completion` | .in_set(NodeSystems::TrackCompletion) | state/run/node/plugin.rs |
| `tick_node_timer` | .in_set(NodeSystems::TickTimer) | state/run/node/plugin.rs |
| `reverse_time_penalty` | .in_set(NodeSystems::ApplyTimePenalty).after(TickTimer).before(apply_time_penalty) | state/run/node/plugin.rs |
| `apply_time_penalty` | .in_set(NodeSystems::ApplyTimePenalty).after(TickTimer) | state/run/node/plugin.rs |
| `handle_node_cleared` | .after(NodeSystems::TrackCompletion) | state/run/node/plugin.rs |
| `handle_timer_expired` | .after(NodeSystems::ApplyTimePenalty).after(handle_node_cleared) | state/run/node/plugin.rs |
| `handle_run_lost` | .after(handle_node_cleared).after(handle_timer_expired) | state/run/node/plugin.rs |

### From state/run/node/tracking/ (moved from run/)

| System | Ordering | Location |
|--------|----------|----------|
| `track_cells_destroyed` | — | state/run/node/plugin.rs |
| `track_bumps` | — | state/run/node/plugin.rs |
| `track_bolts_lost` | — | state/run/node/plugin.rs |
| `track_time_elapsed` | — | state/run/node/plugin.rs |
| `track_evolution_damage` | — | state/run/node/plugin.rs |
| `track_node_cleared_stats` | .after(NodeSystems::TrackCompletion) | state/run/node/plugin.rs |

### From state/run/node/highlights/ (moved from run/)

| System | Ordering | Location |
|--------|----------|----------|
| `detect_mass_destruction` | — | state/run/node/plugin.rs |
| `detect_close_save` | .after(BreakerSystems::GradeBump) | state/run/node/plugin.rs |
| `detect_combo_king` | — | state/run/node/plugin.rs |
| `detect_pinball_wizard` | — | state/run/node/plugin.rs |
| `detect_nail_biter` | .after(NodeSystems::TrackCompletion) | state/run/node/plugin.rs |

### Highlight text juice

| System | Schedule | Gate | Location |
|--------|----------|------|----------|
| `spawn_highlight_text` | Update | NodeState::Playing | state/run/node/plugin.rs |

### Check spawn complete (intentionally unguarded)

| System | Schedule | Gate | Location |
|--------|----------|------|----------|
| `check_spawn_complete` | FixedUpdate | none | state/run/node/plugin.rs |

---

## Update Systems

| System | Current Gate | New Gate | Location |
|--------|-------------|----------|----------|
| `sync_bolt_scale` | PlayingState::Active | NodeState::Playing | bolt/plugin.rs |
| `animate_bump_visual` | PlayingState::Active | NodeState::Playing | breaker/plugin.rs |
| `animate_tilt_visual` | PlayingState::Active | NodeState::Playing | breaker/plugin.rs |
| `sync_breaker_scale` | PlayingState::Active | NodeState::Playing | breaker/plugin.rs |
| `animate_fade_out` | PlayingState::Active | NodeState::Playing | fx/plugin.rs |
| `animate_punch_scale` | PlayingState::Active | NodeState::Playing | fx/plugin.rs |
| `update_timer_display` | PlayingState::Active | NodeState::Playing | state/run/node/hud/ |
| `spawn_highlight_text` | PlayingState::Active | NodeState::Playing | state/run/node/ |
| `update_loading_bar` | GameState::Loading | AppState::Loading | state/app/loading/ |
| `build_chip_catalog` | GameState::Loading | AppState::Loading | chips/ (registration in state/) |
| `handle_main_menu_input` | GameState::MainMenu | MenuState::Main | state/menu/main/ |
| `update_menu_colors` | GameState::MainMenu | MenuState::Main | state/menu/main/ |
| `handle_run_setup_input` | GameState::RunSetup | MenuState::StartGame | state/menu/start_game/ |
| `handle_seed_input` | GameState::RunSetup | MenuState::StartGame | state/menu/start_game/ |
| `update_run_setup_colors` | GameState::RunSetup | MenuState::StartGame | state/menu/start_game/ |
| `update_seed_display` | GameState::RunSetup | MenuState::StartGame | state/menu/start_game/ |
| `handle_chip_input` | GameState::ChipSelect | ChipSelectState::Selecting | state/run/chip_select/ |
| `tick_chip_timer` | GameState::ChipSelect | ChipSelectState::Selecting | state/run/chip_select/ |
| `update_chip_display` | GameState::ChipSelect | ChipSelectState::Selecting | state/run/chip_select/ |
| `track_chips_collected` | GameState::ChipSelect | ChipSelectState::Selecting | state/run/chip_select/ |
| `detect_first_evolution` | GameState::ChipSelect | ChipSelectState::Selecting | state/run/chip_select/ |
| `dispatch_chip_effects` | GameState::ChipSelect | ChipSelectState::Selecting | chips/plugin.rs |
| `handle_run_end_input` | GameState::RunEnd | RunEndState::Active | state/run/run_end/ |
| `toggle_pause` | GameState::Playing | AppState::Game (REWRITE) | state/pause/ |
| `handle_pause_input` | PlayingState::Paused | is_paused() condition (REWRITE) | state/pause/ |
| `track_bump_result` | PlayingState::Active | NodeState::Playing | debug/telemetry/ |
| hot-reload propagation | GameState::Playing | NodeState::Playing | debug/hot_reload/ |

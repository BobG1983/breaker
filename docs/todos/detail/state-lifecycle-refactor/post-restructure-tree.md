# Post-Restructure Folder Tree

Expected `breaker-game/src/` tree after Wave 2 (file moves + merges/splits complete, before state migration).

```
breaker-game/src/
├── lib.rs                          # add: pub mod state; remove: pub mod screen, pub mod ui, pub mod run
├── main.rs
├── app.rs
├── game.rs                         # PluginGroup: drops ScreenPlugin, UiPlugin, RunPlugin; adds StatePlugin
│
├── shared/                         # SLIMMER — state enums + cleanup markers + RunSeed removed
│   ├── mod.rs
│   ├── collision_layers.rs
│   ├── color.rs
│   ├── components.rs               # BaseWidth, BaseHeight, NodeScalingFactor only
│   ├── draw_layer.rs
│   ├── playfield.rs
│   ├── rng.rs                      # GameRng
│   └── size.rs
│
├── state/                          # NEW — all state lifecycle, routing, screens, HUD, run orchestration
│   ├── mod.rs                      # pub mod declarations + re-exports
│   ├── plugin.rs                   # StatePlugin — all state registration, defaults, progress, sub-plugins
│   ├── cleanup.rs                  # CleanupOnExit<S> component, cleanup_entities<T> (← screen/systems/)
│   ├── routing.rs                  # Pass-through + teardown routing systems (plain NextState::set)
│   │
│   ├── types/                      # State enum definitions (passive, no systems)
│   │   ├── mod.rs                  # re-exports all state types
│   │   ├── app_state.rs            # AppState { Loading, Game, Teardown }
│   │   ├── game_state.rs           # GameState { Loading, Menu, Run, Teardown }
│   │   ├── menu_state.rs           # MenuState { Loading, Main, StartGame, Options, Meta, Teardown }
│   │   ├── run_state.rs            # RunState { Loading, Setup, Node, ChipSelect, RunEnd, Teardown }
│   │   ├── node_state.rs           # NodeState { Loading, AnimateIn, Playing, AnimateOut, Teardown }
│   │   ├── chip_select_state.rs    # ChipSelectState { Loading, AnimateIn, Selecting, AnimateOut, Teardown }
│   │   └── run_end_state.rs        # RunEndState { Loading, AnimateIn, Active, AnimateOut, Teardown }
│   │
│   ├── app/                        # AppState-level
│   │   ├── mod.rs
│   │   └── loading/                # AppState::Loading — disk asset loading, progress UI
│   │       ├── mod.rs
│   │       ├── plugin.rs           # ← screen/loading/plugin.rs
│   │       ├── components.rs       # ← screen/loading/components.rs (LoadingScreen, LoadingBarFill, etc.)
│   │       └── systems/
│   │           ├── mod.rs
│   │           ├── spawn_loading_screen.rs
│   │           └── update_loading_bar.rs
│   │
│   ├── game/                       # GameState-level
│   │   ├── mod.rs
│   │   └── loading/                # GameState::Loading — registry stuffing, progress gate
│   │       └── mod.rs              # thin — wires progress transition or resource check
│   │
│   ├── menu/                       # MenuState-level
│   │   ├── mod.rs
│   │   ├── main/                   # MenuState::Main — main menu screen
│   │   │   ├── mod.rs
│   │   │   ├── plugin.rs           # ← screen/main_menu/plugin.rs
│   │   │   ├── components.rs       # ← screen/main_menu/components.rs (MainMenuScreen, MenuItem)
│   │   │   ├── resources.rs        # ← screen/main_menu/resources.rs (MainMenuSelection, MainMenuConfig)
│   │   │   └── systems/
│   │   │       ├── mod.rs
│   │   │       ├── spawn_main_menu.rs
│   │   │       ├── handle_main_menu_input.rs   # REWRITE: NextState<MenuState>(StartGame)
│   │   │       └── update_menu_colors.rs
│   │   └── start_game/             # MenuState::StartGame — breaker/seed selection
│   │       ├── mod.rs
│   │       ├── plugin.rs           # ← screen/run_setup/plugin.rs
│   │       ├── components.rs       # ← screen/run_setup/components.rs
│   │       ├── resources.rs        # ← screen/run_setup/resources.rs
│   │       └── systems/
│   │           ├── mod.rs
│   │           ├── spawn_run_setup.rs
│   │           ├── handle_run_setup_input.rs   # REWRITE: NextState<MenuState>(Teardown)
│   │           ├── handle_seed_input.rs
│   │           ├── update_run_setup_colors.rs
│   │           └── update_seed_display.rs
│   │
│   ├── run/                        # RunState-level — ABSORBS entire run/ domain
│   │   ├── mod.rs
│   │   ├── plugin.rs               # ← run/plugin.rs (rewritten for new states)
│   │   ├── resources/              # ← run/resources/ (RunState resource, RunStats, DifficultyCurve, etc.)
│   │   │   ├── mod.rs
│   │   │   ├── definitions.rs
│   │   │   └── tests.rs
│   │   ├── components.rs           # ← run/components.rs
│   │   ├── messages.rs             # ← run/messages.rs (RunLost, HighlightTriggered)
│   │   ├── definition/             # ← run/definition/ (TierDefinition, NodeType, HighlightConfig)
│   │   │   ├── mod.rs
│   │   │   ├── types.rs
│   │   │   └── tests.rs
│   │   ├── systems/                # Run-level systems (not node-specific)
│   │   │   ├── mod.rs
│   │   │   ├── advance_node.rs     # ← run/systems/ — OnEnter(RunState::Node)
│   │   │   └── setup_run.rs        # NEW — OnExit(RunState::Setup): spawn breaker+bolt
│   │   │
│   │   ├── loading/                # RunState::Loading — run initialization
│   │   │   ├── mod.rs
│   │   │   └── systems/
│   │   │       ├── mod.rs
│   │   │       ├── reset_run_state.rs              # ← run/systems/
│   │   │       ├── generate_node_sequence/         # ← run/systems/ (dir module, may refactor)
│   │   │       └── capture_run_seed.rs             # ← run/systems/
│   │   │
│   │   ├── node/                   # RunState::Node → NodeState
│   │   │   ├── mod.rs
│   │   │   ├── plugin.rs           # ← run/node/plugin.rs (rewritten)
│   │   │   ├── sets.rs             # ← run/node/sets.rs (NodeSystems)
│   │   │   ├── messages.rs         # ← run/node/messages.rs (NodeCleared, TimerExpired, etc.)
│   │   │   ├── resources/          # ← run/node/resources/ (NodeTimer, ClearRemainingCount, etc.)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── definitions.rs
│   │   │   │   └── tests.rs
│   │   │   ├── definition/         # ← run/node/definition/ (NodeLayout, NodeLayoutRegistry)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── types.rs
│   │   │   │   └── tests/
│   │   │   │
│   │   │   ├── systems/            # Node setup + runtime systems (plugin wires to correct schedules)
│   │   │   │   ├── mod.rs
│   │   │   │   │   # ── OnEnter(NodeState::Loading) setup systems ──
│   │   │   │   ├── set_active_layout.rs            # ← run/node/systems/
│   │   │   │   ├── spawn_cells_from_layout/        # ← run/node/systems/ (dir module)
│   │   │   │   ├── init_clear_remaining.rs         # ← run/node/systems/
│   │   │   │   ├── init_node_timer.rs              # ← run/node/systems/
│   │   │   │   ├── check_spawn_complete.rs         # ← run/node/systems/ (REWRITE)
│   │   │   │   ├── spawn_walls.rs                  # ← wall/systems/
│   │   │   │   ├── dispatch_wall_effects.rs        # ← wall/systems/
│   │   │   │   ├── dispatch_cell_effects.rs        # ← cells/systems/
│   │   │   │   ├── apply_node_scale_to_bolt.rs     # ← bolt/systems/
│   │   │   │   ├── apply_node_scale_to_breaker.rs  # ← breaker/systems/
│   │   │   │   ├── reset_bolt/                     # ← bolt/systems/reset_bolt/ (dir module)
│   │   │   │   ├── reset_breaker.rs                # ← breaker/systems/spawn_breaker/ (reset fn only)
│   │   │   │   │   # ── FixedUpdate NodeState::Playing gameplay systems ──
│   │   │   │   ├── track_node_completion.rs        # ← run/node/systems/
│   │   │   │   ├── tick_node_timer.rs              # ← run/node/systems/
│   │   │   │   ├── apply_time_penalty.rs           # ← run/node/systems/
│   │   │   │   └── reverse_time_penalty.rs         # ← run/node/systems/
│   │   │   │
│   │   │   ├── tracking/           # Run stat accumulation during node gameplay
│   │   │   │   ├── mod.rs
│   │   │   │   └── systems/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── track_cells_destroyed.rs    # ← run/systems/
│   │   │   │       ├── track_bumps.rs              # ← run/systems/
│   │   │   │       ├── track_bolts_lost.rs         # ← run/systems/
│   │   │   │       ├── track_time_elapsed.rs       # ← run/systems/
│   │   │   │       ├── track_evolution_damage.rs   # ← run/systems/
│   │   │   │       └── track_node_cleared_stats/   # ← run/systems/ (dir module)
│   │   │   │
│   │   │   ├── highlights/         # Highlight detection during node gameplay
│   │   │   │   ├── mod.rs
│   │   │   │   └── systems/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── detect_mass_destruction.rs  # ← run/highlights/systems/
│   │   │   │       ├── detect_close_save.rs        # ← run/highlights/systems/
│   │   │   │       ├── detect_combo_king.rs        # ← run/highlights/systems/
│   │   │   │       ├── detect_pinball_wizard.rs    # ← run/highlights/systems/
│   │   │   │       └── detect_nail_biter.rs        # ← run/systems/
│   │   │   │
│   │   │   ├── lifecycle/          # Node→run transition decisions + per-node resets
│   │   │   │   ├── mod.rs
│   │   │   │   └── systems/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── handle_node_cleared.rs      # ← run/systems/ (REWRITE)
│   │   │   │       ├── handle_timer_expired.rs     # ← run/systems/ (REWRITE)
│   │   │   │       ├── handle_run_lost.rs          # ← run/systems/ (REWRITE)
│   │   │   │       ├── reset_highlight_tracker.rs  # ← run/systems/
│   │   │   │       └── spawn_highlight_text/       # ← run/systems/ (dir module)
│   │   │   │
│   │   │   └── hud/               # HUD — ABSORBS ui/ systems
│   │   │       ├── mod.rs
│   │   │       ├── components.rs   # ← ui/components.rs (NodeTimerDisplay, SidePanels, StatusPanel)
│   │   │       ├── resources.rs    # ← ui/resources.rs (TimerUiConfig)
│   │   │       ├── sets.rs         # ← ui/sets.rs (UiSystems)
│   │   │       └── systems/
│   │   │           ├── mod.rs
│   │   │           ├── spawn_side_panels.rs        # ← ui/systems/
│   │   │           ├── spawn_timer_hud.rs          # ← ui/systems/
│   │   │           └── update_timer_display.rs     # ← ui/systems/
│   │   │
│   │   ├── chip_select/           # RunState::ChipSelect — ABSORBS screen/chip_select/
│   │   │   ├── mod.rs
│   │   │   ├── plugin.rs          # ← screen/chip_select/plugin.rs
│   │   │   ├── components.rs      # ← screen/chip_select/components.rs
│   │   │   ├── resources.rs       # ← screen/chip_select/resources.rs (ChipOffers, ChipOffering, etc.)
│   │   │   ├── messages.rs        # ← ui/messages.rs (ChipSelected)
│   │   │   └── systems/
│   │   │       ├── mod.rs
│   │   │       ├── generate_chip_offerings.rs      # ← screen/chip_select/systems/
│   │   │       ├── spawn_chip_select.rs            # ← screen/chip_select/systems/
│   │   │       ├── handle_chip_input.rs            # ← screen/chip_select/systems/ (REWRITE)
│   │   │       ├── tick_chip_timer.rs              # ← screen/chip_select/systems/ (REWRITE)
│   │   │       ├── update_chip_display.rs          # ← screen/chip_select/systems/
│   │   │       ├── track_chips_collected.rs        # ← run/systems/
│   │   │       ├── detect_first_evolution.rs       # ← run/systems/
│   │   │       └── select_highlights.rs            # ← run/systems/select_highlights/ (SPLIT: chip_select half)
│   │   │
│   │   └── run_end/               # RunState::RunEnd — ABSORBS screen/run_end/
│   │       ├── mod.rs
│   │       ├── plugin.rs          # ← screen/run_end/plugin.rs
│   │       ├── components.rs      # ← screen/run_end/components.rs
│   │       └── systems/
│   │           ├── mod.rs
│   │           ├── spawn_run_end_screen/           # ← screen/run_end/systems/ (dir module)
│   │           ├── handle_run_end_input.rs         # ← screen/run_end/systems/ (REWRITE)
│   │           ├── detect_most_powerful_evolution.rs # ← run/systems/
│   │           └── select_final_highlights.rs      # ← run/systems/select_highlights/ (SPLIT: run_end half)
│   │
│   ├── pause/                     # Pause overlay (cross-cutting, Time<Virtual>)
│   │   ├── mod.rs
│   │   ├── plugin.rs              # ← screen/pause_menu/plugin.rs (REWRITE)
│   │   ├── components.rs          # ← screen/pause_menu/components.rs
│   │   ├── resources.rs           # ← screen/pause_menu/resources.rs
│   │   └── systems/
│   │       ├── mod.rs
│   │       ├── spawn_pause_menu.rs     # REWRITE (run condition, not OnEnter)
│   │       ├── toggle_pause.rs         # REWRITE (Time<Virtual>)
│   │       └── handle_pause_input.rs   # REWRITE
│   │
│   └── transition/                # Transition overlay (parked, rewired by lifecycle crate later)
│       ├── mod.rs
│       ├── system.rs              # ← fx/transition/system.rs (PARKED — not wired to any state)
│       └── tests.rs               # ← fx/transition/tests.rs
│
│   # ── UNCHANGED DOMAINS (gate changes in plugin.rs only) ──────────
│
├── bolt/                           # Runtime bolt systems stay. Setup systems (reset, scale) moved out.
├── breaker/                        # Runtime breaker systems stay. Setup systems moved out.
├── cells/                          # Runtime cell systems stay. dispatch_cell_effects moved out.
├── chips/                          # Stays. Import path for ChipSelected changes.
├── walls/                          # RENAMED from wall/. spawn_walls + dispatch_wall_effects moved out.
├── effect/                         # Stays. Gate changes in 28+ register() functions.
├── input/                          # Stays. No changes.
├── fx/                             # Stays minus transition/. Gate changes for fade/punch.
├── audio/                          # Stays. Stub, no changes.
└── debug/                          # Stays. Gate changes in 2 plugin files.
```

## Directories Deleted After Restructure

- `src/screen/` — entirely absorbed into `src/state/`
- `src/ui/` — entirely absorbed into `src/state/run/node/hud/` + `src/state/run/chip_select/messages.rs`
- `src/run/` — entirely absorbed into `src/state/run/`
- `src/fx/transition/` — moved to `src/state/transition/`
- `src/shared/game_state.rs` — replaced by `src/state/types/game_state.rs`
- `src/shared/playing_state.rs` — deleted (pause uses Time<Virtual>)
- `src/shared/resources.rs` — RunSeed moves to `src/state/run/resources/`

## Scenario Runner Impact

The scenario runner imports from paths that change:
- `breaker::screen::chip_select::{ChipOffers, ChipOffering}` → `breaker::state::run::chip_select::{ChipOffers, ChipOffering}`
- `breaker::run::*` → `breaker::state::run::*`
- `breaker::ui::messages::ChipSelected` → `breaker::state::run::chip_select::messages::ChipSelected`
- `breaker::shared::GameState` → `breaker::state::types::GameState` (or re-exported from state/)
- `breaker::shared::PlayingState` → DELETED

All `pub mod` declarations must be maintained for cross-crate access.

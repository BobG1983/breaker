# Screen Domain Migration

Module map for the expanded `screen/` domain and migration table showing where current `ui/` and `fx/` code moves.

## Target Module Tree

```
screen/
  mod.rs                            // ScreenPlugin, sub-plugin registration
  plugin.rs                         // ScreenPlugin impl

  // ── Per-screen modules (each has its own plugin) ──

  main_menu/
    mod.rs                          // MainMenuPlugin
    plugin.rs
    components.rs                   // MainMenuMarker, menu entity markers
    resources.rs
    systems/
      mod.rs
      spawn_main_menu.rs            // EXISTING — spawn menu entities
      handle_main_menu_input.rs     // EXISTING — menu input
      update_menu_colors.rs         // EXISTING — visual polish

  run_setup/
    mod.rs                          // RunSetupPlugin
    plugin.rs
    components.rs                   // EXISTING
    resources.rs                    // EXISTING
    systems/
      mod.rs
      spawn_run_setup.rs            // EXISTING
      handle_run_setup_input.rs     // EXISTING
      handle_seed_input.rs          // EXISTING
      update_seed_display.rs        // EXISTING
      update_run_setup_colors.rs    // EXISTING

  chip_select/
    mod.rs                          // ChipSelectPlugin
    plugin.rs
    components.rs                   // EXISTING + card entity markers
    resources.rs                    // EXISTING
    systems/
      mod.rs
      spawn_chip_select.rs          // EXISTING — card layout, rarity treatments
      generate_chip_offerings.rs    // EXISTING
      handle_chip_input.rs          // EXISTING
      tick_chip_timer.rs            // EXISTING
      update_chip_display.rs        // EXISTING

  run_end/
    mod.rs                          // RunEndPlugin
    plugin.rs
    components.rs                   // EXISTING
    systems/
      mod.rs
      spawn_run_end_screen/         // EXISTING (already has tests/)
      handle_run_end_input.rs       // EXISTING

  pause/                            // RENAMED from pause_menu/
    mod.rs                          // PausePlugin
    plugin.rs
    components.rs                   // FROM pause_menu/components.rs
    resources.rs                    // FROM pause_menu/resources.rs
    systems/
      mod.rs
      spawn_pause_menu.rs           // FROM pause_menu/systems/
      handle_pause_input.rs         // FROM pause_menu/systems/
      toggle_pause.rs               // FROM pause_menu/systems/

  loading/
    mod.rs                          // LoadingPlugin
    plugin.rs
    components.rs                   // EXISTING
    systems/
      mod.rs
      spawn_loading_screen.rs       // EXISTING
      update_loading_bar.rs         // EXISTING

  // ── New modules (Phase 5) ──

  playing/
    mod.rs                          // PlayingScreenPlugin
    hud/
      mod.rs                        // HudPlugin
      timer_wall.rs                 // NEW — timer overlay spawn/update
      life_orbs.rs                  // NEW — life orb spawn/dissolve/birth
      node_progress.rs              // NEW — node progress tick display

  transition/
    mod.rs                          // TransitionPlugin, TransitionStyle enum
    flash.rs                        // NEW — flash transition (TriggerScreenFlash)
    sweep.rs                        // NEW — sweep transition (Beam + SparkBurst)
    glitch.rs                       // NEW — glitch transition (ChromaticAberration + RadialDistortion)
    collapse_rebuild.rs             // NEW — collapse/rebuild (FullscreenMaterial)

  systems/
    mod.rs
    cleanup.rs                      // EXISTING — cleanup markers
```

## Migration Table: ui/ → screen/

| Current Location | New Location | Action |
|-----------------|-------------|--------|
| `ui/mod.rs` | — | **Delete** (UiPlugin eliminated) |
| `ui/plugin.rs` | — | **Delete** (systems absorbed into screen sub-plugins) |
| `ui/sets.rs` | — | **Delete** (UiSystems set no longer needed) |
| `ui/components.rs` | — | **Delete** (SidePanelMarker etc. eliminated — side panels removed) |
| `ui/resources.rs` | — | **Delete** (TimerUiDefaults → replaced by timer_wall system reading VfxConfig) |
| `ui/messages.rs` | `screen/chip_select/` | **Move** ChipSelected message to chip_select module (already the only consumer) |
| `ui/systems/spawn_side_panels.rs` | — | **Delete** (side panels removed per architecture) |
| `ui/systems/spawn_timer_hud.rs` | `screen/playing/hud/timer_wall.rs` | **Rewrite** — current Node-based HUD replaced by diegetic timer wall entity |
| `ui/systems/update_timer_display.rs` | `screen/playing/hud/timer_wall.rs` | **Rewrite** — current Text-based timer replaced by shader-driven gauge glow |

## Migration Table: fx/ → screen/ + rantzsoft_vfx

| Current Location | New Location | Action |
|-----------------|-------------|--------|
| `fx/mod.rs` | — | **Delete** (FxPlugin eliminated) |
| `fx/plugin.rs` | — | **Delete** (systems distributed) |
| `fx/components.rs` (FadeOut, PunchScale) | `rantzsoft_vfx` | **Move to crate** — generic animation primitives |
| `fx/systems/animate_fade_out.rs` | `rantzsoft_vfx` | **Move to crate** — generic fade-out system |
| `fx/systems/animate_punch_scale.rs` | `rantzsoft_vfx` | **Move to crate** — generic punch scale system |
| `fx/transition/mod.rs` | `screen/transition/mod.rs` | **Rewrite** — current simple fade replaced by 4 transition styles |
| `fx/transition/system.rs` | `screen/transition/*.rs` | **Rewrite** — transition animation systems using rantzsoft_vfx primitives |
| `fx/transition/tests.rs` | `screen/transition/` | **Rewrite** — tests for new transition system |

## Migration Table: Existing screen/ Renames

| Current Location | New Location | Action |
|-----------------|-------------|--------|
| `screen/pause_menu/` | `screen/pause/` | **Rename** module (shorter, consistent) |
| `screen/chip_select/` | `screen/chip_select/` | **Keep** (already correct) |
| `screen/main_menu/` | `screen/main_menu/` | **Keep** |
| `screen/run_setup/` | `screen/run_setup/` | **Keep** |
| `screen/run_end/` | `screen/run_end/` | **Keep** |
| `screen/loading/` | `screen/loading/` | **Keep** |
| `screen/systems/cleanup.rs` | `screen/systems/cleanup.rs` | **Keep** |

## game.rs Plugin Changes

```rust
// Remove:
app.add_plugins(UiPlugin);
app.add_plugins(FxPlugin);

// Modify (ScreenPlugin absorbs per-screen UI + transitions):
app.add_plugins(ScreenPlugin);  // already exists, gains sub-plugins

// Add:
app.add_plugins(RantzVfxPlugin::default());
// GraphicsConfig registered as a shared resource (not a plugin)
```

## PlayingState Substate Migration

`TransitionOut`, `ChipSelect`, and `TransitionIn` move from `GameState` to `PlayingState` substates. See [transitions.md](transitions.md) for the full rationale.

Current `FxPlugin` registers transitions against `GameState::TransitionOut` / `GameState::TransitionIn`. After migration, `screen/transition/` registers against `PlayingState::TransitionOut` / `PlayingState::TransitionIn`.

This is a breaking change to state machine wiring — all systems that `run_if(in_state(GameState::TransitionOut))` must be updated to `run_if(in_state(PlayingState::TransitionOut))`.

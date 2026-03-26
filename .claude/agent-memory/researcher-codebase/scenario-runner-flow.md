---
name: scenario-runner-flow
description: End-to-end scenario runner lifecycle, input injection, breaker/layout setup, overclock injection, state transitions, and invariant checking flow map
type: reference
---

# Scenario Runner Flow Map

## Crate: breaker-scenario-runner (Bevy 0.18)

### Entry Points

- `main.rs` -- CLI parsing, discovery, serial/parallel/stress execution
- `runner/app.rs::run_scenario()` -- builds Bevy App, inserts ScenarioConfig, adds ScenarioLifecycle plugin, runs app loop

### App Construction (runner/app.rs::build_app)

- Headless: MinimalPlugins + StatesPlugin + AssetPlugin (game assets path) + InputPlugin + ManualDuration(1/64s) + Game::headless()
- Visual: DefaultPlugins + ManualDuration(10/64s) + Game::default()
- Game::headless() includes HeadlessAssetsPlugin (MeshPlugin, ColorMaterial, TextPlugin) but excludes DebugPlugin
- Game::default() includes RenderSetupPlugin (camera) and DebugPlugin

### ScenarioLifecycle Plugin (lifecycle/mod.rs)

Inserts resources: ScenarioFrame, ViolationLog, PreviousGameState, EntityLeakBaseline, ScenarioStats
Adds message: SpawnNodeComplete

#### State Transition Chain

1. App starts in GameState::Loading (default)
2. Game's asset loading completes -> transitions to GameState::MainMenu
3. `OnEnter(MainMenu)` -> `bypass_menu_to_playing`:
   - Sets `SelectedBreaker.0` from config.definition.breaker
   - Sets `ScenarioLayoutOverride.0 = Some(config.definition.layout)`
   - Sets `RunSeed.0 = Some(config.definition.seed.unwrap_or(0))`
   - Pushes initial_overclocks into breaker's EffectChains (if present)
   - Sets NextState to GameState::Playing
4. `OnEnter(Playing)` -> chained sequence:
   - `init_scenario_input` (creates InputDriver from strategy, inserts ScenarioInputDriver)
   - ApplyDeferred
   - `tag_game_entities` (tags Bolt/Breaker entities with ScenarioTagBolt/ScenarioTagBreaker)
   - ApplyDeferred
   - `apply_debug_setup` (teleports, freezes, extra bolts, timer override, previous state override)
   - Ordering: after BoltSystems::InitParams, BreakerSystems::Reset, NodeSystems::InitTimer
5. `OnEnter(ChipSelect)` -> `auto_skip_chip_select` -> NextState::TransitionIn
6. `OnEnter(RunEnd)` -> either `exit_on_run_end` (allow_early_end=true) or `restart_run_on_end` (allow_early_end=false -> MainMenu -> Playing loop)

### Input Injection Chain

Schedule placement: FixedPreUpdate

1. Game's `clear_input_actions` runs in FixedPostUpdate (clears previous tick)
2. Game's `read_input_actions` runs in PreUpdate (reads real keyboard -- produces nothing in headless)
3. Scenario's `inject_scenario_input` runs in FixedPreUpdate:
   - Reads ScenarioInputDriver (wraps InputDriver enum)
   - Calls `driver.actions_for_frame(frame, true)` -- true = is_active
   - Maps each ScenarioGameAction to game's GameAction via `map_action()`
   - Pushes mapped actions into `InputActions.0`
   - Updates ScenarioStats.actions_injected

InputDriver dispatches to:
- ChaosDriver: seeded SmallRng, action_prob roll, chooses from GAMEPLAY_ACTIONS (MoveLeft, MoveRight, Bump, DashLeft, DashRight, TogglePause)
- ScriptedInput: lookup by frame number, returns Vec<GameAction>
- HybridInput: silent scripted phase (0..scripted_frames), then delegates to ChaosDriver

### Breaker Loading

The runner does NOT construct breaker entities directly. It:
1. Sets `SelectedBreaker.0` to the breaker name string
2. The game's breaker plugin reads SelectedBreaker to look up BreakerRegistry and spawn/configure the breaker
3. The runner tags the spawned breaker with ScenarioTagBreaker in tag_game_entities

### Layout Loading

The runner does NOT spawn cells directly. It:
1. Sets `ScenarioLayoutOverride.0 = Some(layout_name)`
2. The game's node system reads ScenarioLayoutOverride in `set_active_layout` to use the named layout instead of index-based selection
3. The game's node system spawns cells from the layout via NodeLayoutRegistry::get_by_name
4. SpawnNodeComplete message fires when all domain spawns are done

### Initial Overclocks

In `bypass_menu_to_playing`:
- Queries `Query<&mut EffectChains, With<Breaker>>`
- For each overclock in config.definition.initial_overclocks:
  - Pushes `(None, overclock_chain.clone())` to EffectChains.0
  - None = no chip_name (breaker-originating chain)
- This runs in OnEnter(MainMenu), BEFORE Playing entry -- breaker entity must already exist

NOTE: This is a potential gap -- if breaker entity hasn't been spawned yet at MainMenu entry, the query finds nothing. Currently works because breaker is spawned at Startup/Loading time.

### Frame Counting and Gating

- `mark_entered_playing_on_spawn_complete`: listens for SpawnNodeComplete message, sets ScenarioStats.entered_playing = true
- `entered_playing` run condition: gates tick_scenario_frame and check_frame_limit
- Frame counting only starts AFTER SpawnNodeComplete fires
- Invariant checkers gated on `|stats| stats.entered_playing`

### Invariant Checking

All invariant checker systems run in FixedUpdate, chained, gated on entered_playing, ordered:
- after deferred_debug_setup
- after tag_game_entities
- after update_breaker_state
- before BoltSystems::BoltLost

### Seed Architecture

Two independent seeds:
1. `ChaosParams.seed` / `HybridParams.seed` -- seeds the input RNG (SmallRng)
2. `ScenarioDefinition.seed` -> `RunSeed.0` -- seeds the game's GameRng (ChaCha8Rng) for gameplay randomness (chip offerings, spawn angles, etc.)

Both default to deterministic values (input seed required in RON, game seed defaults to 0 via unwrap_or(0)).

### Key Types

- ScenarioDefinition: full RON-loaded scenario spec
- ScenarioConfig: Resource wrapping ScenarioDefinition
- ScenarioInputDriver: Resource wrapping InputDriver
- InputDriver: enum (Chaos/Scripted/Hybrid) wrapping strategy implementations
- ScenarioFrame: Resource(u32) counting fixed-update ticks
- ScenarioStats: entered_playing, bolts_tagged, breakers_tagged, etc.
- ViolationLog: Vec<ViolationEntry> for invariant violations
- ScenarioVerdict: pass/fail evaluation with reasons

### Exit Conditions

1. ScenarioFrame >= max_frames -> AppExit::Success
2. GameState::RunEnd (if allow_early_end) -> AppExit::Success (every Update frame)
3. Wall-clock timeout (5 min) -> forced exit in headless run loop
4. System panic -> caught by guarded_update, forced exit

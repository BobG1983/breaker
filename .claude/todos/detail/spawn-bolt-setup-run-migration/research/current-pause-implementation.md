## Behavior Trace: Pause System — PlayingState Machine

Bevy version: **0.18** (confirmed from `CLAUDE.md` and project context; `Cargo.toml` has no workspace-level Bevy pin — game crate uses it directly).

---

### 1. `PlayingState` Definition

**File:** `breaker-game/src/shared/playing_state.rs`

```rust
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum PlayingState {
    #[default]
    Active,
    Paused,
}
```

- `PlayingState` is a **SubState** of `GameState::Playing`.
- It only exists (and is meaningful) while `GameState::Playing` is active.
- It defaults to `Active` when `GameState::Playing` is entered.
- Registered in `ScreenPlugin` via `.add_sub_state::<PlayingState>()`.

`GameState` variants relevant to context: `Loading → MainMenu → RunSetup → Playing → TransitionOut → ChipSelect → TransitionIn → RunEnd`. `Playing` is the only state where `PlayingState` is active.

---

### 2. Trigger: How Pause Is Initiated

#### Step 1 — Raw keyboard → `GameAction::TogglePause`

**File:** `breaker-game/src/input/systems/read_input.rs` — `read_input_actions`
**Schedule:** `PreUpdate` (after `InputSystems`)

- Reads `MessageReader<KeyboardInput>` (one-shot press events).
- Hardcoded: `key == KeyCode::Escape` → pushes `GameAction::TogglePause` into `InputActions`.
- Key-repeat events are explicitly skipped (`event.repeat` check).
- **Not configurable via `InputConfig`** — Escape is hardwired.

`InputActions` is cleared at `FixedPostUpdate` by `clear_input_actions`, so it persists from `PreUpdate` through the entire `FixedUpdate` step.

#### Step 2 — `GameAction::TogglePause` → `PlayingState` transition

**File:** `breaker-game/src/screen/pause_menu/systems/toggle_pause.rs` — `toggle_pause`
**Schedule:** `Update` — `run_if(in_state(GameState::Playing))`

- Reads `Res<InputActions>`.
- Reads `Res<State<PlayingState>>` to check current state.
- Writes `ResMut<NextState<PlayingState>>`:
  - `Active → Paused`
  - `Paused → Active`
- State transition takes effect next frame (Bevy's deferred state system).

---

### 3. Systems Gated on `PlayingState::Active`

All gameplay systems use `run_if(in_state(PlayingState::Active))`. When paused, none of these run.

#### FixedUpdate systems (physics, game logic, timers)

| Plugin / Domain | Systems in set |
|---|---|
| **BoltPlugin** | `launch_bolt`, `hover_bolt`, `spawn_bolt_lost_text`, `dispatch_bolt_effects`, `bolt_cell_collision`, `bolt_wall_collision`, `bolt_breaker_collision`, `clamp_bolt_to_playfield`, `bolt_lost`, `tick_bolt_lifespan`, `cleanup_destroyed_bolts` — the **entire FixedUpdate set** |
| **BreakerPlugin** | `update_bump`, `move_breaker`, `update_breaker_state`, `grade_bump`, `perfect_bump_dash_cancel`, `spawn_bump_grade_text`, `spawn_whiff_text`, `trigger_bump_visual`, `breaker_cell_collision`, `breaker_wall_collision` — the **entire FixedUpdate set** |
| **CellsPlugin** | `handle_cell_hit`, `check_lock_release`, `tick_cell_regen`, `rotate_shield_cells`, `sync_orbit_cell_positions`, `cleanup_cell`, `cell_wall_collision` — the **entire FixedUpdate set** |
| **RunPlugin** | `handle_node_cleared`, `handle_timer_expired`, `handle_run_lost`, `track_cells_destroyed`, `track_bumps`, `track_bolts_lost`, `track_time_elapsed`, `track_evolution_damage`, `track_node_cleared_stats`, `detect_mass_destruction`, `detect_close_save`, `detect_combo_king`, `detect_pinball_wizard`, `detect_nail_biter` — the **entire FixedUpdate set** |
| **NodePlugin** | `track_node_completion`, `tick_node_timer`, `reverse_time_penalty`, `apply_time_penalty` — the **entire FixedUpdate set** |
| **TelemetryPlugin** (debug) | `track_bump_result` |
| **Effect triggers (all in FixedUpdate)** | `bridge_bump`, `bridge_perfect_bump`, `bridge_early_bump`, `bridge_late_bump`, `bridge_bump_whiff`, `bridge_bumped`, `bridge_perfect_bumped`, `bridge_early_bumped`, `bridge_late_bumped`, `bridge_impact` (all variants), `bridge_impacted`, `bridge_cell_destroyed`, `bridge_death`, `bridge_died`, `bridge_bolt_lost`, `bridge_node_end`, `tick_time_expires`, `desugar_until` |
| **Effect effects (all in FixedUpdate)** | `tick_gravity_well`, `apply_gravity_pull`, `tick_anchor`, `maintain_tether_chain` (compound: also `resource_exists::<TetherChainActive>`), `tick_tether_beam`, `tick_chain_lightning`, `tick_pulse`, `tick_shockwave` (various), `tick_piercing_beam` (various), `tick_explode`, `tick_attraction`, `despawn_second_wind_on_contact` |

#### Update systems (visual/UI, not physics)

| Plugin | System | Schedule |
|---|---|---|
| **BoltPlugin** | `sync_bolt_scale` | Update |
| **BreakerPlugin** | `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale` | Update |
| **FxPlugin** | `animate_fade_out`, `animate_punch_scale` | Update |
| **UiPlugin** | `update_timer_display` | Update |
| **RunPlugin** | `spawn_highlight_text` | Update |

---

### 4. Systems Gated on `PlayingState::Paused`

Only **one system** uses `run_if(in_state(PlayingState::Paused))`:

| Plugin | System | Schedule |
|---|---|---|
| **PauseMenuPlugin** | `handle_pause_input` | Update |

`handle_pause_input` handles pause menu navigation (Up/Down keys, Enter) and two resume paths:
- "Resume" → `next_playing_state.set(PlayingState::Active)`
- "Quit" → `next_game_state.set(GameState::MainMenu)` (bypasses unpause entirely)

---

### 5. `OnEnter(PlayingState::Paused)` / `OnExit(PlayingState::Paused)`

| Hook | System | Plugin | What it does |
|---|---|---|---|
| `OnEnter(PlayingState::Paused)` | `spawn_pause_menu` | `PauseMenuPlugin` | Spawns the full-screen overlay UI (semi-transparent black background, "PAUSED" title, "Resume" / "Quit to Menu" items). Also inserts `PauseMenuSelection` resource with `Resume` pre-selected. |
| `OnExit(PlayingState::Paused)` | `cleanup_entities::<PauseMenuScreen>` | `PauseMenuPlugin` | Despawns all entities marked with `PauseMenuScreen`. Fires on both resume (→ Active) and quit (→ GameState::MainMenu triggers state exit). |

**Observation on quit path:** When the player selects "Quit", `next_game_state.set(GameState::MainMenu)` is called. This transitions `GameState` out of `Playing`, which also deactivates `PlayingState`. `OnExit(PlayingState::Paused)` still fires because the substate exits before the parent state does, so `cleanup_entities::<PauseMenuScreen>` runs correctly on both resume and quit.

---

### 6. `OnEnter(PlayingState::Active)` (side effects of unpausing)

Two systems run on `OnEnter(PlayingState::Active)`:

| System | Location | What it does | When it fires |
|---|---|---|---|
| `bridge_node_start` | `effect/triggers/node_start.rs` | Fires `Trigger::NodeStart` globally on all entities with `BoundEffects` and `StagedEffects` | On initial node start **AND on every unpause** |
| `reset_entropy_engine_on_node_start` | `effect/effects/entropy_engine/effect.rs` | Resets `EntropyEngineState.cells_destroyed` to 0 on all entities | On initial node start **AND on every unpause** |

**Critical edge case:** Both `OnEnter(PlayingState::Active)` hooks fire both when the node first loads (initial transition from `GameState` → `Playing` defaults `PlayingState` to `Active`) AND every time the game is unpaused. This means:
- `Trigger::NodeStart` is re-fired globally every time the player unpauses.
- `EntropyEngine` cell-destroyed counters are reset to 0 every time the player unpauses.

This is likely a bug: chips that trigger on `NodeStart` will fire their effects again on every resume, and the `EntropyEngine` state loss on unpause could affect difficulty scaling.

---

### 7. `Time<Virtual>` and Timer Pausing

**No `Time<Virtual>` pausing is used.** There are no calls to:
- `time.set_relative_speed(0.0)`
- `time.pause()`
- Any manipulation of `Time<Virtual>`

The pause implementation is **purely state-gated** — the `PlayingState::Active` condition on `run_if` prevents all gameplay `FixedUpdate` systems from running. The Bevy `FixedUpdate` schedule itself still runs each frame, but none of the gated systems execute.

`Time<Real>` (wall clock) continues unaffected during pause, as intended — it is used only for double-tap dash detection in `read_input_actions`, which explicitly wants real-world time.

The node timer (`tick_node_timer` in `NodePlugin`) is in the `PlayingState::Active`-gated set, so the countdown stops during pause.

---

### 8. FixedUpdate vs Update — Which Systems Are Gated

**FixedUpdate systems** — ALL gated on `PlayingState::Active`:
- Bolt physics, collision, lifespan
- Breaker movement, collision, bump grading
- Cell hit/regen/rotation/cleanup
- Node timer, completion tracking, time penalties
- Run state machines (node cleared, run lost, timer expired)
- All effect trigger bridges (bump, impact, death, timer, etc.)
- All effect timers (gravity well, anchor, pulse, shockwave, etc.)

**FixedUpdate system NOT gated on PlayingState** (intentional exception):
- `check_spawn_complete` (NodePlugin) — explicitly documented: "Intentionally runs without PlayingState::Active guard — must catch spawn signals on the first frame of play."
- `bridge_no_bump` — stub, unimplemented placeholder (no PlayingState guard, no-op).

**Update systems gated on `PlayingState::Active`**:
- `sync_bolt_scale`, `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale`
- `animate_fade_out`, `animate_punch_scale`
- `update_timer_display`
- `spawn_highlight_text`

**Update systems NOT gated on PlayingState** (run regardless of pause):
- `toggle_pause` (gated only on `GameState::Playing`)
- `handle_pause_input` (gated on `PlayingState::Paused`)
- `read_input_actions` (PreUpdate — no state gate; populates InputActions always)
- `clear_input_actions` (FixedPostUpdate — no state gate; always clears)
- `animate_transition` (gated on `GameState::TransitionOut` or `TransitionIn`)
- Debug/egui panels (`debug_ui_system`, `bolt_info_ui`, etc.) — gated on `resource_exists::<DebugOverlays>` only

---

### System Chain — Pause Flow

```
[Keyboard: Escape]
  └─ PreUpdate: read_input_actions → InputActions += TogglePause

[Update]: toggle_pause (run_if GameState::Playing)
  └─ reads InputActions.active(TogglePause)
  └─ NextState<PlayingState> = Paused
  └─ [next frame] state flush: PlayingState → Paused

[State transition frame]:
  OnEnter(PlayingState::Paused):
    spawn_pause_menu → spawns PauseMenuScreen entity + PauseMenuSelection resource

[While Paused]:
  ALL FixedUpdate gameplay systems: SKIP (run_if Active → false)
  ALL Update gameplay systems: SKIP (run_if Active → false)
  Update: handle_pause_input (run_if Paused) → reads ButtonInput<KeyCode> directly
    - Arrow keys: navigate PauseMenuSelection
    - Enter/Space:
        Resume → NextState<PlayingState> = Active
        Quit   → NextState<GameState> = MainMenu

[Resume path]:
  NextState<PlayingState> = Active
  [next frame] state flush:
    OnExit(PlayingState::Paused): cleanup_entities::<PauseMenuScreen> → despawn overlay
    OnEnter(PlayingState::Active):
      bridge_node_start → re-fires Trigger::NodeStart on all BoundEffects entities  ← EDGE CASE
      reset_entropy_engine_on_node_start → resets cells_destroyed to 0              ← EDGE CASE

[Quit path]:
  NextState<GameState> = MainMenu
  [next frame] state flush:
    PlayingState exits (substate deactivated because parent exits)
    OnExit(PlayingState::Paused): cleanup_entities::<PauseMenuScreen> → despawn overlay
```

---

### Edge Cases

1. **`OnEnter(PlayingState::Active)` fires on every unpause** — `bridge_node_start` re-fires `Trigger::NodeStart` globally each time the game is unpaused, not only at node start. Any chip that triggers `When(NodeStart, Do(X))` will activate again on every resume.

2. **`EntropyEngine` counter reset on unpause** — `reset_entropy_engine_on_node_start` runs on `OnEnter(PlayingState::Active)`, so unpausing mid-node resets the escalating chaos counter to zero.

3. **`handle_pause_input` reads `ButtonInput<KeyCode>` directly** (not `InputActions`) — inconsistent with `toggle_pause` which uses `InputActions`. This means `handle_pause_input` does not benefit from the double-buffering/FixedUpdate safety of `InputActions`.

4. **Escape is hardwired in `read_input_actions`** and is not present in `InputConfig` — the pause key cannot be rebound via the config system.

5. **`bridge_no_bump` runs during pause** — it is a no-op stub but lacks a `PlayingState::Active` guard. This is harmless now but will need the guard added when implemented.

6. **`check_spawn_complete` runs during pause** — intentional, so spawn signals aren't missed, but it means spawn coordination works even while paused. Practically safe since spawning only happens on `OnEnter(GameState::Playing)`.

7. **`InputActions` is not cleared during pause** — `clear_input_actions` runs in `FixedPostUpdate` unconditionally. If the player presses Escape while paused, the `TogglePause` action is populated in `InputActions` by `read_input_actions` (PreUpdate, no state gate), but `toggle_pause` (Update, gated on `GameState::Playing`) will still see it. Since `GameState::Playing` remains true while paused, the Escape key correctly triggers `toggle_pause` during a pause session as well — this is the intended unpause path. (The `handle_pause_input` system also provides unpause via the Resume menu item.)

8. **Double unpause paths** — both `toggle_pause` (Escape key, runs during `Paused` state because the guard is only `GameState::Playing`) and `handle_pause_input` (Resume menu item) can transition to `PlayingState::Active`. No deduplication needed since `NextState::set` is idempotent.

---

### Key Files

- `breaker-game/src/shared/playing_state.rs` — `PlayingState` enum definition, SubState derivation
- `breaker-game/src/shared/game_state.rs` — `GameState` top-level state machine
- `breaker-game/src/screen/pause_menu/plugin.rs` — pause menu plugin wiring (OnEnter/OnExit hooks, system gates)
- `breaker-game/src/screen/pause_menu/systems/toggle_pause.rs` — Escape-key toggle system
- `breaker-game/src/screen/pause_menu/systems/handle_pause_input.rs` — pause menu navigation + resume/quit
- `breaker-game/src/screen/pause_menu/systems/spawn_pause_menu.rs` — overlay UI spawn
- `breaker-game/src/input/systems/read_input.rs` — hardwired Escape → `GameAction::TogglePause`
- `breaker-game/src/input/plugin.rs` — `read_input_actions` in PreUpdate, `clear_input_actions` in FixedPostUpdate
- `breaker-game/src/effect/triggers/node_start.rs` — fires on `OnEnter(PlayingState::Active)`, re-fires on every unpause
- `breaker-game/src/effect/effects/entropy_engine/effect.rs` — resets counter on `OnEnter(PlayingState::Active)`
- `breaker-game/src/bolt/plugin.rs` — entire bolt FixedUpdate set gated on Active
- `breaker-game/src/breaker/plugin.rs` — entire breaker FixedUpdate set gated on Active
- `breaker-game/src/cells/plugin.rs` — entire cells FixedUpdate set gated on Active
- `breaker-game/src/run/node/plugin.rs` — node timer + `check_spawn_complete` (ungated, intentional)
- `breaker-game/src/run/plugin.rs` — run stats, highlights, node/run lifecycle gated on Active

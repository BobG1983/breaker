# Migration Map — test_app() to TestAppBuilder

Total functions catalogued: 141 across 129 files (some files have 2–6 variants).

---

## Pattern Categories

### Pattern A: Minimal — single system, no messages, no state (24 files)

`MinimalPlugins` + one or two systems. No messages, no state hierarchy, no physics.

**Canonical builder call (Update schedule):**
```rust
TestAppBuilder::new()
    .with_system(Update, my_system)
    .build()
```

**Canonical builder call (FixedUpdate schedule):**
```rust
TestAppBuilder::new()
    .with_system(FixedUpdate, my_system)
    .build()
```

**Files:**

| File | Deviations from canonical |
|------|--------------------------|
| `src/bolt/systems/normalize_speed_after_constraints.rs` | FixedUpdate |
| `src/bolt/systems/sync_bolt_scale/tests.rs` | Update |
| `src/bolt/systems/tick_birthing/tests.rs` | FixedUpdate |
| `src/bolt/systems/begin_node_birthing.rs` | Update |
| `src/breaker/systems/sync_breaker_scale/tests.rs` | FixedUpdate |
| `src/breaker/builder/tests/spawn_tests.rs` | MinimalPlugins only — no system added at construction (systems added per-test inline) |
| `src/walls/builder/tests/spawn_tests.rs` | MinimalPlugins only — no system added at construction |
| `src/cells/behaviors/regen/systems/tick_cell_regen.rs` | FixedUpdate |
| `src/cells/behaviors/guarded/systems/slide_guardian_cells/tests.rs` | FixedUpdate |
| `src/state/run/node/systems/all_animate_in_complete.rs` | Update; also registers `ChangeState<NodeState>` message |
| `src/state/run/node/systems/apply_node_scale_to_breaker.rs` | Update |
| `src/state/run/node/systems/apply_node_scale_to_bolt.rs` | Update |
| `src/state/run/node/systems/init_clear_remaining.rs` | Startup schedule |
| `src/state/run/node/lifecycle/systems/reset_highlight_tracker.rs` | Update; also inits `HighlightTracker` resource |
| `src/state/pause/systems/toggle_pause.rs` | Update; also inits `ButtonInput<KeyCode>` |
| `src/state/pause/systems/spawn_pause_menu.rs` | Update |
| `src/state/pause/systems/update_pause_menu_colors.rs` | Update; also inserts `PauseMenuSelection` resource |
| `src/state/run/node/hud/systems/spawn_side_panels.rs` | Update |
| `src/state/menu/start_game/systems/spawn_run_setup.rs` | Update; takes `registry: BreakerRegistry` param, inserts it |
| `src/state/menu/start_game/systems/update_run_setup_colors.rs` | Update; takes `selection_index: usize` param, inserts `RunSetupSelection` |
| `src/state/menu/start_game/systems/update_seed_display.rs` | Update; also inits `SeedEntry` resource |
| `src/state/menu/start_game/systems/handle_seed_input.rs` | Update; registers `KeyboardInput` message, inits `SeedEntry` |
| `src/state/run/run_end/systems/spawn_run_end_screen/tests/helpers.rs` | Update; takes `result: NodeResult` param, inserts `NodeOutcome`. `test_app_with_stats` adds `RunStats`. |
| `src/breaker/systems/bump_visual/tests.rs` (`animate_test_app`) | FixedUpdate; `animate_bump_visual` system |

**Notes on this pattern:**
- The `ChangeState<NodeState>` message in `all_animate_in_complete` is just `add_message` — fits as `.with_message::<ChangeState<NodeState>>()`.
- `reset_highlight_tracker` needs `.with_resource::<HighlightTracker>()`.
- `toggle_pause` and `handle_pause_input` need `.insert_resource(ButtonInput::<KeyCode>::default())` — not currently a builder method; use `.insert_resource(...)`.
- `spawn_run_end_screen` test_app_with_stats delegates to `test_app(result)` and inserts `RunStats` on top — straightforward chain.

---

### Pattern B: Minimal + resource(s), single system, no messages (12 files)

`MinimalPlugins` + one or two resources + single system. No messages, no state, no physics.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_resource::<MyResource>()       // or .insert_resource(value)
    .with_system(FixedUpdate, my_system)
    .build()
```

**Files:**

| File | Resources | Deviations |
|------|-----------|------------|
| `src/bolt/systems/bolt_lost/tests/helpers.rs` | `PlayfieldConfig` (init), `GameRng` (init) | FixedUpdate; also registers `BoltLost` message → actually Pattern C |
| `src/bolt/systems/clamp_bolt_to_playfield/tests.rs` | `PlayfieldConfig` (init) | FixedUpdate |
| `src/bolt/systems/launch_bolt/tests.rs` | `InputActions` (init), `GameRng` (init) | FixedUpdate |
| `src/state/run/systems/advance_node.rs` | `NodeOutcome` (inserted with values) | Update |
| `src/state/run/node/systems/tick_node_timer.rs` | `NodeTimer` (inserted with values) | FixedUpdate; also registers `TimerExpired` message → Pattern C |
| `src/state/run/node/systems/init_node_timer.rs` | `ActiveNodeLayout` (inserted) | Startup |
| `src/state/run/node/systems/reset_bolt/tests.rs` | `NodeOutcome` (init), `GameRng` (init) | Update; also registers `BoltSpawned` message → Pattern C |
| `src/state/run/loading/systems/capture_run_seed.rs` | `RunStats`, `GameRng`, `RunSeed` (all init) | Update |
| `src/state/run/loading/systems/reset_run_state.rs` | `NodeOutcome` (inserted with values), `GameRng`, `RunSeed`, `ChipInventory`, `RunStats`, `HighlightTracker` (all init) | Update |
| `src/state/run/node/systems/set_active_layout.rs` | `NodeOutcome` (inserted), `NodeLayoutRegistry` (built), `ScenarioLayoutOverride` (default) | Startup; takes params |
| `src/state/run/node/systems/dispatch_cell_effects/tests/helpers.rs` | `CellTypeRegistry` (param, inserted) | Update |
| `src/state/run/chip_select/systems/track_chips_collected.rs` | `RunStats` (init) | Update; also registers `ChipSelected` message → Pattern C |
| `src/state/run/node/tracking/systems/track_time_elapsed.rs` | `RunStats` (init) | FixedUpdate |
| `src/debug/hot_reload/systems/propagate_bolt_definition/tests/helpers.rs` | `BoltRegistry` (init) | Update |
| `src/debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | `BreakerRegistry` (init), `SelectedBreaker` (init) | Update |
| `src/breaker/queries/tests/helpers.rs` | `QueryMatched` (init) | FixedUpdate |

**Note:** Several files here also register messages; they are more accurately Pattern C. The overlap is noted above.

---

### Pattern C: Message-only bridge — enqueue/bridge system pair, single message (21 functions across 17 files)

`MinimalPlugins` + `add_message` for one message type + (enqueue helper, bridge system) ordered pair. No state, no physics.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_message::<MyMessage>()
    .with_system(FixedUpdate, (enqueue_helper.before(bridge_system), bridge_system))
    .build()
```

**Files:**

| File | Message(s) | Deviations |
|------|-----------|------------|
| `src/effect/triggers/bumped.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/bump/tests.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/perfect_bumped.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/early_bumped.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/late_bumped.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/early_bump.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/perfect_bump.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/late_bump.rs` | `BumpPerformed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/bump_whiff.rs` | `BumpWhiffed` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/node_end.rs` | `NodeCleared` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/bolt_lost.rs` | `BoltLost` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/cell_destroyed.rs` | `CellDestroyedAt` | FixedUpdate; enqueue+bridge pair |
| `src/effect/triggers/died.rs` | `RequestCellDestroyed` + `RequestBoltDestroyed` | FixedUpdate; two enqueue helpers, one bridge |
| `src/effect/triggers/death.rs` | `RequestCellDestroyed` + `RequestBoltDestroyed` | FixedUpdate; two enqueue helpers, one bridge |
| `src/effect/triggers/timer.rs` | none (no message) | FixedUpdate; single `tick_time_expires` system; no enqueue — MinimalPlugins only |
| `src/effect/triggers/until/tests/helpers.rs` | none | FixedUpdate; calls `register(&mut app)` which adds `desugar_until` to FixedUpdate |
| `src/effect/triggers/evaluate/tests/bound_and_staged/helpers.rs` | none | MinimalPlugins only + `Snapshot` resource; no system added at construction |

**Impact/Impacted sub-pattern — 6 variants × 2 files (12 functions total):**

Each of the six collision pair types (bolt-cell, bolt-wall, bolt-breaker, breaker-cell, breaker-wall, cell-wall) gets its own `test_app_*()` helper in both `impact/tests/helpers.rs` and `impacted/tests/helpers.rs`. All follow the same pattern: `MinimalPlugins` + `add_message::<XImpactY>()` + enqueue/bridge pair in FixedUpdate.

| File | Functions |
|------|-----------|
| `src/effect/triggers/impact/tests/helpers.rs` | `test_app_bolt_cell`, `test_app_bolt_wall`, `test_app_bolt_breaker`, `test_app_breaker_cell`, `test_app_breaker_wall`, `test_app_cell_wall` |
| `src/effect/triggers/impacted/tests/helpers.rs` | `test_app_bolt_cell`, `test_app_bolt_wall`, `test_app_bolt_breaker`, `test_app_breaker_cell`, `test_app_breaker_wall`, `test_app_cell_wall` |

**Canonical builder for impact/impacted:**
```rust
TestAppBuilder::new()
    .with_message::<BoltImpactCell>()
    .with_system(FixedUpdate, (enqueue_bolt_impact_cell.before(bridge_impact_bolt_cell), bridge_impact_bolt_cell))
    .build()
```

---

### Pattern D: Physics collision — RantzPhysics2dPlugin + message + collision system (7 files)

`MinimalPlugins` + `RantzPhysics2dPlugin` + one or two messages + collision system ordered after `PhysicsSystems::MaintainQuadtree`. No state.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_physics()
    .with_message::<BoltImpactCell>()
    .with_system(FixedUpdate, bolt_cell_collision.after(PhysicsSystems::MaintainQuadtree))
    .build()
```

**Files:**

| File | Messages | Deviations |
|------|---------|------------|
| `src/bolt/systems/bolt_cell_collision/tests/helpers.rs` (`test_app`) | `BoltImpactCell`, `DamageCell`, `BoltImpactWall` | No collector resources in canonical; 3 messages |
| `src/bolt/systems/bolt_cell_collision/tests/helpers.rs` (`test_app_with_damage_and_wall_messages`) | `BoltImpactCell`, `DamageCell`, `BoltImpactWall` | Also inserts `DamageCellMessages`, `WallHitMessages`, `FullHitMessages` resources + 3 collector systems in FixedUpdate after collision |
| `src/bolt/systems/bolt_breaker_collision/tests/helpers.rs` | `BoltImpactBreaker` | Single message |
| `src/bolt/systems/bolt_wall_collision/tests/helpers.rs` | `BoltImpactWall` | Also inserts `WallHitMessages` resource + `collect_wall_hits` system after collision |
| `src/breaker/systems/breaker_cell_collision.rs` | `BreakerImpactCell` | Also inserts `BreakerCellHitMessages` resource + `collect_breaker_cell_hits` system after collision |
| `src/breaker/systems/breaker_wall_collision/tests.rs` | `BreakerImpactWall` | Also inserts `BreakerWallHitMessages` resource + `collect_breaker_wall_hits` system after collision |
| `src/cells/systems/cell_wall_collision.rs` | `CellImpactWall` | Also inserts `CellWallHitMessages` resource + `collect_cell_wall_hits` system after collision |

**Note:** The collector resource+system pairs in the wall/breaker/cell collision tests are pre-`MessageCollector` patterns — they use bespoke resource types to capture messages. Under the new API, these become `.with_message_capture::<XImpactY>()`.

---

### Pattern E: Physics + DamageCell capture — AoE/beam effect damage tests (5 files, 8 functions)

`MinimalPlugins` + `RantzPhysics2dPlugin` + `DamageCell` message + bespoke `DamageCellCollector` resource + effect-processing system + collector system. Update schedule (not FixedUpdate). Some variants add `PlayfieldConfig` or `Assets<Mesh/ColorMaterial>`.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_physics()
    .with_message_capture::<DamageCell>()
    .with_system(Update, process_explode_requests)
    .build()
```

**Files:**

| File | Function | Deviations |
|------|---------|------------|
| `src/effect/effects/explode/tests/helpers.rs` | `damage_test_app()` | `process_explode_requests` system |
| `src/effect/effects/explode/tests/flash_visual_tests.rs` | `flash_test_app()` | Builds on `damage_test_app()`, adds `Assets<Mesh>` + `Assets<ColorMaterial>` |
| `src/effect/effects/shockwave/tests/helpers.rs` | `damage_test_app()` | `apply_shockwave_damage` system |
| `src/effect/effects/piercing_beam/tests/helpers.rs` | `piercing_beam_damage_test_app()` | `process_piercing_beam` system; also inserts `PlayfieldConfig` |
| `src/effect/effects/piercing_beam/tests/process_tests/flash_visual_tests.rs` | `flash_test_app()` | Builds on `piercing_beam_damage_test_app()`, adds `Assets<Mesh>` + `Assets<ColorMaterial>` |
| `src/effect/effects/tether_beam/tests/helpers.rs` | `damage_test_app()` | `tick_tether_beam` system |
| `src/effect/effects/pulse/tests/helpers.rs` | `damage_test_app()` | `apply_pulse_damage` system |
| `src/effect/effects/pulse/tests/helpers.rs` | `test_app()` | State hierarchy + `tick_pulse_emitter` + `tick_pulse_ring` + `despawn_finished_pulse_ring` — see Pattern F |

**Note:** The `DamageCellCollector` bespoke resource maps directly to `.with_message_capture::<DamageCell>()`. The flash visual variants need `.with_playfield()` to get `Assets<Mesh>` and `Assets<ColorMaterial>`.

---

### Pattern F: State-hierarchy + in_node_playing, single system (13 files)

`MinimalPlugins` + `StatesPlugin` + full state hierarchy (AppState → GameState → RunState → NodeState) + manually navigated to `NodeState::Playing` (or left at a higher level) + system under test. No physics.

**Canonical builder call (navigated to Playing):**
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    .in_node_playing()
    .with_system(Update, my_system)
    .build()
```

**Canonical builder call (state hierarchy but NOT navigated, system fires OnEnter):**
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    .with_system(OnEnter(NodeState::Playing), bridge_node_start)
    .build()
```

**Files:**

| File | Function | Navigation | Deviations |
|------|---------|-----------|------------|
| `src/effect/triggers/node_start.rs` | `test_app()` | Not navigated | `OnEnter(NodeState::Playing)` system; uses `StatesPlugin`, `AppState` + 3 sub-states |
| `src/effect/effects/attraction/tests/helpers.rs` | `test_app()` | Not navigated (enter_playing called separately per test) | Adds `CollisionQuadtree` resource; `apply_attraction` in Update |
| `src/effect/effects/attraction/tests/helpers.rs` | `test_app_with_manage()` | None (no state) | Registers 3 impact messages + `manage_attraction_types` in FixedUpdate — actually Pattern C |
| `src/effect/effects/gravity_well/tests/helpers.rs` | `test_app()` | Not navigated | `tick_gravity_well` + `apply_gravity_pull` in Update |
| `src/effect/effects/shockwave/tests/helpers.rs` | `test_app()` | Not navigated | `tick_shockwave` + `despawn_finished_shockwave` in Update |
| `src/effect/effects/pulse/tests/helpers.rs` | `test_app()` | Not navigated | `tick_pulse_emitter` + `tick_pulse_ring` + `despawn_finished_pulse_ring` in Update |
| `src/effect/effects/entropy_engine/tests/reset_tests.rs` | `test_app_with_reset()` | Not navigated (enter_playing called per test) | Uses `register(&mut app)` for entropy engine systems |
| `src/effect/effects/anchor/tests/helpers.rs` | `test_app()` | None | `tick_anchor` in Update only — no state at all |
| `src/effect/effects/anchor/tests/helpers.rs` | `test_app_fixed()` | None | `tick_anchor` in FixedUpdate — no state |
| `src/effect/effects/anchor/tests/helpers.rs` | `register_test_app()` | Navigated to Playing | Full state hierarchy, navigated manually |
| `src/chips/systems/dispatch_chip_effects/tests/helpers.rs` | `test_app()` | Navigated to `ChipSelectState::Selecting` | Uses `ChipSelectState` sub-state instead of `NodeState`; see Pattern G |
| `src/state/run/node/systems/apply_node_scale_to_breaker.rs` | `test_app()` | None | No state needed — plain Update |
| `src/state/run/node/systems/apply_node_scale_to_bolt.rs` | `test_app()` | None | No state needed — plain Update |

**Note:** Files where `enter_playing` is a separate helper (attraction, shockwave, pulse, entropy_engine, anchor) need `.in_node_playing()` called after `.with_state_hierarchy()`, not during construction. The builder's `.in_node_playing()` is equivalent to those helpers.

---

### Pattern G: Chip-select state hierarchy (4 files)

`MinimalPlugins` + `StatesPlugin` + state hierarchy through `ChipSelectState::Selecting` + chip resources + system under test. Distinct from Pattern F because uses `ChipSelectState` not `NodeState`.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    .in_chip_selecting()
    .with_message::<ChipSelected>()
    .with_resource::<ChipInventory>()
    .with_resource::<ChipCatalog>()
    .with_system(Update, dispatch_chip_effects)
    .build()
```

**Files:**

| File | Function | Messages | Deviations |
|------|---------|---------|------------|
| `src/chips/systems/dispatch_chip_effects/tests/helpers.rs` | `test_app()` | `ChipSelected` | Also inits `PendingChipSelections`; adds `send_chip_selections` before `dispatch_chip_effects` |
| `src/state/run/chip_select/systems/tick_chip_timer.rs` | `test_app(remaining)` | `ChangeState<ChipSelectState>` | Takes `remaining: f32`; inserts `ChipSelectTimer` |
| `src/state/run/chip_select/systems/tick_chip_timer.rs` | `test_app_with_offers(remaining, offers)` | `ChangeState<ChipSelectState>` | Adds `ChipOffers`, `ChipInventory`, `ChipSelectConfig` resources |
| `src/state/run/chip_select/systems/handle_chip_input/tests.rs` | `test_app()` | `ChipSelected`, `ChangeState<ChipSelectState>` | Delegates to `test_app_with_offers(make_offers(3))` |
| `src/state/run/chip_select/systems/handle_chip_input/tests.rs` | `test_app_with_offers(offers)` | `ChipSelected`, `ChangeState<ChipSelectState>` | `ButtonInput<KeyCode>`, `InputConfig`, selection + offers + `ChipInventory` + `ChipSelectConfig` resources |
| `src/state/run/chip_select/systems/handle_chip_input/tests.rs` | `test_app_with_evolution_inventory()` | same | Delegates to `test_app_with_offers`, seeds `ChipInventory` with stacks |
| `src/state/run/run_end/systems/handle_run_end_input.rs` | `test_app()` | `ChangeState<RunEndState>` | Uses `RunEndState` sub-state (not `ChipSelectState`); navigates to `RunState::RunEnd` |
| `src/state/pause/systems/handle_pause_input.rs` | `test_app()` | `ChangeState<NodeState>` | Uses `NodeState`; navigates to `RunState::Node`; adds `PauseMenuSelection`, `NodeOutcome`; pauses `Time<Virtual>` |
| `src/state/menu/main/systems/handle_main_menu_input.rs` | `test_app()` | `AppExit`, `ChangeState<MenuState>` | Uses `MenuState` sub-state; navigates to `GameState::Menu` |
| `src/state/menu/start_game/systems/handle_run_setup_input.rs` | `test_app()` | `ChangeState<MenuState>` | Navigates to `GameState::Menu`; `SelectedBreaker`, `RunSeed`, `SeedEntry`, `BreakerRegistry`, `RunSetupSelection`; spawns `BreakerCard` entities |

**Note:** This pattern covers all tests that need a state hierarchy but with a different terminal state than `NodeState::Playing`. The builder's `in_chip_selecting()` covers the chip-select path; menu/pause/run-end paths navigate manually via `NextState`.

---

### Pattern H: State-guarded node lifecycle — state hierarchy + NodeState navigation + message (7 files)

`MinimalPlugins` + `StatesPlugin` + state hierarchy navigated to `RunState::Node` (but NOT Playing — the system handles the transition into Playing) + messages + system. Tests state machine transitions.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    // Navigate to RunState::Node manually (not .in_node_playing())
    .with_message::<NodeCleared>()
    .with_message::<ChangeState<NodeState>>()
    .with_system(FixedUpdate, handle_node_cleared)
    .build()
```

**Files:**

| File | Function | Messages | Deviations |
|------|---------|---------|------------|
| `src/state/run/node/lifecycle/systems/handle_node_cleared.rs` | `test_app(node_index, layout_count)` | `NodeCleared`, `ChangeState<NodeState>` | Takes params; inserts `NodeLayoutRegistry`, `NodeOutcome`, `SendNodeCleared`; navigates to `RunState::Node` |
| `src/state/run/node/lifecycle/systems/handle_node_cleared.rs` | `test_app_with_sequence(node_index, layout_count, sequence_len)` | `NodeCleared`, `ChangeState<NodeState>` | Adds `NodeSequence` resource |
| `src/state/run/node/lifecycle/systems/handle_timer_expired.rs` | `test_app(result)` | `TimerExpired`, `ChangeState<NodeState>` | Takes `NodeResult` param; inserts `NodeOutcome`; navigates to `RunState::Node` |
| `src/state/run/node/lifecycle/systems/handle_run_lost.rs` | `test_app()` | `RunLost`, `ChangeState<NodeState>` | Inserts `NodeOutcome`, `SendRunLost`; navigates to `RunState::Node` |
| `src/state/run/chip_select/systems/detect_first_evolution.rs` | `test_app()` | `ChipSelected`, `HighlightTriggered` | No state hierarchy — plain MinimalPlugins; see note below |
| `src/state/run/chip_select/systems/snapshot_node_highlights/tests.rs` | `test_app()` | None | No state; plain MinimalPlugins + resources + Update |
| `src/state/transition/tests.rs` | `test_app()` | None | Uses `StatesPlugin` + `init_state::<GameState>()` only (not sub-states); inserts `TransitionConfig` + `GameRng` |

**Note:** `detect_first_evolution` and `snapshot_node_highlights` have no state hierarchy despite being in the chip_select domain — they are closer to Pattern B/C.

---

### Pattern I: Node lifecycle — registries + spawn system + playfield (8 files, 10 functions)

`MinimalPlugins` + playfield/cell config resources + `Assets<Mesh>` + `Assets<ColorMaterial>` + registry resource + spawn system in Startup. No state, no physics.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_playfield()
    .insert_resource(ActiveNodeLayout(layout))
    .insert_resource(cell_type_registry)
    .with_message::<CellsSpawned>()
    .with_system(Startup, spawn_cells_from_layout)
    .build()
```

**Files:**

| File | Function | Deviations |
|------|---------|------------|
| `src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | `test_app(layout)` | Takes `NodeLayout` param; uses default configs |
| `src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | `scaled_test_app(layout)` | Takes `NodeLayout` param; inserts RON-like configs instead of defaults |
| `src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | `test_app_with_sequence(layout)` | Adds `NodeOutcome` + `NodeSequence` resources |
| `src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | `test_app_with_toughness(layout, registry, tier, pos, is_boss)` | Takes 5 params; adds `ToughnessConfig`, `NodeOutcome`, `NodeSequence` |
| `src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | `test_app_with_non_required_cell(layout)` | Builds extended registry with "F" cell type |
| `src/state/run/node/systems/spawn_walls/tests/helpers.rs` | `test_app()` | Registers wall not cells; uses `WallsSpawned` message, `WallRegistry`, `PlayfieldConfig`; Update schedule |
| `src/state/run/systems/setup_run/tests/helpers.rs` | `test_app()` | Startup; registers `BreakerSpawned` + `BoltSpawned` messages; `BreakerRegistry` + `BoltRegistry` + `SelectedBreaker` + `GameRng` + `PlayfieldConfig` + `Assets` |
| `src/state/run/node/hud/systems/spawn_timer_hud.rs` | `test_app()` | Uses `AssetPlugin::default()` + `Font` asset; inserts `TimerUiConfig` + `NodeTimer`; Startup + Update |

---

### Pattern J: Tracking + message + RunStats/HighlightTracker (8 files)

`MinimalPlugins` + `add_message` for one or more messages + `RunStats` and/or `HighlightTracker` resources + enqueue helper + tracking system in FixedUpdate.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_message::<BumpPerformed>()
    .with_resource::<RunStats>()
    .with_resource::<HighlightTracker>()
    .with_system(FixedUpdate, (enqueue_messages, track_bumps).chain())
    .build()
```

**Files:**

| File | Messages | Resources | Deviations |
|------|---------|-----------|------------|
| `src/state/run/node/tracking/systems/track_bumps.rs` | `BumpPerformed` | `RunStats`, `HighlightTracker` | Standard |
| `src/state/run/node/tracking/systems/track_bolts_lost.rs` | `BoltLost` | `RunStats`, `HighlightTracker` | Standard |
| `src/state/run/node/tracking/systems/track_cells_destroyed.rs` | `CellDestroyedAt` | `RunStats` | No `HighlightTracker` |
| `src/state/run/node/tracking/systems/track_evolution_damage.rs` | `DamageCell` | `HighlightTracker` | No `RunStats` |
| `src/state/run/node/tracking/systems/track_node_cleared_stats/tests/helpers.rs` | `NodeCleared`, `HighlightTriggered` | `RunStats`, `HighlightTracker`, `NodeOutcome`, `HighlightConfig`, `NodeTimer` (inserted) | Multiple messages and resources |
| `src/state/run/chip_select/systems/detect_first_evolution.rs` | `ChipSelected`, `HighlightTriggered` | `RunStats`, `HighlightTracker`, `NodeOutcome`, `HighlightConfig` | Two messages; test chip registry resource |
| `src/state/run/run_end/systems/detect_most_powerful_evolution.rs` | `HighlightTriggered` | `RunStats`, `NodeOutcome`, `HighlightTracker` | No input message (reads from HighlightTracker directly) |
| `src/state/run/chip_select/systems/snapshot_node_highlights/tests.rs` | None | `RunStats`, `HighlightTracker`, `NodeOutcome`, `HighlightConfig` | No messages; Update schedule |

---

### Pattern K: Highlight detection — multiple messages + HighlightTriggered capture (5 files)

`MinimalPlugins` + 2-3 messages (at least one input message + `HighlightTriggered`) + `RunStats`, `HighlightTracker`, `NodeOutcome`, `HighlightConfig` + enqueue helpers + detect system + capture collector in FixedUpdate chain.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_message::<BoltImpactCell>()
    .with_message::<BoltImpactBreaker>()
    .with_message_capture::<HighlightTriggered>()
    .with_resource::<RunStats>()
    .with_resource::<HighlightTracker>()
    .with_resource::<NodeOutcome>()
    .insert_resource(HighlightConfig::default())
    .with_system(FixedUpdate, (
        (enqueue_bolt_hit_cell, enqueue_bolt_hit_breaker),
        detect_pinball_wizard,
        collect_highlight_triggered,
    ).chain())
    .build()
```

**Files:**

| File | Input Messages | Extra Resources | Deviations |
|------|--------------|-----------------|------------|
| `src/state/run/node/highlights/systems/detect_pinball_wizard.rs` | `BoltImpactCell`, `BoltImpactBreaker` | `TestBoltImpactCell`, `TestBoltImpactBreaker`, `CapturedHighlightTriggered` | Two injector resources |
| `src/state/run/node/highlights/systems/detect_close_save.rs` | `BumpPerformed` | `PlayfieldConfig` (height=1080.0), `CapturedHighlightTriggered` | PlayfieldConfig override |
| `src/state/run/node/highlights/systems/detect_combo_king.rs` | `CellDestroyedAt`, `BoltImpactBreaker` | `TestCellDestroyed`, `TestBoltImpactBreaker`, `CapturedHighlightTriggered` | Two injector resources |
| `src/state/run/node/highlights/systems/detect_nail_biter.rs` | `NodeCleared` | `PlayfieldConfig` (height=1080.0), `CapturedHighlightTriggered` | PlayfieldConfig override |
| `src/state/run/node/highlights/systems/detect_mass_destruction.rs` | `CellDestroyedAt` | `CapturedHighlightTriggered` | Standard |

**Note:** `CapturedHighlightTriggered` and the per-test injector resources are bespoke collector types. Under the new API: `CapturedHighlightTriggered` → `.with_message_capture::<HighlightTriggered>()`.

---

### Pattern L: Chip dispatch e2e — delegates to Pattern G test_app + entity setup (4 functions, 3 files)

These are not independent `test_app` functions — they call the canonical `test_app()` from `dispatch_chip_effects/tests/helpers.rs` (Pattern G), then insert chip definitions and spawn entities. They return tuples of `(App, Entity, ...)`.

**Files:**

| File | Function | Returns | Deviations |
|------|---------|---------|------------|
| `src/chips/systems/dispatch_chip_effects/tests/desugaring/e2e/all_bolts_tests.rs` | `setup_e2e_all_bolts_app()` | `(App, Entity, Entity, Entity, Entity)` | Calls `test_app()`, inserts chip def, spawns breaker + 3 bolts, selects chip |
| `src/chips/systems/dispatch_chip_effects/tests/desugaring/e2e/all_cells_tests.rs` | `setup_e2e_desugaring_app()` | `(App, Entity, Entity, Entity)` | Calls `test_app()`, inserts chip def, spawns breaker + 2 cells |
| `src/chips/systems/dispatch_chip_effects/tests/desugaring/e2e/all_cells_tests.rs` | `setup_e2e_all_cells_damage_boost_app()` | `(App, Entity, Entity, Entity, Entity)` | Calls `test_app()`, inserts chip def, spawns breaker + 3 cells |
| `src/chips/systems/dispatch_chip_effects/tests/desugaring/e2e/all_walls_tests.rs` | `setup_e2e_all_walls_app()` | `(App, Entity, Entity, Entity)` | Calls `test_app()`, inserts chip def, spawns breaker + 2 walls |

**Migration note:** These don't need to change directly — they will automatically inherit the migrated `test_app()` from their helpers module.

---

### Pattern M: Bump multi-variant — three helpers in one file (1 file, 3 functions)

`src/breaker/systems/bump/tests/helpers.rs` contains three distinct app builders, each testing a different system or combination.

**Functions:**

| Function | What it registers |
|---------|------------------|
| `update_bump_test_app()` | `MinimalPlugins` + `InputActions` + `BumpPerformed` + `BumpWhiffed` messages + `CapturedBumps` + `CapturedWhiffs` resources + `set_bump_action` before `update_bump` + capture systems in FixedUpdate |
| `grade_bump_test_app()` | `MinimalPlugins` + `BoltImpactBreaker` + `BumpPerformed` + `BumpWhiffed` messages + `CapturedBumps` resource + `enqueue_hit` before `grade_bump` + `capture_bumps` in FixedUpdate |
| `combined_bump_test_app()` | Combines both: all messages + `InputActions` + `TestInputActive` + `TestHitMessage` + all systems in FixedUpdate |

**Canonical builder calls:**
```rust
// update_bump_test_app
TestAppBuilder::new()
    .with_resource::<InputActions>()
    .with_message_capture::<BumpPerformed>()
    .with_message_capture::<BumpWhiffed>()
    .with_system(FixedUpdate, (
        set_bump_action.before(update_bump),
        update_bump,
        (capture_bumps, capture_whiffs).after(update_bump),
    ))
    .build()

// grade_bump_test_app
TestAppBuilder::new()
    .with_message::<BoltImpactBreaker>()
    .with_message_capture::<BumpPerformed>()
    .with_message::<BumpWhiffed>()
    .with_system(FixedUpdate, (enqueue_hit.before(grade_bump), grade_bump, capture_bumps.after(grade_bump)))
    .build()
```

---

### Pattern N: Handle cell hit — Assets mesh/material + two messages (1 file, 2 functions)

`src/cells/systems/handle_cell_hit/tests/helpers.rs`

Both `test_app()` and `test_app_two_phase()` are identical: `MinimalPlugins` + `Assets<Mesh>` + `Assets<ColorMaterial>` + `DamageCell` + `RequestCellDestroyed` messages + `handle_cell_hit` in FixedUpdate.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_playfield()          // gets Assets<Mesh> + Assets<ColorMaterial>
    .with_message::<DamageCell>()
    .with_message::<RequestCellDestroyed>()
    .with_system(FixedUpdate, handle_cell_hit)
    .build()
```

**Note:** `test_app_two_phase()` is identical to `test_app()` — it's a duplicate and can be consolidated into one.

---

### Pattern O: Cleanup cell — two messages + enqueue/cleanup chain (1 file)

`src/cells/systems/cleanup_cell.rs`

`MinimalPlugins` + `RequestCellDestroyed` + `CellDestroyedAt` messages + bespoke `EnqueueRequestCellDestroyed` + `CapturedCellDestroyedAt` resources + 3-system FixedUpdate chain.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_message::<RequestCellDestroyed>()
    .with_message_capture::<CellDestroyedAt>()
    .with_system(FixedUpdate, (enqueue_requests, cleanup_cell, capture_cell_destroyed_at).chain())
    .build()
```

---

### Pattern P: Bolt lifespan/cleanup — message + resource + chain (2 files)

Both `tick_bolt_lifespan.rs` and `cleanup_destroyed_bolts.rs` register a `RequestBoltDestroyed` message, a bespoke collector resource, and a 2-system FixedUpdate chain.

**Canonical builder call:**
```rust
TestAppBuilder::new()
    .with_message_capture::<RequestBoltDestroyed>()
    .with_system(FixedUpdate, (tick_bolt_lifespan, capture_request_bolt_destroyed).chain())
    .build()
```

---

### Pattern Q: Dispatch bolt/chip effects — registry resource + system (2 files)

`MinimalPlugins` + registry resource inserted with content + single system in Update.

**Files:**

| File | Function | Registry | System |
|------|---------|---------|--------|
| `src/bolt/systems/dispatch_bolt_effects/tests/helpers.rs` | `test_app_with_dispatch(def)` | `BoltRegistry` with one entry | `dispatch_bolt_effects` |
| `src/chips/systems/build_chip_catalog/tests.rs` | `test_app()` | `ChipTemplateRegistry` + `EvolutionTemplateRegistry` (both init) | `build_chip_catalog` |

---

### Pattern R: Special/one-off (7 files)

Files with patterns that don't fit a clean group.

| File | Function | Pattern description |
|------|---------|-------------------|
| `src/game.rs` | `test_app(game)` | Takes `Game` struct; registers full game plugin group minus `DebugPlugin`; uses `AssetPlugin` + `InputPlugin` — integration smoke test |
| `src/debug/recording/systems/capture_frame.rs` | `test_app_with_config(enabled, filter)` | No MinimalPlugins — uses `TestSchedule` (custom label); inserts `RecordingConfig`, `InputActions`, `RecordingBuffer`, `RecordingFrame` |
| `src/debug/hot_reload/systems/propagate_node_layout_changes.rs` | `test_app()` | Uses `AssetPlugin::default()` + `ColorMaterial` + `Mesh` asset inits; inserts `CellConfig`, `PlayfieldConfig`, `NodeLayoutRegistry` |
| `src/debug/hot_reload/systems/propagate_cell_type_changes.rs` | `test_app()` | Uses `AssetPlugin::default()` + `ColorMaterial` asset; inserts `CellTypeRegistry` |
| `src/state/menu/main/systems/spawn_main_menu.rs` | `test_app()` | `MinimalPlugins` + `StatesPlugin` + `AssetPlugin::default()` + `Font` asset; inserts config resource |
| `src/fx/systems/tick_effect_flash.rs` | `test_app(dt)` | Takes `Duration`; uses `TimeUpdateStrategy::ManualDuration` for deterministic time |
| `src/fx/systems/animate_fade_out.rs` | `test_app(dt)` | Same ManualDuration pattern |
| `src/fx/systems/animate_punch_scale.rs` | `test_app(dt)` | Same ManualDuration pattern |
| `src/input/systems/read_input.rs` | `test_app()` | Inits `InputActions`, `InputConfig`, `DoubleTapState`, `ButtonInput<KeyCode>`; registers `KeyboardInput` message |
| `src/bolt/systems/bolt_lost_feedback.rs` | `test_app()` | Registers `BoltLost` message; 3-system Update chain including `animate_fade_out` |
| `src/breaker/systems/bump_feedback.rs` | `test_app()` | Registers `BumpPerformed` message; 2-system FixedUpdate chain |
| `src/state/run/node/systems/apply_time_penalty.rs` | `test_app_with_send(remaining)` | 3-system FixedUpdate chain with multiple messages and bespoke resources |
| `src/state/run/node/systems/reverse_time_penalty.rs` | `test_app_with_send(remaining, total)` | 2-system FixedUpdate chain; `ReverseTimePenalty` message |
| `src/state/run/node/hud/systems/update_timer_display.rs` | `test_app(remaining, total)` | Inserts `NodeTimer` + `TimerUiConfig`; no messages |
| `src/state/run/chip_select/systems/generate_chip_offerings/tests.rs` | `test_app_with_registry(registry)` | Inserts `ChipCatalog`, `ChipInventory`, `ChipSelectConfig`, `GameRng`; Update |
| `src/state/run/chip_select/systems/generate_chip_offerings/tests.rs` | `test_app_for_evolution(pool, eligible)` | Also inserts `ActiveNodeLayout` with given `NodePool` |
| `src/state/run/chip_select/systems/spawn_chip_select.rs` | `test_app_with_offers(offers)` | Inserts `ChipSelectConfig` + `ChipOffers`; Update |
| `src/state/run/node/lifecycle/systems/spawn_highlight_text/tests/helpers.rs` | `test_app()` | Registers `HighlightTriggered` message; inits `HighlightConfig`, `PlayfieldConfig`, `GameRng`; enqueue+spawn chain in Update |
| `src/state/run/node/systems/init_node_timer.rs` | `test_app_with_node_sequence(timer_secs, timer_mult)` | Adds `NodeOutcome` + `NodeSequence` on top of base `test_app(timer_secs)` |
| `src/state/run/systems/advance_node.rs` | `test_app_with_sequence(...)` | Takes 5 params; builds `NodeSequence` from assignments vector |

---

## Summary Table

| Pattern | Name | Function count | File count |
|---------|------|---------------|------------|
| A | Minimal — single system, no messages | 24 | 24 |
| B | Minimal + resources, no messages | 16 | 16 |
| C | Message-only bridge — enqueue/bridge pair | 29 | 19 |
| D | Physics collision | 7 | 6 |
| E | Physics + DamageCell capture | 8 | 5 |
| F | State hierarchy + system | 13 | 11 |
| G | Chip-select / menu state hierarchy | 10 | 7 |
| H | Node lifecycle state transitions | 7 | 5 |
| I | Node spawn — registries + playfield | 10 | 7 |
| J | Tracking + RunStats/HighlightTracker | 8 | 8 |
| K | Highlight detection — multi-message | 5 | 5 |
| L | E2e setup delegating to Pattern G | 4 | 3 |
| M | Bump multi-variant | 3 | 1 |
| N | Handle cell hit | 2 | 1 |
| O | Cleanup cell | 1 | 1 |
| P | Bolt lifespan/cleanup | 2 | 2 |
| Q | Dispatch registry | 2 | 2 |
| R | Special / one-off | 20 | 18 |
| **Total** | | **171** | **141** |

---

## Migration Priority

**Highest ROI — do these first:**

1. **Pattern C** (29 functions): All bumped/impact/bolt-lost trigger bridges are nearly identical. One builder template covers almost all of them.
2. **Pattern A** (24 functions): Trivially `TestAppBuilder::new().with_system(schedule, system).build()`.
3. **Pattern D** (7 functions): All follow `.with_physics().with_message().with_system(FixedUpdate, x.after(PhysicsSystems::MaintainQuadtree))`.
4. **Pattern F** (13 functions): `.with_state_hierarchy().in_node_playing().with_system(...)`.

**Requires new builder methods or forethought:**

- Pattern R `capture_frame.rs`: Uses `TestSchedule` (not FixedUpdate/Update) — `.with_system(TestSchedule, ...)` should already work if `TestSchedule` implements `ScheduleLabel`.
- Pattern R `fx` files: `TimeUpdateStrategy::ManualDuration` — use `.insert_resource(TimeUpdateStrategy::ManualDuration(dt))`.
- Pattern G `handle_pause_input`: Pauses `Time<Virtual>` directly — no builder method; call `app.world_mut().resource_mut::<Time<Virtual>>().pause()` after `.build()`.
- Pattern I flash variants: Need `AssetPlugin` which `MinimalPlugins` does not include — these tests require `app.add_plugins(AssetPlugin::default())` or the builder needs a `.with_asset_plugin()` method.

---

## Files Not in breaker-game/src

The grep returned 129 files — all are under `breaker-game/src/`. No test_app functions were found in `breaker-scenario-runner/src/` (the graph report `surprising connections` note about `check_aabb_matches_entity_dimensions/tests/helpers.rs` was an inferred edge, not a real occurrence).

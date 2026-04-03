# Crate Migration

Which systems need updating to use `rantzsoft_lifecycle` and what they need to change. This happens in Wave 7, after the lifecycle crate is built (Wave 5-6) and routing tables are written (Wave 6).

---

## Overview

The crate provides:
- `ChangeState<S>` message — replaces `NextState<S>::set()`
- `RoutingTable<S>` — declarative route definitions replace per-system routing logic
- `when()` conditions — parent states react to child completion without child systems knowing
- Transition overlay — replaces parked `state/transition/` systems
- `CleanupOnExit<S>` — already used from Wave 4, crate provides canonical version

## What Changes

### Systems that call `NextState<S>::set()` → send `ChangeState<S>` message

Every system that currently calls `next_state.set(SomeState::Variant)` must switch to sending a `ChangeState<S>` message. The routing table handles the actual state change.

**Within-level transitions (system sends ChangeState for its own level):**

| System | Currently sets | After crate | Location |
|--------|--------------|-------------|----------|
| `handle_node_cleared` | `NextState<NodeState>(AnimateOut)` | `writer.write(ChangeState { target: NodeState::AnimateOut })` | state/run/node/lifecycle/ |
| `handle_timer_expired` | `NextState<NodeState>(AnimateOut)` | same pattern | state/run/node/lifecycle/ |
| `handle_run_lost` | `NextState<NodeState>(AnimateOut)` | same pattern | state/run/node/lifecycle/ |
| `handle_main_menu_input` | `NextState<MenuState>(StartGame)` | `writer.write(ChangeState { target: MenuState::StartGame })` | state/menu/main/ |
| `handle_run_setup_input` | `NextState<MenuState>(Teardown)` | `writer.write(ChangeState { target: MenuState::Teardown })` | state/menu/start_game/ |
| `handle_chip_input` | `NextState<ChipSelectState>(AnimateOut)` | `writer.write(ChangeState { target: ChipSelectState::AnimateOut })` | state/run/chip_select/ |
| `tick_chip_timer` | `NextState<ChipSelectState>(AnimateOut)` | same pattern | state/run/chip_select/ |
| `handle_run_end_input` | `NextState<RunEndState>(AnimateOut)` | same pattern | state/run/run_end/ |
| `check_spawn_complete` | `NextState<NodeState>(AnimateIn)` | same pattern | state/run/node/ |

**Cross-level transitions (removed — routing table handles these):**

These systems currently set a PARENT state directly. After the crate, they don't — the parent's `when()` condition detects the child reaching Teardown and makes the routing decision.

| System | Currently sets | After crate | What replaces it |
|--------|--------------|-------------|------------------|
| `handle_pause_input` (quit) | `NextState<RunState>(Teardown)` | `time.unpause()` only | RunState's when() detects pause-quit resource |
| *(teardown routers in routing.rs)* | `NextState<ParentState>(X)` | REMOVED | Parent's `when()` condition |

### Routing systems in `state/routing.rs` → replaced by routing table

The pass-through routing systems and teardown routing systems are completely replaced by `app.add_route()` calls. The `routing.rs` file is deleted.

**Pass-throughs replaced:**
| Current system | Replaced by |
|----------------|-------------|
| OnEnter(NodeState::AnimateIn) → set Playing | `Route::from(NodeState::AnimateIn).to(NodeState::Playing)` |
| OnEnter(NodeState::AnimateOut) → set Teardown | `Route::from(NodeState::AnimateOut).to(NodeState::Teardown)` |
| OnEnter(ChipSelectState::Loading) → set AnimateIn | `Route::from(ChipSelectState::Loading).to(ChipSelectState::AnimateIn)` |
| *(all other pass-throughs)* | Static routes in routing table |

**Teardown routers replaced:**
| Current system | Replaced by |
|----------------|-------------|
| node_teardown_router (reads RunOutcome) | `Route::from(RunState::Node).to_dynamic(|w| match *w.resource::<RunOutcome>() { ... }).when(...)` |
| chip_select_teardown_router | `Route::from(RunState::ChipSelect).to(RunState::Node).when(...)` |
| run_end_teardown_router | `Route::from(RunState::RunEnd).to(RunState::Teardown).when(...)` |
| menu_teardown_router | `Route::from(GameState::Menu).to(GameState::Run).when(...)` |

### Transitions wired to routes

The parked `state/transition/` systems are replaced by the lifecycle crate's built-in transition overlay. Routes gain `.with_transition()`:

| Route | Transition |
|-------|------------|
| `GameState::Loading → Menu` | `Transition::In(FadeIn)` |
| `GameState::Menu → Run` | `Transition::OutIn { FadeOut, FadeIn }` |
| `GameState::Run → Menu` | `Transition::OutIn { FadeOut, FadeIn }` |
| `RunState::Setup → Node` | `Transition::In(FadeIn)` |
| `RunState::Node → ChipSelect` | `Transition::OutIn { FadeOut, FadeIn }` |
| `RunState::ChipSelect → Node` | `Transition::OutIn { FadeOut, FadeIn }` |
| `RunState::Node → RunEnd` | `Transition::OutIn { FadeOut, FadeIn }` |
| `MenuState::Main → StartGame` | `Transition::OneShot(SlideLeft)` |
| `MenuState::Options → Main` | `Transition::OneShot(SlideRight)` |
| `MenuState::Meta → Main` | `Transition::OneShot(SlideRight)` |

After transitions are wired, delete `state/transition/` entirely.

### CleanupOnExit<S> — re-export from crate

The `CleanupOnExit<S>` component defined in `state/cleanup.rs` is replaced by the crate's canonical version. Update import paths throughout. The `cleanup_entities<T>` utility stays as a game-side helper (or use the crate's `cleanup_on_exit` system).

---

## Signals for Routing Decisions

Some routes need dynamic destinations or conditions. These are the signals (resources/messages) the routing table reads:

| Signal | Type | Set by | Read by (route) | Purpose |
|--------|------|--------|-----------------|---------|
| `RunOutcome` | Resource (enum) | `handle_node_cleared`, `handle_timer_expired`, `handle_run_lost` | RunState::Node → dynamic(ChipSelect or RunEnd) | Determines what happens after a node |
| `NodeCleared` | Message | `track_node_completion` | `handle_node_cleared` (existing) | Node completed — all cells destroyed |
| `TimerExpired` | Message | `tick_node_timer` | `handle_timer_expired` (existing) | Timer ran out |
| `RunLost` | Message | `handle_run_lost` | Future: RunState routing | Lives depleted |
| `BoltSpawned` | Message | `reset_bolt` | `check_spawn_complete` | Bolt ready for gameplay |
| `BreakerSpawned` | Message | `reset_breaker` | `check_spawn_complete` | Breaker ready for gameplay |
| `CellsSpawned` | Message | `spawn_cells_from_layout` | `check_spawn_complete` | Cells ready |
| `WallsSpawned` | Message | `spawn_walls` | `check_spawn_complete` | Walls ready |
| `SpawnNodeComplete` | Message | `check_spawn_complete` | NodeState routing (→AnimateIn) | All spawn signals received |
| `ChipSelected` | Message | `handle_chip_input` | `dispatch_chip_effects` | Player chose a chip |

---

## Systems That DON'T Change for Crate Migration

All FixedUpdate/Update gameplay systems (bolt physics, breaker movement, cell handling, effect triggers, etc.) are unaffected. They don't call `next_state.set()` — they just run during `NodeState::Playing`. Their gate (`run_if(in_state(NodeState::Playing))`) stays the same.

The only systems that change are:
1. Systems that call `next_state.set()` → send `ChangeState<S>` (listed above)
2. Routing systems in `state/routing.rs` → replaced by routing table
3. Transition systems in `state/transition/` → replaced by crate overlay

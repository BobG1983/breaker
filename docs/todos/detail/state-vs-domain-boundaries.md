# State vs Domain Boundary Refactor

## Summary
Define clear rules for what belongs under `state/` (lifecycle, routing, UI, setup, teardown, transitions) vs top-level domains (gameplay logic, runtime systems, tracking, highlights). Refactor `state/run/node/` which currently mixes both.

## Context
The state lifecycle refactor moved `run/` into `state/run/`. This was correct for the routing/lifecycle/UI code, but dragged active gameplay systems along with it. `state/run/node/` now contains spawn systems, collision dispatch, timer ticking, node tracking, highlights — all of which are runtime gameplay, not state management.

The `state/` module should own: state types, routing tables, transitions, AnimateIn/AnimateOut, setup/teardown, loading screens, menus, pause, chip select UI, run end screen. It's about **when things happen and what the player sees between gameplay**.

Gameplay domains should own: the systems that run during `NodeState::Playing` (and similar active states). They're about **what happens during gameplay**.

## Current state/run/node/ contents

| Directory/File | Category | Belongs in |
|----------------|----------|-----------|
| `definition/` (NodeLayout, NodePool) | Data definition | `node/` domain or shared |
| `resources/` (ActiveNodeLayout, NodeLayoutRegistry, NodeTimer, ClearRemaining) | Runtime state | `node/` domain |
| `systems/spawn_cells_from_layout` | Setup (OnEnter) | Could go either way — it's setup but creates gameplay entities |
| `systems/spawn_walls` | Setup (OnEnter) | Same |
| `systems/reset_bolt`, `reset_breaker` | Setup (OnEnter) | Same |
| `systems/check_spawn_complete` | Setup coordination | `state/run/node/` — this IS lifecycle |
| `systems/dispatch_cell_effects` | Runtime gameplay | `node/` or `cells/` domain |
| `systems/tick_node_timer` | Runtime gameplay | `node/` domain |
| `systems/track_node_completion` | Runtime gameplay | `node/` domain |
| `systems/init_node_timer`, `init_clear_remaining` | Setup (OnEnter) | Could go either way |
| `systems/apply_node_scale_to_bolt/breaker` | Setup (OnEnter) | `node/` domain |
| `systems/apply_time_penalty`, `reverse_time_penalty` | Runtime gameplay | `node/` or `effect/` |
| `systems/set_active_layout` | Setup (OnEnter) | `state/run/node/` — this IS lifecycle |
| `lifecycle/` (handle_node_cleared, handle_run_lost, handle_timer_expired) | State transitions | `state/run/node/` — this IS lifecycle |
| `highlights/` (detect_close_save, detect_combo_king, etc.) | Runtime gameplay | `node/` domain |
| `tracking/` (track_bolts_lost, track_bumps, etc.) | Runtime gameplay | `node/` domain |
| `hud/` (timer display, side panels) | UI | `state/run/node/` — this IS state/UI |

## Proposed rule

| `state/**` owns | Top-level domains own |
|-----------------|----------------------|
| State type definitions + registration | Domain-specific components, resources, definitions |
| Routing tables + `when()` conditions | Runtime systems (FixedUpdate gameplay) |
| OnEnter/OnExit lifecycle systems | Domain-internal logic |
| Setup/teardown coordination (check_spawn_complete, set_active_layout) | Spawning entity content (even if triggered by OnEnter) |
| Screen UI (menus, chip select, run end, HUD) | Everything that runs during Playing |
| Transitions, AnimateIn/AnimateOut | Tracking, highlights, timers |
| Loading coordination | Collision dispatch, time penalties |

## Also consider
- `PlayfieldConfig` — currently in `shared/`, may belong in `node/` since it's node-specific config
- `NodeLayoutRegistry` — data loading resource, could be in `node/` or stay in state/run/ for the loading pipeline
- The spawn systems (spawn_cells, spawn_walls, reset_bolt, reset_breaker) run OnEnter but create gameplay entities — they're the bridge between lifecycle and gameplay. Could live in either place. Lean toward `node/` since they create node-specific content.

## Dependencies
- Depends on: State lifecycle refactor (#1) — folder structure should settle first
- Related to: Cross-domain prelude (#2) — import paths change

## Status
`[NEEDS DETAIL]` — needs final rule definition and exhaustive file-by-file mapping after state lifecycle refactor lands

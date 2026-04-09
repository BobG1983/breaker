# Split State Routing — Distribute Routes to Domain Plugins

## Summary

`state/plugin/system.rs` (377 lines) is a centralized routing monolith: every state transition for every state type is registered in one file. As protocols/hazards add new states, this file will grow unboundedly. Each state's routes should live next to the plugin that owns that state.

## The Problem

Today `register_routing()` calls six sub-functions that each know about one state type's routes. But they all live in the same file, which means:

1. **Every new state or transition** requires editing the central file
2. **The file imports from every domain** — menu, run, chip_select, run_end, pause
3. **Routes are disconnected from the systems they gate** — you define a route in `state/plugin/` but the `OnEnter`/`OnExit` systems live in domain plugins

## Current Coupling Analysis

### What `state/` imports from other domains (production code only)

| Domain | Import count | What it needs |
|--------|-------------|---------------|
| `cells` | 3 uses | `CellTypeRegistry`, `CellDefaults`, `ToughnessConfig`, `Toughness`, cell components |
| `shared` | 2 uses | `PlayfieldConfig`, `GameRng`, `RunSeed` |
| `chips` | 1 use | `ChipDefinition`, `EvolutionIngredient` |
| `breaker` | (via prelude) | `BreakerDefinition`, `BumpGrade` |

### What other domains import from `state/`

25 files outside `state/` import from it — mostly:
- **Plugin files** importing state enums (`AppState`, `GameState`, `NodeState`, etc.) for `run_if` guards and `OnEnter`/`OnExit` scheduling
- **Effect systems** importing `NodeState` for node-scoped behavior
- **Components** importing state types for `CleanupOnExit` markers

### Verdict: A Separate Crate Is NOT Worthwhile (Today)

A `breaker-lifecycle` crate would need to:
- Own all state enums — but those are used as `run_if` guards everywhere
- Own routing — but routes reference domain-specific resources (`MainMenuSelection`, `NodeOutcome`)
- Own cleanup — but `CleanupOnExit<S>` markers are spawned by domain code

The circular dependency problem: `state/` needs domain types for route conditions, domains need state types for scheduling. A crate boundary would force trait indirection or a shared types crate, adding complexity without reducing coupling.

**Revisit after the effect refactor and protocol system are built** — if `state/` doubles in size with protocol/hazard states, extraction becomes more justified. For now, the simpler fix is distributing routes.

## Proposed Design: Distributed Route Registration

Each domain plugin registers its own routes in its `Plugin::build()`:

```rust
// In state/run/chip_select/plugin.rs
impl Plugin for ChipSelectPlugin {
    fn build(&self, app: &mut App) {
        register_chip_select_routes(app);  // moved here from state/plugin/system.rs
        // ... existing system registration ...
    }
}
```

### What stays centralized

- **State hierarchy** (`init_state`, `add_sub_state`) — these define the state machine structure and must be registered once, in order
- **`RantzStateflowPlugin` registration** — one call that registers all state types
- **Defaults plugin** — asset loading configuration is cross-cutting
- **Cross-state routes** where the parent watches a child state's teardown (e.g., `GameState::Run → GameState::Menu` watching `RunState::Teardown`) — these are inherently about the relationship between two states, so they stay in the parent

### What gets distributed

| Route function | Moves to |
|----------------|----------|
| `register_node_routes` | `state/run/node/plugin.rs` (NodePlugin) |
| `register_chip_select_routes` | `state/run/chip_select/plugin.rs` (ChipSelectPlugin) |
| `register_run_end_routes` | `state/run/run_end/plugin.rs` (RunEndPlugin) |
| Node/ChipSelect/RunEnd cleanup | Co-located with respective route registration |

| Route function | Stays in `state/plugin/system.rs` |
|----------------|----------------------------------|
| `register_app_routes` | Cross-state (AppState watches GameState) |
| `register_parent_routes` | Cross-state (GameState watches MenuState/RunState) |
| `register_run_routes` | Cross-state (RunState watches NodeState/ChipSelectState/RunEndState) |
| `send_app_exit` | Top-level lifecycle |

### Expected result

`state/plugin/system.rs` shrinks from 377 lines to ~180 (state hierarchy + cross-state routes + defaults). Each domain plugin gains 15-30 lines of self-contained route registration.

## Future: Protocol & Hazard States

When protocols/hazards land (#5), they'll likely add:
- `HazardSelectState` (Loading → Selecting → AnimateOut → Teardown)
- Possibly `ProtocolActiveState` for protocol lifecycle

With distributed routing, these register in their own plugins. Without this refactor, `system.rs` grows by another 40-60 lines per new state type.

## Acceptance Criteria

- [ ] `state/plugin/system.rs` under 200 lines
- [ ] Each leaf state's routes registered in its own plugin
- [ ] Cross-state (parent-watches-child) routes remain centralized
- [ ] All existing route tests pass
- [ ] No functional change — identical state machine behavior
- [ ] `resolve_node_next_state` stays testable (it's already extracted)

## What This Does NOT Do

- Does not extract `state/` into a separate crate (premature today)
- Does not change the state hierarchy or transition types
- Does not touch `rantzsoft_stateflow` — that crate's API is fine
- Does not split the state enum definitions — those are small and stable

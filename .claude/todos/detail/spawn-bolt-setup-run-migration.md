# Run Lifecycle & State Machine Refactor

## Summary
Refactor GameState to introduce hierarchical RunState/NodeState, move breaker+bolt spawning to a single `setup_run` system, and align cleanup markers with the new state boundaries.

## Context
Originally scoped as "move spawn_bolt into setup_run." During interrogation, expanded to include breaker (which now has a builder pattern too) and revealed the need for a proper hierarchical state machine. The current flat `GameState` conflates run-level and node-level concerns — `OnEnter(Playing)` fires on every node entry, forcing guard patterns for run-scoped entities.

## Design Decisions

### State Machine
Current flat `GameState`:
```
Loading → MainMenu → RunSetup → Playing → TransitionOut → ChipSelect → TransitionIn → (Playing again) → RunEnd → MetaProgression
```

New hierarchical states:
```
GameState: Loading | MainMenu | Run | MetaProgression
  RunState (sub-state of GameState::Run): Setup | Node | ChipSelect | RunEnd | Teardown
    NodeState (sub-state of RunState::Node): Setup | Playing | Teardown
```

- `GameState::Run` replaces `GameState::Playing` (and subsumes TransitionOut/TransitionIn/ChipSelect/RunEnd)
- `RunState::Setup` — one-time run initialization (spawn breaker, bolt, run-scoped resources)
- `RunState::Node` — active node (has NodeState sub-state)
- `RunState::ChipSelect` — between nodes
- `RunState::RunEnd` — win/lose screen
- `RunState::Teardown` — cleanup `CleanupOnRunEnd` entities
- `NodeState::Setup` — per-node init (reset_bolt, reset_breaker, spawn cells/walls)
- `NodeState::Playing` — active gameplay (physics, input, collisions)
- `NodeState::Teardown` — cleanup `CleanupOnNodeExit` entities, transition effects

### Entity Lifecycle
- `setup_run` (new system in `run` domain): Runs on `OnEnter(RunState::Setup)`. Spawns primary breaker + primary bolt via builders with `CleanupOnRunEnd`. Single system owns all run-scoped entity creation.
- `reset_bolt`: Runs on `OnEnter(NodeState::Setup)`. Repositions bolt, sends `BoltSpawned`.
- `reset_breaker`: Runs on `OnEnter(NodeState::Setup)`. Repositions breaker, sends `BreakerSpawned`.
- `spawn_bolt` system: Deleted entirely.
- `spawn_or_reuse_breaker` system: Deleted entirely.

### Cleanup
- `CleanupOnRunEnd` entities despawned on `OnEnter(RunState::Teardown)` (was `OnExit(GameState::RunEnd)`)
- `CleanupOnNodeExit` entities despawned on `OnEnter(NodeState::Teardown)` (was `OnExit(GameState::Playing)`)

### Messages
- `reset_bolt` sends `BoltSpawned` on every node entry (semantics: "bolt is ready")
- `reset_breaker` sends `BreakerSpawned` on every node entry (semantics: "breaker is ready")
- `check_spawn_complete` continues to use all 4 signals (Bolt, Breaker, Cells, Walls)

## Scope
- In:
  - New `RunState` and `NodeState` sub-states
  - `setup_run` system spawning primary breaker + bolt
  - Migrate all systems from `OnEnter(GameState::Playing)` to appropriate new state
  - Migrate all systems from `OnExit(GameState::Playing)` to appropriate new state
  - Migrate `FixedUpdate` `run_if(in_state(PlayingState::Active))` to `run_if(in_state(NodeState::Playing))`
  - Delete `spawn_bolt`, `spawn_or_reuse_breaker`
  - Update cleanup systems to new state boundaries
  - `reset_bolt` and `reset_breaker` send spawned messages
  - Remove `PlayingState` (replaced by `NodeState`)
- Out:
  - Effect-spawned bolts (use builder directly, not this system)
  - Wall/cell builder patterns (separate todos)
  - Gameplay behavior changes (pure refactor)

## Dependencies
- Depends on: Bolt builder (done), Breaker builder (done)
- Blocks: Wall builder pattern, Cell builder pattern, Bolt birthing animation, all Phase 5+ work that touches state transitions

## Notes
This is a large refactor touching every plugin that references `GameState::Playing`, `PlayingState`, `OnEnter(GameState::Playing)`, or `OnExit(GameState::Playing)`. Should be broken into waves:
1. State machine types + new states (shared domain)
2. `setup_run` system + delete spawn systems (run/bolt/breaker domains)
3. Migrate OnEnter/OnExit systems to new states (all domains)
4. Migrate FixedUpdate run_if guards (all domains)
5. Cleanup systems to new teardown states (screen domain)

## Status
`ready`

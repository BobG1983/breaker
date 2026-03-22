# State Management

Bevy `States` for top-level game state. `SubStates` where a state only exists within a parent.

## Top-level states (`GameState`)

- `Loading` — asset preload (default/initial state)
- `MainMenu`
- `RunSetup` — breaker/seed selection
- `Playing` — active node (see sub-states below)
- `TransitionOut` — animated transition out of a completed node (clear animation)
- `ChipSelect` — timed chip selection between nodes
- `TransitionIn` — animated transition into the next node (load animation)
- `RunEnd` — win/lose screen
- `MetaProgression` — between-run Flux spending

## Playing sub-states (`PlayingState`)

- `Active` — normal gameplay (default when entering `Playing`)
- `Paused` — all gameplay frozen (pause menu implemented; Escape key toggles Active ↔ Paused)

`PlayingState` only exists when `GameState::Playing` is active — it is automatically created and destroyed by Bevy's sub-state lifecycle. Pausing is modeled as a sub-state (not top-level) because you can only pause from active gameplay. This constraint is encoded in the type system.

Systems that should freeze during pause use `run_if(in_state(PlayingState::Active))`. Systems that should run regardless of pause (e.g., pause menu UI) use `run_if(in_state(GameState::Playing))`.

## TransitionOut / TransitionIn

Bevy doesn't fire `OnExit`/`OnEnter` when transitioning to the same state. Since node spawn/cleanup relies on `OnEnter(Playing)` / `OnExit(Playing)`, advancing between nodes requires leaving and re-entering `Playing`.

The full inter-node flow is: `Playing → TransitionOut → ChipSelect → TransitionIn → Playing`. On `NodeCleared`, `handle_node_cleared` transitions to `TransitionOut`. The `fx` domain's `animate_transition` system drives a timed overlay animation; when the `TransitionOut` animation completes, it sets `NextState(ChipSelect)`. When the player confirms a chip selection (or the timer expires), `handle_chip_input` transitions to `TransitionIn`. The `animate_transition` system drives the `TransitionIn` overlay; on completion it sets `NextState(Playing)`. `advance_node` (in the run domain) runs `OnEnter(TransitionIn)`, incrementing the node index so the correct node is loaded when `Playing` is re-entered.

## Passive types vs. active logic

`GameState`, `PlayingState`, cleanup markers, and playfield constants are passive types defined in `shared.rs` (imported by all domains). State registration, transitions, and cleanup systems live in the `screen/` domain plugin.

# State Management

Bevy `States` for top-level game state. `SubStates` where a state only exists within a parent.

## Top-level states (`GameState`)

- `Loading` — asset preload (default/initial state)
- `MainMenu`
- `RunSetup` — breaker/seed selection
- `Playing` — active node (see sub-states below)
- `UpgradeSelect` — timed upgrade selection
- `RunEnd` — win/lose screen
- `MetaProgression` — between-run Flux spending

## Playing sub-states (`PlayingState`)

- `Active` — normal gameplay (default when entering `Playing`)
- `Paused` — all gameplay frozen

`PlayingState` only exists when `GameState::Playing` is active — it is automatically created and destroyed by Bevy's sub-state lifecycle. Pausing is modeled as a sub-state (not top-level) because you can only pause from active gameplay. This constraint is encoded in the type system.

Systems that should freeze during pause use `run_if(in_state(PlayingState::Active))`. Systems that should run regardless of pause (e.g., pause menu UI) use `run_if(in_state(GameState::Playing))`.

## Passive types vs. active logic

`GameState`, `PlayingState`, cleanup markers, and playfield constants are passive types defined in `shared.rs` (imported by all domains). State registration, transitions, and cleanup systems live in the `screen/` domain plugin.

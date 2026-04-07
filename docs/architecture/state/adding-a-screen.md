# Adding a New Screen

Step-by-step guide for adding a new screen to the game's state hierarchy.

## Prerequisites

Read [state.md](../state.md) for the full state hierarchy and routing model.

## Steps

### 1. Decide where the screen lives

Every screen is a variant of an existing sub-state enum. Pick the parent:

| Parent | When to use |
|--------|-------------|
| `MenuState` | Pre-game screens (settings, meta-progression, credits) |
| `NodeState` | Mid-gameplay screens (unlikely — gameplay is in `Playing`) |
| `ChipSelectState` | Chip selection sub-screens (unlikely — `Selecting` covers it) |
| `RunEndState` | Post-run screens (unlikely — `Active` covers it) |

If none fit, you may need a new sub-state under `RunState` or `GameState`. Discuss with the team first — adding a new sub-state level has routing implications.

### 2. Add the variant

In `breaker-game/src/state/types/<parent>_state.rs`, add the new variant:

```rust
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Menu)]
pub enum MenuState {
    #[default]
    Loading,
    Main,
    StartGame,
    Options,
    Meta,
    Credits,    // ← NEW
    Teardown,
}
```

### 3. Register routes

In `breaker-game/src/state/plugin.rs`, add routes for the new variant in the appropriate `register_*_routes` function:

```rust
// Navigate to Credits from Main menu
app.add_route(Route::from(MenuState::Main).to(MenuState::Credits));

// Back from Credits to Main
app.add_route(Route::from(MenuState::Credits).to(MenuState::Main));
```

**With transitions:**

```rust
use std::sync::Arc;
use rantzsoft_stateflow::{FadeIn, FadeOut, TransitionType};

app.add_route(
    Route::from(MenuState::Main)
        .to(MenuState::Credits)
        .with_transition(TransitionType::OutIn {
            out_e: Arc::new(FadeOut::default()),
            in_e: Arc::new(FadeIn::default()),
        }),
);
```

### 4. Create the screen plugin

Create a new module under the parent's directory:

```
state/menu/credits/
    mod.rs          ← pub(crate) mod plugin; pub(crate) mod systems;
    plugin.rs       ← CreditsPlugin
    systems/
        mod.rs
        spawn_credits_screen.rs
        handle_credits_input.rs
```

The plugin registers its systems:

```rust
pub(crate) struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Credits), spawn_credits_screen)
           .add_systems(
               Update,
               handle_credits_input.run_if(in_state(MenuState::Credits)),
           );
    }
}
```

### 5. Wire the plugin

Add the plugin to `StatePlugin::build` in `state/plugin.rs`:

```rust
.add_plugins((
    LoadingPlugin,
    MainMenuPlugin,
    RunSetupPlugin,
    CreditsPlugin,   // ← NEW
    PauseMenuPlugin,
    // ...
))
```

### 6. Handle input / navigation

The screen's input system sends `ChangeState` to trigger routing:

```rust
fn handle_credits_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<ChangeState<MenuState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        writer.write(ChangeState::new());  // Route decides destination
    }
}
```

### 7. Mark entities for cleanup

Spawn entities with `CleanupOnExit<MenuState>` so they're despawned when MenuState exits:

```rust
use rantzsoft_stateflow::CleanupOnExit;

fn spawn_credits_screen(mut commands: Commands) {
    commands.spawn((
        // ... UI nodes ...
        CleanupOnExit::<MenuState>::default(),
    ));
}
```

If the screen needs its own sub-state with `Loading → AnimateIn → Active → AnimateOut → Teardown`, register `cleanup_on_exit::<CreditsState>` on `OnEnter(CreditsState::Teardown)` in the plugin.

### 8. Add cleanup registration (if using Teardown)

If the screen has a Teardown variant, register the cleanup system in `register_cleanup` in `state/plugin.rs`:

```rust
app.add_systems(
    OnEnter(MenuState::Teardown),
    cleanup_on_exit::<MenuState>,
);
```

## Checklist

- [ ] Variant added to state enum
- [ ] Routes registered (from + to, with transitions if needed)
- [ ] Plugin created with OnEnter spawn + Update input systems
- [ ] Plugin wired in StatePlugin
- [ ] Input system sends `ChangeState` (not `NextState::set`)
- [ ] Entities marked with `CleanupOnExit<S>`
- [ ] Cleanup registered on Teardown (if applicable)
- [ ] Tests: plugin builds, input triggers ChangeState message

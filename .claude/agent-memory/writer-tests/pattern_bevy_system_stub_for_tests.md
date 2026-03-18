---
name: pattern_bevy_system_stub_for_tests
description: How to write Bevy system stubs with todo!() so integration tests compile and fail before implementation
type: project
---

# Bevy System Stubs for Failing Integration Tests

When the spec defines Bevy system signatures but no implementation yet, use `todo!()` stubs.
Tests that tick the app will panic inside the system (via Bevy's catch), causing test failure
with "Encountered a panic in system" — this is correct RED-phase behavior.

## Pattern

```rust
pub fn check_something(
    query: Query<(Entity, &Transform), With<SomeMarker>>,
    resource: Res<SomeResource>,
    mut log: ResMut<SomeLog>,
) {
    // Silence unused variable warnings while keeping the signature compilable
    let _ = (query, resource, &mut log);
    todo!()
}
```

## Type Complexity in Bevy System Parameters

Clippy `type_complexity` fires when a system parameter contains a complex `Query` type with `Or<(...)>` filters.
The fix is a `type` alias at module level (not inside the system fn):

```rust
type TaggedQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), Or<(With<MarkerA>, With<MarkerB>)>>;

pub fn check_something(tagged: TaggedQuery, ...) { ... }
```

Note: use `&'static T` for component references in type aliases (lifetime elision doesn't work in type aliases).

## Adding Bevy as a Dependency to a Non-Bevy Crate

If a crate (e.g. `breaker-runner-scenarios`) has no `bevy` dependency but tests need Bevy types,
add bevy as a **regular** dependency (not dev-only), because the stub types and systems are in production
code, and the crate's module-level code uses Bevy types:

```toml
[dependencies]
bevy = { version = "0.18.1", default-features = false, features = ["2d"] }
```

## Testing PlayingState (SubState) in Integration Tests

`PlayingState` is a `SubState` of `GameState::Playing`. To use it in tests:

```rust
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();  // Required — SubStates must be explicitly registered
    // ...
    app
}
```

Then to enter Playing and then transition to Paused:
```rust
app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
app.update(); // process state transition — PlayingState::Active becomes active
// ... spawn entities, tick once to seed Locals ...
app.world_mut().resource_mut::<NextState<PlayingState>>().set(PlayingState::Paused);
app.update(); // process sub-state transition
// ... now test behavior under Paused ...
tick(&mut app);
```

`Option<Res<State<PlayingState>>>` returns `None` when not in `GameState::Playing`.

## Pre-existing Lint Errors in Other Files

When clippy `pedantic` is deny-level, pre-existing errors in sibling files (like `input.rs`) will
prevent `cargo dclippy` from succeeding even for your new file. Use `cargo dcheck` to verify
compilation only — this is sufficient for confirming the tests will compile and run.

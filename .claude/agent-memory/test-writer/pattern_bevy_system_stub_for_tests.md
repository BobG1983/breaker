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

If a crate (e.g. `breaker-scenario-runner`) has no `bevy` dependency but tests need Bevy types,
add bevy as a **regular** dependency (not dev-only), because the stub types and systems are in production
code, and the crate's module-level code uses Bevy types:

```toml
[dependencies]
bevy = { version = "0.18.1", default-features = false, features = ["2d"] }
```

## Pre-existing Lint Errors in Other Files

When clippy `pedantic` is deny-level, pre-existing errors in sibling files (like `input.rs`) will
prevent `cargo dclippy` from succeeding even for your new file. Use `cargo dcheck` to verify
compilation only — this is sufficient for confirming the tests will compile and run.

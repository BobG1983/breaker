# Wiring a Trigger

Step-by-step checklist for adding a new trigger to the system. Follow [naming-convention.md](naming-convention.md) for system names.

## 1. Add variant to Trigger enum

In the Trigger enum, add the new variant:

```rust
MyEvent,              // unit variant — no parameters
MyEvent(EntityKind),  // parameterized — filters by entity type
MyEvent(f32),         // parameterized — carries a threshold
```

## 2. Determine scope

Decide if this trigger is Local, Global, or Self. See [trigger-api/scope.md](trigger-api/scope.md).

If the event produces both a Local and a Global variant (like Bumped + BumpOccurred), add both variants to the Trigger enum.

## 3. If Local: define participants

If the trigger is Local with named roles:

1. Check if an existing participant enum fits (Bump, Impact, Death, BoltLost)
2. If not, create a new participant enum. See [trigger-api/participant-targets.md](trigger-api/participant-targets.md).

## 4. Add TriggerContext variant (if new category)

If the trigger introduces a new participant relationship not covered by existing TriggerContext variants, add a new variant:

```rust
enum TriggerContext {
    // ...existing...
    MyEvent { role_a: Entity, role_b: Entity },
}
```

## 5. Create the trigger folder

If this is a new trigger category, create a folder in `src/effect/triggers/<category>/`:

```
triggers/my_category/
  mod.rs          # pub(crate) mod bridges; pub(crate) mod register; + re-exports
  bridges.rs      # bridge systems (on_* functions)
  register.rs     # pub(crate) fn register(app: &mut App) — registers bridges, game systems, resources, messages
```

If the trigger has game systems (timers, threshold checks), add them as separate files. If it has resources or messages, add `resources.rs` or `messages.rs`.

If this trigger fits an existing category, add the bridge to the existing `bridges.rs` and update `register.rs`.

## 6. Write the bridge system

In `bridges.rs`, create a bridge system that:
1. Reads the game event message
2. Builds TriggerContext from the message fields
3. Determines which entities to walk (based on scope)
4. Calls the walking algorithm on each entity

See [trigger-api/bridge-systems.md](trigger-api/bridge-systems.md) for the pattern and constraints.

## 7. Write the register function

In `register.rs`, register everything this trigger category needs:

```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, (
        on_my_event,
        on_my_event_occurred,
    ).in_set(EffectSystems::Bridge));

    // Game systems (if any)
    app.add_systems(FixedUpdate,
        tick_my_thing.in_set(EffectSystems::Tick)
    );

    // Resources (if any)
    app.init_resource::<MyRegistry>();
}
```

The plugin calls this function during build. Each trigger category owns its own registration — the plugin does not list individual systems.

## 8. Document

| What | Where |
|------|-------|
| Trigger description | `dispatching-triggers/<category>/<trigger-name>.md` (new) |
| Trigger in RON syntax | `ron-syntax/triggers/<trigger-name>.md` (new) |
| Update trigger list | `ron-syntax/triggers/triggers-list.md` |
| Update Trigger enum | `rust-types/enums/trigger.md` |
| If new participants | `ron-syntax/participants/`, `rust-types/enums/participants/` |
| If new TriggerContext variant | `rust-types/trigger-context.md` |
| Resolution table (if Local) | `target-resolution/on-targets.md` |

## Summary

| Step | Files touched |
|------|--------------|
| Trigger variant | `src/effect/types/trigger.rs` |
| TriggerContext variant (if new category) | `src/effect/types/trigger_context.rs` |
| Participant enum (if new) | `src/effect/types/participants.rs` |
| Trigger folder | `src/effect/triggers/<category>/` (new folder, if new category) |
| Bridge system | `src/effect/triggers/<category>/bridges.rs` |
| Register function | `src/effect/triggers/<category>/register.rs` |
| Game systems (if any) | `src/effect/triggers/<category>/<system_name>.rs` |
| Resources (if any) | `src/effect/triggers/<category>/resources.rs` |
| Messages (if any) | `src/effect/triggers/<category>/messages.rs` |
| Module wiring | `src/effect/triggers/<category>/mod.rs` |
| On() resolution (if new participants) | `src/effect/walking/on.rs` |

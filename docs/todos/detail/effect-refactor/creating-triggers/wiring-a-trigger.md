# Wiring a Trigger

Step-by-step checklist for adding a new trigger to the system.

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

## 5. Write the bridge system

Create a bridge system that:
1. Reads the game event message
2. Builds TriggerContext from the message fields
3. Determines which entities to walk (based on scope)
4. Calls the walking algorithm on each entity

See [trigger-api/bridge-systems.md](trigger-api/bridge-systems.md) for the pattern and constraints.

## 6. Register the bridge

Add the bridge system to the plugin. Schedule it in FixedUpdate, after the game system that produces the event message.

## 7. Document

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
| ParticipantTarget variant (if new) | `src/effect/types/participants.rs` |
| On() resolution (if new participants) | `src/effect/walking/on.rs` |
| Bridge system | `src/effect/bridges/<category>.rs` (new) |
| Plugin registration | `src/effect/plugin.rs` |

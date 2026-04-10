# Participant Targets

When a trigger involves a new kind of interaction with named roles not covered by the existing participant enums (Bump, Impact, Death, BoltLost), you need a new participant enum.

## When you need a new one

You need a new participant enum when:
- Your trigger is Local (fires on specific entities, not globally)
- The involved entities have distinct, named roles (not interchangeable)
- RON authors need to address those roles via On()

You do NOT need a new participant enum when:
- Your trigger is Global with no participants — use TriggerContext::None
- Your trigger is Self — use TriggerContext::None
- Your trigger's participants match an existing relationship — reuse the existing enum

## How to create one

1. **Define the enum** with one variant per role:
```rust
enum MyEventTarget {
    RoleA,
    RoleB,
}
```

2. **Add a variant to ParticipantTarget** wrapping it:
```rust
enum ParticipantTarget {
    // ...existing variants...
    MyEvent(MyEventTarget),
}
```

3. **Add a variant to TriggerContext** carrying the participant entities:
```rust
enum TriggerContext {
    // ...existing variants...
    MyEvent { role_a: Entity, role_b: Entity },
}
```

4. **Add resolution logic** so On(MyEvent(RoleA)) resolves to the correct entity from the context.

5. **Document**:
   - New enum file in `ron-syntax/participants/`
   - New enum file in `rust-types/enums/participants/`
   - Update `participant-list.md` in ron-syntax
   - Update `participant-target.md` in rust-types
   - Add resolution table in `target-resolution/on-targets.md`

## What makes a good participant role

A participant role is a named perspective on the event. "The entity that did the thing" and "the entity the thing was done to" are natural roles. If both sides are interchangeable (two bolts colliding), you still name roles — first/second, initiator/receiver — so On() can address them deterministically.

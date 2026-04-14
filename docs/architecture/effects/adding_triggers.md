# Adding a New Trigger

Step-by-step reference for adding a trigger to the new effect system.

## 1. Add variant to Trigger enum

In `effect_v3/core/types/definitions/enums.rs`:

```rust
enum Trigger {
    // ...
    NewThingBumped,              // local: past-tense verb
    NewThingOccurred,            // global: Occurred suffix
}
```

**Naming convention:**
- **Local** triggers (fire on participant entities): past-tense verb — `PerfectBumped`, `Impacted(Cell)`, `Died`
- **Global** triggers (fire on ALL entities with BoundEffects): `Occurred` suffix — `PerfectBumpOccurred`, `DeathOccurred(Cell)`, `BoltLostOccurred`

## 2. Participant enum (if needed)

If the trigger has named participants that `On(...)` can redirect to, add or reuse a participant enum:

```rust
// Existing participant enums — reuse when possible:
enum BumpTarget { Bolt, Breaker }          // bump triggers
enum ImpactTarget { Impactor, Impactee }   // impact triggers
enum DeathTarget { Victim, Killer }        // death triggers
enum BoltLostTarget { Bolt, Breaker }      // bolt lost triggers
```

If none fit, create a new enum and add a variant to `ParticipantTarget`:

```rust
enum NewTarget { A, B }

enum ParticipantTarget {
    // ...
    New(NewTarget),
}
```

Triggers with no participants (NodeStartOccurred, NodeTimerThresholdOccurred) have no participant enum — `On(...)` is not valid for them.

## 3. TriggerContext variant (if needed)

If the trigger introduces a new concept (not just a new variant of an existing concept), add a `TriggerContext` variant:

```rust
enum TriggerContext {
    Bump(BumpContext),      // bolt + breaker
    Impact(ImpactContext),  // impactor + impactee
    Death(DeathContext),    // victim + killer
    BoltLost(BoltLostContext),
    None { depth: u32 },   // no participants (node lifecycle, timers)
    // New(NewContext),
}
```

Every `TriggerContext` variant carries a `depth: u32` for recursion limiting (MAX_DISPATCH_DEPTH = 10).

## 4. Write bridge system

Bridge systems read existing game messages (Bevy 0.18 `MessageReader`) and call `walk_effects` with the appropriate context and entity list.

**Local bridge** — walks participant entities only:
```rust
fn bridge_new_thing_bumped(
    mut events: MessageReader<NewThingEvent>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let context = TriggerContext::New(NewContext {
            entity_a: event.a,
            entity_b: event.b,
            depth: 0,
        });
        // Walk participant entities
        for entity in [event.a, event.b] {
            if let Ok((entity, bound, mut staged)) = query.get_mut(entity) {
                walk_effects(&Trigger::NewThingBumped, &context, entity,
                    &bound, &mut staged, &mut commands);
            }
        }
    }
}
```

**Global bridge** — walks ALL entities with BoundEffects:
```rust
fn bridge_new_thing_occurred(
    mut events: MessageReader<NewThingEvent>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let context = TriggerContext::New(NewContext {
            entity_a: event.a,
            entity_b: event.b,
            depth: 0,
        });
        // Walk ALL entities
        for (entity, bound, mut staged) in &mut query {
            walk_effects(&Trigger::NewThingOccurred, &context, entity,
                &bound, &mut staged, &mut commands);
        }
    }
}
```

## 5. Register bridge system

Register in the effect plugin with correct ordering:
- After physics / game systems that produce the source message
- Within `EffectV3Systems::Bridge` system set
- Local and global variants are separate systems (same source message, different scope)

## 6. Write behavioral spec

Create `docs/todos/detail/effect-desugaring-node-running-trigger/triggers/<name>.md` documenting the trigger, its source message, participants, locality, and bridge system behavior.

## Key details

### On(ParticipantTarget, ...) redirect

`On(target, ...)` resolves a participant entity from `TriggerContext` at runtime. If the resolved entity has been despawned, log a debug warning and skip. `On(...)` is only valid for triggers that have participants.

### walk_effects ordering

Walk StagedEffects FIRST, then BoundEffects. This prevents a single trigger from both arming and consuming a nested When in the same dispatch call.

### MessageReader (Bevy 0.18)

Bridge systems use `MessageReader<T>` (not `EventReader`) to consume game messages. This is the Bevy 0.18 message API.

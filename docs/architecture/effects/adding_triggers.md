# Adding a New Trigger

Step-by-step reference for adding a trigger to the effect system.

The pattern is: **a variant in the `Trigger` enum, optionally a new `TriggerContext` variant, optionally a new participant role enum, and a bridge system that reads a game message and calls `walk_staged_effects` + `walk_bound_effects`.** Trigger categories are organized one per directory under `effect_v3/triggers/`.

## Where triggers live

```
effect_v3/triggers/
  bump/      — bump-based triggers
  impact/    — collision triggers
  death/     — death triggers
  bolt_lost/ — bolt-lost trigger
  node/      — node lifecycle + threshold
  time/      — TimeExpires and timer ticking
```

A new trigger either fits in an existing category (add a variant + bridge there) or warrants a new category (add a new directory). The decision is "does this share a `TriggerContext` shape with an existing category?" — if yes, reuse the category.

## 1. Add the variant to the Trigger enum

In `effect_v3/types/trigger.rs`:

```rust
pub enum Trigger {
    // ... existing variants
    NewThingHappened,                    // global trigger
    NewThingHappenedTo(EntityKind),      // local-with-kind trigger
}
```

**Naming convention:**
- **Local** triggers (fire on participant entities): past-tense verb that describes what *happened to* the entity — `PerfectBumped`, `Impacted(Cell)`, `Died`, `Killed(EntityKind)`.
- **Global** triggers (fire on every entity with effects): `Occurred` suffix — `PerfectBumpOccurred`, `DeathOccurred(Cell)`, `BoltLostOccurred`. The suffix signals "this happened somewhere; we're sweeping all entities."

If your trigger has an `f32` payload (e.g. a threshold or duration), use `OrderedFloat<f32>` so `Trigger` retains its `Hash`/`Eq` derives:

```rust
NewThresholdReached(OrderedFloat<f32>)
```

## 2. Decide on a TriggerContext variant

The `TriggerContext` carries the entities involved in the event. The walker uses it to resolve `Tree::On(ParticipantTarget, _)` redirects.

```rust
pub enum TriggerContext {
    Bump     { bolt: Option<Entity>, breaker: Entity },
    Impact   { impactor: Entity,     impactee: Entity },
    Death    { victim: Entity,       killer: Option<Entity> },
    BoltLost { bolt: Entity,         breaker: Entity },
    None,
}
```

Three options for your new trigger:

- **Reuse an existing variant.** If your trigger's participants match an existing variant's shape (e.g. another two-entity event with `victim`/`killer` semantics), reuse `Death`.
- **Add a new variant.** If your trigger has a distinct participant set, add a variant in `effect_v3/types/trigger_context.rs`.
- **Use `TriggerContext::None`.** For triggers that have no participants (`NodeStartOccurred`, `TimeExpires`), bridges build `TriggerContext::None`.

If you add a new variant, you do not need to add a `depth` field — recursion limiting is not modeled in `TriggerContext`.

## 3. Decide on a ParticipantTarget extension

If your trigger has named participants that `On(...)` can redirect to, you may need to extend `ParticipantTarget`. Existing role enums:

```rust
pub enum BumpTarget       { Bolt, Breaker }
pub enum ImpactTarget     { Impactor, Impactee }
pub enum DeathTarget      { Victim, Killer }
pub enum BoltLostTarget   { Bolt, Breaker }
```

If none fit, add a new role enum and a corresponding `ParticipantTarget` variant in `effect_v3/types/participants.rs`:

```rust
pub enum NewThingTarget { Source, Target }

pub enum ParticipantTarget {
    // ...
    NewThing(NewThingTarget),
}
```

Then add the resolution arms in `walking/on/system.rs` `resolve_participant`:

```rust
(ParticipantTarget::NewThing(NewThingTarget::Source),
 TriggerContext::NewThing { source, .. }) => Some(*source),
(ParticipantTarget::NewThing(NewThingTarget::Target),
 TriggerContext::NewThing { target, .. }) => Some(*target),
```

Triggers with no participants (`NodeStartOccurred`, `NodeTimerThresholdOccurred`, `TimeExpires`) have no participant role enum. `Tree::On` is not valid against them — the resolution would fall through to `_ => None` and silently skip.

## 4. Write the bridge system(s)

A bridge reads a game message, builds a `TriggerContext`, and calls `walk_staged_effects` + `walk_bound_effects` for each entity that should see the trigger.

The bridge functions live in `effect_v3/triggers/<category>/bridges/system.rs`. The standard shape:

```rust
use bevy::prelude::*;
use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::{Trigger, TriggerContext},
    walking::{walk_bound_effects, walk_staged_effects},
};

/// Local bridge — walks specific participant entities only.
pub fn on_new_thing_happened_to(
    mut events: MessageReader<NewThingMessage>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let context = TriggerContext::NewThing {
            source: event.source,
            target: event.target,
        };
        let trigger = Trigger::NewThingHappenedTo(event.target_kind);

        // Walk both participants
        for entity in [event.source, event.target] {
            if let Ok((entity, bound, mut staged)) = query.get_mut(entity) {
                walk_staged_effects(entity, &trigger, &context, &staged.0, &mut commands);
                walk_bound_effects(entity, &trigger, &context, &bound.0, &mut commands);
            }
        }
    }
}

/// Global bridge — walks every entity with effects.
pub fn on_new_thing_occurred(
    mut events: MessageReader<NewThingMessage>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let context = TriggerContext::NewThing {
            source: event.source,
            target: event.target,
        };
        let trigger = Trigger::NewThingHappened;

        for (entity, bound, mut staged) in &mut query {
            walk_staged_effects(entity, &trigger, &context, &staged.0, &mut commands);
            walk_bound_effects(entity, &trigger, &context, &bound.0, &mut commands);
        }
    }
}
```

**Always call `walk_staged_effects` before `walk_bound_effects`** for the same `(entity, trigger, context)` tuple — see `evaluation.md` for why.

## 5. Register the bridge

In `effect_v3/triggers/<category>/register.rs`:

```rust
use bevy::prelude::*;
use super::bridges;
use crate::effect_v3::EffectV3Systems;

pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_new_thing_happened_to,
            bridges::on_new_thing_occurred,
        )
            .in_set(EffectV3Systems::Bridge),
    );
}
```

If the bridges depend on a producing system from another domain, add `.after(...)` constraints:

```rust
.add_systems(
    FixedUpdate,
    bridges::on_new_thing_occurred
        .after(SomeOtherDomainSystems::ProduceMessage)
        .in_set(EffectV3Systems::Bridge),
)
```

## 6. Wire the category into EffectV3Plugin

If this is a new trigger category (new directory under `triggers/`), add the registration call to `EffectV3Plugin::build`:

```rust
triggers::bump::register::register(app);
triggers::impact::register::register(app);
// ...
triggers::new_category::register::register(app);
```

And in `effect_v3/triggers/mod.rs`:

```rust
pub mod new_category;
```

For an existing category, no plugin change is needed — the category's `register` function already exists and now wires the new bridge alongside the others.

## Common patterns

### Run condition

Most game triggers should only fire while the node is playing:

```rust
.add_systems(
    FixedUpdate,
    bridges::on_my_event.run_if(in_state(NodeState::Playing)).in_set(EffectV3Systems::Bridge),
)
```

### Trigger with payload

If the trigger carries a payload (e.g. `Trigger::NodeTimerThresholdOccurred(OrderedFloat<f32>)`), the bridge constructs the variant from the message data:

```rust
let trigger = Trigger::NodeTimerThresholdOccurred(OrderedFloat(event.threshold));
```

The walker compares triggers by structural equality, so the payload must match exactly between the chip's authored trigger and the bridge's dispatched trigger. Author-side and bridge-side must both round-trip through the same `OrderedFloat<f32>`.

### Time-based triggers

The `time` category owns `TimeExpires(seconds)` triggers and the timer ticking systems that decrement per-entity countdowns. If your new trigger needs per-entity time tracking, look at `triggers/time/` for the pattern: a dedicated component on the entity, a tick system that decrements, and a bridge that fires the `Trigger` when the countdown hits zero.

### Entity-kind filtering

Triggers that take an `EntityKind` payload (`Impacted(EntityKind)`, `DeathOccurred(EntityKind)`) let chips filter on entity type. The bridge fires multiple variants per event:

```rust
// On a single BoltImpactCell, fire both Impacted(Bolt) and Impacted(Cell)
// — the bridge dispatches the trigger that matches each participant.
```

`EntityKind::Any` is valid in chip-authored payloads (matches everything) but is not produced by bridges — bridges always emit the concrete kind.

## Behavioral spec

Add unit tests in `bridges/tests.rs` (or `bridges/tests/`) that:

1. Build a small world with one entity carrying a relevant `BoundEffects` entry.
2. Send the source message.
3. Run the bridge system once.
4. Assert the expected effect command was queued.

The bridge tests live in the same module as the bridge so they can use `use super::*;`.

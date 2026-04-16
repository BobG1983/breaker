# Effect System Architecture

The effect system is a data-driven trigger→effect pipeline. RON-defined effect trees are stamped onto game entities at chip selection, breaker initialization, cell initialization, and on entity spawns matching a `Spawn` root node. At runtime, game events are bridged into `Trigger` dispatches; the walker walks each entity's `BoundEffects` and `StagedEffects` and queues effect commands.

**Core principle: effects act on the entity they live on.** The owning entity is the implicit target. `On(participant, ...)` redirects to a participant from the trigger context; `commands.stamp_effect` installs onto a different entity at dispatch time. There is no global "target this" mechanism.

## Three Layers

1. **Triggers** (`effect_v3/triggers/<category>/`) — normal Bevy bridge systems. Read game messages, build a `TriggerContext`, call `walk_staged_effects` and then `walk_bound_effects` on the relevant entities.
2. **Walker** (`effect_v3/walking/`) — pattern-match on `Tree` nodes and queue commands. Pure logic — no `&mut World`. The walker is what `walk_*_effects` calls into via `evaluate_tree`.
3. **Commands** (`effect_v3/commands/`) — `EffectCommandsExt` extension trait + concrete `Command` structs. Get `&mut World` at command flush; call `fire_dispatch` / `reverse_dispatch` on the relevant config struct.

Per-effect implementations (`effect_v3/effects/<name>/`) sit beneath the dispatch layer. Each config struct implements `Fireable` (and `Reversible` if applicable). The enum-to-trait jump happens exactly once, in `fire_dispatch` and `reverse_dispatch`.

## Contents

- [Domain Structure](structure.md) — directory layout and per-module shapes
- [Core Types](core_types.md) — EffectType, ReversibleEffectType, Tree, ScopedTree, RootNode, StampTarget, Trigger, Condition, EntityKind, ParticipantTarget, TriggerContext
- [Tree Node Types](node_types.md) — Fire, When, Once, During, Until, Sequence, On + Tree-vs-ScopedTree restrictions
- [Targets and Participants](targets.md) — StampTarget vs EntityKind vs ParticipantTarget, resolution rules
- [Storage Components](components.md) — BoundEffects, StagedEffects, ArmedFiredParticipants, SpawnStampRegistry, EffectStack
- [Commands Extension](commands.md) — EffectCommandsExt trait and the eight concrete commands
- [Evaluation Flow](evaluation.md) — walk_bound_effects, walk_staged_effects, per-node evaluators, entry-specific consumption
- [Conditions and During](conditions.md) — Condition predicates, evaluate_conditions polling, DuringActive state machine
- [Until State Machine](until.md) — UntilEvaluateCommand, UntilApplied, the four shapes
- [Reversal](reversal.md) — Reversible trait, reverse_dispatch, reverse_all_by_source semantics
- [Dispatch](dispatch.md) — chip dispatch flow, spawn-stamp watchers, deferred install for non-Breaker targets
- [Trigger System Pattern](trigger_systems.md) — bridge system shape, register pattern, scope conventions
- [Trigger Reference](trigger_reference.md) — every Trigger variant, scope, participants, bridge mapping
- [System Ordering](ordering.md) — EffectV3Systems sets and FixedUpdate placement
- [Per-Effect Reference](effect_reference.md) — all 30 effects: config, fire, reversal, category, runtime systems
- [Adding Effects](adding_effects.md) — step-by-step guide for a new EffectType variant
- [Adding Triggers](adding_triggers.md) — step-by-step guide for a new Trigger variant
- [Adding Conditions](adding_conditions.md) — step-by-step guide for a new Condition variant
- [Adding Collisions](adding_collisions.md) — step-by-step guide for a new collision message + Impact triggers
- [Collision Messages](collisions.md) — impact detection in entity domains, message naming
- [Examples](examples.md) — RON examples for common chip shapes
- [Death Pipeline](death_pipeline.md) — KillYourself, Destroyed, killer attribution

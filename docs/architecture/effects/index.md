# Effect System Architecture

The effect system is a data-driven trigger→effect pipeline. RON-defined effect trees are dispatched onto game entities at chip selection, breaker initialization, and cell initialization. At runtime, game events are bridged into triggers. Trigger systems evaluate each entity's effect chains, firing effects and arming deeper chains.

**Core principle: effects act on the entity they live on.** The entity IS the implicit target.

## Three Layers

1. **Triggers** (`effect/triggers/`) — normal Bevy systems. Read game messages, walk entity chains, queue commands.
2. **Commands** (`effect/commands.rs`) — bridge layer. `EffectCommandsExt` trait. Custom Commands get `&mut World` at `apply_deferred`.
3. **Effects** (`effect/effects/`) — per-effect modules with `fire()` and `reverse()` functions. Each gets entity + `&mut World`.

No layer reaches into another's internals.

## Contents

- [Core Types](core_types.md) — all type definitions: EffectKind, Trigger, EffectNode, Target, RootEffect, BoundEffects, StagedEffects
- [Components](components.md) — BoundEffects and StagedEffects struct definitions and semantics
- [Node Types](node_types.md) — When, Do, Once, On (with brief Until/Reverse summaries)
- [Until and Desugaring](until.md) — Until mechanics, desugaring system, Reverse node, detailed examples
- [Commands Extension](commands.md) — EffectCommandsExt, FireEffectCommand, ReverseEffectCommand, TransferCommand
- [Evaluation Flow](evaluation.md) — how trigger systems walk chains, Until desugaring system, timer system
- [System Ordering](ordering.md) — FixedUpdate ordering chain from collisions to effect runtime
- [Reversal](reversal.md) — how every effect reverses, Until's Reverse node, two kinds of reversal
- [Dispatch](dispatch.md) — chip, breaker, and cell initialization (lives outside effect domain)
- [Target Resolution](targets.md) — On target resolution at dispatch vs runtime, All* desugaring
- [Collision Messages](collisions.md) — impact detection in entity domains, message naming
- [Trigger Systems](trigger_systems.md) — bridge pattern, scopes, On resolution, full Impact/Impacted example
- [Domain Structure](structure.md) — file layout for the effect domain
- [Examples](examples.md) — Overclock, wall redirect, cascade, nested triggers, passive piercing, recurring Until
- [Adding Effects](adding_effects.md) — step-by-step guide
- [Adding Triggers](adding_triggers.md) — step-by-step guide
- [Adding Collisions](adding_collisions.md) — step-by-step guide

## Design Reference

- [design/effects/](../../design/effects/index.md) — what each effect IS and does (game design)
- [design/triggers/](../../design/triggers/index.md) — what each trigger IS, global vs targeted (game design)

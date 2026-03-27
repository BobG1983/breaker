---
name: effect-system
description: CRITICAL — Effect system source of truth pointers. READ THE DOCS BEFORE TOUCHING ANY EFFECT CODE.
type: stable
---

# Effect System — Source of Truth

## ⚠️ READ THESE DOCS FIRST ⚠️

The effect system has been redesigned twice due to implementation diverging from design. The docs below are THE authoritative source. Do NOT rely on existing code in `effect/` or `effect_old/` — it may not match the design. Do NOT make assumptions. Read the docs.

## Architecture (HOW it works)

**`docs/architecture/effects/index.md`** — start here. Links to all architecture docs:
- `core_types.md` — ALL type definitions: EffectKind, Trigger, EffectNode, Target, RootEffect, ImpactTarget, AttractionType
- `components.md` — BoundEffects (permanent) and StagedEffects (consumed)
- `node_types.md` — When, Do, Once, On, Until (with detailed Until→When+Reverse desugaring)
- `commands.md` — EffectCommandsExt trait, FireEffectCommand, ReverseEffectCommand, TransferCommand
- `evaluation.md` — how trigger systems walk chains step by step
- `reversal.md` — how every effect reverses, Until's Reverse node
- `dispatch.md` — lives OUTSIDE effect domain (chips, breaker, cells)
- `targets.md` — On target resolution at dispatch vs runtime
- `collisions.md` — impact detection in entity domains, message naming (BoltImpactCell etc.)
- `trigger_systems.md` — bridge pattern, global vs targeted scopes
- `structure.md` — file layout
- `examples.md` — Overclock, wall redirect, cascade, nested triggers, recurring Until, passive piercing
- `adding_effects.md`, `adding_triggers.md`, `adding_collisions.md` — step-by-step guides

## Design (WHAT things are)

**`docs/design/effects/index.md`** — every effect, what it does, its parameters, its reversal behavior
**`docs/design/triggers/index.md`** — every trigger, its scope (global/targeted), what it means

## Key Rules (memorize these)

1. Effects act on self (the entity they live on)
2. On only redirects — never fires on current entity
3. BoundEffects = permanent, StagedEffects = consumed
4. Commands extension bridges triggers → effects (fire_effect, reverse_effect, transfer_effect)
5. EffectKind enum with inline fields — fire()/reverse() methods call per-module functions
6. No Effect trait — exhaustive match provides compile-time enforcement
7. No typed events, no observers for dispatch — custom Commands with &mut World
8. Trigger systems are normal Bevy systems — no exclusive world access
9. Dispatch lives in entity domains (chips, breaker, cells), NOT effect domain
10. Collision detection lives in entity domains, fires impact messages
11. Until desugars: fires Do children, pushes When children to BoundEffects, replaces itself with When(trigger, [Reverse(effects, chains)])
12. ALL effects define reverse() — there are no no-ops
13. Impact(X) = global, Impacted(X) = targeted on BOTH participants

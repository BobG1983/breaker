---
name: effect-system
description: Effect system architecture — entity-as-implicit-target, evaluation algorithm, trigger scopes, reversal model, On routing
type: stable
---

# Effect System Model

Effects act on the entity they live on. The entity IS the implicit target. No targets field, no typed events, no observer event structs.

## Components

- **EffectChains**: permanent trees on entity. Never consumed. Re-evaluates each trigger.
- **ArmedEffects**: working set. Consumed when matched. Fed by EffectChains evaluation and On redirects.

## Dispatch

`RootEffect::On(target, children)` at chip selection/breaker init pushes children to target's **EffectChains**.

- `Bolt` → primary bolt (new bolts inherit if `SpawnBolts(inherit: true)`)
- `AllBolts` → all bolts
- `Breaker` → breaker entity
- `Cell`/`AllCells`/`Wall` → no-op at dispatch

Passives: `When(Selected, [Do(Piercing(1))])` — Selected trigger fires at dispatch time.

## Evaluation Algorithm

For entity receiving trigger T:

**Collect** (loop ArmedEffects then EffectChains):
- When(trigger, children): if T matches → Do children to `to_fire`, else to `to_arm`. ArmedEffects entries consumed; EffectChains entries stay.
- Until: if T matches until-trigger → reverse children, remove Until
- Once: if children match → fire/arm, remove Once. Else keep.
- On(target, children): resolve target → `to_transfer`

**Execute**:
1. Fire `to_fire` on this entity
2. Push `to_arm` to ArmedEffects
3. Transfer: bare Do → fire on target entity; non-Do → push to target's ArmedEffects

## Trigger Scopes

**Global** (sweep all entities): PerfectBump, EarlyBump, LateBump, Bump, BumpWhiff, NoBump, Death, BoltLost, CellDestroyed, NodeTimerThreshold

**Targeted** (specific entity): PerfectBumped, EarlyBumped, LateBumped, Bumped, Impacted(*), Died

**Impacted fires both directions**: bolt hits cell → Impacted(Cell) on bolt + Impacted(Bolt) on cell. Any entity type on either side.

## Effect Execution

Direct function call. Handler receives entity + effect params, queries entity for components (Position2D, Velocity2D, etc.), no-op if missing.

## Reversal

Each reversible effect has a reverse function in its own file. `Effect::reverse()` method dispatches to it (one-line match arms). Until stores children and calls reverse on expiry. Until never imports effect internals.

## On Target Resolution

- At dispatch: Bolt=primary bolt, Breaker=breaker, plurals=all
- At runtime: singular=from trigger context, plural=all via query. Trigger systems fill out context from bridged messages.

## Key Design Rules

1. Effects act on self (entity they live on)
2. On only redirects — never fires on current entity
3. EffectChains = permanent, ArmedEffects = consumed
4. No typed events — direct function calls
5. Each effect owns its fire + reverse functions
6. Bridge systems only translate messages → triggers

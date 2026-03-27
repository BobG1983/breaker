# Evaluation Flow

There is no shared `evaluate()` function. Each trigger system walks chains directly. The pattern is the same across all triggers.

**Trigger systems never encounter `Until` nodes.** Until is desugared by a dedicated system that runs after all trigger systems — see [Until Desugaring System](#until-desugaring-system). By the time trigger systems walk chains, all Untils have been replaced with When+Reverse entries.

## For Each Entity Being Evaluated

### Step 1: Walk StagedEffects (entries consumed on match)

For each node in StagedEffects:

**When(trigger, children)**: If trigger matches, evaluate each child:
- `Do(effect)` → `commands.fire_effect(entity, effect)`
- `On(target, on_children, permanent)` → resolve target, transfer on_children to target (bare Do fires on target, non-Do pushes to target's StagedEffects or BoundEffects if `permanent: true`)
- Any other child → push to this entity's StagedEffects (re-arm at deeper level)
- Consume this entry

**Once(children)**: Evaluate children against trigger. A bare `Do` always matches (ready to fire). A `When(trigger, ...)` matches only if the current trigger matches. Other node types (`On`, `Once`, `Until`) are treated as non-Do children that get armed if any sibling matches.
- If any child matches: fire/arm children, remove the Once
- If no child matches: keep the Once

**On(target, children, permanent)**: Resolve target from the trigger system's message data:
- Bare Do children → `commands.fire_effect(target_entity, effect)`
- Non-Do children → push to target's StagedEffects (default) or BoundEffects (if `permanent: true`)

### Step 2: Walk BoundEffects (entries stay — permanent)

Same matching logic as StagedEffects, but entries are **never removed** (except by Reverse cleanup or Once consumption).

When a When matches, evaluate each child:
- `Do(effect)` → `commands.fire_effect(entity, effect)`
- `On(target, on_children, permanent)` → resolve target, transfer on_children to target (StagedEffects or BoundEffects per permanent flag)
- Any other child → push to StagedEffects

## On Unwrapping

When a `When` matches and one of its children is an `On`, the On is resolved **immediately** — its children go to the **target entity**, not back onto the current entity's StagedEffects. This applies in both StagedEffects and BoundEffects evaluation. The `permanent` flag controls whether non-Do children land in the target's StagedEffects (default) or BoundEffects (`permanent: true`).

Example: `When(Impacted(Cell), [On(AllCells, [Do(Shockwave(...))])])`

When Impacted(Cell) fires:
1. When matches → child is `On(AllCells, [Do(Shockwave)])`
2. On resolves AllCells → all cell entities
3. Bare Do → `commands.fire_effect(each_cell, Shockwave)` — fires on each cell, not on the bolt whose chain triggered this

If the On child is non-Do:
`When(Impacted(Cell), [On(Cell, [When(Died, [Do(Shockwave(...))])])])`

1. When matches → child is `On(Cell, [When(Died, Do(Shockwave))])`
2. On resolves Cell → the cell from the trigger context
3. Non-Do → `commands.transfer_effect(cell_entity, [When(Died, Do(Shockwave))])` — pushes to the cell's StagedEffects

## Until Desugaring System

Until desugaring is handled by a **dedicated system**, not by trigger systems during chain walking. This system:

1. Runs **after all trigger bridge systems** in FixedUpdate
2. Queries all entities with BoundEffects and StagedEffects
3. Finds any `Until` nodes in either component
4. Desugars each Until:
   - `Do` children → `commands.fire_effect(entity, effect)` (fire immediately)
   - `When` children → push to the entity's BoundEffects (recurring)
   - Replace the Until with `When(trigger, [Reverse(effects, chains)])` in StagedEffects
5. The original Until entry is removed from whichever component it was in

This means:
- **Dispatch** can freely push Until nodes to BoundEffects — the desugaring system handles them on the next tick
- **Trigger systems** never see Until — they only see When, Do, Once, On, and Reverse
- **Ordering is explicit**: collision systems → trigger bridges → Until desugaring → apply_deferred

## Timer System

The timer system (`effect/triggers/timer.rs`) is a normal Bevy system that runs every FixedUpdate tick after trigger bridges. It handles `TimeExpires` reversal:

1. Queries all entities with StagedEffects
2. For each `When(TimeExpires(remaining), children)` entry:
   - Decrements `remaining` by `delta_time`
   - When `remaining <= 0`: evaluates children (which will be `[Reverse(...)]`), consuming the entry
3. This is the **only trigger that modifies StagedEffects entries in-place** (decrementing the timer) rather than matching-and-consuming

## Key Invariants

- StagedEffects entries are consumed when matched. BoundEffects entries are permanent.
- StagedEffects is walked first, then BoundEffects.
- Non-Do children of matching When push to StagedEffects, EXCEPT:
  - `On` children are resolved immediately — their children go to the target entity
- Trigger systems never encounter Until — the desugaring system handles it separately.
- On never fires on the current entity — it only transfers to other entities.

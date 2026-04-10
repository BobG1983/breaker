# When

## Receives
`When(Trigger, Box<Tree>)` — a repeating gate with a trigger and an inner tree.

## Behavior

1. Compare the When's trigger to the trigger that was fired.
2. If they do not match, stop. Do nothing.
3. If they match, check the inner tree's outermost node:
   a. If the inner tree is a **trigger gate** (When, Once, Until) — **arm it**: call `stage_effect(entity, source, inner_tree)` to install it into StagedEffects for a future trigger. Do not evaluate it recursively. This is true even if the inner gate's trigger matches the current trigger — `When(Bumped, When(Bumped, Fire(X)))` means "bumped twice", not "bumped and immediately fire".
   b. If the inner tree is NOT a trigger gate (Fire, Sequence, On, Route, During) — evaluate it recursively.
4. The When entry stays in storage. It will match again on the next occurrence of the same trigger.

## Arming

Step 3a is the "arming" mechanism described in `arming-effects.md`. Example:

`When(Bumped, When(Impacted(Cell), Fire(Explode(...))))`

When Bumped fires, the outer When matches. The inner tree is `When(Impacted(Cell), ...)` — a trigger gate. It is armed into StagedEffects via `stage_effect`. It will fire when the next `Impacted(Cell)` trigger occurs.

Another example: `When(Bumped, When(Bumped, Fire(X)))` — "bumped twice." The outer When matches on the first bump and arms the inner When into StagedEffects. The inner When fires on the second bump.

## Constraints

- DO leave the When in place after matching — it re-arms automatically.
- DO NOT remove the When from BoundEffects after matching. That is Once's behavior, not When's.
- DO pass the trigger context through to the inner tree evaluation so On nodes inside can resolve participants.
- DO use `stage_effect` (deferred command) for arming, not direct StagedEffects mutation.

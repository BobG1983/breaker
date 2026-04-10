# Once

## Receives
`Once(Trigger, Box<Tree>)` — a one-shot gate with a trigger and an inner tree.

## Behavior

1. Compare the Once's trigger to the trigger that was fired.
2. If they do not match, stop. Do nothing.
3. If they match, evaluate the inner tree recursively (same arming rules as When — see `when.md` step 3).
4. Queue `remove_effect(entity, Bound, source, tree)` to remove this Once entry from BoundEffects. It has fired and is done.

## Constraints

- DO remove the Once from BoundEffects after matching via `remove_effect` command (deferred). This is the key difference from When.
- DO evaluate the inner tree before queuing removal — the tree fires, then the Once is cleaned up.
- DO pass the trigger context through to the inner tree evaluation.
- DO use the same arming rules as When for nested trigger gates inside the inner tree.

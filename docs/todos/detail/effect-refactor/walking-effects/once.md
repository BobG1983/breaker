# Once

## Receives
`Once(Trigger, Box<Tree>)` — a one-shot gate with a trigger and an inner tree.

## Behavior

1. Compare the Once's trigger to the trigger that was fired.
2. If they do not match, stop. Do nothing.
3. If they match, evaluate the inner tree recursively.
4. Mark this Once entry for removal from BoundEffects. It has fired and is done.

## Constraints

- DO remove the Once from BoundEffects after matching. This is the key difference from When.
- DO evaluate the inner tree before removing — the tree fires, then the Once is cleaned up.
- DO pass the trigger context through to the inner tree evaluation.

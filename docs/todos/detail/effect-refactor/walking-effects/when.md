# When

## Receives
`When(Trigger, Box<Tree>)` — a repeating gate with a trigger and an inner tree.

## Behavior

1. Compare the When's trigger to the trigger that was fired.
2. If they do not match, stop. Do nothing.
3. If they match, evaluate the inner tree recursively.
4. The When entry stays in storage. It will match again on the next occurrence of the same trigger.

## Constraints

- DO leave the When in place after matching — it re-arms automatically.
- DO NOT remove the When from BoundEffects after matching. That is Once's behavior, not When's.
- DO pass the trigger context through to the inner tree evaluation so On nodes inside can resolve participants.

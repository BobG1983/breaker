# Arming Effects

"Arming" is shorthand for taking a sub-tree from a BoundEffects entry and inserting it into StagedEffects.

For example, when a `When(Bumped, When(Impacted(Cell), Fire(Explode(...))))` entry in BoundEffects matches a Bumped trigger, the inner `When(Impacted(Cell), Fire(Explode(...)))` is armed — it moves into StagedEffects to await the next Impacted(Cell) trigger as a one-shot.
